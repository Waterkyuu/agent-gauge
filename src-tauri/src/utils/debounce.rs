use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const EVENT_QUEUE_CAPACITY: usize = 1;

/// Indicates that the debounce worker has already stopped accepting change signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DebouncerStopped;

/// Sends change signals without blocking the file-watcher callback thread.
#[derive(Clone)]
pub(crate) struct DebounceTrigger {
    /// Bounded channel that collapses duplicate signals while one signal is already queued.
    sender: SyncSender<DebounceMessage>,
}

impl DebounceTrigger {
    /// Queues a change for the debounce worker.
    ///
    /// This is called by event producers such as filesystem watchers. A full queue means another
    /// change is already pending, so the new signal is safely treated as part of the same batch.
    pub(crate) fn signal_change(&self) -> Result<(), DebouncerStopped> {
        match self.sender.try_send(DebounceMessage::Changed) {
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(TrySendError::Disconnected(_)) => Err(DebouncerStopped),
        }
    }
}

/// Owns one debounce worker and stops it when the surrounding watcher is dropped.
pub(crate) struct EventDebouncer {
    /// Channel used by `Drop` to request worker shutdown.
    shutdown_sender: SyncSender<DebounceMessage>,
    /// Join handle retained so the background thread cannot outlive this owner.
    worker: Option<JoinHandle<()>>,
}

impl EventDebouncer {
    /// Starts a worker that runs `on_batch` after changes settle or reach `maximum_delay`.
    ///
    /// The returned trigger is cloned into event producers. For example, with a 300 ms quiet
    /// period and a one-second maximum delay, several rapid calls to `signal_change` produce one
    /// callback after the burst, while continuous calls still produce a callback every second.
    pub(crate) fn start(
        quiet_period: Duration,
        maximum_delay: Duration,
        on_batch: impl FnMut() + Send + 'static,
    ) -> io::Result<(Self, DebounceTrigger)> {
        let (sender, receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let trigger = DebounceTrigger {
            sender: sender.clone(),
        };
        let worker = thread::Builder::new()
            .name("event-debouncer".to_string())
            .spawn(move || {
                run_debounce_worker(receiver, quiet_period, maximum_delay, on_batch);
            })?;

        Ok((
            Self {
                shutdown_sender: sender,
                worker: Some(worker),
            },
            trigger,
        ))
    }
}

impl Drop for EventDebouncer {
    fn drop(&mut self) {
        if self
            .shutdown_sender
            .send(DebounceMessage::Shutdown)
            .is_err()
        {
            // A disconnected receiver means the worker has already stopped, so no shutdown
            // request remains to be delivered.
        }

        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                // A callback panic has already terminated the worker. Drop cannot propagate that
                // panic without risking a second panic while another thread is unwinding.
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebounceMessage {
    Changed,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebounceDecision {
    /// No change is waiting for a callback.
    Idle,
    /// A change is pending and the worker should wait for the remaining duration.
    Wait(Duration),
    /// The quiet period or maximum delay has elapsed, so the callback should run.
    Run,
}

/// Tracks one batch using timestamps supplied by the worker.
///
/// Keeping this timing decision separate from channels and threads makes the boundary conditions
/// deterministic to test without sleeping or depending on scheduler timing.
struct DebounceSchedule {
    /// Required silence after the newest change before the callback may run.
    quiet_period: Duration,
    /// Longest time a continuous burst may postpone the callback.
    maximum_delay: Duration,
    /// Time of the first change in the current batch.
    first_change_at: Option<Instant>,
    /// Time of the newest change in the current batch.
    latest_change_at: Option<Instant>,
}

impl DebounceSchedule {
    /// Creates an idle schedule using the caller's quiet period and maximum batch delay.
    fn new(quiet_period: Duration, maximum_delay: Duration) -> Self {
        Self {
            quiet_period,
            maximum_delay,
            first_change_at: None,
            latest_change_at: None,
        }
    }

    /// Adds a change to the current batch or starts a new batch when the schedule is idle.
    fn record_change(&mut self, changed_at: Instant) {
        self.first_change_at.get_or_insert(changed_at);
        self.latest_change_at = Some(changed_at);
    }

    /// Returns whether the worker should stay idle, wait longer, or run its callback now.
    fn decision_at(&self, now: Instant) -> DebounceDecision {
        let (Some(first_change_at), Some(latest_change_at)) =
            (self.first_change_at, self.latest_change_at)
        else {
            return DebounceDecision::Idle;
        };

        let quiet_elapsed = now.saturating_duration_since(latest_change_at);
        let total_elapsed = now.saturating_duration_since(first_change_at);
        if quiet_elapsed >= self.quiet_period || total_elapsed >= self.maximum_delay {
            return DebounceDecision::Run;
        }

        DebounceDecision::Wait(
            self.quiet_period
                .saturating_sub(quiet_elapsed)
                .min(self.maximum_delay.saturating_sub(total_elapsed)),
        )
    }

    /// Clears the completed batch so the next change starts with a fresh maximum-delay window.
    fn mark_run(&mut self) {
        self.first_change_at = None;
        self.latest_change_at = None;
    }
}

/// Receives change signals until shutdown and executes one callback for each settled batch.
fn run_debounce_worker(
    receiver: Receiver<DebounceMessage>,
    quiet_period: Duration,
    maximum_delay: Duration,
    mut on_batch: impl FnMut(),
) {
    let mut schedule = DebounceSchedule::new(quiet_period, maximum_delay);

    loop {
        match receiver.recv() {
            Ok(DebounceMessage::Changed) => schedule.record_change(Instant::now()),
            Ok(DebounceMessage::Shutdown) | Err(_) => return,
        }

        loop {
            match schedule.decision_at(Instant::now()) {
                DebounceDecision::Idle => break,
                DebounceDecision::Run => {
                    schedule.mark_run();
                    on_batch();
                    break;
                }
                DebounceDecision::Wait(wait) => match receiver.recv_timeout(wait) {
                    Ok(DebounceMessage::Changed) => schedule.record_change(Instant::now()),
                    Ok(DebounceMessage::Shutdown) | Err(RecvTimeoutError::Disconnected) => return,
                    Err(RecvTimeoutError::Timeout) => {}
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DebounceDecision, DebounceSchedule};
    use std::time::{Duration, Instant};

    #[test]
    fn waits_until_the_latest_change_has_been_quiet() {
        let start = Instant::now();
        let mut schedule =
            DebounceSchedule::new(Duration::from_millis(100), Duration::from_millis(500));

        schedule.record_change(start);
        schedule.record_change(start + Duration::from_millis(80));

        assert_eq!(
            schedule.decision_at(start + Duration::from_millis(100)),
            DebounceDecision::Wait(Duration::from_millis(80))
        );
        assert_eq!(
            schedule.decision_at(start + Duration::from_millis(180)),
            DebounceDecision::Run
        );
    }

    #[test]
    fn runs_at_the_maximum_delay_during_continuous_changes() {
        let start = Instant::now();
        let mut schedule =
            DebounceSchedule::new(Duration::from_millis(100), Duration::from_millis(300));

        schedule.record_change(start);
        schedule.record_change(start + Duration::from_millis(90));
        schedule.record_change(start + Duration::from_millis(180));
        schedule.record_change(start + Duration::from_millis(270));

        assert_eq!(
            schedule.decision_at(start + Duration::from_millis(299)),
            DebounceDecision::Wait(Duration::from_millis(1))
        );
        assert_eq!(
            schedule.decision_at(start + Duration::from_millis(300)),
            DebounceDecision::Run
        );
    }

    #[test]
    fn returns_to_idle_after_the_callback_runs() {
        let start = Instant::now();
        let mut schedule =
            DebounceSchedule::new(Duration::from_millis(100), Duration::from_millis(500));
        schedule.record_change(start);

        schedule.mark_run();

        assert_eq!(schedule.decision_at(start), DebounceDecision::Idle);
    }
}
