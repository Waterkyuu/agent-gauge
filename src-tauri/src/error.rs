use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppError {
    CodexProbeFailed,
    WorkerFailed,
}

/// Stable and redacted error contract exposed across the Tauri IPC boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IpcError {
    /// Machine-readable category used by the frontend.
    pub(crate) code: &'static str,
    /// Safe user-facing explanation without local paths or process details.
    pub(crate) message: &'static str,
}

impl From<AppError> for IpcError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::CodexProbeFailed => Self {
                code: "CODEX_PROBE_FAILED",
                message: "无法检查本地 Codex 登录状态。",
            },
            AppError::WorkerFailed => Self {
                code: "WORKER_FAILED",
                message: "后台任务意外终止。",
            },
        }
    }
}
