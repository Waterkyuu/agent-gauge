use crate::domain::agent_activity::{AgentActivity, AgentActivityKind, AgentActivityStatus};
use crate::services::activity::SystemAgentActivityMonitor;
use serde::Serialize;
use tauri::State;

/// Snapshot payload shared by the command and changed event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentActivitiesResponse {
    /// Recent task summaries sorted by latest source activity.
    activities: Vec<AgentActivityResponse>,
}

/// Privacy-safe task summary sent across the Tauri boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentActivityResponse {
    /// Opaque local identifier that does not reveal the source session ID.
    id: String,
    /// Product identifier used by the frontend Agent presentation map.
    agent: &'static str,
    /// Shared four-state lifecycle identifier.
    status: &'static str,
    /// Last source modification time in Unix milliseconds.
    updated_at_ms: u64,
}

impl From<Vec<AgentActivity>> for AgentActivitiesResponse {
    fn from(activities: Vec<AgentActivity>) -> Self {
        Self {
            activities: activities
                .into_iter()
                .map(|activity| AgentActivityResponse {
                    id: activity.id,
                    agent: match activity.agent {
                        AgentActivityKind::Claude => "claude",
                        AgentActivityKind::Codex => "codex",
                        AgentActivityKind::WorkBuddy => "workbuddy",
                    },
                    status: match activity.status {
                        AgentActivityStatus::Running => "running",
                        AgentActivityStatus::Waiting => "waiting",
                        AgentActivityStatus::Finish => "finish",
                        AgentActivityStatus::Error => "error",
                    },
                    updated_at_ms: activity.updated_at_ms,
                })
                .collect(),
        }
    }
}

/// Returns the monitor's cached snapshot without scanning product files on the UI thread.
#[tauri::command]
pub(crate) fn check_agent_activities(
    monitor: State<'_, SystemAgentActivityMonitor>,
) -> AgentActivitiesResponse {
    monitor.current_activities().into()
}

#[cfg(test)]
mod tests {
    use super::AgentActivitiesResponse;
    use crate::domain::agent_activity::{AgentActivity, AgentActivityKind, AgentActivityStatus};

    #[test]
    fn serializes_the_privacy_safe_activity_contract_in_camel_case() {
        let response = AgentActivitiesResponse::from(vec![AgentActivity {
            id: "codex-1234".to_string(),
            agent: AgentActivityKind::Codex,
            status: AgentActivityStatus::Waiting,
            updated_at_ms: 42,
        }]);

        let value = serde_json::to_value(response).expect("activity response should serialize");

        assert_eq!(value["activities"][0]["id"], "codex-1234");
        assert_eq!(value["activities"][0]["agent"], "codex");
        assert_eq!(value["activities"][0]["status"], "waiting");
        assert_eq!(value["activities"][0]["updatedAtMs"], 42);
        assert_eq!(
            value["activities"][0].as_object().map(|item| item.len()),
            Some(4)
        );
    }
}
