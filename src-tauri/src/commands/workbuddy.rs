use crate::adapters::workbuddy::{
    read_workbuddy_config, workbuddy_local_storage_path, SystemWorkBuddyAdapter,
};
use crate::dto::agent::AgentRunResponse;
use crate::dto::workbuddy::{WorkBuddyConfigStatus, WorkBuddyLoginStatus};
use crate::error::{AppError, IpcError};
use crate::platform::workbuddy_config::{WorkBuddyConfigWatchEvent, WorkBuddyConfigWatcherState};
use crate::services::agent::run_agent_task;
use crate::services::workbuddy::check_workbuddy_login as check_workbuddy_login_service;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

const WORKBUDDY_CONFIG_CHANGED_EVENT: &str = "workbuddy-config-changed";

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
pub async fn check_workbuddy_config(
    app: tauri::AppHandle,
    watcher_state: tauri::State<'_, WorkBuddyConfigWatcherState>,
) -> Result<WorkBuddyConfigStatus, IpcError> {
    if let Some(local_storage_path) = workbuddy_local_storage_path() {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| IpcError::from(AppError::WorkerFailed))?;
        let previous_config = Arc::new(Mutex::new(None::<WorkBuddyConfigStatus>));
        watcher_state
            .start_if_available(local_storage_path, move |event| {
                if event == WorkBuddyConfigWatchEvent::Failed {
                    if let Ok(mut previous) = previous_config.lock() {
                        *previous = None;
                    }
                    return;
                }

                let Ok(config) = read_workbuddy_config().map(WorkBuddyConfigStatus::from) else {
                    return;
                };
                let Ok(mut previous) = previous_config.lock() else {
                    return;
                };
                if previous.as_ref() == Some(&config) {
                    return;
                }
                *previous = Some(config.clone());
                drop(previous);

                if window.emit(WORKBUDDY_CONFIG_CHANGED_EVENT, config).is_err() {
                    // The comparison page may be unmounted; a later snapshot command still reads
                    // the current configuration before it renders WorkBuddy as logged in.
                }
            })
            .map_err(|_| IpcError::from(AppError::WorkBuddyConfigReadFailed))?;
    }

    let config = tauri::async_runtime::spawn_blocking(read_workbuddy_config)
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(WorkBuddyConfigStatus::from)
        .map_err(IpcError::from)?;

    Ok(config)
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
