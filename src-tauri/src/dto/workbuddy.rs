use crate::adapters::workbuddy::WorkBuddyConfigSnapshot;
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

/// Model configuration returned by the initial command and native change event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkBuddyConfigStatus {
    /// Contains the model selected for newly created WorkBuddy tasks.
    pub(crate) model: Option<String>,
    /// Contains the effective thinking level selected for new WorkBuddy tasks.
    pub(crate) reasoning_effort: Option<String>,
}

impl From<WorkBuddyConfigSnapshot> for WorkBuddyConfigStatus {
    fn from(snapshot: WorkBuddyConfigSnapshot) -> Self {
        Self {
            model: snapshot.model,
            reasoning_effort: snapshot.reasoning_effort,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkBuddyConfigStatus;

    #[test]
    fn serializes_workbuddy_config_for_tauri_events() {
        let status = WorkBuddyConfigStatus {
            model: Some("kimi-k3".to_string()),
            reasoning_effort: Some("high".to_string()),
        };

        let value = serde_json::to_value(status).expect("config status should serialize");

        assert_eq!(value["model"], "kimi-k3");
        assert_eq!(value["reasoningEffort"], "high");
    }
}
