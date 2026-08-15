# AGENTS.md

## Role and Scope

Act as a senior Rust and Tauri backend engineer. Code under `src-tauri/` must be secure,
cross-platform, observable, testable, and efficient enough for a long-running desktop process.

AgentGauge monitors local AI agent processes and logs. Treat process metadata, prompts, command
arguments, filesystem paths, repository names, model output, tokens, and environment data as
sensitive local information.

These rules apply to every file under `src-tauri/`, including Rust source, Cargo manifests, Tauri
configuration, capabilities, permissions, migrations, fixtures, and integration tests.

## Required Commands

Run these commands from the repository root after changing Rust code and before committing, in this
order:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

Also run the following after changing `Cargo.toml`, `tauri.conf.json`, capabilities, permissions,
plugins, build scripts, or platform-specific code:

```bash
pnpm tauri build --no-bundle
```

If a required command fails, fix the failure and rerun the complete sequence. Do not suppress a
warning merely to make CI green. A narrowly scoped `#[allow(...)]` is acceptable only when it has a
comment explaining why the lint is incorrect for that exact location.

## Project Structure

Keep `main.rs` and `lib.rs` small. `main.rs` only starts the library entrypoint. `lib.rs` is the
composition root that registers state, plugins, commands, and application lifecycle hooks.

Use this structure as the backend grows:

```text
src-tauri/src/
├── main.rs                 # Native binary entrypoint only
├── lib.rs                  # Tauri composition root only
├── commands/               # Thin IPC transport handlers grouped by domain
├── dto/                    # Serialized request, response, and event contracts
├── domain/                 # Core entities, value objects, and domain rules
├── services/               # Application use cases and orchestration
├── adapters/               # Codex, Claude Code, WorkBuddy, and future agent adapters
├── repositories/           # Persistence interfaces and implementations
├── platform/               # Operating-system-specific implementations
│   ├── macos/
│   └── windows/
├── state/                  # Tauri-managed application state
├── error.rs                # Shared application and IPC errors
└── utils/                  # Small reusable helpers with no domain ownership
```

Do not create empty layers preemptively. Add a module when it has a concrete responsibility.

Dependency direction must be:

```text
commands -> services -> domain
                    -> repository traits
                    -> adapter traits

platform implementations -> repository or adapter traits
lib.rs -> concrete implementations and dependency wiring
```

Domain and service modules must not depend on Tauri types. Tauri-specific types belong at the IPC
and composition boundaries. This allows business logic to run in unit tests without a webview or an
application runtime.

## Rust Coding Rules

### Naming and Visibility

- Follow standard Rust naming: `snake_case` for modules, functions, and variables;
  `UpperCamelCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants.
- Name functions after their concrete operation and result. Do not use catch-all verbs such as
  `handle`, `process`, `manage`, or `resolve` when a precise verb such as `find`, `parse`, `load`,
  `map`, `validate`, or `collect` describes the behavior. For example, use
  `find_usable_codex_executable`, not `resolve_codex_executable`.
- Use the narrowest visibility. Prefer private items, then `pub(crate)`. Use `pub` only for a real
  crate boundary or a Tauri command that must be public from a separate command module.
- Do not use wildcard imports except in an explicitly designed prelude or a test module where the
  imported scope is obvious.
- Do not create a `utils` function when the behavior belongs to a domain type or service.

### Ownership and Types

- Model valid domain states with enums, newtypes, and constructors that validate invariants.
- Prefer `&str`, slices, and references at synchronous internal boundaries when ownership is not
  required. Tauri async commands should own their inputs, such as `String` and `PathBuf`.
- Do not clone values merely to satisfy the borrow checker. Reconsider ownership first. A clone is
  acceptable when it represents a deliberate snapshot or crosses a task boundary.
- Prefer `Path` and `PathBuf` over string paths. Never assemble paths by concatenating separators.
- Prefer standard conversion traits such as `From`, `TryFrom`, `AsRef`, and `Into` over ad hoc
  conversion methods.
- Public and IPC-facing structs should derive useful traits where semantically correct, including
  `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`.
- Avoid `serde_json::Value` in typed application code. Use explicit DTOs. Dynamic JSON is allowed
  only at an adapter boundary whose upstream schema is genuinely dynamic.
- Do not use `unsafe`. If an operating-system API makes it unavoidable, isolate it in the relevant
  `platform` module, document every safety invariant, and add focused tests around the safe wrapper.

### Functions and Modules

- Keep functions focused on one responsibility and keep side effects at explicit boundaries.
- Prefer pure functions for parsing, normalization, metric calculation, and state transitions.
- Do not split a short, single-use operation into a wrapper with no reusable meaning.
- Use early returns to keep the main execution path readable.
- Avoid boolean parameters when an enum communicates intent more clearly.
- Comments explain why a decision or invariant exists, not what the syntax does.
- Every field in a `struct` must have a field-level `///` documentation comment, including named
  and tuple fields in private, protocol, DTO, and test-only structs.
- Add rustdoc to public domain types, public traits, reusable APIs, and non-obvious command contracts.
  Include examples when they clarify intended usage.

## Error Handling

- Recoverable failures return `Result<T, E>`. Reserve `panic!` for broken internal invariants that
  indicate a programming defect.
- Do not use `unwrap()` or `expect()` in runtime request handling, adapters, repositories, parsing,
  file access, process handling, or background tasks.
- An `expect()` is allowed at the application startup boundary only when failure makes startup
  impossible and the message explains the failed invariant.
- Define typed internal errors. Use a shared `AppError` or domain-specific error enum and preserve
  the original source error.
- Convert internal errors to a stable serializable IPC error at the command boundary. An IPC error
  must contain a machine-readable code and a safe user-facing message.
- Never return raw operating-system errors, SQL statements, full paths, prompts, command arguments,
  tokens, secrets, or stack traces to the frontend.
- Do not return `Result<T, String>` from service or repository layers. String errors are permitted
  only as a final compatibility boundary when the framework requires them.
- Do not discard errors with `let _ = ...`. Either handle, propagate, log, or explicitly document
  why the result is safely ignored.
- Add context at boundary transitions so logs identify which operation failed without exposing
  sensitive values.

## Tauri Command Rules

- Commands are transport adapters, not business logic containers. A command validates its request,
  obtains managed state, calls one service method, and maps the result to an IPC response.
- Place commands in domain-specific modules once more than a few commands exist. Command names must
  be globally unique across all modules.
- Register every command in one `tauri::generate_handler![...]` invocation. Multiple
  `invoke_handler` calls are forbidden because only the last handler is used.
- Use typed request and response DTOs with `#[serde(rename_all = "camelCase")]` so TypeScript and
  Rust contracts are explicit and consistent.
- Validate all frontend input again in Rust. The webview is not a trusted security boundary.
- Set explicit length, count, range, and path constraints before allocating memory or touching the
  filesystem.
- Prefer request-response commands for operations that return a result.
- Use targeted events for small asynchronous notifications. Do not use global events when a single
  window is the intended recipient.
- Use Tauri channels for ordered or higher-volume streams such as process output, agent events, and
  log tailing. Events are not suitable for high-throughput data.
- Never use webview JavaScript evaluation for application data flow.
- Keep serialized payloads bounded. Store large data locally and return identifiers, pagination, or
  summaries instead of unbounded arrays or file contents.

Example command shape:

```rust
#[tauri::command]
pub async fn list_runs(
    request: ListRunsRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ListRunsResponse, IpcError> {
    request.validate()?;
    state.run_service.list_runs(request).await.map_err(Into::into)
}
```

The service invoked above must not import `tauri::State` or other webview types.

## State and Concurrency

- Register long-lived dependencies through `tauri::Builder::manage` and access them with
  `tauri::State` only at the command boundary.
- Prefer immutable state and thread-safe service handles. Shared mutable state must have a clearly
  documented owner and synchronization strategy.
- Use `Arc` only when ownership is genuinely shared across tasks or services.
- Do not hold a `Mutex` or `RwLock` guard across `.await`.
- Prefer async-aware synchronization for state accessed by async tasks. A standard mutex is allowed
  only for very short, non-async critical sections.
- Use bounded channels. Unbounded process or log event queues are forbidden.
- Every spawned task must have an owner, shutdown signal, and observable failure path. Do not detach
  infinite background tasks without cancellation.
- Apply explicit timeouts to external processes, network calls, filesystem waits, and adapter
  initialization.
- Use backoff with jitter for retryable operations. Set a maximum attempt count or a cancellation
  condition. Never retry validation, authorization, or permanent errors.
- Move CPU-heavy or blocking filesystem and process work off the async executor using the appropriate
  blocking-task mechanism.

## Process and Agent Adapter Rules

- Define a shared adapter trait and normalized event model. Codex, Claude Code, WorkBuddy, macOS,
  and Windows implementations must not calculate UI metrics independently.
- Keep process discovery, log decoding, event normalization, and metric calculation as separate
  responsibilities.
- Prefer stable structured logs and documented APIs over terminal scraping. A scraper must be
  versioned, fixture-tested, and able to enter a degraded state when the upstream format changes.
- Every source event needs a stable deduplication key, source timestamp, source identifier, adapter
  version, and confidence level where inference is involved.
- Incrementally tail files using persisted offsets. Handle truncation, rotation, partial UTF-8,
  duplicate delivery, application restart, and system sleep.
- Bound line length, event size, batch size, retained buffers, and concurrency.
- Never execute a command through a shell using interpolated input. Use `std::process::Command` or an
  async equivalent with the executable and each argument supplied separately.
- Allowlist executables and operations initiated by the application. Validate executable identity
  when relying on a discovered binary.
- Capture child process output with bounded buffers, apply a timeout, and terminate owned child
  processes during cancellation or application shutdown.
- Never terminate, pause, or modify a monitored third-party Agent unless the user explicitly starts
  a product feature that requires that action.
- Passive monitoring must not materially change the monitored Agent's runtime behavior.

## Filesystem and Persistence

- Store application data under a Tauri-resolved application data directory, never an assumed
  current working directory.
- Treat paths received from the frontend, logs, plugins, or external processes as untrusted.
- Define an allowed root before reading or writing. Normalize and validate paths without allowing
  `..` traversal or symlink escape from that root.
- Do not follow symlinks when doing so can cross a security boundary. Document the intended symlink
  policy for each adapter.
- Use atomic writes for settings and metadata that must not be partially persisted.
- Use transactions for multi-step database changes. Parameterize every query.
- Version database schemas and metric calculation rules. Migrations must be forward-safe, tested on
  realistic fixtures, and never silently destroy user data.
- Persist timestamps in UTC with an explicit format. Convert to local time only in the presentation
  layer.
- Store durations as integer units with the unit encoded in the field name or type.
- Paginate list queries and add indexes for frequently filtered fields such as agent, status,
  project, and start time.

## Cross-Platform Rules

- macOS and Windows are first-class targets. Do not place operating-system assumptions in domain,
  command, or service modules.
- Put platform differences behind traits in `platform/macos` and `platform/windows` modules.
- Keep `#[cfg(target_os = ...)]` near the concrete platform implementation, not scattered through
  business logic.
- Use `PathBuf` and platform APIs instead of hardcoded `/`, drive letters, `$HOME`, `%APPDATA%`, or
  shell-specific commands.
- Do not assume process identifiers are globally stable or that executable names uniquely identify
  a session.
- Normalize platform observations into the same domain types and state transitions.
- Add platform-specific tests or fixtures for process output, path handling, permissions, log
  rotation, encoding, and timestamp behavior.
- A platform that cannot supply a metric must return an explicit unavailable value, not `0` or a
  fabricated estimate.

## Security and Privacy

- Follow least privilege. Every Tauri plugin permission must be explicitly required by a feature.
- Scope capabilities to exact windows or webviews and the narrowest paths or commands possible.
- Do not grant filesystem, shell, process, or remote URL wildcards for convenience.
- Do not enable remote webview origins for local system capabilities unless the feature has a
  documented threat model and explicit approval.
- Review `capabilities/`, custom permissions, CSP, and plugin features together whenever adding an
  IPC or system-access feature.
- Never read environment variable values, credentials, SSH keys, browser storage, or unrelated
  directories as part of agent discovery.
- Redact secrets, prompts, outputs, command arguments, usernames, repository paths, and tokens before
  logging or exporting data.
- Privacy modes must be enforced before persistence, not only hidden in the frontend.
- Use explicit allowlists for exported fields. Do not serialize internal structs wholesale into a
  report.
- Any telemetry or network transmission must be opt-in and must not reuse consent granted for local
  collection.

## Logging and Observability

- Use structured logging through a single logging facade. Do not use `println!` or `dbg!` in runtime
  code.
- Include safe correlation fields such as run ID, adapter kind, event type, and duration.
- Never put sensitive raw values into log fields. Prefer redacted identifiers or one-way local
  hashes when correlation is necessary.
- Use suitable levels: `error` for failed operations requiring attention, `warn` for degraded but
  recoverable behavior, `info` for lifecycle milestones, and `debug` or `trace` for opt-in
  diagnostics.
- Avoid logging each high-frequency event at `info`. Aggregate counts and durations.
- Metrics and logs must not affect the measured Agent enough to invalidate timing results.

## Performance

- Prefer event-driven file watching and process observation over frequent polling.
- If polling is unavoidable, document the interval and resource budget and suspend or back off when
  idle.
- Do not perform filesystem scans, database migrations, or process enumeration on the UI thread.
- Stream and batch large inputs. Do not read complete growing log files into memory.
- Measure before adding caches. Every cache needs an invalidation rule and memory bound.
- Avoid premature micro-optimization, but prevent unbounded work by design.
- Performance-sensitive changes need a repeatable benchmark or before-and-after measurement.

## Dependency Rules

- Prefer the standard library and established, maintained crates with a narrow purpose.
- Explain why a new dependency is needed and review its license, maintenance, transitive dependency
  cost, build scripts, enabled features, and platform support.
- Disable default features when they add unused functionality or system access.
- Keep JavaScript and Rust Tauri package versions compatible.
- Commit `Cargo.lock` for this desktop application.
- Do not add a dependency to avoid writing a small, well-tested pure function.

## Testing Rules

- Use red-green-refactor for new behavior and bug fixes.
- Place unit tests beside the module in `#[cfg(test)] mod tests`.
- Place black-box crate tests in `src-tauri/tests/` and shared fixtures in
  `src-tauri/tests/fixtures/`.
- Test observable behavior and contracts, not private implementation details.
- Domain and service tests must run without starting Tauri.
- Use trait-based fakes for process, clock, filesystem, and persistence boundaries. Do not add test
  branches to production code.
- Adapter parsers require versioned fixtures for valid input, malformed input, partial records,
  duplicates, log rotation, and upstream format changes.
- Test all state-machine transitions, including cancellation, failure, unknown termination, restart,
  and system sleep.
- Test IPC request and response serialization so Rust `camelCase` contracts stay aligned with
  TypeScript.
- Tests must not depend on the developer's installed Agents, home-directory contents, network,
  locale, timezone, execution order, or wall-clock timing.
- Use temporary directories for filesystem tests and clean up owned child processes.
- Add platform-gated tests for true operating-system behavior and keep portable contract tests for
  every platform implementation.

## Definition of Done

A Rust backend change is complete only when:

- Module boundaries and dependency direction remain clear.
- Inputs are validated and errors are typed and safely serialized.
- Async work is cancellable, bounded, and does not block the executor.
- macOS and Windows behavior is implemented or explicitly represented as unavailable.
- Capabilities and permissions grant only the required access.
- Sensitive data is redacted before logs, persistence, events, and exports.
- New behavior has deterministic tests, including relevant failure paths.
- Required format, Clippy, test, check, and applicable Tauri build commands pass.
- Any schema, IPC contract, capability, migration, or adapter compatibility change is documented.
