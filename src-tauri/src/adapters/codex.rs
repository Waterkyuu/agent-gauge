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

const APP_SERVER_START_TIMEOUT: Duration = Duration::from_secs(30);
const CODEX_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexAuthentication {
    pub(crate) installed: bool,
    pub(crate) logged_in: bool,
    pub(crate) authentication_method: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait CodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError>;
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
                    let logged_in = output.status.success();
                    let authentication_method = logged_in.then(|| {
                        String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .strip_prefix("Logged in using ")
                            .unwrap_or("authenticated credentials")
                            .to_string()
                    });
                    let runtime_defaults = logged_in
                        .then(|| resolve_codex_runtime_defaults(&executable))
                        .transpose()?;

                    return Ok(CodexAuthentication {
                        installed: true,
                        logged_in,
                        authentication_method,
                        model: runtime_defaults
                            .as_ref()
                            .map(|defaults| defaults.model.clone()),
                        reasoning_effort: runtime_defaults
                            .and_then(|defaults| defaults.reasoning_effort),
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
            model: None,
            reasoning_effort: None,
        })
    }
}

impl AgentAdapter for SystemCodexAdapter {
    fn run_task(&self, query: &str) -> Result<AgentRunOutput, AppError> {
        let executable = resolve_codex_executable()?;
        with_app_server(&executable, |stdin, event_receiver| {
            run_app_server_task(stdin, event_receiver, query)
        })
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
#[serde(rename_all = "camelCase")]
struct AppServerResult {
    thread: Option<AppServerThread>,
    model: Option<String>,
    reasoning_effort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppServerThread {
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexRuntimeDefaults {
    thread_id: String,
    model: String,
    reasoning_effort: Option<String>,
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
            reasoning_output_tokens: Some(usage.reasoning_output_tokens),
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

fn resolve_codex_runtime_defaults(executable: &OsStr) -> Result<CodexRuntimeDefaults, AppError> {
    with_app_server(executable, initialize_app_server_thread)
}

fn with_app_server<T>(
    executable: &OsStr,
    operation: impl FnOnce(&mut ChildStdin, &Receiver<Result<String, AppError>>) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let mut child = Command::new(executable)
        .args(["app-server", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| AppError::CodexProtocolFailed)?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::CodexProtocolFailed);
        }
    };
    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::CodexProtocolFailed);
        }
    };
    let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
    let reader_handle = thread::spawn(move || read_app_server_events(stdout, event_sender));
    let operation_result = operation(&mut stdin, &event_receiver);
    let termination_result = terminate_child(&mut child);
    let reader_result = reader_handle
        .join()
        .map_err(|_| AppError::CodexProtocolFailed);

    let output = operation_result?;
    termination_result?;
    reader_result?;
    Ok(output)
}

fn initialize_app_server_thread(
    stdin: &mut ChildStdin,
    event_receiver: &Receiver<Result<String, AppError>>,
) -> Result<CodexRuntimeDefaults, AppError> {
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
    let result = thread_response
        .result
        .ok_or(AppError::CodexProtocolFailed)?;

    Ok(CodexRuntimeDefaults {
        thread_id: result
            .thread
            .map(|thread| thread.id)
            .ok_or(AppError::CodexProtocolFailed)?,
        model: result.model.ok_or(AppError::CodexProtocolFailed)?,
        reasoning_effort: result.reasoning_effort,
    })
}

fn run_app_server_task(
    stdin: &mut ChildStdin,
    event_receiver: &Receiver<Result<String, AppError>>,
    query: &str,
) -> Result<AgentRunOutput, AppError> {
    let runtime_defaults = initialize_app_server_thread(stdin, event_receiver)?;
    let turn_request = serde_json::json!({
        "method": "turn/start",
        "id": 2,
        "params": {
            "threadId": runtime_defaults.thread_id,
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
) -> Result<AgentRunOutput, AppError> {
    let mut collector = AgentRunMetricsCollector::default();
    let mut response = String::new();

    loop {
        let remaining = CODEX_RUN_TIMEOUT
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::CodexTimedOut)?;
        let line = receive_line(event_receiver, remaining)?;
        let message: AppServerMessage =
            serde_json::from_str(&line).map_err(|_| AppError::CodexProtocolFailed)?;

        match message.method.as_deref() {
            Some(
                "tool/requestUserInput"
                | "item/tool/requestUserInput"
                | "mcpServer/elicitation/request",
            ) => return Err(AppError::CodexNeedsInput),
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

                return Ok(AgentRunOutput {
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
    candidates.extend([
        OsString::from("/Applications/Codex.app/Contents/Resources/codex"),
        OsString::from("/Applications/ChatGPT.app/Contents/Resources/codex"),
    ]);

    candidates
}

#[cfg(test)]
mod tests {
    use super::{codex_executable_candidates, collect_run_events};
    use crate::error::AppError;
    use std::sync::mpsc;
    use std::time::Instant;

    #[cfg(target_os = "macos")]
    #[test]
    fn discovers_codex_bundled_with_the_chatgpt_desktop_app() {
        let candidates = codex_executable_candidates();

        assert!(candidates.iter().any(|candidate| {
            candidate == "/Applications/ChatGPT.app/Contents/Resources/codex"
        }));
    }

    #[test]
    fn reports_when_codex_requests_user_input() {
        for method in [
            "tool/requestUserInput",
            "item/tool/requestUserInput",
            "mcpServer/elicitation/request",
        ] {
            let (sender, receiver) = mpsc::sync_channel(1);
            sender
                .send(Ok(format!(
                    r#"{{"method":"{method}","id":7,"params":{{}}}}"#
                )))
                .expect("fixture should be queued");
            drop(sender);

            let result = collect_run_events(&receiver, Instant::now());

            assert_eq!(result, Err(AppError::CodexNeedsInput));
        }
    }
}
