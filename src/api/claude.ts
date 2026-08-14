import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentRunResult, AgentRuntimeStatus } from "@/types/agent";

/** Checks the local Claude Code credential state through the Tauri backend. */
const checkClaudeLogin = () => invoke<AgentRuntimeStatus>("check_claude_login");

/**
 * Subscribes to changes in the user-level Claude runtime settings.
 *
 * @example
 * onClaudeConfigChanged(refreshClaudeStatus);
 */
const onClaudeConfigChanged = (listener: () => void) =>
	listen<void>("claude-config-changed", listener);

/**
 * Sends one natural-language task to the local Claude Code runtime.
 *
 * @example
 * runClaudeTask("解释这个仓库");
 */
const runClaudeTask = (query: string) =>
	invoke<AgentRunResult>("run_claude_task", { request: { query } });

export { checkClaudeLogin, onClaudeConfigChanged, runClaudeTask };
