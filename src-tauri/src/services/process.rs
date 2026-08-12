use crate::adapters::process::{AgentProcessAdapter, AgentProcessStates};
use crate::error::AppError;

/// Reads one normalized snapshot of supported local Agent processes.
pub(crate) fn check_agent_processes(
    adapter: &impl AgentProcessAdapter,
) -> Result<AgentProcessStates, AppError> {
    adapter.check_processes()
}
