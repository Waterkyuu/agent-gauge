use crate::error::AppError;
use crate::platform::process::running_process_names;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AgentProcessStates {
    pub(crate) claude: bool,
    pub(crate) codex: bool,
    pub(crate) workbuddy: bool,
}

pub(crate) trait AgentProcessAdapter {
    fn check_processes(&self) -> Result<AgentProcessStates, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemAgentProcessAdapter;

impl AgentProcessAdapter for SystemAgentProcessAdapter {
    fn check_processes(&self) -> Result<AgentProcessStates, AppError> {
        running_process_names().map(process_states_from_names)
    }
}

fn process_states_from_names<I, S>(process_names: I) -> AgentProcessStates
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut states = AgentProcessStates::default();

    for process_name in process_names {
        let normalized_path = process_name
            .as_ref()
            .trim()
            .replace('\\', "/")
            .to_ascii_lowercase();
        let is_workbuddy_desktop =
            normalized_path.ends_with("/workbuddy ai.app/contents/macos/electron");
        let executable_name = Path::new(&normalized_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let executable_name = executable_name
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
