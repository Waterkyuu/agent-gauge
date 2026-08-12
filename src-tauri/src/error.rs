use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppError {
    CodexProbeFailed,
    CodexProtocolFailed,
    CodexTaskFailed,
    CodexTimedOut,
    InvalidQuery,
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
            AppError::CodexProtocolFailed => Self {
                code: "CODEX_PROTOCOL_FAILED",
                message: "无法读取本地 Codex 事件流。",
            },
            AppError::CodexTaskFailed => Self {
                code: "CODEX_TASK_FAILED",
                message: "Codex 未能完成任务。",
            },
            AppError::CodexTimedOut => Self {
                code: "CODEX_TIMED_OUT",
                message: "等待 Codex 完成任务超时。",
            },
            AppError::InvalidQuery => Self {
                code: "INVALID_QUERY",
                message: "任务内容不能为空且不能超过 16000 个字符。",
            },
            AppError::WorkerFailed => Self {
                code: "WORKER_FAILED",
                message: "后台任务意外终止。",
            },
        }
    }
}
