use crate::adapters::agent::AgentAdapter;
use crate::domain::codex_run::{AgentRunMetricsCollector, AgentRunOutput, TokenUsage};
use crate::error::AppError;
use serde::Deserialize;
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

const WORKBUDDY_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const WORKBUDDY_PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkBuddyAuthentication {
    pub(crate) installed: bool,
    pub(crate) logged_in: bool,
    pub(crate) authentication_method: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait WorkBuddyAdapter {
    fn check_authentication(&self) -> Result<WorkBuddyAuthentication, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemWorkBuddyAdapter;

impl WorkBuddyAdapter for SystemWorkBuddyAdapter {
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
    #[serde(rename = "type")]
    message_type: String,
    subtype: Option<String>,
    event: Option<StreamEvent>,
    result: Option<String>,
    usage: Option<StreamUsage>,
    is_error: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamUsage {
    input_tokens: u64,
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct AcpMessage {
    id: Option<u64>,
    result: Option<AcpResult>,
    error: Option<AcpError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpResult {
    session_id: Option<String>,
    models: Option<AcpModels>,
    config_options: Option<Vec<AcpConfigOption>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpModels {
    current_model_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcpConfigOption {
    id: String,
    current_value: String,
}

#[derive(Debug, Deserialize)]
struct AcpError {}

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
    let mut child = Command::new(executable)
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
        .stderr(Stdio::null())
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

        if message.message_type == "stream_event" {
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
    let mut candidates = vec![OsString::from("codebuddy"), OsString::from("cbc")];

    #[cfg(target_os = "macos")]
    candidates.push(OsString::from(
        "/Applications/WorkBuddy AI.app/Contents/Resources/app.asar.unpacked/cli/bin/codebuddy",
    ));

    candidates
}

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

fn initialize_acp_session(
    stdin: &mut ChildStdin,
    event_receiver: &Receiver<Result<String, AppError>>,
) -> Result<WorkBuddyAuthentication, AppError> {
    write_acp_message(
        stdin,
        r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"agent-gauge","version":"0.1.0"}}}"#,
    )?;
    wait_for_acp_response(event_receiver, 0)?;
    let cwd = std::env::current_dir().map_err(|_| AppError::WorkBuddyProbeFailed)?;
    let session_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "session/new",
        "params": {"cwd": cwd, "mcpServers": []}
    });
    write_acp_message(stdin, &session_request.to_string())?;
    let response = wait_for_acp_response(event_receiver, 1)?;

    authentication_from_acp_response(response)
}

fn authentication_from_acp_response(
    response: AcpMessage,
) -> Result<WorkBuddyAuthentication, AppError> {
    if response.error.is_some() {
        return Ok(WorkBuddyAuthentication {
            installed: true,
            logged_in: false,
            authentication_method: None,
            model: None,
            reasoning_effort: None,
        });
    }

    let result = response.result.ok_or(AppError::WorkBuddyProbeFailed)?;
    let logged_in = result.session_id.is_some();
    let reasoning_effort = result.config_options.and_then(|options| {
        options
            .into_iter()
            .find(|option| option.id == "thought_level")
            .map(|option| option.current_value)
    });

    Ok(WorkBuddyAuthentication {
        installed: true,
        logged_in,
        authentication_method: logged_in.then(|| "WorkBuddy account".to_string()),
        model: result.models.map(|models| models.current_model_id),
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
        authentication_from_acp_response, collect_workbuddy_events, AcpMessage, StreamUsage,
    };
    use crate::domain::codex_run::TokenUsage;
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
    fn reads_model_and_thought_level_from_the_acp_session() {
        let response: AcpMessage = serde_json::from_str(
            r#"{"id":1,"result":{"sessionId":"session-1","models":{"currentModelId":"fast-model"},"configOptions":[{"id":"thought_level","currentValue":"enabled"}]}}"#,
        )
        .expect("fixture should deserialize");

        let authentication = authentication_from_acp_response(response)
            .expect("valid session should produce authentication state");

        assert!(authentication.installed);
        assert!(authentication.logged_in);
        assert_eq!(authentication.model.as_deref(), Some("fast-model"));
        assert_eq!(authentication.reasoning_effort.as_deref(), Some("enabled"));
    }
}
