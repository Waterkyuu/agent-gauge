use crate::adapters::agent::AgentAdapter;
use crate::domain::codex_run::{AgentRunMetricsCollector, AgentRunOutput, TokenUsage};
use crate::error::AppError;
use serde::Deserialize;
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

const CLAUDE_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClaudeAuthentication {
    pub(crate) installed: bool,
    pub(crate) logged_in: bool,
    pub(crate) authentication_method: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait ClaudeAdapter {
    fn check_authentication(&self) -> Result<ClaudeAuthentication, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemClaudeAdapter;

impl ClaudeAdapter for SystemClaudeAdapter {
    fn check_authentication(&self) -> Result<ClaudeAuthentication, AppError> {
        let executable = match resolve_claude_executable() {
            Ok(executable) => executable,
            Err(AppError::ClaudeNotInstalled) => return Ok(not_installed_authentication()),
            Err(error) => return Err(error),
        };
        let output = Command::new(executable)
            .args(["auth", "status", "--json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| AppError::ClaudeProbeFailed)?;
        let stdout = String::from_utf8(output.stdout).map_err(|_| AppError::ClaudeProbeFailed)?;

        if output.status.success() {
            return authentication_from_status(&stdout);
        }

        authentication_from_status(&stdout).or_else(|_| Ok(logged_out_authentication()))
    }
}

impl AgentAdapter for SystemClaudeAdapter {
    fn run_task(&self, query: &str) -> Result<AgentRunOutput, AppError> {
        let executable = resolve_claude_executable()?;
        run_claude_task(&executable, query)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthStatus {
    logged_in: bool,
    auth_method: Option<String>,
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
    content_block: Option<StreamContentBlock>,
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    name: Option<String>,
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

impl From<StreamUsage> for TokenUsage {
    fn from(usage: StreamUsage) -> Self {
        let input_tokens = usage
            .input_tokens
            .saturating_add(usage.cache_creation_input_tokens)
            .saturating_add(usage.cache_read_input_tokens);

        Self {
            total_tokens: input_tokens.saturating_add(usage.output_tokens),
            input_tokens,
            cached_input_tokens: usage.cache_read_input_tokens,
            cache_write_input_tokens: usage.cache_creation_input_tokens,
            output_tokens: usage.output_tokens,
            reasoning_output_tokens: None,
        }
    }
}

fn not_installed_authentication() -> ClaudeAuthentication {
    ClaudeAuthentication {
        installed: false,
        logged_in: false,
        authentication_method: None,
        model: None,
        reasoning_effort: None,
    }
}

fn logged_out_authentication() -> ClaudeAuthentication {
    ClaudeAuthentication {
        installed: true,
        logged_in: false,
        authentication_method: None,
        model: None,
        reasoning_effort: None,
    }
}

fn authentication_from_status(status: &str) -> Result<ClaudeAuthentication, AppError> {
    let status: AuthStatus =
        serde_json::from_str(status).map_err(|_| AppError::ClaudeProbeFailed)?;
    let authentication_method = status.logged_in.then(|| {
        status
            .auth_method
            .filter(|method| !method.is_empty())
            .map(|method| match method.as_str() {
                "oauth_token" => "Claude account".to_string(),
                _ => method,
            })
            .unwrap_or_else(|| "authenticated credentials".to_string())
    });

    Ok(ClaudeAuthentication {
        installed: true,
        logged_in: status.logged_in,
        authentication_method,
        model: None,
        reasoning_effort: None,
    })
}

fn resolve_claude_executable() -> Result<OsString, AppError> {
    for executable in claude_executable_candidates() {
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
            Err(_) => return Err(AppError::ClaudeProbeFailed),
        }
    }

    Err(AppError::ClaudeNotInstalled)
}

fn run_claude_task(executable: &OsStr, query: &str) -> Result<AgentRunOutput, AppError> {
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
            "plan",
            "--no-session-persistence",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| AppError::ClaudeProtocolFailed)?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child)?;
            return Err(AppError::ClaudeProtocolFailed);
        }
    };
    let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
    let reader_handle = thread::spawn(move || read_stream_events(stdout, event_sender));
    let result = collect_claude_events(&event_receiver, started_at);

    if result.is_err() {
        terminate_child(&mut child)?;
    } else {
        let status = child.wait().map_err(|_| AppError::ClaudeProtocolFailed)?;
        if !status.success() {
            return Err(AppError::ClaudeTaskFailed);
        }
    }
    reader_handle
        .join()
        .map_err(|_| AppError::ClaudeProtocolFailed)?;

    result
}

fn collect_claude_events(
    event_receiver: &Receiver<Result<String, AppError>>,
    started_at: Instant,
) -> Result<AgentRunOutput, AppError> {
    let mut collector = AgentRunMetricsCollector::default();
    let mut response = String::new();

    loop {
        let remaining = CLAUDE_RUN_TIMEOUT
            .checked_sub(started_at.elapsed())
            .ok_or(AppError::ClaudeTimedOut)?;
        let line = receive_line(event_receiver, remaining)?;
        let message: StreamMessage =
            serde_json::from_str(&line).map_err(|_| AppError::ClaudeProtocolFailed)?;

        if message.message_type == "stream_event" {
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
                return Err(AppError::ClaudeNeedsInput);
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
                return Err(AppError::ClaudeTaskFailed);
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
        Err(RecvTimeoutError::Timeout) => Err(AppError::ClaudeTimedOut),
        Err(RecvTimeoutError::Disconnected) => Err(AppError::ClaudeProtocolFailed),
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
                    .send(Err(AppError::ClaudeProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
            Ok(_) => {
                let line = String::from_utf8(bytes).map_err(|_| AppError::ClaudeProtocolFailed);
                if event_sender.send(line).is_err() {
                    break;
                }
            }
            Err(_) => {
                if event_sender
                    .send(Err(AppError::ClaudeProtocolFailed))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

fn terminate_child(child: &mut Child) -> Result<(), AppError> {
    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
        Err(_) => return Err(AppError::ClaudeProtocolFailed),
    }
    child
        .wait()
        .map(|_| ())
        .map_err(|_| AppError::ClaudeProtocolFailed)
}

fn claude_executable_candidates() -> Vec<OsString> {
    let mut candidates = vec![OsString::from("claude")];

    #[cfg(target_os = "macos")]
    candidates.extend([
        OsString::from("/usr/local/bin/claude"),
        OsString::from("/opt/homebrew/bin/claude"),
    ]);

    candidates
}

#[cfg(test)]
mod tests {
    use super::{authentication_from_status, collect_claude_events, StreamUsage};
    use crate::domain::codex_run::TokenUsage;
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn reads_authenticated_claude_account_status() {
        let authentication = authentication_from_status(
            r#"{"loggedIn":true,"authMethod":"oauth_token","apiProvider":"firstParty"}"#,
        )
        .expect("valid status should produce authentication state");

        assert!(authentication.installed);
        assert!(authentication.logged_in);
        assert_eq!(
            authentication.authentication_method.as_deref(),
            Some("Claude account")
        );
    }

    #[test]
    fn normalizes_claude_cache_tokens_into_total_input() {
        let usage = TokenUsage::from(StreamUsage {
            input_tokens: 120,
            output_tokens: 30,
            cache_creation_input_tokens: 2_000,
            cache_read_input_tokens: 800,
        });

        assert_eq!(usage.total_tokens, 2_950);
        assert_eq!(usage.input_tokens, 2_920);
        assert_eq!(usage.cached_input_tokens, 800);
        assert_eq!(usage.cache_write_input_tokens, 2_000);
        assert_eq!(usage.reasoning_output_tokens, None);
    }

    #[test]
    fn collects_streamed_claude_text_and_final_usage() {
        let (sender, receiver) = mpsc::sync_channel(3);
        sender
            .send(Ok(
                r#"{"type":"system","subtype":"init","model":"claude-sonnet-4-5"}"#.to_string(),
            ))
            .expect("fixture should be queued");
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"OK"}}}"#.to_string()))
            .expect("fixture should be queued");
        sender
            .send(Ok(r#"{"type":"result","subtype":"success","is_error":false,"result":"OK","usage":{"input_tokens":10,"output_tokens":2,"cache_creation_input_tokens":7,"cache_read_input_tokens":3}}"#.to_string()))
            .expect("fixture should be queued");

        let output =
            collect_claude_events(&receiver, Instant::now()).expect("valid stream should complete");

        assert_eq!(output.response, "OK");
        assert!(output.metrics.time_to_first_token.is_some());
        assert_eq!(
            output.metrics.token_usage.map(|usage| usage.total_tokens),
            Some(22)
        );
    }

    #[test]
    fn reports_when_claude_asks_the_user_a_question() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Ok(r#"{"type":"stream_event","event":{"type":"content_block_start","content_block":{"type":"tool_use","name":"AskUserQuestion"}}}"#.to_string()))
            .expect("fixture should be queued");
        drop(sender);

        let result = collect_claude_events(&receiver, Instant::now());

        assert_eq!(result, Err(crate::error::AppError::ClaudeNeedsInput));
    }
}
