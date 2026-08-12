use serde::Serialize;

/// Describes whether the local Claude Code runtime can run authenticated tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeLoginStatus {
    /// Indicates whether a Claude Code executable was found locally.
    pub(crate) installed: bool,
    /// Indicates whether Claude Code reports active credentials.
    pub(crate) logged_in: bool,
    /// Contains the safe authentication mode reported by Claude Code when available.
    pub(crate) authentication_method: Option<String>,
    /// Contains the effective model when Claude Code can report it without running a task.
    pub(crate) model: Option<String>,
    /// Contains the effective reasoning effort when available.
    pub(crate) reasoning_effort: Option<String>,
}
