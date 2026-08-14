use crate::domain::codex_run::{AgentRunMetrics, AgentRunOutput, TokenUsage, ToolCallMetric};
use serde::Serialize;

/// Token consumption reported for one completed Agent task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenUsageResponse {
    /// Total tokens reported by the source Agent.
    total_tokens: u64,
    /// Tokens included in model input.
    input_tokens: u64,
    /// Input tokens served from cache.
    cached_input_tokens: u64,
    /// Input tokens written into cache.
    cache_write_input_tokens: u64,
    /// Tokens included in model output.
    output_tokens: u64,
    /// Output tokens consumed by reasoning when reported.
    reasoning_output_tokens: Option<u64>,
}

/// One tool invocation recorded during an Agent task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolCallMetricResponse {
    /// Stable tool name supplied by the source protocol.
    name: String,
    /// Wall-clock execution duration in milliseconds.
    duration_ms: u64,
}

/// Normalized result shared by every supported local Agent runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentRunResponse {
    /// Final assistant response assembled from the source stream.
    response: String,
    /// Full task wall-clock duration in milliseconds.
    total_duration_ms: u64,
    /// Delay until the first assistant text delta in milliseconds.
    time_to_first_token_ms: Option<u64>,
    /// Token usage when the source runtime reports it.
    token_usage: Option<TokenUsageResponse>,
    /// Sum of explicit thinking intervals in milliseconds.
    thinking_duration_ms: u64,
    /// Number of tool invocations recorded for the task.
    tool_call_count: usize,
    /// Tool invocations retained in source start order.
    tool_calls: Vec<ToolCallMetricResponse>,
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

impl From<ToolCallMetric> for ToolCallMetricResponse {
    fn from(tool_call: ToolCallMetric) -> Self {
        Self {
            name: tool_call.name,
            duration_ms: duration_millis(tool_call.duration),
        }
    }
}

impl From<AgentRunOutput> for AgentRunResponse {
    fn from(output: AgentRunOutput) -> Self {
        let AgentRunMetrics {
            total_duration,
            time_to_first_token,
            token_usage,
            thinking_duration,
            tool_calls,
        } = output.metrics;

        Self {
            response: output.response,
            total_duration_ms: duration_millis(total_duration),
            time_to_first_token_ms: time_to_first_token.map(duration_millis),
            token_usage: token_usage.map(Into::into),
            thinking_duration_ms: duration_millis(thinking_duration),
            tool_call_count: tool_calls.len(),
            tool_calls: tool_calls.into_iter().map(Into::into).collect(),
        }
    }
}

/// Converts a duration to the bounded millisecond representation used over IPC.
fn duration_millis(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::AgentRunResponse;
    use crate::domain::codex_run::{AgentRunMetrics, AgentRunOutput, ToolCallMetric};
    use std::time::Duration;

    #[test]
    fn converts_execution_metrics_into_the_shared_agent_response() {
        let response = AgentRunResponse::from(AgentRunOutput {
            response: "done".to_string(),
            metrics: AgentRunMetrics {
                total_duration: Duration::from_secs(5),
                time_to_first_token: Some(Duration::from_millis(400)),
                token_usage: None,
                thinking_duration: Duration::from_millis(900),
                tool_calls: vec![ToolCallMetric {
                    name: "Read".to_string(),
                    duration: Duration::from_millis(250),
                }],
            },
        });

        assert_eq!(response.thinking_duration_ms, 900);
        assert_eq!(response.tool_call_count, 1);
        assert_eq!(response.tool_calls[0].name, "Read");
        assert_eq!(response.tool_calls[0].duration_ms, 250);
    }
}
