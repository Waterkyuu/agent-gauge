use serde::Serialize;

/// Describes whether the local WorkBuddy runtime can run authenticated tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkBuddyLoginStatus {
    /// Indicates whether the WorkBuddy application or CodeBuddy CLI was found locally.
    pub(crate) installed: bool,
    /// Indicates whether the discovered WorkBuddy runtime accepted an authenticated ACP session.
    pub(crate) logged_in: bool,
    /// Contains the safe authentication mode when available.
    pub(crate) authentication_method: Option<String>,
    /// Contains the effective model selected for new WorkBuddy tasks.
    pub(crate) model: Option<String>,
    /// Contains the effective reasoning effort selected for new WorkBuddy tasks.
    pub(crate) reasoning_effort: Option<String>,
}
