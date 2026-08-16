use std::io;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const EVENT_QUEUE_CAPACITY: usize = 1;
const QUIET_PERIOD: Duration = Duration::from_millis(300);
const MAXIMUM_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DebouncerStopped;

/// Non-blocking handle used by event producers.
#[derive(Clone)]
pub(crate) struct DebounceTrigger {
    sender: SyncSender<DebounceMessage>,
}

impl DebounceTrigger {
    /// Queues a change, or merges it with the change already waiting in the bounded queue.
    pub(crate) fn signal_change(&self) -> Result<(), DebouncerStopped> {
        match self.sender.try_send(DebounceMessage::Changed) {
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(TrySendError::Disconnected(_)) => Err(DebouncerStopped),
        }
    }
}

/// Owns the debounce worker and joins it on drop.
pub(crate) struct EventDebouncer {
    shutdown_sender: SyncSender<DebounceMessage>,
    worker: Option<JoinHandle<()>>,
}

impl EventDebouncer {
    /// Runs `on_batch` after 300 ms of quiet, capped at one second per observed batch.
    pub(crate) fn start(
        on_batch: impl FnMut() + Send + 'static,
    ) -> io::Result<(Self, DebounceTrigger)> {
        let (sender, receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let trigger = DebounceTrigger {
            sender: sender.clone(),
        };
        let worker = thread::Builder::new()
            .name("event-debouncer".to_string())
            .spawn(move || run_debounce_worker(receiver, on_batch))?;

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
        let _ = self.shutdown_sender.send(DebounceMessage::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Clone, Copy)]
enum DebounceMessage {
    Changed,
    Shutdown,
}

fn run_debounce_worker(receiver: Receiver<DebounceMessage>, on_batch: impl FnMut()) {
    run_debounce_worker_with_policy(receiver, QUIET_PERIOD, MAXIMUM_DELAY, on_batch);
}

fn run_debounce_worker_with_policy(
    receiver: Receiver<DebounceMessage>,
    quiet_period: Duration,
    maximum_delay: Duration,
    mut on_batch: impl FnMut(),
) {
    loop {
        match receiver.recv() {
            Ok(DebounceMessage::Changed) => {}
            Ok(DebounceMessage::Shutdown) | Err(_) => return,
        }

        let first_change_at = Instant::now();
        let mut latest_change_at = first_change_at;

        loop {
            let wait = remaining_wait(
                first_change_at,
                latest_change_at,
                Instant::now(),
                quiet_period,
                maximum_delay,
            );
            if wait.is_zero() {
                on_batch();
                break;
            }

            match receiver.recv_timeout(wait) {
                Ok(DebounceMessage::Changed) => latest_change_at = Instant::now(),
                Ok(DebounceMessage::Shutdown) | Err(RecvTimeoutError::Disconnected) => return,
                Err(RecvTimeoutError::Timeout) => {
                    on_batch();
                    break;
                }
            }
        }
    }
}

fn remaining_wait(
    first_change_at: Instant,
    latest_change_at: Instant,
    now: Instant,
    quiet_period: Duration,
    maximum_delay: Duration,
) -> Duration {
    quiet_period
        .saturating_sub(now.saturating_duration_since(latest_change_at))
        .min(maximum_delay.saturating_sub(now.saturating_duration_since(first_change_at)))
}

#[cfg(test)]
mod tests {
    use super::{
        remaining_wait, run_debounce_worker_with_policy, DebounceMessage, MAXIMUM_DELAY,
        QUIET_PERIOD,
    };
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn worker_coalesces_queued_changes_into_one_callback() {
        let (message_sender, message_receiver) = mpsc::sync_channel(4);
        let (callback_sender, callback_receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            run_debounce_worker_with_policy(
                message_receiver,
                Duration::from_millis(10),
                Duration::from_millis(50),
                move || callback_sender.send(()).unwrap(),
            );
        });

        for _ in 0..3 {
            message_sender.send(DebounceMessage::Changed).unwrap();
        }

        callback_receiver
            .recv_timeout(Duration::from_millis(200))
            .expect("queued changes should produce a callback");
        assert!(callback_receiver
            .recv_timeout(Duration::from_millis(30))
            .is_err());

        message_sender.send(DebounceMessage::Shutdown).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn worker_discards_a_pending_batch_on_shutdown() {
        let (message_sender, message_receiver) = mpsc::sync_channel(2);
        let (callback_sender, callback_receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            run_debounce_worker_with_policy(
                message_receiver,
                Duration::from_secs(1),
                Duration::from_secs(1),
                move || callback_sender.send(()).unwrap(),
            );
        });

        message_sender.send(DebounceMessage::Changed).unwrap();
        message_sender.send(DebounceMessage::Shutdown).unwrap();
        worker.join().unwrap();

        assert!(callback_receiver.try_recv().is_err());
    }

    #[test]
    fn wait_ends_at_the_quiet_or_maximum_deadline() {
        let start = Instant::now();

        assert_eq!(
            remaining_wait(
                start,
                start,
                start + Duration::from_millis(299),
                QUIET_PERIOD,
                MAXIMUM_DELAY,
            ),
            Duration::from_millis(1)
        );
        assert_eq!(
            remaining_wait(
                start,
                start + Duration::from_millis(900),
                start + MAXIMUM_DELAY,
                QUIET_PERIOD,
                MAXIMUM_DELAY,
            ),
            Duration::ZERO
        );
    }
}
