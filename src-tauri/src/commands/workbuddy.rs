use crate::adapters::workbuddy::{read_workbuddy_config, SystemWorkBuddyAdapter};
use crate::dto::agent::AgentRunResponse;
use crate::dto::workbuddy::{WorkBuddyConfigStatus, WorkBuddyLoginStatus};
use crate::error::{AppError, IpcError};
use crate::services::agent::run_agent_task;
use crate::services::workbuddy::check_workbuddy_login as check_workbuddy_login_service;
use serde::Deserialize;

/// User input accepted by the WorkBuddy task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkBuddyTaskRequest {
    /// Natural-language task sent to the local WorkBuddy runtime.
    query: String,
}

/// Checks whether the locally installed WorkBuddy runtime has an active account.
#[tauri::command]
pub async fn check_workbuddy_login() -> Result<WorkBuddyLoginStatus, IpcError> {
    tauri::async_runtime::spawn_blocking(|| check_workbuddy_login_service(&SystemWorkBuddyAdapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}

/// Reads the current WorkBuddy model configuration without starting its ACP runtime.
#[tauri::command]
pub async fn check_workbuddy_config() -> Result<WorkBuddyConfigStatus, IpcError> {
    tauri::async_runtime::spawn_blocking(read_workbuddy_config)
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(Into::into)
        .map_err(Into::into)
}

/// Sends a bounded query to WorkBuddy and waits for the streamed task result.
#[tauri::command]
pub async fn run_workbuddy_task(
    request: RunWorkBuddyTaskRequest,
) -> Result<AgentRunResponse, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        run_agent_task(&SystemWorkBuddyAdapter, &request.query)
    })
    .await
    .map_err(|_| IpcError::from(AppError::WorkerFailed))?
    .map(Into::into)
    .map_err(Into::into)
}
