use crate::adapters::codex::{CodexRuntimeDefaultsCache, SystemCodexAdapter};
use crate::dto::agent::AgentRunResponse;
use crate::dto::codex::CodexLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::agent::run_agent_task;
use crate::services::codex::check_codex_login as resolve_codex_login;
use serde::Deserialize;

/// User input accepted by the Codex task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCodexTaskRequest {
    /// Natural-language task sent to the local Codex runtime.
    query: String,
}

/// Checks whether a locally installed Codex CLI currently has active credentials.
#[tauri::command]
pub async fn check_codex_login(
    runtime_defaults_cache: tauri::State<'_, CodexRuntimeDefaultsCache>,
) -> Result<CodexLoginStatus, IpcError> {
    let adapter = SystemCodexAdapter::new(runtime_defaults_cache.inner().clone());
    tauri::async_runtime::spawn_blocking(move || resolve_codex_login(&adapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}

/// Sends a bounded query to local Codex and waits for its streamed turn to finish.
#[tauri::command]
pub async fn run_codex_task(
    request: RunCodexTaskRequest,
    runtime_defaults_cache: tauri::State<'_, CodexRuntimeDefaultsCache>,
) -> Result<AgentRunResponse, IpcError> {
    let adapter = SystemCodexAdapter::new(runtime_defaults_cache.inner().clone());
    tauri::async_runtime::spawn_blocking(move || run_agent_task(&adapter, &request.query))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(Into::into)
        .map_err(Into::into)
}
