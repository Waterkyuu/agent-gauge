import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentRunResult, AgentRuntimeStatus } from "@/types/agent";

/** Checks the local Codex credential state through the Tauri backend. */
const checkCodexLogin = () => invoke<AgentRuntimeStatus>("check_codex_login");

/** Subscribes to native changes in the effective local Codex configuration. */
const onCodexConfigChanged = (listener: () => void) =>
	listen<void>("codex-config-changed", listener);

/**
 * Sends one natural-language task to the local Codex App Server.
 *
 * @example
 * runCodexTask("解释这个仓库");
 */
const runCodexTask = (query: string) =>
	invoke<AgentRunResult>("run_codex_task", { request: { query } });

export { checkCodexLogin, onCodexConfigChanged, runCodexTask };
