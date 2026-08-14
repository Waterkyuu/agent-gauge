use crate::adapters::claude::{ClaudeRuntimeSettingsCache, SystemClaudeAdapter};
use crate::domain::codex_run::{AgentRunMetrics, AgentRunOutput, TokenUsage};
use crate::dto::claude::ClaudeLoginStatus;
use crate::error::{AppError, IpcError};
use crate::services::agent::run_agent_task;
use crate::services::claude::check_claude_login as resolve_claude_login;
use serde::{Deserialize, Serialize};

/// User input accepted by the Claude Code task command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunClaudeTaskRequest {
    /// Natural-language task sent to the local Claude Code runtime.
    query: String,
}

/// Token consumption reported for a completed Claude Code task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeTokenUsageResponse {
    total_tokens: u64,
    input_tokens: u64,
    cached_input_tokens: u64,
    cache_write_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: Option<u64>,
}

/// Completed response and latency metrics for one local Claude Code task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunClaudeTaskResponse {
    response: String,
    total_duration_ms: u64,
    time_to_first_token_ms: Option<u64>,
    token_usage: Option<ClaudeTokenUsageResponse>,
}

impl From<TokenUsage> for ClaudeTokenUsageResponse {
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

impl From<AgentRunOutput> for RunClaudeTaskResponse {
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
) -> Result<RunClaudeTaskResponse, IpcError> {
    let adapter = SystemClaudeAdapter::new(runtime_settings_cache.inner().clone());
    tauri::async_runtime::spawn_blocking(move || run_agent_task(&adapter, &request.query))
        .await
        .map_err(|_| IpcError::from(AppError::WorkerFailed))?
        .map(Into::into)
        .map_err(Into::into)
}
