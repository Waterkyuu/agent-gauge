import type { AgentKind, AgentRunResult } from "@/types/agent";

type ComparisonResultInput = {
	/** Agent product that produced this outcome. */
	agent: AgentKind;
	/** Model configuration captured before execution. */
	model: string | null;
	/** Reasoning configuration captured before execution. */
	reasoningEffort: string | null;
} & (
	| {
			/** Successful outcome discriminator. */
			status: "succeeded";
			/** Completed response and metrics. */
			result: AgentRunResult;
	  }
	| {
			/** Failed outcome discriminator. */
			status: "failed";
			/** Safe localized failure detail. */
			errorMessage: string;
	  }
);

type SaveComparisonHistoryRequest = {
	/** Shared task sent to every selected Agent. */
	query: string;
	/** Final outcome for every selected Agent. */
	results: ComparisonResultInput[];
};

type ComparisonCursor = {
	/** UTC timestamp of the final item in the prior page. */
	createdAtMs: number;
	/** Primary key of the final item in the prior page. */
	id: number;
};

type ComparisonAgentSummary = {
	/** Agent represented by this summary. */
	agent: AgentKind;
	/** Success or failure state. */
	status: "succeeded" | "failed";
};

type ComparisonSummary = {
	/** Persistent comparison identifier. */
	id: number;
	/** Shared task text. */
	query: string;
	/** Aggregate completion state. */
	status: "completed" | "partial" | "failed";
	/** Metric calculation contract version. */
	metricVersion: number;
	/** UTC Unix timestamp in milliseconds. */
	createdAtMs: number;
	/** Selected Agent outcomes without response bodies. */
	agents: ComparisonAgentSummary[];
};

type ComparisonHistoryPage = {
	/** Summaries ordered newest first. */
	items: ComparisonSummary[];
	/** Cursor for the next page. */
	nextCursor: ComparisonCursor | null;
};

type ComparisonResultDetail = {
	/** Agent product represented by this result. */
	agent: AgentKind;
	/** Model configuration captured at execution time. */
	model: string | null;
	/** Reasoning configuration captured at execution time. */
	reasoningEffort: string | null;
	/** Success or failure state. */
	status: "succeeded" | "failed";
	/** Successful response and metrics. */
	result: AgentRunResult | null;
	/** Safe failure detail. */
	errorMessage: string | null;
};

type ComparisonHistoryDetail = {
	/** Persistent comparison identifier. */
	id: number;
	/** Shared task text. */
	query: string;
	/** Aggregate completion state. */
	status: "completed" | "partial" | "failed";
	/** Metric calculation contract version. */
	metricVersion: number;
	/** UTC Unix timestamp in milliseconds. */
	createdAtMs: number;
	/** Complete outcomes for every selected Agent. */
	results: ComparisonResultDetail[];
};

export type {
	ComparisonAgentSummary,
	ComparisonCursor,
	ComparisonHistoryDetail,
	ComparisonHistoryPage,
	ComparisonResultDetail,
	ComparisonResultInput,
	ComparisonSummary,
	SaveComparisonHistoryRequest,
};
