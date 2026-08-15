use crate::adapters::agent::AgentAdapter;
use crate::domain::agent_run::{AgentRunMetricsCollector, AgentRunOutput, TokenUsage};
use crate::error::AppError;
use crate::platform::claude_config::claude_settings_path;
use serde::Deserialize;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::time::{Duration, Instant};

const CLAUDE_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const MAX_CLAUDE_SETTINGS_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;
const MAX_RUNTIME_SETTINGS_RESOLUTION_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClaudeAuthentication {
    /// Indicates whether a usable Claude Code executable was found locally.
    pub(crate) installed: bool,
    /// Indicates whether Claude Code reports active credentials.
    pub(crate) logged_in: bool,
    /// Safe authentication mode parsed from Claude Code's status response.
    pub(crate) authentication_method: Option<String>,
    /// Effective model read from the local Claude settings.
    pub(crate) model: Option<String>,
    /// Effective reasoning effort read from the local Claude settings.
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait ClaudeAdapter {
    fn check_authentication(&self) -> Result<ClaudeAuthentication, AppError>;
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SystemClaudeAdapter {
    /// Shared cache of effective runtime settings used by authentication probes.
    runtime_settings_cache: ClaudeRuntimeSettingsCache,
}

impl SystemClaudeAdapter {
    pub(crate) fn new(runtime_settings_cache: ClaudeRuntimeSettingsCache) -> Self {
        Self {
            runtime_settings_cache,
        }
    }
}

impl ClaudeAdapter for SystemClaudeAdapter {
    /// Detects Claude Code and parses its structured authentication status response.
    ///
    /// Claude may return useful logged-out JSON with a non-zero exit status, so parsing is tried
    /// before falling back to the normalized logged-out state.
    fn check_authentication(&self) -> Result<ClaudeAuthentication, AppError> {
        let executable = match resolve_claude_executable() {
            Ok(executable) => executable,
            Err(AppError::ClaudeNotInstalled) => {
                self.runtime_settings_cache.invalidate();
                return Ok(not_installed_authentication());
            }
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

        let mut authentication = if output.status.success() {
            authentication_from_status(&stdout)?
        } else {
            authentication_from_status(&stdout).unwrap_or_else(|_| logged_out_authentication())
        };

        if authentication.logged_in {
            let runtime_settings = self
                .runtime_settings_cache
                .resolve(|| Ok(read_claude_runtime_settings()))?;
            authentication.model = runtime_settings.model;
            authentication.reasoning_effort = runtime_settings.reasoning_effort;
        } else {
            self.runtime_settings_cache.invalidate();
        }

        Ok(authentication)
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
    /// Login state reported by `claude auth status`.
    logged_in: bool,
    /// Authentication method reported by `claude auth status`.
    auth_method: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeSettings {
    /// Model explicitly selected in the user-level Claude settings.
    model: Option<String>,
    /// Thinking effort explicitly selected in the user-level Claude settings.
    effort_level: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClaudeRuntimeSettings {
    /// Effective model parsed from the local settings file.
    model: Option<String>,
    /// Effective reasoning effort parsed from the local settings file.
    reasoning_effort: Option<String>,
}

#[derive(Debug, Clone)]
struct CachedClaudeRuntimeSettings {
    /// Cache revision at which the runtime settings were resolved.
    revision: u64,
    /// Runtime settings retained for the matching revision.
    value: ClaudeRuntimeSettings,
}

#[derive(Debug, Default)]
struct ClaudeRuntimeSettingsCacheState {
    /// Indicates whether native file watching makes cached values safe to serve.
    enabled: AtomicBool,
    /// Generation incremented whenever the watched settings may have changed.
    revision: AtomicU64,
    /// Cached settings paired with the generation that produced them.
    value: Mutex<Option<CachedClaudeRuntimeSettings>>,
}

/// Shares parsed Claude model and effort settings across authentication probes.
#[derive(Debug, Clone, Default)]
pub(crate) struct ClaudeRuntimeSettingsCache {
    /// Thread-safe state shared by every clone of the cache handle.
    state: Arc<ClaudeRuntimeSettingsCacheState>,
}

impl ClaudeRuntimeSettingsCache {
    pub(crate) fn enable(&self) {
        self.state.enabled.store(true, Ordering::Release);
    }

    pub(crate) fn disable(&self) {
        self.state.enabled.store(false, Ordering::Release);
        self.invalidate();
    }

    pub(crate) fn invalidate(&self) {
        self.state.revision.fetch_add(1, Ordering::AcqRel);
        *self.lock_value() = None;
    }

    fn resolve(
        &self,
        mut resolver: impl FnMut() -> Result<ClaudeRuntimeSettings, AppError>,
    ) -> Result<ClaudeRuntimeSettings, AppError> {
        if !self.state.enabled.load(Ordering::Acquire) {
            return resolver();
        }

        for _ in 0..MAX_RUNTIME_SETTINGS_RESOLUTION_ATTEMPTS {
            let revision = self.state.revision.load(Ordering::Acquire);
            if let Some(cached) = self
                .lock_value()
                .as_ref()
                .filter(|cached| cached.revision == revision)
            {
                return Ok(cached.value.clone());
            }

            let resolved = resolver()?;
            let mut cached_value = self.lock_value();
            if self.state.revision.load(Ordering::Acquire) == revision {
                *cached_value = Some(CachedClaudeRuntimeSettings {
                    revision,
                    value: resolved.clone(),
                });
                return Ok(resolved);
            }
        }

        resolver()
    }

    fn lock_value(&self) -> MutexGuard<'_, Option<CachedClaudeRuntimeSettings>> {
        match self.state.value.lock() {
            Ok(value) => value,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    /// Top-level Claude stream message discriminator.
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
    /// Low-level event discriminator from the Claude streaming protocol.
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
    /// Input tokens written into Claude's prompt cache.
    #[serde(default)]
    cache_creation_input_tokens: u64,
    /// Input tokens served from Claude's prompt cache.
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
                // Present the implementation-specific OAuth token label as the account type the
                // user recognizes in the UI.
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

/// Reads only the bounded user-level fields needed for the runtime status card.
fn read_claude_runtime_settings() -> ClaudeRuntimeSettings {
    let Some(path) = claude_settings_path() else {
        return ClaudeRuntimeSettings::default();
    };
    let content = match read_bounded_claude_settings(&path) {
        Ok(Some(content)) => content,
        Ok(None) | Err(()) => return ClaudeRuntimeSettings::default(),
    };

    runtime_settings_from_json(&content).unwrap_or_default()
}

fn read_bounded_claude_settings(path: &Path) -> Result<Option<String>, ()> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CLAUDE_SETTINGS_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 > MAX_CLAUDE_SETTINGS_BYTES {
        return Err(());
    }

    String::from_utf8(bytes).map(Some).map_err(|_| ())
}

fn runtime_settings_from_json(content: &str) -> Result<ClaudeRuntimeSettings, ()> {
    let settings: ClaudeSettings = serde_json::from_str(content).map_err(|_| ())?;

    Ok(ClaudeRuntimeSettings {
        model: non_empty_setting(settings.model),
        reasoning_effort: non_empty_setting(settings.effort_level),
    })
}

fn non_empty_setting(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

/// Resolves the first Claude executable candidate whose version command succeeds.
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

        if message.message_type == "assistant" {
            // Full assistant messages contain stable tool ids; partial stream events do not span
            // the actual execution, so they are unsuitable for tool-duration measurement.
            for content in message
                .message
                .map(|message| message.content)
                .unwrap_or_default()
            {
                if content.content_type == "tool_use" {
                    if content.name.as_deref() == Some("AskUserQuestion") {
                        return Err(AppError::ClaudeNeedsInput);
                    }
                    if let (Some(id), Some(name)) = (content.id, content.name) {
                        collector.record_tool_started(&id, &name, started_at.elapsed());
                    }
                }
            }
            continue;
        }

        if message.message_type == "user" {
            // Claude returns tool results as user content with the originating tool_use id.
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
                // Content-block indexes are stable for the lifetime of one thinking block.
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
    // Tauri desktop processes may inherit a smaller PATH than an interactive macOS shell.
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
    use super::{
        authentication_from_status, collect_claude_events, runtime_settings_from_json,
        ClaudeRuntimeSettingsCache, StreamUsage,
    };
    use crate::domain::agent_run::TokenUsage;
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
    fn reads_model_and_effort_from_claude_settings() {
        let settings = runtime_settings_from_json(
            r#"{
                "model": "claude-sonnet-4-6",
                "effortLevel": "high",
                "permissions": { "allow": ["Read"] }
            }"#,
        )
        .expect("valid settings should provide runtime configuration");

        assert_eq!(settings.model.as_deref(), Some("claude-sonnet-4-6"));
        assert_eq!(settings.reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    fn rejects_invalid_claude_settings_without_fabricating_runtime_values() {
        assert!(runtime_settings_from_json("{invalid").is_err());
    }

    #[test]
    fn caches_claude_runtime_settings_until_configuration_is_invalidated() {
        let cache = ClaudeRuntimeSettingsCache::default();
        cache.enable();
        let mut resolution_count = 0;
        let mut resolve = || {
            resolution_count += 1;
            Ok(super::ClaudeRuntimeSettings {
                model: Some(format!("model-{resolution_count}")),
                reasoning_effort: Some("high".to_string()),
            })
        };

        let first = cache
            .resolve(&mut resolve)
            .expect("initial Claude settings should resolve");
        let cached = cache
            .resolve(&mut resolve)
            .expect("Claude settings should come from the cache");
        cache.invalidate();
        let refreshed = cache
            .resolve(&mut resolve)
            .expect("invalidated Claude settings should resolve again");

        assert_eq!(first.model.as_deref(), Some("model-1"));
        assert_eq!(cached.model.as_deref(), Some("model-1"));
        assert_eq!(refreshed.model.as_deref(), Some("model-2"));
        assert_eq!(resolution_count, 2);
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
    fn records_claude_thinking_and_tool_use_messages() {
        let (sender, receiver) = mpsc::sync_channel(6);
        for fixture in [
            r#"{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"thinking"}}}"#,
            r#"{"type":"stream_event","event":{"type":"content_block_stop","index":0}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Read"}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_1"}]}}"#,
            r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"OK"}}}"#,
            r#"{"type":"result","subtype":"success","is_error":false,"result":"OK"}"#,
        ] {
            sender
                .send(Ok(fixture.to_string()))
                .expect("fixture should be queued");
        }

        let output = collect_claude_events(&receiver, Instant::now())
            .expect("valid tool lifecycle should complete");

        assert_eq!(output.metrics.tool_calls.len(), 1);
        assert_eq!(output.metrics.tool_calls[0].name, "Read");
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
