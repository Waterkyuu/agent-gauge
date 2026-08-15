use crate::adapters::process::{AgentProcessStates, SystemAgentProcessAdapter};
use crate::error::{AppError, IpcError};
use crate::services::process::check_agent_processes as resolve_agent_processes;
use serde::Serialize;

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

/// Reads a lightweight process snapshot without rerunning authentication probes.
#[tauri::command]
pub async fn check_agent_processes() -> Result<AgentProcessStatesResponse, IpcError> {
    tauri::async_runtime::spawn_blocking(|| resolve_agent_processes(&SystemAgentProcessAdapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(Into::into)
        .map_err(Into::into)
}
