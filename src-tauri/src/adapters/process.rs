use crate::error::AppError;
use crate::platform::process::running_process_names;
use std::path::Path;
use sysinfo::System;

/// One point-in-time snapshot of whether each supported Agent has a matching local process.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AgentProcessStates {
    /// Indicates whether an exact Claude Code executable match was observed.
    pub(crate) claude: bool,
    /// Indicates whether an exact Codex executable match was observed.
    pub(crate) codex: bool,
    /// Indicates whether an exact WorkBuddy executable or application match was observed.
    pub(crate) workbuddy: bool,
}

/// Abstracts operating-system process discovery for the service and its tests.
pub(crate) trait AgentProcessAdapter {
    fn check_processes(&mut self) -> Result<AgentProcessStates, AppError>;
}

pub(crate) struct SystemAgentProcessAdapter {
    /// Native process table retained and refreshed by the application-wide monitor.
    system: System,
}

impl Default for SystemAgentProcessAdapter {
    fn default() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl AgentProcessAdapter for SystemAgentProcessAdapter {
    fn check_processes(&mut self) -> Result<AgentProcessStates, AppError> {
        Ok(process_states_from_names(running_process_names(
            &mut self.system,
        )))
    }
}

/// Maps platform process observations to the shared Agent running-state snapshot.
///
/// Matching intentionally uses exact executable basenames so helper renderers such as
/// `Codex Helper` do not make an idle Agent appear active.
fn process_states_from_names<I, S>(process_names: I) -> AgentProcessStates
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut states = AgentProcessStates::default();

    for process_name in process_names {
        let lowercase_path = process_name
            .as_ref()
            .trim()
            .replace('\\', "/")
            .to_ascii_lowercase();
        // WorkBuddy's macOS bundle keeps the generic Electron executable name, so its containing
        // application path is the stable identity available from the native process table.
        let is_workbuddy_desktop =
            lowercase_path.ends_with("/workbuddy ai.app/contents/macos/electron");
        let executable_name = Path::new(&lowercase_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let executable_name = executable_name
            // Windows `tasklist` reports image names with the `.exe` suffix.
            .strip_suffix(".exe")
            .unwrap_or(executable_name);

        if is_workbuddy_desktop {
            states.workbuddy = true;
            continue;
        }

        match executable_name {
            "claude" => states.claude = true,
            "codex" => states.codex = true,
            "cbc" | "codebuddy" | "workbuddy" | "workbuddy ai" => states.workbuddy = true,
            _ => {}
        }
    }

    states
}

#[cfg(test)]
mod tests {
    use super::process_states_from_names;

    #[test]
    fn detects_supported_agents_from_executable_names() {
        let states = process_states_from_names([
            "/usr/local/bin/claude",
            "/Applications/Codex.app/Contents/MacOS/Codex",
            "WorkBuddy AI.exe",
        ]);

        assert!(states.claude);
        assert!(states.codex);
        assert!(states.workbuddy);
    }

    #[test]
    fn detects_the_workbuddy_desktop_electron_process() {
        let states =
            process_states_from_names(["/Applications/WorkBuddy AI.app/Contents/MacOS/Electron"]);

        assert!(states.workbuddy);
    }

    #[test]
    fn ignores_helpers_and_unrelated_processes() {
        let states = process_states_from_names([
            "Claude Helper",
            "Codex Helper (Renderer)",
            "node",
            "agent-gauge",
        ]);

        assert!(!states.claude);
        assert!(!states.codex);
        assert!(!states.workbuddy);
    }
}
