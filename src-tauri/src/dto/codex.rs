use serde::Serialize;

/// Describes whether the local Codex CLI can run authenticated tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CodexLoginStatus {
    /// Indicates whether a Codex executable was found locally.
    pub(crate) installed: bool,
    /// Indicates whether the discovered Codex CLI has active credentials.
    pub(crate) logged_in: bool,
    /// Contains the safe authentication mode reported by Codex when available.
    pub(crate) authentication_method: Option<String>,
    /// Contains the effective model selected for new local Codex tasks.
    pub(crate) model: Option<String>,
    /// Contains the effective reasoning effort selected for new local Codex tasks.
    pub(crate) reasoning_effort: Option<String>,
}
