use crate::adapters::agent::AgentAdapter;
use crate::domain::agent_run::{AgentRunMetricsCollector, AgentRunOutput, TokenUsage};
use crate::error::AppError;
use crate::platform::codex_config::codex_config_paths;
use serde::Deserialize;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::time::{Duration, Instant};

const APP_SERVER_START_TIMEOUT: Duration = Duration::from_secs(30);
const CODEX_RUN_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_EVENT_BYTES: u64 = 1024 * 1024;
const MAX_CODEX_CONFIG_BYTES: u64 = 1024 * 1024;
const EVENT_QUEUE_CAPACITY: usize = 64;
const MAX_RUNTIME_DEFAULT_RESOLUTION_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexAuthentication {
    /// Indicates whether a usable Codex executable was found locally.
    pub(crate) installed: bool,
    /// Indicates whether the Codex CLI reports active credentials.
    pub(crate) logged_in: bool,
    /// Safe authentication mode parsed from the Codex CLI status output.
    pub(crate) authentication_method: Option<String>,
    /// Effective model selected for newly created Codex threads.
    pub(crate) model: Option<String>,
    /// Effective reasoning effort selected for newly created Codex threads.
    pub(crate) reasoning_effort: Option<String>,
}

pub(crate) trait CodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError>;
}

#[derive(Debug, Clone)]
pub(crate) struct SystemCodexAdapter {
    /// Shared cache of effective runtime defaults used by authentication probes.
    runtime_defaults_cache: CodexRuntimeDefaultsCache,
}

impl SystemCodexAdapter {
    pub(crate) fn new(runtime_defaults_cache: CodexRuntimeDefaultsCache) -> Self {
        Self {
            runtime_defaults_cache,
        }
    }
}

impl CodexAdapter for SystemCodexAdapter {
    /// Checks the installed, login, and effective runtime status of the local Codex CLI.
    ///
    /// The login-status service calls this method through [`CodexAdapter`]; the task execution path
    /// uses a separate executable check. Each candidate receives `codex login status` in order.
    /// Successfully starting any candidate proves that Codex is installed. Exit code zero means
    /// logged in, while a non-zero exit code means installed but logged out.
    ///
    /// Missing candidates are skipped so bundled macOS executables remain available as fallbacks.
    /// If every candidate is missing, this method clears cached runtime settings and returns an
    /// installed and login status of `false`. Any other process-start failure returns
    /// [`AppError::CodexProbeFailed`].
    ///
    /// For a logged-in installation, the returned [`CodexAuthentication`] also contains the safe
    /// authentication label plus the effective model and reasoning effort. Logged-out and missing
    /// installations clear those cached settings so later probes cannot display stale values.
    ///
    /// For example, a candidate that starts and exits non-zero returns `installed: true` with
    /// `logged_in: false`; a machine where every candidate is missing returns both fields as
    /// `false` without treating absence as a probe error.
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
                    // The official CLI contract states that `codex login status` exits with 0
                    // when credentials are present and prints the active authentication mode:
                    // https://learn.chatgpt.com/docs/developer-commands?surface=cli#cli-codex-login
                    let logged_in = output.status.success();
                    // Current Codex versions print, for example, `Logged in using ChatGPT`.
                    // Authentication-mode parsing is display-only; an unexpected format falls
                    // back to a safe generic label without changing the exit-code login result.
                    let authentication_method = logged_in.then(|| {
                        String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .strip_prefix("Logged in using ")
                            .unwrap_or("authenticated credentials")
                            .to_string()
                    });
                    // Explicit config values are authoritative for this display. App Server remains
                    // the fallback when either field cannot be resolved safely from local TOML.
                    let runtime_settings = if logged_in {
                        Some(
                            self.runtime_defaults_cache
                                .resolve(|| resolve_codex_runtime_settings(&executable))?,
                        )
                    } else {
                        self.runtime_defaults_cache.invalidate();
                        None
                    };

                    return Ok(CodexAuthentication {
                        installed: true,
                        logged_in,
                        authentication_method,
                        model: runtime_settings
                            .as_ref()
                            .map(|settings| settings.model.clone()),
                        reasoning_effort: runtime_settings
                            .and_then(|settings| settings.reasoning_effort),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(_) => return Err(AppError::CodexProbeFailed),
            }
        }

        self.runtime_defaults_cache.invalidate();
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
        let executable = find_usable_codex_executable()?;
        with_app_server(&executable, |stdin, event_receiver| {
            run_app_server_task(stdin, event_receiver, query)
        })
    }
}

#[derive(Debug, Deserialize)]
struct AppServerMessage {
    /// JSON-RPC request identifier when the message is a response.
    id: Option<u64>,
    /// App Server notification method when the message is an event.
    method: Option<String>,
    /// Event payload supplied by an App Server notification.
    params: Option<AppServerParams>,
    /// Successful JSON-RPC response payload.
    result: Option<AppServerResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppServerParams {
    /// Incremental assistant text emitted by a streaming notification.
    delta: Option<String>,
    /// Latest cumulative token usage for the active thread.
    token_usage: Option<ThreadTokenUsage>,
    /// Turn state emitted by a turn lifecycle notification.
    turn: Option<AppServerTurn>,
    /// Item state emitted by an item lifecycle notification.
    item: Option<AppServerThreadItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppServerThreadItem {
    /// Lifecycle identifier shared by item/started and item/completed.
    id: String,
    /// Discriminator for reasoning, command, MCP, and other app-server items.
    #[serde(rename = "type")]
    item_type: String,
    /// MCP server name when this item represents an MCP tool call.
    server: Option<String>,
    /// Tool name reported for MCP, dynamic, and collaboration calls.
    tool: Option<String>,
    /// Dynamic-tool namespace when one is present.
    namespace: Option<String>,
}

impl AppServerThreadItem {
    /// Normalizes every executable app-server item into one concise display name.
    fn tool_name(&self) -> Option<String> {
        match self.item_type.as_str() {
            "commandExecution" | "fileChange" | "webSearch" | "imageView" | "imageGeneration"
            | "sleep" => Some(self.item_type.clone()),
            "mcpToolCall" => Some(
                [self.server.as_deref(), self.tool.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            "dynamicToolCall" => Some(
                [self.namespace.as_deref(), self.tool.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            "collabAgentToolCall" => self.tool.clone(),
            _ => None,
        }
        .filter(|name| !name.is_empty())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppServerResult {
    /// Thread created by a successful thread/start response.
    thread: Option<AppServerThread>,
    /// Model resolved by a successful account or thread response.
    model: Option<String>,
    /// Reasoning effort resolved by a successful account or thread response.
    reasoning_effort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppServerThread {
    /// Stable thread identifier assigned by Codex App Server.
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexRuntimeDefaults {
    /// Temporary thread identifier used while resolving runtime defaults.
    thread_id: String,
    /// Model selected by App Server for the temporary thread.
    model: String,
    /// Reasoning effort selected by App Server for the temporary thread.
    reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexRuntimeSettings {
    /// Effective model after merging explicit configuration and App Server defaults.
    model: String,
    /// Effective reasoning effort after merging configuration layers.
    reasoning_effort: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CodexConfigLayer {
    /// Model explicitly configured in this TOML layer.
    model: Option<String>,
    /// Reasoning effort explicitly configured in this TOML layer.
    model_reasoning_effort: Option<String>,
}

#[derive(Debug, Clone)]
struct CachedCodexRuntimeDefaults {
    /// Cache revision at which the runtime settings were resolved.
    revision: u64,
    /// Effective runtime settings retained for the matching revision.
    value: CodexRuntimeSettings,
}

#[derive(Debug, Default)]
struct CodexRuntimeDefaultsCacheState {
    /// Indicates whether native file watching makes cached values safe to serve.
    enabled: AtomicBool,
    /// Generation incremented whenever watched configuration may have changed.
    revision: AtomicU64,
    /// Cached runtime settings paired with the generation that produced them.
    value: Mutex<Option<CachedCodexRuntimeDefaults>>,
}

/// Shares the latest effective model defaults across authentication probes.
///
/// The cache is enabled only after the native configuration watcher starts successfully. When
/// watching is unavailable, every probe resolves fresh defaults so the UI cannot become stale.
#[derive(Debug, Clone, Default)]
pub(crate) struct CodexRuntimeDefaultsCache {
    /// Thread-safe state shared by every clone of the cache handle.
    state: Arc<CodexRuntimeDefaultsCacheState>,
}

impl CodexRuntimeDefaultsCache {
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
        mut resolver: impl FnMut() -> Result<CodexRuntimeSettings, AppError>,
    ) -> Result<CodexRuntimeSettings, AppError> {
        if !self.state.enabled.load(Ordering::Acquire) {
            return resolver();
        }

        for _ in 0..MAX_RUNTIME_DEFAULT_RESOLUTION_ATTEMPTS {
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
                *cached_value = Some(CachedCodexRuntimeDefaults {
                    revision,
                    value: resolved.clone(),
                });
                return Ok(resolved);
            }
        }

        // Rapid consecutive writes can invalidate both bounded attempts. Return a fresh value
        // without caching it; the next scheduled probe can establish a stable cached snapshot.
        resolver()
    }

    fn lock_value(&self) -> MutexGuard<'_, Option<CachedCodexRuntimeDefaults>> {
        match self.state.value.lock() {
            Ok(value) => value,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AppServerTurn {
    /// Current turn lifecycle status reported by App Server.
    status: String,
}

#[derive(Debug, Deserialize)]
struct ThreadTokenUsage {
    /// Most recent cumulative usage snapshot for the active turn.
    last: TokenUsageBreakdown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenUsageBreakdown {
    /// Total input and output tokens reported by Codex.
    total_tokens: u64,
    /// All tokens included in model input.
    input_tokens: u64,
    /// Input tokens served from an existing cache entry.
    cached_input_tokens: u64,
    /// Input tokens written into the prompt cache.
    /// Codex CLI 0.144.3 no longer returns `cacheWriteInputTokens`, so its absence defaults to zero.
    #[serde(default)]
    cache_write_input_tokens: u64,
    /// Tokens generated in the model output.
    output_tokens: u64,
    /// Output tokens consumed by model reasoning.
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

fn find_usable_codex_executable() -> Result<OsString, AppError> {
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

/// Resolves explicit TOML settings first and delegates incomplete state to Codex itself.
fn resolve_codex_runtime_settings(executable: &OsStr) -> Result<CodexRuntimeSettings, AppError> {
    if let Some(settings) = read_codex_runtime_settings() {
        return Ok(settings);
    }

    resolve_codex_runtime_defaults(executable).map(|defaults| CodexRuntimeSettings {
        model: defaults.model,
        reasoning_effort: defaults.reasoning_effort,
    })
}

/// Reads bounded configuration layers in precedence order without exposing unrelated settings.
fn read_codex_runtime_settings() -> Option<CodexRuntimeSettings> {
    let mut contents = Vec::new();

    for path in codex_config_paths() {
        match read_bounded_codex_config(&path) {
            Ok(Some(content)) => contents.push(content),
            Ok(None) => {}
            Err(()) => return None,
        }
    }

    runtime_settings_from_config_layers(contents.iter().map(String::as_str))
}

fn read_bounded_codex_config(path: &Path) -> Result<Option<String>, ()> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CODEX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 > MAX_CODEX_CONFIG_BYTES {
        return Err(());
    }

    String::from_utf8(bytes).map(Some).map_err(|_| ())
}

fn runtime_settings_from_config_layers<'a>(
    layers: impl IntoIterator<Item = &'a str>,
) -> Option<CodexRuntimeSettings> {
    let mut model = None;
    let mut reasoning_effort = None;

    for content in layers {
        let layer: CodexConfigLayer = toml::from_str(content).ok()?;
        if let Some(value) = non_empty_config_value(layer.model) {
            model = Some(value);
        }
        if let Some(value) = non_empty_config_value(layer.model_reasoning_effort) {
            reasoning_effort = Some(value);
        }
    }

    Some(CodexRuntimeSettings {
        model: model?,
        reasoning_effort: Some(reasoning_effort?),
    })
}

fn non_empty_config_value(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

/// Starts an ephemeral App Server session to resolve the model defaults used by new Codex tasks.
fn resolve_codex_runtime_defaults(executable: &OsStr) -> Result<CodexRuntimeDefaults, AppError> {
    with_app_server(executable, initialize_app_server_thread)
}

/// Runs one bounded App Server exchange and always terminates the child before returning.
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
    // npm-installed Codex uses a launcher process. Closing its inherited input also lets the
    // native App Server exit, so the stdout reader cannot outlive the launcher indefinitely.
    drop(stdin);
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
            Some("item/started") => {
                if let Some(item) = message.params.and_then(|params| params.item) {
                    let elapsed = started_at.elapsed();
                    // Reasoning is timed separately; all executable item kinds enter the tool list.
                    if item.item_type == "reasoning" {
                        collector.record_thinking_started(&item.id, elapsed);
                    } else if let Some(name) = item.tool_name() {
                        collector.record_tool_started(&item.id, &name, elapsed);
                    }
                }
            }
            Some("item/completed") => {
                if let Some(item) = message.params.and_then(|params| params.item) {
                    let elapsed = started_at.elapsed();
                    // The stable item id prevents concurrent calls from being paired incorrectly.
                    if item.item_type == "reasoning" {
                        collector.record_thinking_finished(&item.id, elapsed);
                    } else if item.tool_name().is_some() {
                        collector.record_tool_finished(&item.id, elapsed);
                    }
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
    // GUI installations may bundle Codex without placing it on the desktop app's PATH.
    let mut candidates = vec![OsString::from("codex")];

    #[cfg(target_os = "macos")]
    candidates.extend([
        // OpenAI's troubleshooting guide documents this retained compatibility path for the
        // Codex executable bundled with the ChatGPT desktop app:
        // https://learn.chatgpt.com/docs/reference/troubleshooting.md
        OsString::from("/Applications/Codex.app/Contents/Resources/codex"),
        // Current ChatGPT builds also place the executable here. Keep this as an observed fallback;
        // the official guide above does not promise this internal bundle layout as a stable path.
        OsString::from("/Applications/ChatGPT.app/Contents/Resources/codex"),
    ]);

    candidates
}

#[cfg(test)]
mod tests {
    use super::{
        codex_executable_candidates, collect_run_events, runtime_settings_from_config_layers,
        with_app_server, CodexRuntimeDefaultsCache, CodexRuntimeSettings,
    };
    use crate::error::AppError;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[cfg(unix)]
    #[test]
    fn closes_app_server_stdin_before_waiting_for_the_stdout_reader() {
        use std::os::unix::fs::PermissionsExt;

        let script_path = std::env::temp_dir().join(format!(
            "agent-gauge-codex-wrapper-test-{}",
            std::process::id()
        ));
        std::fs::write(
            &script_path,
            r#"#!/bin/sh
cat <&0 &
reader_pid=$!
(
  sleep 2
  kill "$reader_pid" 2>/dev/null
) </dev/null >/dev/null 2>&1 &
echo READY
wait "$reader_pid"
"#,
        )
        .expect("wrapper fixture should be written");
        let mut permissions = std::fs::metadata(&script_path)
            .expect("wrapper fixture metadata should be readable")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script_path, permissions)
            .expect("wrapper fixture should be executable");

        let started_at = Instant::now();
        let result = with_app_server(script_path.as_os_str(), |_stdin, receiver| {
            let ready = super::receive_line(receiver, Duration::from_secs(1))?;
            assert_eq!(ready.trim(), "READY");
            Ok(())
        });
        let elapsed = started_at.elapsed();
        std::fs::remove_file(script_path).expect("wrapper fixture should be removable");

        assert_eq!(result, Ok(()));
        assert!(
            elapsed < Duration::from_secs(1),
            "cleanup waited for the detached server process: {elapsed:?}"
        );
    }

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

    #[test]
    fn records_codex_reasoning_and_tool_item_lifecycles() {
        let (sender, receiver) = mpsc::sync_channel(5);
        for fixture in [
            r#"{"method":"item/started","params":{"item":{"id":"reason-1","type":"reasoning"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"reason-1","type":"reasoning"}}}"#,
            r#"{"method":"item/started","params":{"item":{"id":"tool-1","type":"mcpToolCall","server":"github","tool":"search"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"tool-1","type":"mcpToolCall","server":"github","tool":"search"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"status":"completed"}}}"#,
        ] {
            sender
                .send(Ok(fixture.to_string()))
                .expect("fixture should be queued");
        }

        let output = collect_run_events(&receiver, Instant::now())
            .expect("valid item lifecycle should complete");

        assert_eq!(output.metrics.tool_calls.len(), 1);
        assert_eq!(output.metrics.tool_calls[0].name, "github.search");
    }

    #[test]
    fn accepts_token_usage_without_cache_write_tokens() {
        let (sender, receiver) = mpsc::sync_channel(3);
        for fixture in [
            r#"{"method":"item/agentMessage/delta","params":{"delta":"OK"}}"#,
            r#"{"method":"thread/tokenUsage/updated","params":{"tokenUsage":{"last":{"totalTokens":16400,"inputTokens":16395,"cachedInputTokens":9984,"outputTokens":5,"reasoningOutputTokens":0}}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"status":"completed"}}}"#,
        ] {
            sender
                .send(Ok(fixture.to_string()))
                .expect("fixture should be queued");
        }

        let output = collect_run_events(&receiver, Instant::now())
            .expect("current Codex token usage should complete");
        let usage = output
            .metrics
            .token_usage
            .expect("token usage should be retained");

        assert_eq!(output.response, "OK");
        assert_eq!(usage.cache_write_input_tokens, 0);
    }

    #[test]
    fn reads_complete_runtime_settings_from_layered_codex_config() {
        let settings = runtime_settings_from_config_layers([
            r#"
                model = "gpt-5.6-sol"
                model_reasoning_effort = "medium"
                [features]
                js_repl = false
            "#,
            r#"model_reasoning_effort = "high""#,
        ])
        .expect("complete configuration should provide runtime settings");

        assert_eq!(settings.model, "gpt-5.6-sol");
        assert_eq!(settings.reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    fn requires_app_server_fallback_for_incomplete_or_invalid_codex_config() {
        assert_eq!(
            runtime_settings_from_config_layers([r#"model = "gpt-5.6-sol""#]),
            None
        );
        assert_eq!(
            runtime_settings_from_config_layers(["model = [invalid"]),
            None
        );
    }

    #[test]
    fn caches_runtime_defaults_until_the_configuration_is_invalidated() {
        let cache = CodexRuntimeDefaultsCache::default();
        cache.enable();
        let mut resolution_count = 0;
        let mut resolve = || {
            resolution_count += 1;
            Ok(CodexRuntimeSettings {
                model: format!("model-{resolution_count}"),
                reasoning_effort: Some("high".to_string()),
            })
        };

        let first = cache
            .resolve(&mut resolve)
            .expect("initial runtime defaults should resolve");
        let cached = cache
            .resolve(&mut resolve)
            .expect("runtime defaults should come from the cache");
        cache.invalidate();
        let refreshed = cache
            .resolve(&mut resolve)
            .expect("invalidated runtime defaults should resolve again");

        assert_eq!(first.model, "model-1");
        assert_eq!(cached.model, "model-1");
        assert_eq!(refreshed.model, "model-2");
        assert_eq!(resolution_count, 2);
    }

    #[test]
    fn bypasses_the_cache_when_configuration_monitoring_is_unavailable() {
        let cache = CodexRuntimeDefaultsCache::default();
        let mut resolution_count = 0;
        let mut resolve = || {
            resolution_count += 1;
            Ok(CodexRuntimeSettings {
                model: format!("model-{resolution_count}"),
                reasoning_effort: None,
            })
        };

        let first = cache
            .resolve(&mut resolve)
            .expect("initial runtime defaults should resolve");
        let second = cache
            .resolve(&mut resolve)
            .expect("uncached runtime defaults should resolve again");

        assert_eq!(first.model, "model-1");
        assert_eq!(second.model, "model-2");
        assert_eq!(resolution_count, 2);
    }
}
