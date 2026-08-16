use crate::domain::comparison::{
    AgentKind, ComparisonDetail, ComparisonPage, ComparisonResultStatus,
};
use serde::{Deserialize, Serialize};

/// Stable Agent identifier accepted and returned over IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AgentKindRequest {
    Codex,
    Claude,
    WorkBuddy,
}

impl From<AgentKindRequest> for AgentKind {
    fn from(value: AgentKindRequest) -> Self {
        match value {
            AgentKindRequest::Codex => Self::Codex,
            AgentKindRequest::Claude => Self::Claude,
            AgentKindRequest::WorkBuddy => Self::WorkBuddy,
        }
    }
}

impl From<AgentKind> for AgentKindRequest {
    fn from(value: AgentKind) -> Self {
        match value {
            AgentKind::Codex => Self::Codex,
            AgentKind::Claude => Self::Claude,
            AgentKind::WorkBuddy => Self::WorkBuddy,
        }
    }
}

/// Token counters received from one successful comparison result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenUsageRequest {
    /// Total tokens reported by the Agent.
    pub(crate) total_tokens: u64,
    /// Tokens included in model input.
    pub(crate) input_tokens: u64,
    /// Input tokens served from cache.
    pub(crate) cached_input_tokens: u64,
    /// Input tokens written into cache.
    pub(crate) cache_write_input_tokens: u64,
    /// Tokens included in model output.
    pub(crate) output_tokens: u64,
    /// Reasoning tokens when reported separately.
    pub(crate) reasoning_output_tokens: Option<u64>,
}

/// One tool invocation received from a successful comparison result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolCallRequest {
    /// One-based invocation order.
    pub(crate) sequence: u64,
    /// Stable tool name reported by the Agent protocol.
    pub(crate) name: String,
    /// Wall-clock invocation duration in milliseconds.
    pub(crate) duration_ms: u64,
}

/// Completed metrics and response received from one Agent.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentRunResultRequest {
    /// Final response assembled from the Agent stream.
    pub(crate) response: String,
    /// Complete task duration in milliseconds.
    pub(crate) total_duration_ms: u64,
    /// Delay until the first assistant text in milliseconds.
    pub(crate) time_to_first_token_ms: Option<u64>,
    /// Token counters when reported by the Agent.
    pub(crate) token_usage: Option<TokenUsageRequest>,
    /// Sum of explicit thinking intervals in milliseconds.
    pub(crate) thinking_duration_ms: u64,
    /// Number of tools invoked during this task.
    pub(crate) tool_call_count: u64,
    /// Tool calls retained in source order.
    pub(crate) tool_calls: Vec<ToolCallRequest>,
}

/// Success payload or safe failure detail for one Agent.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum ComparisonOutcomeRequest {
    Succeeded {
        /// Successful response and measured metrics.
        result: AgentRunResultRequest,
    },
    Failed {
        /// Safe localized failure shown in history.
        error_message: String,
    },
}

/// One final Agent outcome submitted for persistence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonResultRequest {
    /// Agent product that produced this outcome.
    pub(crate) agent: AgentKindRequest,
    /// Model configuration captured before execution.
    pub(crate) model: Option<String>,
    /// Reasoning configuration captured before execution.
    pub(crate) reasoning_effort: Option<String>,
    /// Success metrics or safe failure detail.
    #[serde(flatten)]
    pub(crate) outcome: ComparisonOutcomeRequest,
}

/// Complete comparison submitted after all selected Agents finish.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveComparisonRequest {
    /// Shared task sent to every selected Agent.
    pub(crate) query: String,
    /// Final outcome for every selected Agent.
    pub(crate) results: Vec<ComparisonResultRequest>,
}

/// Identifier returned after one comparison commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveComparisonResponse {
    /// Newly persisted comparison identifier.
    pub(crate) id: i64,
}

/// Keyset cursor accepted by the history list command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonCursorDto {
    /// UTC timestamp of the final item in the prior page.
    pub(crate) created_at_ms: i64,
    /// Primary key of the final item in the prior page.
    pub(crate) id: i64,
}

/// Bounded history list request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListComparisonsRequest {
    /// Cursor returned by the prior page.
    pub(crate) cursor: Option<ComparisonCursorDto>,
    /// Requested page size, capped by the service.
    pub(crate) limit: Option<u64>,
}

/// Primary-key request for one history detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetComparisonRequest {
    /// Persistent comparison identifier.
    pub(crate) id: i64,
}

/// Lightweight Agent outcome shown in a history row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonAgentSummaryResponse {
    /// Agent represented by this summary.
    pub(crate) agent: AgentKindRequest,
    /// Success or failure state.
    pub(crate) status: &'static str,
}

/// Lightweight comparison returned by the history list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonSummaryResponse {
    /// Persistent comparison identifier.
    pub(crate) id: i64,
    /// Shared task text.
    pub(crate) query: String,
    /// Aggregate completion state.
    pub(crate) status: &'static str,
    /// Metric calculation contract version.
    pub(crate) metric_version: i64,
    /// UTC Unix timestamp in milliseconds.
    pub(crate) created_at_ms: i64,
    /// Selected Agent outcomes without response bodies.
    pub(crate) agents: Vec<ComparisonAgentSummaryResponse>,
}

/// One page of history summaries and its continuation cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListComparisonsResponse {
    /// Summaries ordered newest first.
    pub(crate) items: Vec<ComparisonSummaryResponse>,
    /// Cursor for the next page.
    pub(crate) next_cursor: Option<ComparisonCursorDto>,
}

/// Full Agent result returned by the detail command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonResultResponse {
    /// Agent product represented by this result.
    pub(crate) agent: AgentKindRequest,
    /// Model configuration captured at execution time.
    pub(crate) model: Option<String>,
    /// Reasoning configuration captured at execution time.
    pub(crate) reasoning_effort: Option<String>,
    /// Success or failure state.
    pub(crate) status: &'static str,
    /// Successful response and metrics.
    pub(crate) result: Option<AgentRunResultRequest>,
    /// Safe failure message.
    pub(crate) error_message: Option<String>,
}

/// Complete historical comparison returned for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComparisonDetailResponse {
    /// Persistent comparison identifier.
    pub(crate) id: i64,
    /// Shared task text.
    pub(crate) query: String,
    /// Aggregate completion state.
    pub(crate) status: &'static str,
    /// Metric calculation contract version.
    pub(crate) metric_version: i64,
    /// UTC Unix timestamp in milliseconds.
    pub(crate) created_at_ms: i64,
    /// Complete outcomes for every selected Agent.
    pub(crate) results: Vec<ComparisonResultResponse>,
}

impl From<ComparisonPage> for ListComparisonsResponse {
    fn from(page: ComparisonPage) -> Self {
        Self {
            items: page
                .items
                .into_iter()
                .map(|item| ComparisonSummaryResponse {
                    id: item.id,
                    query: item.query,
                    status: item.status.as_str(),
                    metric_version: item.metric_version,
                    created_at_ms: item.created_at_ms,
                    agents: item
                        .agents
                        .into_iter()
                        .map(|agent| ComparisonAgentSummaryResponse {
                            agent: agent.agent.into(),
                            status: agent.status.as_str(),
                        })
                        .collect(),
                })
                .collect(),
            next_cursor: page.next_cursor.map(|cursor| ComparisonCursorDto {
                created_at_ms: cursor.created_at_ms,
                id: cursor.id,
            }),
        }
    }
}

impl From<ComparisonDetail> for ComparisonDetailResponse {
    fn from(detail: ComparisonDetail) -> Self {
        Self {
            id: detail.id,
            query: detail.query,
            status: detail.status.as_str(),
            metric_version: detail.metric_version,
            created_at_ms: detail.created_at_ms,
            results: detail
                .results
                .into_iter()
                .map(|item| {
                    let tool_call_count = item.tool_calls.len() as u64;
                    let result = (item.status == ComparisonResultStatus::Succeeded).then(|| {
                        AgentRunResultRequest {
                            response: item.response.unwrap_or_default(),
                            total_duration_ms: item.total_duration_ms.unwrap_or_default() as u64,
                            time_to_first_token_ms: item
                                .time_to_first_token_ms
                                .map(|value| value as u64),
                            token_usage: item.token_usage.map(|usage| TokenUsageRequest {
                                total_tokens: usage.total_tokens as u64,
                                input_tokens: usage.input_tokens as u64,
                                cached_input_tokens: usage.cached_input_tokens as u64,
                                cache_write_input_tokens: usage.cache_write_input_tokens as u64,
                                output_tokens: usage.output_tokens as u64,
                                reasoning_output_tokens: usage
                                    .reasoning_output_tokens
                                    .map(|value| value as u64),
                            }),
                            thinking_duration_ms: item.thinking_duration_ms.unwrap_or_default()
                                as u64,
                            tool_call_count,
                            tool_calls: item
                                .tool_calls
                                .into_iter()
                                .map(|tool| ToolCallRequest {
                                    sequence: tool.sequence as u64,
                                    name: tool.name,
                                    duration_ms: tool.duration_ms as u64,
                                })
                                .collect(),
                        }
                    });
                    ComparisonResultResponse {
                        agent: item.agent.into(),
                        model: item.model,
                        reasoning_effort: item.reasoning_effort,
                        status: item.status.as_str(),
                        result,
                        error_message: item.error_message,
                    }
                })
                .collect(),
        }
    }
}
