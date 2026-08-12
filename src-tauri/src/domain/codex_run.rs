use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TokenUsage {
    pub(crate) total_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) cached_input_tokens: u64,
    pub(crate) cache_write_input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning_output_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexRunMetrics {
    pub(crate) total_duration: Duration,
    pub(crate) time_to_first_token: Option<Duration>,
    pub(crate) token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexRunOutput {
    pub(crate) response: String,
    pub(crate) metrics: CodexRunMetrics,
}

#[derive(Debug, Default)]
pub(crate) struct CodexRunMetricsCollector {
    time_to_first_token: Option<Duration>,
    token_usage: Option<TokenUsage>,
}

impl CodexRunMetricsCollector {
    /// Captures the first non-empty streamed assistant content observation.
    pub(crate) fn record_agent_delta(&mut self, delta: &str, elapsed: Duration) {
        if self.time_to_first_token.is_none() && !delta.is_empty() {
            self.time_to_first_token = Some(elapsed);
        }
    }

    /// Replaces the usage snapshot because app-server reports the latest turn totals cumulatively.
    pub(crate) fn record_token_usage(&mut self, usage: TokenUsage) {
        self.token_usage = Some(usage);
    }

    /// Finalizes the immutable metric snapshot when app-server reports turn completion.
    pub(crate) fn finish(self, total_duration: Duration) -> CodexRunMetrics {
        CodexRunMetrics {
            total_duration,
            time_to_first_token: self.time_to_first_token,
            token_usage: self.token_usage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CodexRunMetricsCollector, TokenUsage};
    use std::time::Duration;

    #[test]
    fn records_first_non_empty_agent_delta_once() {
        let mut collector = CodexRunMetricsCollector::default();

        collector.record_agent_delta("", Duration::from_millis(40));
        collector.record_agent_delta("首", Duration::from_millis(75));
        collector.record_agent_delta("个 token 后的内容", Duration::from_millis(120));

        let metrics = collector.finish(Duration::from_millis(200));

        assert_eq!(metrics.time_to_first_token, Some(Duration::from_millis(75)));
    }

    #[test]
    fn returns_duration_and_latest_turn_token_usage() {
        let mut collector = CodexRunMetricsCollector::default();
        let usage = TokenUsage {
            total_tokens: 120,
            input_tokens: 80,
            cached_input_tokens: 40,
            cache_write_input_tokens: 0,
            output_tokens: 30,
            reasoning_output_tokens: 10,
        };

        collector.record_token_usage(usage.clone());
        let metrics = collector.finish(Duration::from_millis(450));

        assert_eq!(metrics.total_duration, Duration::from_millis(450));
        assert_eq!(metrics.token_usage, Some(usage));
    }
}
