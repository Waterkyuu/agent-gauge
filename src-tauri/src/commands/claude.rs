use crate::adapters::claude::{ClaudeRuntimeSettingsCache, SystemClaudeAdapter};
use crate::dto::agent::AgentRunResponse;
use crate::dto::claude::ClaudeLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::agent::run_agent_task;
use crate::services::claude::check_claude_login as resolve_claude_login;
use serde::Deserialize;

/// User input accepted by the Claude Code task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunClaudeTaskRequest {
    /// Natural-language task sent to the local Claude Code runtime.
    query: String,
}

/// Checks whether the locally installed Claude Code runtime has active credentials.
#[tauri::command]
pub async fn check_claude_login(
    runtime_settings_cache: tauri::State<'_, ClaudeRuntimeSettingsCache>,
) -> Result<ClaudeLoginStatus, IpcError> {
    let adapter = SystemClaudeAdapter::new(runtime_settings_cache.inner().clone());
    tauri::async_runtime::spawn_blocking(move || resolve_claude_login(&adapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}

/// Sends a bounded query to Claude Code and waits for the streamed task result.
#[tauri::command]
pub async fn run_claude_task(
    request: RunClaudeTaskRequest,
    runtime_settings_cache: tauri::State<'_, ClaudeRuntimeSettingsCache>,
) -> Result<AgentRunResponse, IpcError> {
    let adapter = SystemClaudeAdapter::new(runtime_settings_cache.inner().clone());
    tauri::async_runtime::spawn_blocking(move || run_agent_task(&adapter, &request.query))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(Into::into)
        .map_err(Into::into)
}
