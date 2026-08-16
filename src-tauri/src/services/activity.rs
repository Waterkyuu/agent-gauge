use crate::adapters::activity::{AgentActivityAdapter, SystemAgentActivityAdapter};
use crate::adapters::process::AgentProcessStates;
use crate::domain::agent_activity::AgentActivity;
use crate::utils::debounce::EventDebouncer;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::sync::{Arc, Mutex, MutexGuard};

/// Application monitor type wired to the real local Agent sources.
pub(crate) type SystemAgentActivityMonitor = AgentActivityMonitor<SystemAgentActivityAdapter>;

/// Owns the latest protocol-derived task snapshot and native source watchers.
pub(crate) struct AgentActivityMonitor<A: AgentActivityAdapter + Send + Sync + 'static> {
    /// Shared state used by process and filesystem callbacks.
    state: Arc<AgentActivityMonitorState<A>>,
    /// Native watcher retained for the application lifetime.
    _watcher: Option<Mutex<RecommendedWatcher>>,
    /// Debounce worker retained so bursts produce one bounded rescan.
    _debouncer: Option<EventDebouncer>,
}

/// Lightweight clone passed to the independent process monitor callback.
pub(crate) struct AgentActivityMonitorHandle<A: AgentActivityAdapter + Send + Sync + 'static> {
    /// Shared activity state owned by the full monitor.
    state: Arc<AgentActivityMonitorState<A>>,
}

struct AgentActivityMonitorState<A: AgentActivityAdapter + Send + Sync + 'static> {
    /// Read-only product adapter used for each bounded refresh.
    adapter: A,
    /// Latest process snapshot used only to detect unfinished task termination.
    processes: Mutex<AgentProcessStates>,
    /// Latest activities exposed to the Tauri command.
    activities: Mutex<Vec<AgentActivity>>,
    /// Serializes process and filesystem refreshes.
    refresh_lock: Mutex<()>,
    /// Emits changed snapshots without binding the service to Tauri.
    on_change: Mutex<Box<dyn FnMut(Vec<AgentActivity>) + Send>>,
}

impl<A: AgentActivityAdapter + Send + Sync + 'static> AgentActivityMonitor<A> {
    /// Builds the initial snapshot and subscribes to existing product-owned directories.
    pub(crate) fn start(
        adapter: A,
        processes: AgentProcessStates,
        on_change: impl FnMut(Vec<AgentActivity>) + Send + 'static,
    ) -> notify::Result<Self> {
        let mut monitor = Self::create(adapter, processes, on_change);
        let state = Arc::clone(&monitor.state);
        let (debouncer, trigger) = EventDebouncer::start(move || refresh_activity_state(&state))
            .map_err(notify::Error::io)?;
        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| {
            if result.as_ref().is_ok_and(event_affects_agent_activity) {
                // A disconnected trigger means application teardown already owns cleanup.
                let _monitor_is_active = trigger.signal_change().is_ok();
            }
        })?;
        let mut watched_any_path = false;
        for path in monitor.state.adapter.watch_paths() {
            if watcher.watch(&path, RecursiveMode::Recursive).is_ok() {
                watched_any_path = true;
            }
        }
        monitor._watcher = watched_any_path.then(|| Mutex::new(watcher));
        monitor._debouncer = Some(debouncer);
        Ok(monitor)
    }

    /// Returns a handle that can refresh the same state when a product process starts or stops.
    pub(crate) fn handle(&self) -> AgentActivityMonitorHandle<A> {
        AgentActivityMonitorHandle {
            state: Arc::clone(&self.state),
        }
    }

    /// Returns the latest successful activity snapshot without touching product files.
    pub(crate) fn current_activities(&self) -> Vec<AgentActivity> {
        lock_or_recover(&self.state.activities).clone()
    }

    /// Creates shared monitor state without filesystem watching for deterministic unit tests.
    fn create(
        adapter: A,
        processes: AgentProcessStates,
        on_change: impl FnMut(Vec<AgentActivity>) + Send + 'static,
    ) -> Self {
        let activities = adapter.list_activities(processes);
        Self {
            state: Arc::new(AgentActivityMonitorState {
                adapter,
                processes: Mutex::new(processes),
                activities: Mutex::new(activities),
                refresh_lock: Mutex::new(()),
                on_change: Mutex::new(Box::new(on_change)),
            }),
            _watcher: None,
            _debouncer: None,
        }
    }
}

impl<A: AgentActivityAdapter + Send + Sync + 'static> AgentActivityMonitorHandle<A> {
    /// Stores a process transition and refreshes the protocol-derived task snapshot.
    pub(crate) fn update_process_states(&self, processes: AgentProcessStates) {
        let changed = {
            let mut current = lock_or_recover(&self.state.processes);
            if *current == processes {
                false
            } else {
                *current = processes;
                true
            }
        };
        if changed {
            refresh_activity_state(&self.state);
        }
    }
}

/// Replaces and emits the snapshot only when an adapter observation changed.
fn refresh_activity_state<A: AgentActivityAdapter + Send + Sync + 'static>(
    state: &AgentActivityMonitorState<A>,
) {
    let _refresh_guard = lock_or_recover(&state.refresh_lock);
    let processes = *lock_or_recover(&state.processes);
    let next = state.adapter.list_activities(processes);
    let changed = {
        let mut current = lock_or_recover(&state.activities);
        if *current == next {
            false
        } else {
            *current = next.clone();
            true
        }
    };
    if changed {
        lock_or_recover(&state.on_change)(next);
    }
}

/// Accepts only mutating JSONL or LevelDB data-file events from registered source roots.
fn event_affects_agent_activity(event: &Event) -> bool {
    let mutating = matches!(
        event.kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );
    mutating
        && event.paths.iter().any(|path| {
            path.extension()
                .and_then(OsStr::to_str)
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("jsonl")
                        || extension.eq_ignore_ascii_case("log")
                        || extension.eq_ignore_ascii_case("ldb")
                        || extension.eq_ignore_ascii_case("sst")
                })
                || path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name == "CURRENT")
        })
}

/// Recovers short non-async state guards after a prior callback panic.
fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::AgentActivityMonitor;
    use crate::adapters::activity::AgentActivityAdapter;
    use crate::adapters::process::AgentProcessStates;
    use crate::domain::agent_activity::{AgentActivity, AgentActivityKind, AgentActivityStatus};
    use std::sync::mpsc;
    use std::time::Duration;

    struct FakeActivityAdapter;

    impl AgentActivityAdapter for FakeActivityAdapter {
        fn list_activities(&self, processes: AgentProcessStates) -> Vec<AgentActivity> {
            vec![AgentActivity {
                id: "codex-test".to_string(),
                agent: AgentActivityKind::Codex,
                status: if processes.codex {
                    AgentActivityStatus::Running
                } else {
                    AgentActivityStatus::Error
                },
                updated_at_ms: 1,
            }]
        }

        fn watch_paths(&self) -> Vec<std::path::PathBuf> {
            Vec::new()
        }
    }

    #[test]
    fn process_changes_recompute_protocol_activities_and_emit_only_changed_snapshots() {
        let (sender, receiver) = mpsc::channel();
        let monitor = AgentActivityMonitor::create(
            FakeActivityAdapter,
            AgentProcessStates::default(),
            move |activities| {
                sender
                    .send(activities)
                    .expect("test receiver should remain open")
            },
        );

        assert_eq!(
            monitor.current_activities()[0].status,
            AgentActivityStatus::Error
        );

        monitor.handle().update_process_states(AgentProcessStates {
            codex: true,
            ..AgentProcessStates::default()
        });
        let changed = receiver
            .recv_timeout(Duration::from_millis(100))
            .expect("changed process state should emit activities");
        assert_eq!(changed[0].status, AgentActivityStatus::Running);

        monitor.handle().update_process_states(AgentProcessStates {
            codex: true,
            ..AgentProcessStates::default()
        });
        assert!(receiver.recv_timeout(Duration::from_millis(20)).is_err());
    }
}
