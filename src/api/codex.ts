import { invoke } from "@tauri-apps/api/core";

type CodexLoginStatus = {
	/** Whether a local Codex executable was discovered. */
	installed: boolean;
	/** Whether the local Codex executable has active credentials. */
	loggedIn: boolean;
	/** Safe authentication mode reported by Codex. */
	authenticationMethod: string | null;
	/** Effective model selected for new local Codex tasks. */
	model: string | null;
	/** Effective reasoning effort selected for new local Codex tasks. */
	reasoningEffort: string | null;
};

type TokenUsage = {
	/** Total tokens consumed by the turn. */
	totalTokens: number;
	/** Tokens included in the model input. */
	inputTokens: number;
	/** Input tokens served from cache. */
	cachedInputTokens: number;
	/** Input tokens written into cache. */
	cacheWriteInputTokens: number;
	/** Tokens included in the model output. */
	outputTokens: number;
	/** Output tokens consumed by reasoning. */
	reasoningOutputTokens: number;
};

type CodexRunResult = {
	/** Incrementally assembled assistant response. */
	response: string;
	/** Milliseconds from turn submission until completion. */
	totalDurationMs: number;
	/** Milliseconds from turn submission until the first assistant delta. */
	timeToFirstTokenMs: number | null;
	/** Token usage reported for this turn. */
	tokenUsage: TokenUsage | null;
};

/** Checks the local Codex credential state through the Tauri backend. */
const checkCodexLogin = () => invoke<CodexLoginStatus>("check_codex_login");

/** Sends one natural-language task to the local Codex App Server. Example: `runCodexTask("解释这个仓库")`. */
const runCodexTask = (query: string) =>
	invoke<CodexRunResult>("run_codex_task", { request: { query } });

export type { CodexLoginStatus, CodexRunResult, TokenUsage };
export { checkCodexLogin, runCodexTask };
