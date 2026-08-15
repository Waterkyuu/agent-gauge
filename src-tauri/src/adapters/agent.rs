use crate::domain::agent_run::AgentRunOutput;
use crate::error::AppError;

/// Normalized execution boundary shared by every locally monitored agent product.
pub(crate) trait AgentAdapter {
    /// Runs one task and returns normalized response, latency, and token metrics.
    fn run_task(&self, query: &str) -> Result<AgentRunOutput, AppError>;
}
