use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TokenUsage {
    pub(crate) total_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) cached_input_tokens: u64,
    pub(crate) cache_write_input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolCallMetric {
    /// Stable tool name reported by the source Agent protocol.
    pub(crate) name: String,
    /// Wall-clock time between the tool request and its matching result.
    pub(crate) duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentRunMetrics {
    pub(crate) total_duration: Duration,
    pub(crate) time_to_first_token: Option<Duration>,
    pub(crate) token_usage: Option<TokenUsage>,
    /// Sum of explicit reasoning or thinking content-block intervals.
    pub(crate) thinking_duration: Duration,
    /// Tool invocations retained in source start order.
    pub(crate) tool_calls: Vec<ToolCallMetric>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentRunOutput {
    pub(crate) response: String,
    pub(crate) metrics: AgentRunMetrics,
}

#[derive(Debug, Default)]
pub(crate) struct AgentRunMetricsCollector {
    time_to_first_token: Option<Duration>,
    token_usage: Option<TokenUsage>,
    thinking_duration: Duration,
    active_thinking_intervals: Vec<(String, Duration)>,
    tool_calls: Vec<PendingToolCall>,
}

#[derive(Debug)]
struct PendingToolCall {
    /// Protocol identifier used to match the later tool result.
    id: String,
    /// User-visible tool name captured at invocation time.
    name: String,
    /// Task-relative wall-clock time when the invocation was observed.
    started_at: Duration,
    /// Completed duration, or none while the tool remains active.
    duration: Option<Duration>,
}

impl AgentRunMetricsCollector {
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

    /// Starts one named thinking interval unless the source identifier is already active.
    pub(crate) fn record_thinking_started(&mut self, id: &str, elapsed: Duration) {
        if self
            .active_thinking_intervals
            .iter()
            .any(|(active_id, _)| active_id == id)
        {
            return;
        }
        self.active_thinking_intervals
            .push((id.to_string(), elapsed));
    }

    /// Adds one completed thinking interval to the task total.
    pub(crate) fn record_thinking_finished(&mut self, id: &str, elapsed: Duration) {
        let Some(index) = self
            .active_thinking_intervals
            .iter()
            .position(|(active_id, _)| active_id == id)
        else {
            return;
        };
        let (_, started_at) = self.active_thinking_intervals.remove(index);
        self.thinking_duration += elapsed.saturating_sub(started_at);
    }

    /// Records one tool invocation in source start order.
    pub(crate) fn record_tool_started(&mut self, id: &str, name: &str, elapsed: Duration) {
        if self.tool_calls.iter().any(|call| call.id == id) {
            return;
        }
        self.tool_calls.push(PendingToolCall {
            id: id.to_string(),
            name: name.to_string(),
            started_at: elapsed,
            duration: None,
        });
    }

    /// Completes the matching tool invocation when its result is observed.
    pub(crate) fn record_tool_finished(&mut self, id: &str, elapsed: Duration) {
        let Some(call) = self
            .tool_calls
            .iter_mut()
            .find(|call| call.id == id && call.duration.is_none())
        else {
            return;
        };
        call.duration = Some(elapsed.saturating_sub(call.started_at));
    }

    /// Finalizes the immutable metric snapshot when app-server reports turn completion.
    pub(crate) fn finish(mut self, total_duration: Duration) -> AgentRunMetrics {
        // A successful task may finish before a source emits its final stop/result event. Closing
        // active intervals here preserves every observed invocation without inventing a timeout.
        for (_, started_at) in self.active_thinking_intervals {
            self.thinking_duration += total_duration.saturating_sub(started_at);
        }
        let tool_calls = self
            .tool_calls
            .into_iter()
            .map(|call| ToolCallMetric {
                name: call.name,
                duration: call
                    .duration
                    .unwrap_or_else(|| total_duration.saturating_sub(call.started_at)),
            })
            .collect();

        AgentRunMetrics {
            total_duration,
            time_to_first_token: self.time_to_first_token,
            token_usage: self.token_usage,
            thinking_duration: self.thinking_duration,
            tool_calls,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentRunMetricsCollector, TokenUsage};
    use std::time::Duration;

    #[test]
    fn records_first_non_empty_agent_delta_once() {
        let mut collector = AgentRunMetricsCollector::default();

        collector.record_agent_delta("", Duration::from_millis(40));
        collector.record_agent_delta("首", Duration::from_millis(75));
        collector.record_agent_delta("个 token 后的内容", Duration::from_millis(120));

        let metrics = collector.finish(Duration::from_millis(200));

        assert_eq!(metrics.time_to_first_token, Some(Duration::from_millis(75)));
    }

    #[test]
    fn returns_duration_and_latest_turn_token_usage() {
        let mut collector = AgentRunMetricsCollector::default();
        let usage = TokenUsage {
            total_tokens: 120,
            input_tokens: 80,
            cached_input_tokens: 40,
            cache_write_input_tokens: 0,
            output_tokens: 30,
            reasoning_output_tokens: Some(10),
        };

        collector.record_token_usage(usage.clone());
        let metrics = collector.finish(Duration::from_millis(450));

        assert_eq!(metrics.total_duration, Duration::from_millis(450));
        assert_eq!(metrics.token_usage, Some(usage));
    }

    #[test]
    fn records_tool_calls_in_start_order_with_measured_durations() {
        let mut collector = AgentRunMetricsCollector::default();

        collector.record_tool_started("tool-1", "Read", Duration::from_millis(100));
        collector.record_tool_started("tool-2", "Bash", Duration::from_millis(150));
        collector.record_tool_finished("tool-2", Duration::from_millis(350));
        collector.record_tool_finished("tool-1", Duration::from_millis(500));

        let metrics = collector.finish(Duration::from_millis(600));

        assert_eq!(
            metrics.tool_calls,
            vec![
                super::ToolCallMetric {
                    name: "Read".to_string(),
                    duration: Duration::from_millis(400),
                },
                super::ToolCallMetric {
                    name: "Bash".to_string(),
                    duration: Duration::from_millis(200),
                },
            ]
        );
    }

    #[test]
    fn accumulates_thinking_intervals_and_closes_active_metrics_at_finish() {
        let mut collector = AgentRunMetricsCollector::default();

        collector.record_thinking_started("thinking-1", Duration::from_millis(50));
        collector.record_thinking_finished("thinking-1", Duration::from_millis(250));
        collector.record_thinking_started("thinking-2", Duration::from_millis(300));
        collector.record_tool_started("tool-1", "WebSearch", Duration::from_millis(400));

        let metrics = collector.finish(Duration::from_millis(700));

        assert_eq!(metrics.thinking_duration, Duration::from_millis(600));
        assert_eq!(
            metrics.tool_calls,
            vec![super::ToolCallMetric {
                name: "WebSearch".to_string(),
                duration: Duration::from_millis(300),
            }]
        );
    }
}
