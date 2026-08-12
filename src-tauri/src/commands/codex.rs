use crate::adapters::codex::SystemCodexAdapter;
use crate::dto::codex::CodexLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::codex::check_codex_login as resolve_codex_login;

/// Checks whether a locally installed Codex CLI currently has active credentials.
#[tauri::command]
pub async fn check_codex_login() -> Result<CodexLoginStatus, IpcError> {
    tauri::async_runtime::spawn_blocking(|| resolve_codex_login(&SystemCodexAdapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}
