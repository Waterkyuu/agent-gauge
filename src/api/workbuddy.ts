import { invoke } from "@tauri-apps/api/core";
import type { AgentRunResult, AgentRuntimeStatus } from "@/types/agent";

/** Checks the local WorkBuddy account state through the Tauri backend. */
const checkWorkBuddyLogin = () =>
	invoke<AgentRuntimeStatus>("check_workbuddy_login");

/**
 * Sends one natural-language task to the local WorkBuddy runtime.
 *
 * @example
 * runWorkBuddyTask("解释这个仓库");
 */
const runWorkBuddyTask = (query: string) =>
	invoke<AgentRunResult>("run_workbuddy_task", { request: { query } });

export { checkWorkBuddyLogin, runWorkBuddyTask };
