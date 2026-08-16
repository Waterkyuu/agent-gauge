/// Supported local Agent product that owns an observed task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentActivityKind {
    Claude,
    Codex,
    WorkBuddy,
}

/// User-facing lifecycle shared by every supported Agent product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentActivityStatus {
    Running,
    Waiting,
    Finish,
    Error,
}

/// Privacy-safe summary of one task observed from a local Agent event source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentActivity {
    /// Opaque identifier derived locally without exposing the source session identifier.
    pub(crate) id: String,
    /// Product that owns the task.
    pub(crate) agent: AgentActivityKind,
    /// Latest lifecycle state derived from the product's own protocol.
    pub(crate) status: AgentActivityStatus,
    /// Last filesystem observation time in Unix milliseconds.
    pub(crate) updated_at_ms: u64,
}
