use crate::adapters::agent::AgentAdapter;
use crate::domain::codex_run::AgentRunOutput;
use crate::error::AppError;

/// Runs one local agent task after enforcing the bounded query contract.
pub(crate) fn run_agent_task(
    adapter: &impl AgentAdapter,
    query: &str,
) -> Result<AgentRunOutput, AppError> {
    if query.trim().is_empty() || query.len() > 16_000 {
        return Err(AppError::InvalidQuery);
    }

    adapter.run_task(query)
}

#[cfg(test)]
mod tests {
    use super::run_agent_task;
    use crate::adapters::agent::AgentAdapter;
    use crate::domain::codex_run::AgentRunOutput;
    use crate::error::AppError;

    struct FakeAgentAdapter;

    impl AgentAdapter for FakeAgentAdapter {
        fn run_task(&self, _query: &str) -> Result<AgentRunOutput, AppError> {
            Err(AppError::CodexTaskFailed)
        }
    }

    #[test]
    fn rejects_empty_and_oversized_queries_before_calling_an_agent() {
        assert_eq!(
            run_agent_task(&FakeAgentAdapter, "  "),
            Err(AppError::InvalidQuery)
        );
        assert_eq!(
            run_agent_task(&FakeAgentAdapter, &"a".repeat(16_001)),
            Err(AppError::InvalidQuery)
        );
    }
}
