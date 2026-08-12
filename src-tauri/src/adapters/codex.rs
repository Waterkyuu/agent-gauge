use crate::domain::codex_run::{CodexRunMetricsCollector, CodexRunOutput, TokenUsage};
use crate::error::AppError;
use serde::Deserialize;
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

const APP_SERVER_START_TIMEOUT: Duration = Duration::from_secs(30);
const CODEX_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexAuthentication {
    pub(crate) installed: bool,
    pub(crate) logged_in: bool,
    pub(crate) authentication_method: Option<String>,
}

pub(crate) trait CodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError>;
    fn run_task(&self, query: &str) -> Result<CodexRunOutput, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemCodexAdapter;

impl CodexAdapter for SystemCodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError> {
        for executable in codex_executable_candidates() {
            let output = Command::new(&executable)
                .args(["login", "status"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();

            match output {
                Ok(output) => {
                    let authentication_method = output.status.success().then(|| {
                        String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .strip_prefix("Logged in using ")
                            .unwrap_or("authenticated credentials")
                            .to_string()
                    });

                    return Ok(CodexAuthentication {
                        installed: true,
                        logged_in: output.status.success(),
                        authentication_method,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(_) => return Err(AppError::CodexProbeFailed),
            }
        }

        Ok(CodexAuthentication {
            installed: false,
            logged_in: false,
            authentication_method: None,
        })
    }

    fn run_task(&self, query: &str) -> Result<CodexRunOutput, AppError> {
        let executable = resolve_codex_executable()?;
        let mut child = Command::new(executable)
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| AppError::CodexProtocolFailed)?;
        let stdout = child.stdout.take().ok_or(AppError::CodexProtocolFailed)?;
        let mut stdin = child.stdin.take().ok_or(AppError::CodexProtocolFailed)?;
        let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let reader_handle = thread::spawn(move || read_app_server_events(stdout, event_sender));
        let task_result = run_app_server_task(&mut stdin, &event_receiver, query);
        let termination_result = terminate_child(&mut child);
        let reader_result = reader_handle
            .join()
            .map_err(|_| AppError::CodexProtocolFailed);

        let output = task_result?;
        termination_result?;
        reader_result?;
        Ok(output)
    }
}

#[derive(Debug, Deserialize)]
struct AppServerMessage {
    id: Option<u64>,
    method: Option<String>,
    params: Option<AppServerParams>,
    result: Option<AppServerResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppServerParams {
    delta: Option<String>,
    token_usage: Option<ThreadTokenUsage>,
    turn: Option<AppServerTurn>,
}

#[derive(Debug, Deserialize)]
struct AppServerResult {
    thread: Option<AppServerThread>,
}

#[derive(Debug, Deserialize)]
struct AppServerThread {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AppServerTurn {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ThreadTokenUsage {
    last: TokenUsageBreakdown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageBreakdown {
    total_tokens: u64,
    input_tokens: u64,
    cached_input_tokens: u64,
    cache_write_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: u64,
}

impl From<TokenUsageBreakdown> for TokenUsage {
    fn from(usage: TokenUsageBreakdown) -> Self {
        Self {
            total_tokens: usage.total_tokens,
            input_tokens: usage.input_tokens,
            cached_input_tokens: usage.cached_input_tokens,
            cache_write_input_tokens: usage.cache_write_input_tokens,
            output_tokens: usage.output_tokens,
            reasoning_output_tokens: usage.reasoning_output_tokens,
        }
    }
}

fn resolve_codex_executable() -> Result<OsString, AppError> {
    for executable in codex_executable_candidates() {
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
            Err(_) => return Err(AppError::CodexProbeFailed),
        }
    }

    Err(AppError::CodexProbeFailed)
}

fn run_app_server_task(
    stdin: &mut ChildStdin,
    event_receiver: &Receiver<Result<String, AppError>>,
    query: &str,
) -> Result<CodexRunOutput, AppError> {
    write_message(
        stdin,
        r#"{"method":"initialize","id":0,"params":{"clientInfo":{"name":"agent_gauge","title":"AgentGauge","version":"0.1.0"}}}"#,
    )?;
    wait_for_response(event_receiver, 0, APP_SERVER_START_TIMEOUT)?;
    write_message(stdin, r#"{"method":"initialized","params":{}}"#)?;
    write_message(
        stdin,
        r#"{"method":"thread/start","id":1,"params":{"approvalPolicy":"never","sandbox":"workspace-write","ephemeral":true,"serviceName":"agent_gauge"}}"#,
    )?;
    let thread_response = wait_for_response(event_receiver, 1, APP_SERVER_START_TIMEOUT)?;
    let thread_id = thread_response
        .result
        .and_then(|result| result.thread)
        .map(|thread| thread.id)
        .ok_or(AppError::CodexProtocolFailed)?;
    let turn_request = serde_json::json!({
        "method": "turn/start",
        "id": 2,
        "params": {
            "threadId": thread_id,
            "input": [{"type": "text", "text": query}]
        }
    });
    let started_at = Instant::now();
    write_message(stdin, &turn_request.to_string())?;

    collect_run_events(event_receiver, started_at)
}

fn collect_run_events(
    event_receiver: &Receiver<Result<String, AppError>>,
    started_at: Instant,
) -> Result<CodexRunOutput, AppError> {
    let mut collector = CodexRunMetricsCollector::default();
    let mut response = String::new();

    loop {
        let remaining = CODEX_RUN_TIMEOUT
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::CodexTimedOut)?;
        let line = receive_line(event_receiver, remaining)?;
        let message: AppServerMessage =
            serde_json::from_str(&line).map_err(|_| AppError::CodexProtocolFailed)?;

        match message.method.as_deref() {
            Some("item/agentMessage/delta") => {
                if let Some(delta) = message.params.and_then(|params| params.delta) {
                    collector.record_agent_delta(&delta, started_at.elapsed());
                    response.push_str(&delta);
                }
            }
            Some("thread/tokenUsage/updated") => {
                if let Some(usage) = message.params.and_then(|params| params.token_usage) {
                    collector.record_token_usage(usage.last.into());
                }
            }
            Some("turn/completed") => {
                let completed = message
                    .params
                    .and_then(|params| params.turn)
                    .is_some_and(|turn| turn.status == "completed");
                if !completed {
                    return Err(AppError::CodexTaskFailed);
                }

                return Ok(CodexRunOutput {
                    response,
                    metrics: collector.finish(started_at.elapsed()),
                });
            }
            _ => {}
        }
    }
}

fn wait_for_response(
    event_receiver: &Receiver<Result<String, AppError>>,
    response_id: u64,
    timeout: Duration,
) -> Result<AppServerMessage, AppError> {
    let started_at = Instant::now();

    loop {
        let remaining = timeout
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::CodexTimedOut)?;
        let line = receive_line(event_receiver, remaining)?;
        let message: AppServerMessage =
            serde_json::from_str(&line).map_err(|_| AppError::CodexProtocolFailed)?;
        if message.id == Some(response_id) {
            return Ok(message);
        }
    }
}

fn receive_line(
    event_receiver: &Receiver<Result<String, AppError>>,
    timeout: Duration,
) -> Result<String, AppError> {
    match event_receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(AppError::CodexTimedOut),
        Err(RecvTimeoutError::Disconnected) => Err(AppError::CodexProtocolFailed),
    }
}

fn read_app_server_events(
    stdout: impl io::Read,
    event_sender: SyncSender<Result<String, AppError>>,
) {
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
                    .send(Err(AppError::CodexProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
            Ok(_) => {
                let line = String::from_utf8(bytes).map_err(|_| AppError::CodexProtocolFailed);
                if event_sender.send(line).is_err() {
                    break;
                }
            }
            Err(_) => {
                if event_sender
                    .send(Err(AppError::CodexProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

fn write_message(stdin: &mut ChildStdin, message: &str) -> Result<(), AppError> {
    stdin
        .write_all(message.as_bytes())
        .and_then(|()| stdin.write_all(b"\n"))
        .and_then(|()| stdin.flush())
        .map_err(|_| AppError::CodexProtocolFailed)
}

fn terminate_child(child: &mut Child) -> Result<(), AppError> {
    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
        Err(_) => return Err(AppError::CodexProtocolFailed),
    }
    child
        .wait()
        .map(|_| ())
        .map_err(|_| AppError::CodexProtocolFailed)
}

fn codex_executable_candidates() -> Vec<OsString> {
    let mut candidates = vec![OsString::from("codex")];

    #[cfg(target_os = "macos")]
    candidates.push(OsString::from(
        "/Applications/Codex.app/Contents/Resources/codex",
    ));

    candidates
}
