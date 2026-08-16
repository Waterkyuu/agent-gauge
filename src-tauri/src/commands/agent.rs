use crate::adapters::process::AgentProcessStates;
use crate::error::IpcError;
use crate::services::process::AgentProcessMonitor;
use serde::Serialize;
use tauri::State;

/// Running-state snapshot for every supported local Agent product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProcessStatesResponse {
    /// Indicates whether a Claude Code process is currently running.
    claude: bool,
    /// Indicates whether a Codex process is currently running.
    codex: bool,
    /// Indicates whether a WorkBuddy process is currently running.
    workbuddy: bool,
}

impl From<AgentProcessStates> for AgentProcessStatesResponse {
    fn from(states: AgentProcessStates) -> Self {
        Self {
            claude: states.claude,
            codex: states.codex,
            workbuddy: states.workbuddy,
        }
    }
}

/// Reads the latest process snapshot retained by the application-wide monitor.
#[tauri::command]
pub fn check_agent_processes(
    monitor: State<'_, AgentProcessMonitor>,
) -> Result<AgentProcessStatesResponse, IpcError> {
    monitor.current_states().map(Into::into).map_err(Into::into)
}
