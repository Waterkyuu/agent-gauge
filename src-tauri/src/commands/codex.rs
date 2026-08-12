use crate::adapters::codex::SystemCodexAdapter;
use crate::domain::codex_run::{AgentRunMetrics, AgentRunOutput, TokenUsage};
use crate::dto::codex::CodexLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::codex::{
    check_codex_login as resolve_codex_login, run_codex_task as execute_codex_task,
};
use serde::{Deserialize, Serialize};

/// User input accepted by the Codex task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCodexTaskRequest {
    /// Natural-language task sent to the local Codex runtime.
    query: String,
}

/// Token consumption reported for the completed Codex turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageResponse {
    /// All input, cached input, output, and reasoning tokens reported by Codex.
    total_tokens: u64,
    /// Tokens included in the model input.
    input_tokens: u64,
    /// Input tokens served from cache.
    cached_input_tokens: u64,
    /// Input tokens written into cache.
    cache_write_input_tokens: u64,
    /// Tokens included in the model output.
    output_tokens: u64,
    /// Output tokens consumed by model reasoning.
    reasoning_output_tokens: Option<u64>,
}

/// Completed response and latency metrics for one local Codex turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCodexTaskResponse {
    /// Incrementally assembled assistant response.
    response: String,
    /// Milliseconds from sending the turn request until completion.
    total_duration_ms: u64,
    /// Milliseconds from sending the turn request until the first non-empty assistant delta.
    time_to_first_token_ms: Option<u64>,
    /// Token usage for this turn when supplied by Codex.
    token_usage: Option<TokenUsageResponse>,
}

impl From<TokenUsage> for TokenUsageResponse {
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

impl From<AgentRunOutput> for RunCodexTaskResponse {
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

/// Checks whether a locally installed Codex CLI currently has active credentials.
#[tauri::command]
pub async fn check_codex_login() -> Result<CodexLoginStatus, IpcError> {
    tauri::async_runtime::spawn_blocking(|| resolve_codex_login(&SystemCodexAdapter))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map_err(Into::into)
}

/// Sends a bounded query to local Codex and waits for its streamed turn to finish.
#[tauri::command]
pub async fn run_codex_task(
    request: RunCodexTaskRequest,
) -> Result<RunCodexTaskResponse, IpcError> {
    tauri::async_runtime::spawn_blocking(move || {
        execute_codex_task(&SystemCodexAdapter, &request.query)
    })
    .await
    .map_err(|_| IpcError::from(AppError::WorkerFailed))?
    .map(Into::into)
    .map_err(Into::into)
}
