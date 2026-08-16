use crate::adapters::process::AgentProcessStates;
use crate::domain::agent_activity::{AgentActivity, AgentActivityKind, AgentActivityStatus};
use leveldb_forensic::{decode_local_storage, LocalStorageRecord};
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BOARD_ACTIVITIES: usize = 48;
const MAX_SCAN_ENTRIES: usize = 10_000;
const MAX_TRANSCRIPTS_PER_PRODUCT: usize = 16;
const MAX_TRANSCRIPT_BYTES: u64 = 2 * 1024 * 1024;
const WORKBUDDY_STATUS_SNAPSHOT_KEY: &str = "codebuddy-conversation-status-snapshot";

/// Allowed local roots used by the passive activity adapter.
#[derive(Debug, Clone, Default)]
pub(crate) struct AgentActivitySourcePaths {
    /// Codex rollout directory documented by the local Codex runtime.
    pub(crate) codex_sessions: Option<PathBuf>,
    /// Claude Code transcript directory documented by Claude Code.
    pub(crate) claude_projects: Option<PathBuf>,
    /// CodeBuddy transcript directory used by WorkBuddy's bundled Agent runtime.
    pub(crate) codebuddy_projects: Option<PathBuf>,
    /// WorkBuddy Chromium Local Storage directory containing conversation status snapshots.
    pub(crate) workbuddy_local_storage: Option<PathBuf>,
}

/// Abstracts read-only task discovery for the activity monitor and deterministic tests.
pub(crate) trait AgentActivityAdapter {
    fn list_activities(&self, processes: AgentProcessStates) -> Vec<AgentActivity>;
    fn watch_paths(&self) -> Vec<PathBuf>;
}

/// Reads bounded structured task records written by supported local Agent products.
#[derive(Debug, Clone)]
pub(crate) struct SystemAgentActivityAdapter {
    /// Product-owned roots selected from the current user's home directory.
    sources: AgentActivitySourcePaths,
}

impl Default for SystemAgentActivityAdapter {
    fn default() -> Self {
        let sources = dirs::home_dir().map_or_else(AgentActivitySourcePaths::default, |home| {
            AgentActivitySourcePaths {
                codex_sessions: Some(home.join(".codex").join("sessions")),
                claude_projects: Some(home.join(".claude").join("projects")),
                codebuddy_projects: Some(home.join(".codebuddy").join("projects")),
                workbuddy_local_storage: Some(
                    home.join(".workbuddy-ai")
                        .join("app")
                        .join("session")
                        .join("Local Storage")
                        .join("leveldb"),
                ),
            }
        });
        Self { sources }
    }
}

impl AgentActivityAdapter for SystemAgentActivityAdapter {
    /// Collects recent tasks while keeping prompts, output, paths, and source identifiers private.
    fn list_activities(&self, processes: AgentProcessStates) -> Vec<AgentActivity> {
        let mut activities = Vec::new();
        self.collect_transcripts(
            &mut activities,
            self.sources.codex_sessions.as_deref(),
            AgentActivityKind::Codex,
            processes.codex,
        );
        self.collect_transcripts(
            &mut activities,
            self.sources.claude_projects.as_deref(),
            AgentActivityKind::Claude,
            processes.claude,
        );
        self.collect_transcripts(
            &mut activities,
            self.sources.codebuddy_projects.as_deref(),
            AgentActivityKind::WorkBuddy,
            processes.workbuddy,
        );

        if let Some(path) = self.sources.workbuddy_local_storage.as_deref() {
            if let Some((snapshot, updated_at_ms)) = latest_workbuddy_snapshot(path) {
                activities.extend(workbuddy_activities_from_snapshot(&snapshot, updated_at_ms));
            }
        }

        activities.sort_by(|left, right| {
            right
                .updated_at_ms
                .cmp(&left.updated_at_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        activities.dedup_by(|left, right| left.id == right.id);
        activities.truncate(MAX_BOARD_ACTIVITIES);
        activities
    }

    /// Returns only existing product-owned directories suitable for native watching.
    fn watch_paths(&self) -> Vec<PathBuf> {
        [
            self.sources.codex_sessions.as_ref(),
            self.sources.claude_projects.as_ref(),
            self.sources.codebuddy_projects.as_ref(),
            self.sources.workbuddy_local_storage.as_ref(),
        ]
        .into_iter()
        .flatten()
        .filter(|path| fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()))
        .cloned()
        .collect()
    }
}

impl SystemAgentActivityAdapter {
    /// Adds the latest bounded transcripts from one product root to the shared snapshot.
    fn collect_transcripts(
        &self,
        activities: &mut Vec<AgentActivity>,
        root: Option<&Path>,
        agent: AgentActivityKind,
        process_running: bool,
    ) {
        let Some(root) = root else {
            return;
        };
        for path in recent_jsonl_files(root) {
            if let Some(activity) = activity_from_transcript(&path, agent, process_running) {
                activities.push(activity);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct CodexRolloutEvent {
    /// Structured payload written by Codex for one rollout event.
    payload: Option<CodexRolloutPayload>,
    /// Official App Server notification method when protocol events are persisted.
    method: Option<String>,
    /// Official App Server notification parameters.
    params: Option<CodexNotificationParams>,
}

#[derive(Debug, Deserialize)]
struct CodexRolloutPayload {
    /// Event or response-item discriminator.
    #[serde(rename = "type")]
    event_type: String,
    /// Tool name when the response item invokes a tool.
    name: Option<String>,
    /// Identifier that pairs a tool invocation with its output.
    call_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexNotificationParams {
    /// Current thread lifecycle for thread/status/changed notifications.
    status: Option<CodexThreadStatus>,
    /// Current or terminal turn lifecycle for turn notifications.
    turn: Option<CodexTurnStatus>,
}

#[derive(Debug, Deserialize)]
struct CodexThreadStatus {
    /// Thread lifecycle discriminator such as active or systemError.
    #[serde(rename = "type")]
    status_type: String,
    /// Reasons an active turn cannot proceed without the user.
    #[serde(rename = "activeFlags", default)]
    active_flags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CodexTurnStatus {
    /// Official turn lifecycle such as inProgress, completed, interrupted, or failed.
    status: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeTranscriptEvent {
    /// Transcript event discriminator.
    #[serde(rename = "type")]
    event_type: Option<String>,
    /// Official Claude Code hook lifecycle discriminator.
    hook_event_name: Option<String>,
    /// Official Notification subtype such as permission_prompt or idle_prompt.
    notification_type: Option<String>,
    /// System-event subtype used for terminal failures.
    subtype: Option<String>,
    /// Conversation message carried by user and assistant events.
    message: Option<ClaudeTranscriptMessage>,
    /// Claude Code marker written for API failures in persisted transcripts.
    #[serde(rename = "isApiErrorMessage", default)]
    is_api_error_message: bool,
}

#[derive(Debug, Deserialize)]
struct ClaudeTranscriptMessage {
    /// Message role supplied by the transcript protocol.
    role: String,
    /// Reason the assistant stopped its current response.
    stop_reason: Option<String>,
    /// String or block-based message content.
    content: Option<ClaudeTranscriptContent>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClaudeTranscriptContent {
    Text(String),
    Blocks(Vec<ClaudeTranscriptBlock>),
}

#[derive(Debug, Deserialize)]
struct ClaudeTranscriptBlock {
    /// Content-block discriminator such as tool_use or tool_result.
    #[serde(rename = "type")]
    block_type: String,
    /// Tool identifier created by a tool_use block.
    id: Option<String>,
    /// Tool name supplied by a tool_use block.
    name: Option<String>,
    /// Identifier that links a tool_result block to the request.
    tool_use_id: Option<String>,
}

/// Maps persisted Codex turn events to the official shared board lifecycle.
fn codex_status_from_jsonl(contents: &str, process_running: bool) -> Option<AgentActivityStatus> {
    let mut status = None;
    let mut pending_user_input = None;

    for event in contents
        .lines()
        .filter_map(|line| serde_json::from_str::<CodexRolloutEvent>(line).ok())
    {
        match event.method.as_deref() {
            Some("thread/status/changed") => {
                if let Some(thread_status) = event
                    .params
                    .as_ref()
                    .and_then(|params| params.status.as_ref())
                {
                    status = match thread_status.status_type.as_str() {
                        "active"
                            if thread_status.active_flags.iter().any(|flag| {
                                matches!(flag.as_str(), "waitingOnApproval" | "waitingOnUserInput")
                            }) =>
                        {
                            Some(AgentActivityStatus::Waiting)
                        }
                        "active" => Some(AgentActivityStatus::Running),
                        "idle" => Some(AgentActivityStatus::Finish),
                        "systemError" => Some(AgentActivityStatus::Error),
                        _ => status,
                    };
                }
                continue;
            }
            Some("turn/started") => {
                status = Some(AgentActivityStatus::Running);
                continue;
            }
            Some("turn/completed") => {
                status = match event
                    .params
                    .as_ref()
                    .and_then(|params| params.turn.as_ref())
                    .map(|turn| turn.status.as_str())
                {
                    Some("completed") => Some(AgentActivityStatus::Finish),
                    Some("interrupted" | "failed") => Some(AgentActivityStatus::Error),
                    Some("inProgress") => Some(AgentActivityStatus::Running),
                    _ => status,
                };
                continue;
            }
            _ => {}
        }
        let Some(payload) = event.payload else {
            continue;
        };
        match payload.event_type.as_str() {
            "task_started" => {
                status = Some(AgentActivityStatus::Running);
                pending_user_input = None;
            }
            "function_call" | "custom_tool_call"
                if payload.name.as_deref() == Some("request_user_input") =>
            {
                status = Some(AgentActivityStatus::Waiting);
                pending_user_input = payload.call_id;
            }
            "function_call_output" | "custom_tool_call_output"
                if payload.call_id == pending_user_input =>
            {
                status = Some(AgentActivityStatus::Running);
                pending_user_input = None;
            }
            "task_complete" => {
                status = Some(AgentActivityStatus::Finish);
                pending_user_input = None;
            }
            "turn_aborted" => {
                status = Some(AgentActivityStatus::Error);
                pending_user_input = None;
            }
            "reasoning"
            | "agent_reasoning"
            | "agent_message"
            | "message"
            | "token_count"
            | "function_call"
            | "custom_tool_call"
            | "function_call_output"
            | "custom_tool_call_output"
                if status.is_none() =>
            {
                status = Some(AgentActivityStatus::Running);
            }
            _ => {}
        }
    }

    match status {
        Some(AgentActivityStatus::Running | AgentActivityStatus::Waiting) if !process_running => {
            Some(AgentActivityStatus::Error)
        }
        value => value,
    }
}

/// Maps Claude Code transcript messages to prompt, question, completion, and failure states.
fn claude_status_from_jsonl(contents: &str, process_running: bool) -> Option<AgentActivityStatus> {
    let mut status = None;
    let mut pending_question = None;

    for event in contents
        .lines()
        .filter_map(|line| serde_json::from_str::<ClaudeTranscriptEvent>(line).ok())
    {
        match event.hook_event_name.as_deref() {
            Some("PermissionRequest") => {
                status = Some(AgentActivityStatus::Waiting);
                continue;
            }
            Some("Notification")
                if matches!(
                    event.notification_type.as_deref(),
                    Some("permission_prompt" | "idle_prompt" | "elicitation_dialog")
                ) =>
            {
                status = Some(AgentActivityStatus::Waiting);
                continue;
            }
            Some("Stop") => {
                status = Some(AgentActivityStatus::Finish);
                continue;
            }
            Some("StopFailure") => {
                status = Some(AgentActivityStatus::Error);
                continue;
            }
            Some("UserPromptSubmit" | "PreToolUse" | "PostToolUse" | "PostToolUseFailure") => {
                status = Some(AgentActivityStatus::Running);
                continue;
            }
            _ => {}
        }
        if event.is_api_error_message || event.subtype.as_deref() == Some("api_error") {
            status = Some(AgentActivityStatus::Error);
            pending_question = None;
            continue;
        }
        let Some(message) = event.message else {
            continue;
        };
        match (event.event_type.as_deref(), message.role.as_str()) {
            (Some("user"), "user") => {
                let answered_question = match &message.content {
                    Some(ClaudeTranscriptContent::Text(text)) => {
                        let _ = text;
                        false
                    }
                    Some(ClaudeTranscriptContent::Blocks(blocks)) => blocks.iter().any(|block| {
                        block.block_type == "tool_result" && block.tool_use_id == pending_question
                    }),
                    None => false,
                };
                if answered_question || pending_question.is_none() {
                    status = Some(AgentActivityStatus::Running);
                    if answered_question {
                        pending_question = None;
                    }
                }
            }
            (Some("assistant"), "assistant") => {
                if let Some(ClaudeTranscriptContent::Blocks(blocks)) = &message.content {
                    if let Some(question) = blocks.iter().find(|block| {
                        block.block_type == "tool_use"
                            && block.name.as_deref() == Some("AskUserQuestion")
                    }) {
                        pending_question = question.id.clone();
                        status = Some(AgentActivityStatus::Waiting);
                        continue;
                    }
                }
                status = match message.stop_reason.as_deref() {
                    Some("end_turn" | "stop_sequence") => Some(AgentActivityStatus::Finish),
                    Some("max_tokens" | "refusal") => Some(AgentActivityStatus::Error),
                    Some("tool_use") | None => Some(AgentActivityStatus::Running),
                    Some(_) => status,
                };
            }
            _ => {}
        }
    }

    match status {
        Some(AgentActivityStatus::Running | AgentActivityStatus::Waiting) if !process_running => {
            Some(AgentActivityStatus::Error)
        }
        value => value,
    }
}

/// Maps WorkBuddy's documented conversation protocol states to the shared four-state board.
fn workbuddy_status_from_protocol(status: &str) -> Option<AgentActivityStatus> {
    match status.to_ascii_lowercase().as_str() {
        "working" | "running" | "planning" | "connecting" => Some(AgentActivityStatus::Running),
        "pending" => Some(AgentActivityStatus::Waiting),
        "idle" | "completed" | "archived" => Some(AgentActivityStatus::Finish),
        "failed" | "error" | "terminated" | "cancelled" | "canceled" => {
            Some(AgentActivityStatus::Error)
        }
        _ => None,
    }
}

/// Reads one bounded transcript tail and returns a privacy-safe latest-task summary.
fn activity_from_transcript(
    path: &Path,
    agent: AgentActivityKind,
    process_running: bool,
) -> Option<AgentActivity> {
    let contents = read_bounded_file_tail(path)?;
    let status = match agent {
        AgentActivityKind::Codex => codex_status_from_jsonl(&contents, process_running),
        AgentActivityKind::Claude | AgentActivityKind::WorkBuddy => {
            claude_status_from_jsonl(&contents, process_running)
        }
    }?;
    let updated_at_ms = fs::symlink_metadata(path)
        .ok()?
        .modified()
        .ok()
        .and_then(system_time_millis)?;

    Some(AgentActivity {
        id: opaque_activity_id(agent, &path.to_string_lossy()),
        agent,
        status,
        updated_at_ms,
    })
}

/// Decodes WorkBuddy's conversation-status map without exposing conversation identifiers.
fn workbuddy_activities_from_snapshot(contents: &str, updated_at_ms: u64) -> Vec<AgentActivity> {
    let Ok(statuses) = serde_json::from_str::<HashMap<String, String>>(contents) else {
        return Vec::new();
    };
    let mut activities: Vec<_> = statuses
        .into_iter()
        .filter_map(|(source_id, protocol_status)| {
            Some(AgentActivity {
                id: opaque_activity_id(AgentActivityKind::WorkBuddy, &source_id),
                agent: AgentActivityKind::WorkBuddy,
                status: workbuddy_status_from_protocol(&protocol_status)?,
                updated_at_ms,
            })
        })
        .collect();
    activities.sort_by(|left, right| left.id.cmp(&right.id));
    activities
}

/// Finds the latest non-deleted WorkBuddy status snapshot retained by Chromium LevelDB.
fn latest_workbuddy_snapshot(path: &Path) -> Option<(String, u64)> {
    if !fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()) {
        return None;
    }
    let records = decode_local_storage(path).ok()?;
    let (_, value) = records
        .into_iter()
        .filter_map(|record| match record {
            LocalStorageRecord::Data {
                script_key,
                value,
                seq,
                deleted: false,
                ..
            } if !script_key.lossy
                && !value.lossy
                && script_key.text == WORKBUDDY_STATUS_SNAPSHOT_KEY =>
            {
                Some((seq, value.text))
            }
            _ => None,
        })
        .max_by_key(|(seq, _)| *seq)?;
    let updated_at_ms = directory_modified_millis(path)?;
    Some((value, updated_at_ms))
}

/// Recursively collects only the newest bounded JSONL files below one trusted product root.
fn recent_jsonl_files(root: &Path) -> Vec<PathBuf> {
    if !fs::symlink_metadata(root).is_ok_and(|metadata| metadata.is_dir()) {
        return Vec::new();
    }
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    let mut visited_entries = 0;

    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            visited_entries += 1;
            if visited_entries > MAX_SCAN_ENTRIES {
                break;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
            {
                let modified = entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(system_time_millis)
                    .unwrap_or_default();
                files.push((modified, entry.path()));
            }
        }
        if visited_entries > MAX_SCAN_ENTRIES {
            break;
        }
    }

    files.sort_by_key(|item| Reverse(item.0));
    files.truncate(MAX_TRANSCRIPTS_PER_PRODUCT);
    files.into_iter().map(|(_, path)| path).collect()
}

/// Reads at most the final two MiB so a growing transcript cannot allocate without bound.
fn read_bounded_file_tail(path: &Path) -> Option<String> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return None;
    }
    let mut file = File::open(path).ok()?;
    let start = metadata.len().saturating_sub(MAX_TRANSCRIPT_BYTES);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len().saturating_sub(start)).unwrap_or_default(),
    );
    file.take(MAX_TRANSCRIPT_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;
    let mut contents = String::from_utf8(bytes).ok()?;
    if start > 0 {
        let first_newline = contents.find('\n')?;
        contents.drain(..=first_newline);
    }
    Some(contents)
}

/// Returns the newest data-file modification time inside the WorkBuddy LevelDB directory.
fn directory_modified_millis(path: &Path) -> Option<u64> {
    fs::read_dir(path)
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() || file_type.is_symlink() {
                return None;
            }
            entry
                .metadata()
                .ok()?
                .modified()
                .ok()
                .and_then(system_time_millis)
        })
        .max()
}

/// Converts a filesystem timestamp to the bounded Unix-millisecond IPC representation.
fn system_time_millis(time: SystemTime) -> Option<u64> {
    let millis = time.duration_since(UNIX_EPOCH).ok()?.as_millis();
    u64::try_from(millis).ok()
}

/// Hashes a sensitive source identifier into an opaque product-scoped display identifier.
fn opaque_activity_id(agent: AgentActivityKind, source_id: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source_id.hash(&mut hasher);
    format!("{}-{:016x}", agent_id_prefix(agent), hasher.finish())
}

/// Returns the stable public product prefix used by opaque task identifiers.
fn agent_id_prefix(agent: AgentActivityKind) -> &'static str {
    match agent {
        AgentActivityKind::Claude => "claude",
        AgentActivityKind::Codex => "codex",
        AgentActivityKind::WorkBuddy => "workbuddy",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        activity_from_transcript, claude_status_from_jsonl, codex_status_from_jsonl,
        workbuddy_activities_from_snapshot, workbuddy_status_from_protocol,
    };
    use crate::domain::agent_activity::{AgentActivityKind, AgentActivityStatus};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_test_file(name: &str, contents: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after the Unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "agent-gauge-activity-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("temporary activity directory should be writable");
        let path = directory.join(name);
        fs::write(&path, contents).expect("temporary transcript should be writable");
        path
    }

    #[test]
    fn codex_process_without_an_active_turn_has_no_run_status() {
        assert_eq!(codex_status_from_jsonl("", true), None);
    }

    #[test]
    fn codex_recognizes_an_active_turn_when_its_start_is_before_the_bounded_tail() {
        let tail = r#"{"type":"response_item","payload":{"type":"reasoning"}}
{"type":"event_msg","payload":{"type":"agent_message"}}"#;

        assert_eq!(
            codex_status_from_jsonl(tail, true),
            Some(AgentActivityStatus::Running)
        );
    }

    #[test]
    fn codex_uses_turn_events_and_unanswered_user_input() {
        let running = r#"{"type":"event_msg","payload":{"type":"task_started"}}
{"type":"response_item","payload":{"type":"reasoning"}}"#;
        let waiting = format!(
            "{running}\n{}",
            r#"{"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"call-1"}}"#
        );
        let resumed = format!(
            "{waiting}\n{}",
            r#"{"type":"response_item","payload":{"type":"function_call_output","call_id":"call-1"}}"#
        );
        let completed = format!(
            "{resumed}\n{}",
            r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#
        );

        assert_eq!(
            codex_status_from_jsonl(running, true),
            Some(AgentActivityStatus::Running)
        );
        assert_eq!(
            codex_status_from_jsonl(&waiting, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            codex_status_from_jsonl(&resumed, true),
            Some(AgentActivityStatus::Running)
        );
        assert_eq!(
            codex_status_from_jsonl(&completed, true),
            Some(AgentActivityStatus::Finish)
        );
    }

    #[test]
    fn codex_uses_official_thread_and_turn_notifications() {
        let waiting_for_approval = r#"{"method":"turn/started","params":{"turn":{"status":"inProgress"}}}
{"method":"thread/status/changed","params":{"status":{"type":"active","activeFlags":["waitingOnApproval"]}}}"#;
        let waiting_for_answer = r#"{"method":"thread/status/changed","params":{"status":{"type":"active","activeFlags":["waitingOnUserInput"]}}}"#;
        let completed = r#"{"method":"turn/completed","params":{"turn":{"status":"completed"}}}"#;
        let failed = r#"{"method":"turn/completed","params":{"turn":{"status":"failed"}}}"#;

        assert_eq!(
            codex_status_from_jsonl(waiting_for_approval, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            codex_status_from_jsonl(waiting_for_answer, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            codex_status_from_jsonl(completed, true),
            Some(AgentActivityStatus::Finish)
        );
        assert_eq!(
            codex_status_from_jsonl(failed, true),
            Some(AgentActivityStatus::Error)
        );
    }

    #[test]
    fn codex_interruption_and_disappeared_active_process_are_errors() {
        let interrupted = r#"{"type":"event_msg","payload":{"type":"task_started"}}
{"type":"event_msg","payload":{"type":"turn_aborted"}}"#;
        let unfinished = r#"{"type":"event_msg","payload":{"type":"task_started"}}"#;

        assert_eq!(
            codex_status_from_jsonl(interrupted, true),
            Some(AgentActivityStatus::Error)
        );
        assert_eq!(
            codex_status_from_jsonl(unfinished, false),
            Some(AgentActivityStatus::Error)
        );
    }

    #[test]
    fn claude_uses_prompt_question_stop_and_failure_events() {
        let running = r#"{"type":"user","message":{"role":"user","content":"do work"}}"#;
        let waiting = format!(
            "{running}\n{}",
            r#"{"type":"assistant","message":{"role":"assistant","stop_reason":"tool_use","content":[{"type":"tool_use","id":"tool-1","name":"AskUserQuestion"}]}}"#
        );
        let completed = format!(
            "{running}\n{}",
            r#"{"type":"assistant","message":{"role":"assistant","stop_reason":"end_turn","content":[{"type":"text","text":"done"}]}}"#
        );
        let failed = format!(
            "{running}\n{}",
            r#"{"type":"system","subtype":"api_error"}"#
        );

        assert_eq!(
            claude_status_from_jsonl(running, true),
            Some(AgentActivityStatus::Running)
        );
        assert_eq!(
            claude_status_from_jsonl(&waiting, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            claude_status_from_jsonl(&completed, true),
            Some(AgentActivityStatus::Finish)
        );
        assert_eq!(
            claude_status_from_jsonl(&failed, true),
            Some(AgentActivityStatus::Error)
        );
    }

    #[test]
    fn claude_uses_official_hook_lifecycle_events() {
        let waiting_for_permission = r#"{"hook_event_name":"UserPromptSubmit"}
{"hook_event_name":"PermissionRequest"}"#;
        let waiting_after_notification =
            r#"{"hook_event_name":"Notification","notification_type":"permission_prompt"}"#;
        let completed = r#"{"hook_event_name":"Stop"}"#;
        let failed = r#"{"hook_event_name":"StopFailure"}"#;

        assert_eq!(
            claude_status_from_jsonl(waiting_for_permission, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            claude_status_from_jsonl(waiting_after_notification, true),
            Some(AgentActivityStatus::Waiting)
        );
        assert_eq!(
            claude_status_from_jsonl(completed, true),
            Some(AgentActivityStatus::Finish)
        );
        assert_eq!(
            claude_status_from_jsonl(failed, true),
            Some(AgentActivityStatus::Error)
        );
    }

    #[test]
    fn workbuddy_maps_its_protocol_statuses_to_the_board() {
        for status in ["planning", "working", "running", "connecting"] {
            assert_eq!(
                workbuddy_status_from_protocol(status),
                Some(AgentActivityStatus::Running)
            );
        }
        assert_eq!(
            workbuddy_status_from_protocol("pending"),
            Some(AgentActivityStatus::Waiting)
        );
        for status in ["idle", "completed", "archived"] {
            assert_eq!(
                workbuddy_status_from_protocol(status),
                Some(AgentActivityStatus::Finish)
            );
        }
        for status in ["failed", "error", "terminated", "cancelled", "canceled"] {
            assert_eq!(
                workbuddy_status_from_protocol(status),
                Some(AgentActivityStatus::Error)
            );
        }
        assert_eq!(workbuddy_status_from_protocol("deleted"), None);
    }

    #[test]
    fn transcript_activity_exposes_only_an_opaque_local_identifier() {
        let path = temporary_test_file(
            "rollout-private-session-id.jsonl",
            r#"{"type":"event_msg","payload":{"type":"task_started"}}
{"type":"event_msg","payload":{"type":"task_complete"}}"#,
        );

        let activity = activity_from_transcript(&path, AgentActivityKind::Codex, true)
            .expect("completed transcript should produce one activity");

        assert_eq!(activity.status, AgentActivityStatus::Finish);
        assert!(activity.id.starts_with("codex-"));
        assert!(!activity.id.contains("private-session-id"));
        fs::remove_dir_all(path.parent().expect("test file should have a parent"))
            .expect("temporary activity directory should be removable");
    }

    #[test]
    fn workbuddy_snapshot_keeps_supported_conversations_and_ignores_deleted_entries() {
        let activities = workbuddy_activities_from_snapshot(
            r#"{"conversation-1":"planning","conversation-2":"pending","conversation-3":"failed","conversation-4":"deleted"}"#,
            42,
        );

        assert_eq!(activities.len(), 3);
        assert_eq!(activities[0].updated_at_ms, 42);
        assert!(activities
            .iter()
            .all(|activity| activity.agent == AgentActivityKind::WorkBuddy));
        assert!(activities
            .iter()
            .all(|activity| !activity.id.contains("conversation-")));
    }
}
