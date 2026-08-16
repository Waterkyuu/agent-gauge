use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Refreshes only executable metadata and returns one name or path for each visible process.
///
/// The caller retains `System` between calls so a one-second monitor reuses native process state
/// instead of creating a new `ps` or `tasklist` child process for every snapshot.
pub(crate) fn running_process_names(system: &mut System) -> Vec<String> {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::OnlyIfNotSet)
            .without_tasks(),
    );

    system
        .processes()
        .values()
        .map(|process| {
            process
                .exe()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| process.name().to_string_lossy().into_owned())
        })
        .collect()
}
