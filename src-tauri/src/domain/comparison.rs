/// Supported local Agent products that can participate in one comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AgentKind {
    Codex,
    Claude,
    WorkBuddy,
}

impl AgentKind {
    /// Returns the stable database and IPC identifier for this Agent.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::WorkBuddy => "workbuddy",
        }
    }

    /// Parses a trusted persisted Agent identifier.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "workbuddy" => Some(Self::WorkBuddy),
            _ => None,
        }
    }
}

/// Aggregate completion state for one comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComparisonStatus {
    Completed,
    Partial,
    Failed,
}

impl ComparisonStatus {
    /// Returns the stable value stored in SQLite.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Failed => "failed",
        }
    }

    /// Parses a status protected by the database check constraint.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "completed" => Some(Self::Completed),
            "partial" => Some(Self::Partial),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Completion state for one Agent inside a comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComparisonResultStatus {
    Succeeded,
    Failed,
}

impl ComparisonResultStatus {
    /// Returns the stable value stored in SQLite.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    /// Parses a result status protected by the database check constraint.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Token counters persisted for one successful Agent result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewTokenUsage {
    /// Total input and output tokens reported by the Agent.
    pub(crate) total_tokens: i64,
    /// Tokens included in model input.
    pub(crate) input_tokens: i64,
    /// Input tokens served from cache.
    pub(crate) cached_input_tokens: i64,
    /// Input tokens written into cache.
    pub(crate) cache_write_input_tokens: i64,
    /// Tokens included in model output.
    pub(crate) output_tokens: i64,
    /// Reasoning tokens when reported separately.
    pub(crate) reasoning_output_tokens: Option<i64>,
}

/// One ordered tool invocation prepared for persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewToolCall {
    /// One-based invocation order within the Agent run.
    pub(crate) sequence: i64,
    /// Stable tool name reported by the Agent protocol.
    pub(crate) name: String,
    /// Wall-clock invocation duration in milliseconds.
    pub(crate) duration_ms: i64,
}

/// Final outcome prepared for one Agent result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NewComparisonOutcome {
    Succeeded {
        /// Final response assembled from the Agent stream.
        response: String,
        /// Complete task duration in milliseconds.
        total_duration_ms: i64,
        /// Delay until the first assistant text in milliseconds.
        time_to_first_token_ms: Option<i64>,
        /// Sum of explicit thinking intervals in milliseconds.
        thinking_duration_ms: i64,
        /// Token counters when reported by the Agent.
        token_usage: Option<NewTokenUsage>,
        /// Tool calls retained in source order.
        tool_calls: Vec<NewToolCall>,
    },
    Failed {
        /// Safe user-facing failure message.
        error_message: String,
    },
}

/// One Agent result prepared for transactional persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewComparisonResult {
    /// Agent product that produced this outcome.
    pub(crate) agent: AgentKind,
    /// Model configuration captured when the comparison started.
    pub(crate) model: Option<String>,
    /// Reasoning configuration captured when the comparison started.
    pub(crate) reasoning_effort: Option<String>,
    /// Success metrics or safe failure detail.
    pub(crate) outcome: NewComparisonOutcome,
}

/// Complete comparison prepared for one atomic write transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewComparisonRun {
    /// Shared task submitted to every selected Agent.
    pub(crate) query: String,
    /// Version of the metric calculation contract.
    pub(crate) metric_version: i64,
    /// UTC Unix timestamp in milliseconds.
    pub(crate) created_at_ms: i64,
    /// Final result for every selected Agent.
    pub(crate) results: Vec<NewComparisonResult>,
}

/// Stable keyset pagination cursor for the history index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ComparisonCursor {
    /// UTC millisecond timestamp of the final item in the prior page.
    pub(crate) created_at_ms: i64,
    /// Primary key of the final item in the prior page.
    pub(crate) id: i64,
}

/// Lightweight Agent state shown in the history list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonAgentSummary {
    /// Agent represented by this summary.
    pub(crate) agent: AgentKind,
    /// Whether this Agent succeeded or failed.
    pub(crate) status: ComparisonResultStatus,
}

/// Comparison fields loaded by the paginated history index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonSummary {
    /// Persistent comparison identifier.
    pub(crate) id: i64,
    /// Shared task text, used for the list title and preview.
    pub(crate) query: String,
    /// Aggregate completion state.
    pub(crate) status: ComparisonStatus,
    /// Metric contract version used for this record.
    pub(crate) metric_version: i64,
    /// UTC Unix timestamp in milliseconds.
    pub(crate) created_at_ms: i64,
    /// Agent outcomes loaded without response bodies.
    pub(crate) agents: Vec<ComparisonAgentSummary>,
}

/// One page of comparison summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonPage {
    /// Summaries ordered newest first.
    pub(crate) items: Vec<ComparisonSummary>,
    /// Cursor for the next page, or none when the end is reached.
    pub(crate) next_cursor: Option<ComparisonCursor>,
}

/// Fully reconstructed result for one Agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonResultDetail {
    /// Persistent result identifier.
    pub(crate) id: i64,
    /// Agent product represented by this result.
    pub(crate) agent: AgentKind,
    /// Model configuration captured at execution time.
    pub(crate) model: Option<String>,
    /// Reasoning configuration captured at execution time.
    pub(crate) reasoning_effort: Option<String>,
    /// Success or failure state.
    pub(crate) status: ComparisonResultStatus,
    /// Final Agent response for a successful result.
    pub(crate) response: Option<String>,
    /// Safe failure message for a failed result.
    pub(crate) error_message: Option<String>,
    /// Complete task duration in milliseconds.
    pub(crate) total_duration_ms: Option<i64>,
    /// Delay until the first assistant text in milliseconds.
    pub(crate) time_to_first_token_ms: Option<i64>,
    /// Sum of explicit thinking intervals in milliseconds.
    pub(crate) thinking_duration_ms: Option<i64>,
    /// Token counters when reported by the Agent.
    pub(crate) token_usage: Option<NewTokenUsage>,
    /// Ordered tool invocations for this result.
    pub(crate) tool_calls: Vec<NewToolCall>,
}

/// Complete comparison loaded for the history detail view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonDetail {
    /// Persistent comparison identifier.
    pub(crate) id: i64,
    /// Shared task submitted to every Agent.
    pub(crate) query: String,
    /// Aggregate completion state.
    pub(crate) status: ComparisonStatus,
    /// Metric calculation contract version.
    pub(crate) metric_version: i64,
    /// UTC Unix timestamp in milliseconds.
    pub(crate) created_at_ms: i64,
    /// Complete results in insertion order.
    pub(crate) results: Vec<ComparisonResultDetail>,
}
