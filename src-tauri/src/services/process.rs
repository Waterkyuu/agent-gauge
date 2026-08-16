use crate::adapters::process::{AgentProcessAdapter, AgentProcessStates};
use crate::error::AppError;
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Reads one snapshot of supported local Agent processes.
pub(crate) fn check_agent_processes(
    adapter: &mut impl AgentProcessAdapter,
) -> Result<AgentProcessStates, AppError> {
    adapter.check_processes()
}

/// Owns the reusable process source, latest snapshot, and application-lifetime worker thread.
pub(crate) struct AgentProcessMonitor {
    /// Latest successful process snapshot shared with the Tauri snapshot command.
    current_states: Arc<Mutex<AgentProcessStates>>,
    /// Sends the shutdown signal consumed by the worker thread during application teardown.
    stop_sender: SyncSender<()>,
    /// Join handle retained so dropping the monitor waits for the worker to finish.
    worker: Option<JoinHandle<()>>,
}

impl AgentProcessMonitor {
    /// Reads the initial snapshot, then checks the retained adapter after every interval.
    ///
    /// The callback runs on the worker thread only when a successful snapshot differs from the
    /// previous snapshot. For example, a `false -> true` Codex transition invokes it once.
    pub(crate) fn start(
        mut adapter: impl AgentProcessAdapter + Send + 'static,
        interval: Duration,
        mut on_change: impl FnMut(AgentProcessStates) + Send + 'static,
    ) -> Result<Self, AppError> {
        let initial_states = check_agent_processes(&mut adapter)?;
        let current_states = Arc::new(Mutex::new(initial_states));
        let worker_states = Arc::clone(&current_states);
        let (stop_sender, stop_receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || loop {
            match stop_receiver.recv_timeout(interval) {
                Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            let Ok(next_states) = check_agent_processes(&mut adapter) else {
                continue;
            };
            let Ok(mut stored_states) = worker_states.lock() else {
                break;
            };
            if *stored_states == next_states {
                continue;
            }

            *stored_states = next_states;
            drop(stored_states);
            on_change(next_states);
        });

        Ok(Self {
            current_states,
            stop_sender,
            worker: Some(worker),
        })
    }

    /// Returns the most recent successful process snapshot without scanning the operating system.
    ///
    /// For example, immediately after startup this returns the snapshot read by [`Self::start`].
    pub(crate) fn current_states(&self) -> Result<AgentProcessStates, AppError> {
        self.current_states
            .lock()
            .map(|states| *states)
            .map_err(|_| AppError::ProcessProbeFailed)
    }
}

impl Drop for AgentProcessMonitor {
    fn drop(&mut self) {
        let _ = self.stop_sender.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AgentProcessMonitor;
    use crate::adapters::process::{AgentProcessAdapter, AgentProcessStates};
    use crate::error::AppError;
    use std::collections::VecDeque;
    use std::sync::mpsc;
    use std::time::Duration;

    struct FakeProcessAdapter {
        /// Ordered snapshots returned by consecutive monitor checks.
        states: VecDeque<AgentProcessStates>,
        /// Last snapshot repeated after the ordered test data is exhausted.
        last_state: AgentProcessStates,
    }

    impl AgentProcessAdapter for FakeProcessAdapter {
        fn check_processes(&mut self) -> Result<AgentProcessStates, AppError> {
            if let Some(state) = self.states.pop_front() {
                self.last_state = state;
            }
            Ok(self.last_state)
        }
    }

    #[test]
    fn emits_only_when_the_process_snapshot_changes() {
        let stopped = AgentProcessStates::default();
        let started = AgentProcessStates {
            codex: true,
            ..AgentProcessStates::default()
        };
        let adapter = FakeProcessAdapter {
            states: VecDeque::from([stopped, stopped, started, started]),
            last_state: stopped,
        };
        let (event_sender, event_receiver) = mpsc::channel();
        let monitor =
            AgentProcessMonitor::start(adapter, Duration::from_millis(5), move |states| {
                let _ = event_sender.send(states);
            })
            .expect("the initial fake snapshot should succeed");

        assert_eq!(
            event_receiver.recv_timeout(Duration::from_millis(100)),
            Ok(started)
        );
        assert!(event_receiver
            .recv_timeout(Duration::from_millis(20))
            .is_err());
        assert_eq!(monitor.current_states(), Ok(started));
    }
}
