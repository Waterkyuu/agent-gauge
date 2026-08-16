use crate::utils::debounce::EventDebouncer;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Native outcomes relevant to the WorkBuddy LevelDB configuration snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkBuddyConfigWatchEvent {
    Changed,
    Failed,
}

/// Owns the LevelDB directory watcher and its debounce worker for the application lifetime.
pub(crate) struct WorkBuddyConfigWatcher {
    /// Live watcher handle retained to keep the operating-system subscription active.
    _watcher: Mutex<RecommendedWatcher>,
    /// Debounce worker that merges LevelDB's burst of physical file mutations.
    _debouncer: EventDebouncer,
}

impl WorkBuddyConfigWatcher {
    /// Watches one existing WorkBuddy LevelDB directory for data-file mutations.
    pub(crate) fn start(
        local_storage_path: PathBuf,
        on_event: impl Fn(WorkBuddyConfigWatchEvent) + Send + Sync + 'static,
    ) -> notify::Result<Option<Self>> {
        if !fs::symlink_metadata(&local_storage_path).is_ok_and(|metadata| metadata.is_dir()) {
            return Ok(None);
        }

        let on_event = Arc::new(on_event);
        let debounced_on_event = Arc::clone(&on_event);
        let (debouncer, debounce_trigger) = EventDebouncer::start(move || {
            debounced_on_event(WorkBuddyConfigWatchEvent::Changed);
        })
        .map_err(notify::Error::io)?;
        let watched_path = local_storage_path.clone();
        let mut watcher = recommended_watcher(move |result: notify::Result<Event>| match result {
            Ok(event) if event_affects_local_storage(&event, &watched_path) => {
                if debounce_trigger.signal_change().is_err() {
                    on_event(WorkBuddyConfigWatchEvent::Failed);
                }
            }
            Ok(_) => {}
            Err(_) => on_event(WorkBuddyConfigWatchEvent::Failed),
        })?;

        watcher.watch(&local_storage_path, RecursiveMode::NonRecursive)?;

        Ok(Some(Self {
            _watcher: Mutex::new(watcher),
            _debouncer: debouncer,
        }))
    }
}

/// Returns whether a native event mutated a LevelDB data file inside the watched directory.
fn event_affects_local_storage(event: &Event, local_storage_path: &Path) -> bool {
    let mutating_event = matches!(
        event.kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    );

    mutating_event
        && event.paths.iter().any(|changed_path| {
            changed_path.parent() == Some(local_storage_path)
                && changed_path
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|extension| {
                        extension.eq_ignore_ascii_case("log")
                            || extension.eq_ignore_ascii_case("ldb")
                            || extension.eq_ignore_ascii_case("sst")
                    })
        })
}

#[cfg(test)]
mod tests {
    use super::event_affects_local_storage;
    use notify::event::{AccessKind, AccessMode, ModifyKind};
    use notify::{Event, EventKind};
    use std::path::PathBuf;

    #[test]
    fn accepts_leveldb_data_file_mutations() {
        let storage_path = PathBuf::from("/home/test/Local Storage/leveldb");

        for file_name in ["000004.log", "000005.ldb", "000006.sst"] {
            let event = Event::new(EventKind::Modify(ModifyKind::Any))
                .add_path(storage_path.join(file_name));

            assert!(event_affects_local_storage(&event, &storage_path));
        }
    }

    #[test]
    fn ignores_reads_metadata_and_files_outside_the_leveldb_directory() {
        let storage_path = PathBuf::from("/home/test/Local Storage/leveldb");
        let read_event = Event::new(EventKind::Access(AccessKind::Close(AccessMode::Read)))
            .add_path(storage_path.join("000004.log"));
        let metadata_event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(storage_path.join("CURRENT"));
        let outside_event = Event::new(EventKind::Modify(ModifyKind::Any))
            .add_path(PathBuf::from("/home/test/other/000004.log"));

        assert!(!event_affects_local_storage(&read_event, &storage_path));
        assert!(!event_affects_local_storage(&metadata_event, &storage_path));
        assert!(!event_affects_local_storage(&outside_event, &storage_path));
    }
}
