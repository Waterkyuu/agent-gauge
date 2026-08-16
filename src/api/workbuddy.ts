import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
	AgentRunResult,
	AgentRuntimeConfig,
	AgentRuntimeStatus,
} from "@/types/agent";

/** Checks the local WorkBuddy account state through the Tauri backend. */
const checkWorkBuddyLogin = () =>
	invoke<AgentRuntimeStatus>("check_workbuddy_login");

/** Reads the current WorkBuddy model configuration from the Tauri backend. */
const checkWorkBuddyConfig = () =>
	invoke<AgentRuntimeConfig>("check_workbuddy_config");

/**
 * Subscribes to debounced native WorkBuddy model configuration changes.
 *
 * @example
 * onWorkBuddyConfigChanged(setWorkBuddyConfig);
 */
const onWorkBuddyConfigChanged = (
	listener: (config: AgentRuntimeConfig) => void,
) =>
	listen<AgentRuntimeConfig>("workbuddy-config-changed", (event) => {
		listener(event.payload);
	});

/**
 * Sends one natural-language task to the local WorkBuddy runtime.
 *
 * @example
 * runWorkBuddyTask("解释这个仓库");
 */
const runWorkBuddyTask = (query: string) =>
	invoke<AgentRunResult>("run_workbuddy_task", { request: { query } });

export {
	checkWorkBuddyConfig,
	checkWorkBuddyLogin,
	onWorkBuddyConfigChanged,
	runWorkBuddyTask,
};
