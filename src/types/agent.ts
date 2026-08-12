type AgentKind = "claude" | "codex" | "workbuddy";

type AgentProcessStates = {
	/** Whether a Claude Code process is currently running. */
	claude: boolean;
	/** Whether a Codex process is currently running. */
	codex: boolean;
	/** Whether a WorkBuddy process is currently running. */
	workbuddy: boolean;
};

type AgentRuntimeStatus = {
	/** Whether the local agent product was discovered. */
	installed: boolean;
	/** Whether the local agent product has active credentials. */
	loggedIn: boolean;
	/** Safe authentication mode reported by the agent. */
	authenticationMethod: string | null;
	/** Effective model selected for new tasks. */
	model: string | null;
	/** Effective reasoning effort selected for new tasks. */
	reasoningEffort: string | null;
};

type TokenUsage = {
	/** Total tokens consumed by the task. */
	totalTokens: number;
	/** Tokens included in the model input. */
	inputTokens: number;
	/** Input tokens served from cache. */
	cachedInputTokens: number;
	/** Input tokens written into cache. */
	cacheWriteInputTokens: number;
	/** Tokens included in the model output. */
	outputTokens: number;
	/** Output tokens consumed by reasoning when reported by the agent. */
	reasoningOutputTokens: number | null;
};

type AgentRunResult = {
	/** Incrementally assembled assistant response. */
	response: string;
	/** Milliseconds from task submission until completion. */
	totalDurationMs: number;
	/** Milliseconds from task submission until the first assistant text delta. */
	timeToFirstTokenMs: number | null;
	/** Token usage reported for this task. */
	tokenUsage: TokenUsage | null;
};

export type {
	AgentKind,
	AgentProcessStates,
	AgentRunResult,
	AgentRuntimeStatus,
	TokenUsage,
};
