use crate::utils::debounce::EventDebouncer;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const CONFIG_CHANGE_QUIET_PERIOD: Duration = Duration::from_millis(300);
const CONFIG_CHANGE_MAXIMUM_DELAY: Duration = Duration::from_secs(1);

/// Native outcomes relevant to the Codex runtime-defaults cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CodexConfigWatchEvent {
    Changed,
    Failed,
}

/// Owns the native watcher so its operating-system subscriptions live for the application lifetime.
pub(crate) struct CodexConfigWatcher {
    /// Live watcher handle retained to keep all operating-system subscriptions active.
    _watcher: Mutex<RecommendedWatcher>,
    /// Debounce worker that merges duplicate filesystem events before notifying consumers.
    _debouncer: EventDebouncer,
}

impl CodexConfigWatcher {
    /// Watches existing Codex configuration directories and filters their events to exact files.
    pub(crate) fn start(
        config_paths: Vec<PathBuf>,
        on_event: impl Fn(CodexConfigWatchEvent) + Send + Sync + 'static,
    ) -> notify::Result<Option<Self>> {
        let mut watched_directories = config_paths
            .iter()
            .filter_map(|path| path.parent().map(Path::to_path_buf))
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        watched_directories.sort_unstable();
        watched_directories.dedup();

        if watched_directories.is_empty() {
            return Ok(None);
        }

        let on_event = Arc::new(on_event);
        let debounced_on_event = Arc::clone(&on_event);
        let (debouncer, debounce_trigger) = EventDebouncer::start(
            CONFIG_CHANGE_QUIET_PERIOD,
            CONFIG_CHANGE_MAXIMUM_DELAY,
            move || debounced_on_event(CodexConfigWatchEvent::Changed),
        )
        .map_err(notify::Error::io)?;

        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| match result {
            Ok(event) if event_affects_config(&event, &config_paths) => {
                if debounce_trigger.signal_change().is_err() {
                    on_event(CodexConfigWatchEvent::Failed);
                }
            }
            Ok(_) => {}
            Err(_) => {
                // Runtime backend failures disable caching through the callback; periodic login
                // probes then continue resolving uncached defaults instead of serving stale data.
                on_event(CodexConfigWatchEvent::Failed);
            }
        })?;

        for directory in watched_directories {
            // Watching the parent directory preserves notifications when editors save by replacing
            // or renaming `config.toml` instead of modifying the existing inode in place.
            watcher.watch(&directory, RecursiveMode::NonRecursive)?;
        }

        Ok(Some(Self {
            _watcher: Mutex::new(watcher),
            _debouncer: debouncer,
        }))
    }
}

/// Returns the user configuration plus existing project configuration layers for the launch cwd.
pub(crate) fn codex_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".codex").join("config.toml"));
    }

    if let Ok(current_directory) = std::env::current_dir() {
        for ancestor in current_directory.ancestors() {
            let project_config = ancestor.join(".codex").join("config.toml");
            if project_config.parent().is_some_and(Path::is_dir) {
                paths.push(project_config);
            }
        }
    }

    paths.sort_unstable();
    paths.dedup();
    paths
}

fn event_affects_config(event: &Event, config_paths: &[PathBuf]) -> bool {
    let mutating_event = matches!(
        event.kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );

    mutating_event
        && event
            .paths
            .iter()
            .any(|changed_path| config_paths.iter().any(|path| path == changed_path))
}

#[cfg(test)]
mod tests {
    use super::event_affects_config;
    use notify::event::{AccessKind, AccessMode};
    use notify::{Event, EventKind};
    use std::path::PathBuf;

    #[test]
    fn accepts_mutations_for_an_exact_codex_config_path() {
        let config_path = PathBuf::from("/home/test/.codex/config.toml");
        let event = Event::new(EventKind::Any).add_path(config_path.clone());

        assert!(event_affects_config(&event, &[config_path]));
    }

    #[test]
    fn ignores_reads_and_unrelated_files_in_the_codex_directory() {
        let config_path = PathBuf::from("/home/test/.codex/config.toml");
        let read_event = Event::new(EventKind::Access(AccessKind::Close(AccessMode::Read)))
            .add_path(config_path.clone());
        let unrelated_event =
            Event::new(EventKind::Any).add_path(PathBuf::from("/home/test/.codex/auth.json"));

        assert!(!event_affects_config(
            &read_event,
            std::slice::from_ref(&config_path)
        ));
        assert!(!event_affects_config(&unrelated_event, &[config_path]));
    }
}
