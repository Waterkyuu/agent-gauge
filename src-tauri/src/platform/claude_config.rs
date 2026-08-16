use crate::utils::debounce::EventDebouncer;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Native outcomes relevant to the Claude runtime-settings cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClaudeConfigWatchEvent {
    Changed,
    Failed,
}

/// Owns the native watcher for the user-level Claude settings file.
pub(crate) struct ClaudeConfigWatcher {
    /// Live watcher handle retained to keep the operating-system subscription active.
    _watcher: Mutex<RecommendedWatcher>,
    /// Debounce worker that merges duplicate filesystem events before notifying consumers.
    _debouncer: EventDebouncer,
}

impl ClaudeConfigWatcher {
    /// Watches the parent directory so atomic replacements of `settings.json` remain observable.
    pub(crate) fn start(
        settings_path: PathBuf,
        on_event: impl Fn(ClaudeConfigWatchEvent) + Send + Sync + 'static,
    ) -> notify::Result<Option<Self>> {
        let Some(settings_directory) = settings_path.parent().filter(|path| path.is_dir()) else {
            return Ok(None);
        };
        let settings_directory = settings_directory.to_path_buf();
        let on_event = Arc::new(on_event);
        let debounced_on_event = Arc::clone(&on_event);
        let (debouncer, debounce_trigger) = EventDebouncer::start(move || {
            debounced_on_event(ClaudeConfigWatchEvent::Changed);
        })
        .map_err(notify::Error::io)?;
        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| match result {
            Ok(event) if event_affects_config(&event, std::slice::from_ref(&settings_path)) => {
                if debounce_trigger.signal_change().is_err() {
                    on_event(ClaudeConfigWatchEvent::Failed);
                }
            }
            Ok(_) => {}
            Err(_) => on_event(ClaudeConfigWatchEvent::Failed),
        })?;

        watcher.watch(&settings_directory, RecursiveMode::NonRecursive)?;

        Ok(Some(Self {
            _watcher: Mutex::new(watcher),
            _debouncer: debouncer,
        }))
    }
}

/// Returns the single user-level settings file monitored for Claude defaults.
pub(crate) fn claude_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".claude").join("settings.json"))
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
    fn accepts_mutations_for_an_exact_claude_settings_path() {
        let settings_path = PathBuf::from("/home/test/.claude/settings.json");
        let event = Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
            .add_path(settings_path.clone());

        assert!(event_affects_config(&event, &[settings_path]));
    }

    #[test]
    fn ignores_reads_and_unrelated_claude_files() {
        let settings_path = PathBuf::from("/home/test/.claude/settings.json");
        let read_event = Event::new(EventKind::Access(AccessKind::Close(AccessMode::Read)))
            .add_path(settings_path.clone());
        let unrelated_event = Event::new(EventKind::Any)
            .add_path(PathBuf::from("/home/test/.claude/.credentials.json"));

        assert!(!event_affects_config(
            &read_event,
            std::slice::from_ref(&settings_path)
        ));
        assert!(!event_affects_config(&unrelated_event, &[settings_path]));
    }
}
