use crate::adapters::workbuddy::SystemWorkBuddyAdapter;
use crate::domain::codex_run::{AgentRunMetrics, AgentRunOutput, TokenUsage};
use crate::dto::workbuddy::WorkBuddyLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::agent::run_agent_task;
use crate::services::workbuddy::check_workbuddy_login as resolve_workbuddy_login;
use serde::{Deserialize, Serialize};

/// User input accepted by the WorkBuddy task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkBuddyTaskRequest {
    /// Natural-language task sent to the local WorkBuddy runtime.
    query: String,
}

/// Token consumption reported for a completed WorkBuddy task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyTokenUsageResponse {
    total_tokens: u64,
    input_tokens: u64,
    cached_input_tokens: u64,
    cache_write_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: Option<u64>,
}

/// Completed response and latency metrics for one local WorkBuddy task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWorkBuddyTaskResponse {
    response: String,
    total_duration_ms: u64,
    time_to_first_token_ms: Option<u64>,
    token_usage: Option<WorkBuddyTokenUsageResponse>,
}

impl From<TokenUsage> for WorkBuddyTokenUsageResponse {
    fn from(usage: TokenUsage) -> Self {
        Self {
            total_tokens: usage.total_tokens,
            input_tokens: usage.input_tokens,
            cached_input_tokens: usage.cached_input_tokens,
            cache_write_input_tokens: usage.cache_write_input_tokens,
            output_tokens: usage.output_tokens,
            reasoning_output_tokens: usage.reasoning_output_tokens,
        }
    }
}

impl From<AgentRunOutput> for RunWorkBuddyTaskResponse {
    fn from(output: AgentRunOutput) -> Self {
        let AgentRunMetrics {
            total_duration,
            time_to_first_token,
            token_usage,
        } = output.metrics;

        Self {
            response: output.response,
            total_duration_ms: duration_millis(total_duration),
            time_to_first_token_ms: time_to_first_token.map(duration_millis),
            token_usage: token_usage.map(Into::into),
        }
    }
}

fn duration_millis(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Checks whether the locally installed WorkBuddy runtime has an active account.
#[tauri::command]
pub async fn check_workbuddy_login() -> Result<WorkBuddyLoginStatus, IpcError> {
    tauri::async_runtime::spawn_blocking(|| resolve_workbuddy_login(&SystemWorkBuddyAdapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}

/// Sends a bounded query to WorkBuddy and waits for the streamed task result.
#[tauri::command]
pub async fn run_workbuddy_task(
    request: RunWorkBuddyTaskRequest,
) -> Result<RunWorkBuddyTaskResponse, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        run_agent_task(&SystemWorkBuddyAdapter, &request.query)
    })
    .await
    .map_err(|_| IpcError::from(AppError::WorkerFailed))?
    .map(Into::into)
    .map_err(Into::into)
}
