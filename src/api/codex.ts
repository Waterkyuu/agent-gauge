import { invoke } from "@tauri-apps/api/core";
import type { AgentRunResult, AgentRuntimeStatus } from "@/types/agent";

/** Checks the local Codex credential state through the Tauri backend. */
const checkCodexLogin = () => invoke<AgentRuntimeStatus>("check_codex_login");

/**
 * Sends one natural-language task to the local Codex App Server.
 *
 * @example
 * runCodexTask("解释这个仓库");
 */
const runCodexTask = (query: string) =>
	invoke<AgentRunResult>("run_codex_task", { request: { query } });

export { checkCodexLogin, runCodexTask };
