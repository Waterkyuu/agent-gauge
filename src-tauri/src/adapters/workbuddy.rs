use crate::adapters::agent::AgentAdapter;
use crate::domain::agent_run::{AgentRunMetricsCollector, AgentRunOutput, TokenUsage};
use crate::error::AppError;
use leveldb_forensic::{decode_local_storage, LocalStorageRecord};
use serde::Deserialize;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

const WORKBUDDY_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const WORKBUDDY_PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;
const WORKBUDDY_GLOBAL_MODEL_KEY_PREFIX: &str = "cb-newtask:model";
const MAX_WORKBUDDY_LOCAL_STORAGE_BYTES: u64 = 16 * 1024 * 1024;
const ACP_AUTH_REQUIRED_CODE: i64 = -32000;
const JSON_RPC_METHOD_NOT_FOUND_CODE: i64 = -32601;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkBuddyGlobalSelection {
    /// Model identifier selected for newly created WorkBuddy tasks.
    id: String,
    /// Indicates whether thinking is enabled for the selected model.
    is_thinking: bool,
    /// Explicit thinking effort selected when WorkBuddy supplies one.
    reasoning_effort: Option<String>,
}

/// Model configuration read from WorkBuddy's LevelDB-backed Local Storage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkBuddyConfigSnapshot {
    /// Model identifier selected for newly created WorkBuddy tasks.
    pub(crate) model: Option<String>,
    /// Effective thinking level derived from the selected model configuration.
    pub(crate) reasoning_effort: Option<String>,
}

impl WorkBuddyGlobalSelection {
    fn thought_level(&self) -> &str {
        if !self.is_thinking {
            return "disabled";
        }

        self.reasoning_effort.as_deref().unwrap_or("enabled")
    }
}

fn global_selection_from_local_storage(
    records: &[LocalStorageRecord],
) -> Option<WorkBuddyGlobalSelection> {
    let (_, deleted, value) = records
        .iter()
        .filter_map(|record| match record {
            LocalStorageRecord::Data {
                origin,
                script_key,
                value,
                seq,
                deleted,
            } if origin == "file://"
                && !script_key.lossy
                && (script_key.text == WORKBUDDY_GLOBAL_MODEL_KEY_PREFIX
                    || script_key
                        .text
                        .strip_prefix(WORKBUDDY_GLOBAL_MODEL_KEY_PREFIX)
                        .is_some_and(|suffix| suffix.starts_with(':'))) =>
            {
                Some((*seq, *deleted, value))
            }
            _ => None,
        })
        .max_by_key(|(seq, _, _)| *seq)?;

    if deleted || value.lossy {
        return None;
    }

    serde_json::from_str(&value.text).ok()
}

pub(crate) fn workbuddy_local_storage_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".workbuddy-ai")
            .join("app")
            .join("session")
            .join("Local Storage")
            .join("leveldb")
    })
}

fn is_bounded_workbuddy_local_storage(path: &Path) -> bool {
    if !fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()) {
        return false;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    let mut total_bytes = 0_u64;

    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        let Ok(file_type) = entry.file_type() else {
            return false;
        };
        if file_type.is_symlink() {
            return false;
        }
        if !file_type.is_file()
            || !entry
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("ldb")
                        || extension.eq_ignore_ascii_case("sst")
                        || extension.eq_ignore_ascii_case("log")
                })
        {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            return false;
        };
        total_bytes = total_bytes.saturating_add(metadata.len());
        if total_bytes > MAX_WORKBUDDY_LOCAL_STORAGE_BYTES {
            return false;
        }
    }

    true
}

fn read_workbuddy_global_selection() -> Option<WorkBuddyGlobalSelection> {
    let path = workbuddy_local_storage_path()?;
    read_workbuddy_global_selection_from_path(&path).ok()?
}

fn read_workbuddy_global_selection_from_path(
    path: &Path,
) -> Result<Option<WorkBuddyGlobalSelection>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    if !is_bounded_workbuddy_local_storage(path) {
        return Err(AppError::WorkBuddyConfigReadFailed);
    }
    let records = decode_local_storage(path).map_err(|_| AppError::WorkBuddyConfigReadFailed)?;

    Ok(global_selection_from_local_storage(&records))
}

/// Reads the model and thinking level currently selected for new WorkBuddy tasks.
pub(crate) fn read_workbuddy_config() -> Result<WorkBuddyConfigSnapshot, AppError> {
    let path = workbuddy_local_storage_path().ok_or(AppError::WorkBuddyConfigReadFailed)?;
    let selection = read_workbuddy_global_selection_from_path(&path)?;

    Ok(
        selection.map_or_else(WorkBuddyConfigSnapshot::default, |selection| {
            WorkBuddyConfigSnapshot {
                reasoning_effort: Some(selection.thought_level().to_string()),
                model: Some(selection.id),
            }
        }),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkBuddyAuthentication {
    /// Indicates whether a usable WorkBuddy application or CLI was found locally.
    pub(crate) installed: bool,
    /// Indicates whether WorkBuddy accepted an authenticated ACP session.
    pub(crate) logged_in: bool,
    /// Safe authentication mode derived from the ACP user response.
    pub(crate) authentication_method: Option<String>,
    /// Effective model selected for newly created WorkBuddy tasks.
    pub(crate) model: Option<String>,
    /// Effective reasoning effort selected for newly created WorkBuddy tasks.
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait WorkBuddyAdapter {
    fn check_authentication(&self) -> Result<WorkBuddyAuthentication, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemWorkBuddyAdapter;

impl WorkBuddyAdapter for SystemWorkBuddyAdapter {
    /// Detects WorkBuddy and opens a temporary ACP session to verify account access.
    ///
    /// WorkBuddy does not expose a separate authentication-status command; successful session
    /// creation is therefore the authoritative local login signal.
    fn check_authentication(&self) -> Result<WorkBuddyAuthentication, AppError> {
        let executable = match resolve_workbuddy_executable() {
            Ok(executable) => executable,
            Err(AppError::WorkBuddyNotInstalled) => {
                return Ok(WorkBuddyAuthentication {
                    installed: false,
                    logged_in: false,
                    authentication_method: None,
                    model: None,
                    reasoning_effort: None,
                });
            }
            Err(error) => return Err(error),
        };

        probe_workbuddy_runtime(&executable)
    }
}

impl AgentAdapter for SystemWorkBuddyAdapter {
    fn run_task(&self, query: &str) -> Result<AgentRunOutput, AppError> {
        let executable = resolve_workbuddy_executable()?;
        run_workbuddy_task(&executable, query)
    }
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    /// Top-level WorkBuddy stream message discriminator.
    #[serde(rename = "type")]
    message_type: String,
    /// Optional subtype that refines result and control messages.
    subtype: Option<String>,
    /// Low-level streaming event carried by this message.
    event: Option<StreamEvent>,
    /// Final assistant response carried by a result message.
    result: Option<String>,
    /// Token usage carried by a result message.
    usage: Option<StreamUsage>,
    /// Indicates whether a result message represents a failed task.
    is_error: Option<bool>,
    /// Full conversation message carried by an assistant message event.
    message: Option<StreamConversationMessage>,
}

#[derive(Debug, Deserialize)]
struct StreamEvent {
    /// Low-level event discriminator from the WorkBuddy streaming protocol.
    #[serde(rename = "type")]
    event_type: String,
    /// Content block announced by a block-start event.
    content_block: Option<StreamContentBlock>,
    /// Incremental content carried by a block-delta event.
    delta: Option<StreamDelta>,
    /// Source content-block position associated with the event.
    index: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct StreamContentBlock {
    /// Content-block discriminator such as thinking or tool_use.
    #[serde(rename = "type")]
    block_type: String,
    /// Tool name when the block represents a tool invocation.
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamConversationMessage {
    /// Full assistant/user content emitted between low-level stream events.
    #[serde(default)]
    content: Vec<StreamConversationContent>,
}

#[derive(Debug, Deserialize)]
struct StreamConversationContent {
    /// Content discriminator such as tool_use or tool_result.
    #[serde(rename = "type")]
    content_type: String,
    /// Unique identifier present on a tool_use block.
    id: Option<String>,
    /// Tool name present on a tool_use block.
    name: Option<String>,
    /// Identifier that links a tool_result back to its tool_use block.
    tool_use_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    /// Delta discriminator that identifies the incremental content kind.
    #[serde(rename = "type")]
    delta_type: Option<String>,
    /// Incremental assistant text when supplied by the delta.
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamUsage {
    /// Non-cache input tokens reported for the task.
    input_tokens: u64,
    /// Tokens generated in the model output.
    output_tokens: u64,
    /// Input tokens written into WorkBuddy's prompt cache.
    #[serde(default)]
    cache_creation_input_tokens: u64,
    /// Input tokens served from WorkBuddy's prompt cache.
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct AcpMessage {
    /// JSON-RPC response identifier.
    id: Option<u64>,
    /// Successful ACP response payload.
    result: Option<AcpResult>,
    /// Structured ACP failure payload.
    error: Option<AcpError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpResult {
    /// Session identifier returned after successful ACP initialization.
    session_id: Option<String>,
    /// Available and currently selected ACP models.
    models: Option<AcpModels>,
    /// Runtime configuration choices exposed by WorkBuddy.
    config_options: Option<Vec<AcpConfigOption>>,
    /// Authenticated user information returned by ACP.
    user_info: Option<AcpUserInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpUserInfo {
    /// Stable user identifier proving that ACP returned an authenticated account.
    user_id: String,
    /// Authentication type reported for the current account.
    auth_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpModels {
    /// Models that the current ACP session can select.
    #[serde(default)]
    available_models: Vec<AcpModel>,
    /// Identifier of the model selected for the current ACP session.
    current_model_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpModel {
    /// Stable identifier used to select the model through ACP.
    model_id: String,
    /// User-visible model label reported by WorkBuddy.
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpConfigOption {
    /// Stable identifier of the configurable runtime option.
    id: String,
    /// Value selected for this runtime option.
    current_value: String,
}

#[derive(Debug, Deserialize)]
struct AcpError {
    /// JSON-RPC error code returned by WorkBuddy ACP.
    code: i64,
}

impl From<StreamUsage> for TokenUsage {
    fn from(usage: StreamUsage) -> Self {
        Self {
            total_tokens: usage.input_tokens.saturating_add(usage.output_tokens),
            input_tokens: usage.input_tokens,
            cached_input_tokens: usage.cache_read_input_tokens,
            cache_write_input_tokens: usage.cache_creation_input_tokens,
            output_tokens: usage.output_tokens,
            reasoning_output_tokens: None,
        }
    }
}

fn resolve_workbuddy_executable() -> Result<OsString, AppError> {
    for executable in workbuddy_executable_candidates() {
        match Command::new(&executable)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) if status.success() => return Ok(executable),
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return Err(AppError::WorkBuddyProbeFailed),
        }
    }

    Err(AppError::WorkBuddyNotInstalled)
}

fn run_workbuddy_task(executable: &OsStr, query: &str) -> Result<AgentRunOutput, AppError> {
    let started_at = Instant::now();
    let global_selection = read_workbuddy_global_selection();
    let mut child = build_workbuddy_task_command(executable, query, global_selection.as_ref())
        .spawn()
        .map_err(|_| AppError::WorkBuddyProtocolFailed)?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::WorkBuddyProtocolFailed);
        }
    };
    let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
    let reader_handle = thread::spawn(move || read_stream_events(stdout, event_sender));
    let result = collect_workbuddy_events(&event_receiver, started_at);

    if result.is_err() {
        terminate_child(&mut child)?;
    } else {
        let status = child
            .wait()
            .map_err(|_| AppError::WorkBuddyProtocolFailed)?;
        if !status.success() {
            return Err(AppError::WorkBuddyTaskFailed);
        }
    }
    reader_handle
        .join()
        .map_err(|_| AppError::WorkBuddyProtocolFailed)?;

    result
}

fn build_workbuddy_task_command(
    executable: &OsStr,
    query: &str,
    global_selection: Option<&WorkBuddyGlobalSelection>,
) -> Command {
    let mut command = Command::new(executable);
    if let Some(selection) = global_selection {
        command.args(["--model", selection.id.as_str()]);
        if selection.is_thinking {
            if let Some(effort) = selection.reasoning_effort.as_deref().filter(|effort| {
                matches!(
                    *effort,
                    "minimal" | "low" | "medium" | "high" | "xhigh" | "max"
                )
            }) {
                command.args(["--effort", effort]);
            }
        }
    }
    command
        .args([
            "--print",
            query,
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--verbose",
            "--permission-mode",
            "acceptEdits",
            "--no-session-persistence",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command
}

fn collect_workbuddy_events(
    event_receiver: &Receiver<Result<String, AppError>>,
    started_at: Instant,
) -> Result<AgentRunOutput, AppError> {
    let mut collector = AgentRunMetricsCollector::default();
    let mut response = String::new();

    loop {
        let remaining = WORKBUDDY_RUN_TIMEOUT
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::WorkBuddyTimedOut)?;
        let line = receive_line(event_receiver, remaining)?;
        let message: StreamMessage =
            serde_json::from_str(&line).map_err(|_| AppError::WorkBuddyProtocolFailed)?;

        if message.message_type == "assistant" {
            // WorkBuddy mirrors the stream-json tool lifecycle but keeps its own collector state.
            for content in message
                .message
                .map(|message| message.content)
                .unwrap_or_default()
            {
                if content.content_type == "tool_use" {
                    if content.name.as_deref() == Some("AskUserQuestion") {
                        return Err(AppError::WorkBuddyNeedsInput);
                    }
                    if let (Some(id), Some(name)) = (content.id, content.name) {
                        collector.record_tool_started(&id, &name, started_at.elapsed());
                    }
                }
            }
            continue;
        }

        if message.message_type == "user" {
            // A tool_result closes only the matching tool_use id, including concurrent calls.
            for content in message
                .message
                .map(|message| message.content)
                .unwrap_or_default()
            {
                if content.content_type == "tool_result" {
                    if let Some(id) = content.tool_use_id {
                        collector.record_tool_finished(&id, started_at.elapsed());
                    }
                }
            }
            continue;
        }

        if message.message_type == "stream_event" {
            if let Some(event) = message.event.as_ref() {
                // Thinking blocks use stream indexes because they have no tool-style identifier.
                let interval_id = format!("thinking-{}", event.index.unwrap_or(0));
                if event.event_type == "content_block_start"
                    && event
                        .content_block
                        .as_ref()
                        .is_some_and(|block| block.block_type == "thinking")
                {
                    collector.record_thinking_started(&interval_id, started_at.elapsed());
                } else if event.event_type == "content_block_stop" {
                    collector.record_thinking_finished(&interval_id, started_at.elapsed());
                }
            }
            if message
                .event
                .as_ref()
                .filter(|event| event.event_type == "content_block_start")
                .and_then(|event| event.content_block.as_ref())
                .is_some_and(|block| {
                    block.block_type == "tool_use"
                        && block.name.as_deref() == Some("AskUserQuestion")
                })
            {
                return Err(AppError::WorkBuddyNeedsInput);
            }
            if let Some(delta) = message
                .event
                .filter(|event| event.event_type == "content_block_delta")
                .and_then(|event| event.delta)
                .filter(|delta| delta.delta_type.as_deref() == Some("text_delta"))
                .and_then(|delta| delta.text)
            {
                collector.record_agent_delta(&delta, started_at.elapsed());
                response.push_str(&delta);
            }
            continue;
        }

        if message.message_type == "result" {
            let succeeded =
                message.subtype.as_deref() == Some("success") && message.is_error == Some(false);
            if !succeeded {
                return Err(AppError::WorkBuddyTaskFailed);
            }
            if let Some(usage) = message.usage {
                collector.record_token_usage(usage.into());
            }
            if response.is_empty() {
                response = message.result.unwrap_or_default();
            }
            return Ok(AgentRunOutput {
                response,
                metrics: collector.finish(started_at.elapsed()),
            });
        }
    }
}

fn receive_line(
    event_receiver: &Receiver<Result<String, AppError>>,
    timeout: Duration,
) -> Result<String, AppError> {
    match event_receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(AppError::WorkBuddyTimedOut),
        Err(RecvTimeoutError::Disconnected) => Err(AppError::WorkBuddyProtocolFailed),
    }
}

fn read_stream_events(stdout: impl io::Read, event_sender: SyncSender<Result<String, AppError>>) {
    let mut reader = BufReader::new(stdout);

    loop {
        let mut bytes = Vec::new();
        let result = reader
            .by_ref()
            .take(MAX_EVENT_BYTES + 1)
            .read_until(b'\n', &mut bytes);
        match result {
            Ok(0) => break,
            Ok(_) if bytes.len() as u64 > MAX_EVENT_BYTES => {
                if event_sender
                    .send(Err(AppError::WorkBuddyProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
            Ok(_) => {
                let line = String::from_utf8(bytes).map_err(|_| AppError::WorkBuddyProtocolFailed);
                if event_sender.send(line).is_err() {
                    break;
                }
            }
            Err(_) => {
                if event_sender
                    .send(Err(AppError::WorkBuddyProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

fn workbuddy_executable_candidates() -> Vec<OsString> {
    // The product has shipped under both CLI names, and the desktop bundle may not modify PATH.
    let mut candidates = vec![OsString::from("codebuddy"), OsString::from("cbc")];

    #[cfg(target_os = "macos")]
    candidates.push(OsString::from(
        "/Applications/WorkBuddy AI.app/Contents/Resources/app.asar.unpacked/cli/bin/codebuddy",
    ));

    candidates
}

/// Runs the minimum ACP exchange needed to determine whether WorkBuddy can create a session.
///
/// The probe owns this child process and terminates it after receiving the authentication result;
/// leaving it alive would make the separate process monitor report a false running state.
fn probe_workbuddy_runtime(executable: &OsStr) -> Result<WorkBuddyAuthentication, AppError> {
    let mut child = Command::new(executable)
        .arg("--acp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| AppError::WorkBuddyProbeFailed)?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::WorkBuddyProbeFailed);
        }
    };
    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::WorkBuddyProbeFailed);
        }
    };
    let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
    let reader_handle = thread::spawn(move || read_stream_events(stdout, event_sender));
    let probe_result = initialize_acp_session(&mut stdin, &event_receiver);
    terminate_child(&mut child)?;
    reader_handle
        .join()
        .map_err(|_| AppError::WorkBuddyProtocolFailed)?;

    probe_result
}

/// Initializes ACP, requests a disposable session, and normalizes its runtime configuration.
fn initialize_acp_session(
    stdin: &mut ChildStdin,
    event_receiver: &Receiver<Result<String, AppError>>,
) -> Result<WorkBuddyAuthentication, AppError> {
    write_acp_message(
        stdin,
        r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"agent-gauge","version":"0.1.0"}}}"#,
    )?;
    let initialize_response = wait_for_acp_response(event_receiver, 0)?;
    if initialize_response.error.is_some() {
        return Err(AppError::WorkBuddyProbeFailed);
    }
    // CodeBuddy's read-only extension reports the current account without creating a session.
    write_acp_message(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"_codebuddy.ai/getUserInfo","params":{}}"#,
    )?;
    let user_info_response = wait_for_acp_response(event_receiver, 1)?;
    let global_selection = read_workbuddy_global_selection();
    if let Some(authentication) =
        authentication_from_user_info_response(user_info_response, global_selection.as_ref())?
    {
        return Ok(authentication);
    }

    // Older CodeBuddy releases may not expose getUserInfo. In that case only, create a
    // disposable session as a compatibility probe and read its effective configuration.
    let cwd = std::env::current_dir().map_err(|_| AppError::WorkBuddyProbeFailed)?;
    let session_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {"cwd": cwd, "mcpServers": []}
    });
    write_acp_message(stdin, &session_request.to_string())?;
    let response = wait_for_acp_response(event_receiver, 2)?;

    authentication_from_acp_response(response, global_selection.as_ref())
}

/// Converts CodeBuddy's read-only account response; `None` requests the legacy session fallback.
fn authentication_from_user_info_response(
    response: AcpMessage,
    global_selection: Option<&WorkBuddyGlobalSelection>,
) -> Result<Option<WorkBuddyAuthentication>, AppError> {
    if let Some(error) = response.error {
        return if error.code == JSON_RPC_METHOD_NOT_FOUND_CODE {
            Ok(None)
        } else {
            Err(AppError::WorkBuddyProbeFailed)
        };
    }

    let user_info = response
        .result
        .ok_or(AppError::WorkBuddyProbeFailed)?
        .user_info;
    let logged_in = user_info
        .as_ref()
        .is_some_and(|user| !user.user_id.trim().is_empty());
    let authentication_method = user_info
        .and_then(|user| user.auth_type)
        .filter(|auth_type| !auth_type.trim().is_empty())
        .or_else(|| logged_in.then(|| "WorkBuddy account".to_string()));
    let model = logged_in
        .then(|| global_selection.map(|selection| selection.id.clone()))
        .flatten();
    let reasoning_effort = logged_in
        .then(|| global_selection.map(|selection| selection.thought_level().to_string()))
        .flatten();

    Ok(Some(WorkBuddyAuthentication {
        installed: true,
        logged_in,
        authentication_method,
        model,
        reasoning_effort,
    }))
}

fn authentication_from_acp_response(
    response: AcpMessage,
    global_selection: Option<&WorkBuddyGlobalSelection>,
) -> Result<WorkBuddyAuthentication, AppError> {
    if let Some(error) = response.error {
        if error.code == ACP_AUTH_REQUIRED_CODE {
            return Ok(WorkBuddyAuthentication {
                installed: true,
                logged_in: false,
                authentication_method: None,
                model: None,
                reasoning_effort: None,
            });
        }
        return Err(AppError::WorkBuddyProbeFailed);
    }

    let result = response.result.ok_or(AppError::WorkBuddyProbeFailed)?;
    // ACP returns a session identifier only after the local runtime accepts the active account.
    let logged_in = result.session_id.is_some();
    let model = result.models.map(|models| {
        let model_id = global_selection
            .map(|selection| selection.id.as_str())
            .unwrap_or(&models.current_model_id);
        models
            .available_models
            .into_iter()
            .find(|model| model.model_id == model_id)
            .map_or_else(|| model_id.to_string(), |model| model.name)
    });
    let reasoning_effort = global_selection
        .map(|selection| selection.thought_level().to_string())
        .or_else(|| {
            result.config_options.and_then(|options| {
                options
                    .into_iter()
                    .find(|option| option.id == "thought_level")
                    .map(|option| option.current_value)
            })
        });

    Ok(WorkBuddyAuthentication {
        installed: true,
        logged_in,
        authentication_method: logged_in.then(|| "WorkBuddy account".to_string()),
        model,
        reasoning_effort,
    })
}

fn wait_for_acp_response(
    event_receiver: &Receiver<Result<String, AppError>>,
    response_id: u64,
) -> Result<AcpMessage, AppError> {
    let started_at = Instant::now();

    loop {
        let remaining = WORKBUDDY_PROBE_TIMEOUT
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::WorkBuddyProbeFailed)?;
        let line = receive_line(event_receiver, remaining)?;
        let json_start = line.find('{').ok_or(AppError::WorkBuddyProtocolFailed)?;
        let message: AcpMessage = serde_json::from_str(&line[json_start..])
            .map_err(|_| AppError::WorkBuddyProtocolFailed)?;
        if message.id == Some(response_id) {
            return Ok(message);
        }
    }
}

fn write_acp_message(stdin: &mut ChildStdin, message: &str) -> Result<(), AppError> {
    stdin
        .write_all(message.as_bytes())
        .and_then(|()| stdin.write_all(b"\n"))
        .and_then(|()| stdin.flush())
        .map_err(|_| AppError::WorkBuddyProtocolFailed)
}

fn terminate_child(child: &mut Child) -> Result<(), AppError> {
    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
        Err(_) => return Err(AppError::WorkBuddyProtocolFailed),
    }
    child
        .wait()
        .map(|_| ())
        .map_err(|_| AppError::WorkBuddyProtocolFailed)
}

#[cfg(test)]
mod tests {
    use super::{
        authentication_from_acp_response, authentication_from_user_info_response,
        build_workbuddy_task_command, collect_workbuddy_events,
        global_selection_from_local_storage, AcpMessage, StreamUsage,
    };
    use crate::domain::agent_run::TokenUsage;
    use leveldb_forensic::{Encoding, LocalStorageRecord, StorageValue};
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn normalizes_final_workbuddy_usage_without_double_counting_cache_tokens() {
        let usage = TokenUsage::from(StreamUsage {
            input_tokens: 6_311,
            output_tokens: 33,
            cache_creation_input_tokens: 6_197,
            cache_read_input_tokens: 114,
        });

        assert_eq!(usage.total_tokens, 6_344);
        assert_eq!(usage.input_tokens, 6_311);
        assert_eq!(usage.cached_input_tokens, 114);
        assert_eq!(usage.cache_write_input_tokens, 6_197);
        assert_eq!(usage.reasoning_output_tokens, None);
    }

    #[test]
    fn collects_first_text_delta_and_completed_metrics() {
        let (sender, receiver) = mpsc::sync_channel(4);
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"thinking_delta","thinking":"hidden"}}}"#.to_string()))
            .expect("fixture should be queued");
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"OK"}}}"#.to_string()))
            .expect("fixture should be queued");
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"input_tokens":10,"output_tokens":2}}}"#.to_string()))
            .expect("fixture should be queued");
        sender
            .send(Ok(r#"{"type":"result","subtype":"success","is_error":false,"result":"OK","usage":{"input_tokens":10,"output_tokens":2,"cache_creation_input_tokens":7,"cache_read_input_tokens":3}}"#.to_string()))
            .expect("fixture should be queued");

        let output = collect_workbuddy_events(&receiver, Instant::now())
            .expect("valid stream should complete");

        assert_eq!(output.response, "OK");
        assert!(output.metrics.time_to_first_token.is_some());
        assert_eq!(
            output.metrics.token_usage.map(|usage| usage.total_tokens),
            Some(12)
        );
    }

    #[test]
    fn records_workbuddy_thinking_and_tool_use_messages() {
        let (sender, receiver) = mpsc::sync_channel(5);
        for fixture in [
            r#"{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"thinking"}}}"#,
            r#"{"type":"stream_event","event":{"type":"content_block_stop","index":0}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Bash"}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_1"}]}}"#,
            r#"{"type":"result","subtype":"success","is_error":false,"result":"OK"}"#,
        ] {
            sender
                .send(Ok(fixture.to_string()))
                .expect("fixture should be queued");
        }

        let output = collect_workbuddy_events(&receiver, Instant::now())
            .expect("valid tool lifecycle should complete");

        assert_eq!(output.metrics.tool_calls.len(), 1);
        assert_eq!(output.metrics.tool_calls[0].name, "Bash");
    }

    #[test]
    fn reads_model_name_and_thought_level_from_the_acp_session() {
        let response: AcpMessage = serde_json::from_str(
            r#"{"id":1,"result":{"sessionId":"session-1","models":{"availableModels":[{"modelId":"fast-model","name":"Fast"},{"modelId":"kimi-k3","name":"Kimi-K3"}],"currentModelId":"kimi-k3"},"configOptions":[{"id":"thought_level","currentValue":"enabled"}]}}"#,
        )
        .expect("fixture should deserialize");

        let authentication = authentication_from_acp_response(response, None)
            .expect("valid session should produce authentication state");

        assert!(authentication.installed);
        assert!(authentication.logged_in);
        assert_eq!(authentication.model.as_deref(), Some("Kimi-K3"));
        assert_eq!(authentication.reasoning_effort.as_deref(), Some("enabled"));
    }

    #[test]
    fn reads_login_state_without_creating_an_acp_session() {
        let response: AcpMessage = serde_json::from_str(
            r#"{"id":1,"result":{"userInfo":{"userId":"user-1","authType":"external"}}}"#,
        )
        .expect("fixture should deserialize");
        let global_selection = super::WorkBuddyGlobalSelection {
            id: "kimi-k3".to_string(),
            is_thinking: true,
            reasoning_effort: Some("high".to_string()),
        };

        let authentication =
            authentication_from_user_info_response(response, Some(&global_selection))
                .expect("valid user info should produce authentication state")
                .expect("supported user info method should not request a session fallback");

        assert!(authentication.logged_in);
        assert_eq!(
            authentication.authentication_method.as_deref(),
            Some("external")
        );
        assert_eq!(authentication.model.as_deref(), Some("kimi-k3"));
        assert_eq!(authentication.reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    fn reports_logged_out_when_acp_has_no_current_user() {
        let response: AcpMessage =
            serde_json::from_str(r#"{"id":1,"result":{}}"#).expect("fixture should deserialize");

        let authentication = authentication_from_user_info_response(response, None)
            .expect("empty user info should be a valid logged-out state")
            .expect("supported user info method should not request a session fallback");

        assert!(!authentication.logged_in);
        assert_eq!(authentication.authentication_method, None);
    }

    #[test]
    fn requests_session_fallback_when_user_info_method_is_unsupported() {
        let response: AcpMessage = serde_json::from_str(
            r#"{"id":1,"error":{"code":-32601,"message":"Method not found"}}"#,
        )
        .expect("fixture should deserialize");

        let authentication = authentication_from_user_info_response(response, None)
            .expect("method-not-found should select the compatibility fallback");

        assert_eq!(authentication, None);
    }

    #[test]
    fn preserves_non_authentication_acp_probe_failures() {
        let response: AcpMessage =
            serde_json::from_str(r#"{"id":1,"error":{"code":-32002,"message":"Internal error"}}"#)
                .expect("fixture should deserialize");

        let result = authentication_from_user_info_response(response, None);

        assert_eq!(result, Err(crate::error::AppError::WorkBuddyProbeFailed));
    }

    #[test]
    fn global_selection_overrides_new_acp_session_defaults() {
        let response: AcpMessage = serde_json::from_str(
            r#"{"id":1,"result":{"sessionId":"session-1","models":{"availableModels":[{"modelId":"fast-model","name":"Fast"},{"modelId":"kimi-k3","name":"Kimi-K3"}],"currentModelId":"fast-model"},"configOptions":[{"id":"thought_level","currentValue":"enabled"}]}}"#,
        )
        .expect("fixture should deserialize");
        let global_selection = super::WorkBuddyGlobalSelection {
            id: "kimi-k3".to_string(),
            is_thinking: true,
            reasoning_effort: None,
        };

        let authentication = authentication_from_acp_response(response, Some(&global_selection))
            .expect("valid session should produce authentication state");

        assert_eq!(authentication.model.as_deref(), Some("Kimi-K3"));
        assert_eq!(authentication.reasoning_effort.as_deref(), Some("enabled"));
    }

    #[test]
    fn task_command_uses_global_model_and_keeps_default_reasoning() {
        let selection = super::WorkBuddyGlobalSelection {
            id: "kimi-k3".to_string(),
            is_thinking: true,
            reasoning_effort: None,
        };

        let command =
            build_workbuddy_task_command("codebuddy".as_ref(), "test prompt", Some(&selection));
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--model", "kimi-k3"]));
        assert!(!args.iter().any(|arg| arg == "--effort"));
    }

    #[test]
    fn task_command_uses_explicit_global_reasoning() {
        let selection = super::WorkBuddyGlobalSelection {
            id: "kimi-k3".to_string(),
            is_thinking: true,
            reasoning_effort: Some("high".to_string()),
        };

        let command =
            build_workbuddy_task_command("codebuddy".as_ref(), "test prompt", Some(&selection));
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--effort", "high"]));
    }

    #[test]
    fn treats_the_latest_global_model_tombstone_as_deleted() {
        let records = vec![
            LocalStorageRecord::Data {
                origin: "file://".to_string(),
                script_key: StorageValue {
                    text: "cb-newtask:model:user-1".to_string(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                value: StorageValue {
                    text: r#"{"id":"fast-model","isThinking":true}"#.to_string(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                seq: 4,
                deleted: false,
            },
            LocalStorageRecord::Data {
                origin: "file://".to_string(),
                script_key: StorageValue {
                    text: "cb-newtask:model:user-1".to_string(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                value: StorageValue {
                    text: r#"{"id":"kimi-k3","isThinking":true}"#.to_string(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                seq: 9,
                deleted: false,
            },
            LocalStorageRecord::Data {
                origin: "file://".to_string(),
                script_key: StorageValue {
                    text: "cb-newtask:model:user-1".to_string(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                value: StorageValue {
                    text: String::new(),
                    raw: Vec::new(),
                    encoding: Encoding::Latin1,
                    lossy: false,
                },
                seq: 10,
                deleted: true,
            },
        ];

        let selection = global_selection_from_local_storage(&records);

        assert_eq!(selection, None);
    }

    #[test]
    fn reports_when_workbuddy_asks_the_user_a_question() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"content_block_start","content_block":{"type":"tool_use","name":"AskUserQuestion"}}}"#.to_string()))
            .expect("fixture should be queued");
        drop(sender);

        let result = collect_workbuddy_events(&receiver, Instant::now());

        assert_eq!(result, Err(crate::error::AppError::WorkBuddyNeedsInput));
    }
}
