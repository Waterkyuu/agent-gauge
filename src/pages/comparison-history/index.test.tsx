import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const apiMocks = vi.hoisted(() => ({
	getComparisonHistory: vi.fn(),
	listComparisonHistory: vi.fn(),
}));

vi.mock("@/api/comparison", () => ({
	getComparisonHistory: apiMocks.getComparisonHistory,
	listComparisonHistory: apiMocks.listComparisonHistory,
}));

import ComparisonHistoryPage from ".";

/**
 * Renders history with an isolated cache so tests never share query state.
 * @example renderHistoryPage();
 */
const renderHistoryPage = () => {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	});

	return render(
		<QueryClientProvider client={queryClient}>
			<ComparisonHistoryPage />
		</QueryClientProvider>,
	);
};

const SUMMARIES = [
	{
		id: 2,
		query: "检查第二次性能",
		status: "partial",
		metricVersion: 1,
		createdAtMs: 1_700_000_001_000,
		agents: [
			{ agent: "codex", status: "succeeded" },
			{ agent: "claude", status: "failed" },
		],
	},
	{
		id: 1,
		query: "检查第一次性能",
		status: "completed",
		metricVersion: 1,
		createdAtMs: 1_700_000_000_000,
		agents: [{ agent: "codex", status: "succeeded" }],
	},
] as const;

const DETAIL = {
	id: 2,
	query: "检查第二次性能",
	status: "partial",
	metricVersion: 1,
	createdAtMs: 1_700_000_001_000,
	results: [
		{
			agent: "codex",
			model: "gpt-5",
			reasoningEffort: "high",
			status: "succeeded",
			result: {
				response: "历史响应",
				totalDurationMs: 1500,
				timeToFirstTokenMs: 120,
				tokenUsage: null,
				thinkingDurationMs: 300,
				toolCallCount: 0,
				toolCalls: [],
			},
			errorMessage: null,
		},
	],
} as const;

// Covers list, selection, detail, and empty states for persisted comparisons.
describe("ComparisonHistoryPage", () => {
	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.listComparisonHistory.mockResolvedValue({
			items: SUMMARIES,
			nextCursor: null,
		});
		apiMocks.getComparisonHistory.mockResolvedValue(DETAIL);
	});

	it("loads the newest comparison and renders its complete result", async () => {
		renderHistoryPage();

		expect(await screen.findByText("历史响应")).toBeInTheDocument();
		expect(apiMocks.getComparisonHistory).toHaveBeenCalledWith(2);
		expect(screen.getByText("gpt-5")).toBeInTheDocument();
	});

	it("loads another detail when its history row is selected", async () => {
		renderHistoryPage();
		await screen.findByText("历史响应");

		fireEvent.click(screen.getByRole("button", { name: /检查第一次性能/ }));

		await waitFor(() => {
			expect(apiMocks.getComparisonHistory).toHaveBeenLastCalledWith(1);
		});
	});

	it("shows a useful empty state when no comparisons exist", async () => {
		apiMocks.listComparisonHistory.mockResolvedValue({
			items: [],
			nextCursor: null,
		});
		renderHistoryPage();

		expect(await screen.findByText("还没有历史对比")).toBeInTheDocument();
		expect(
			screen.getByText("完成一次性能对比后，结果会自动保存在这里。"),
		).toHaveClass("w-full", "max-w-[20rem]");
	});

	it("reuses a cached detail when a previously selected row is opened again", async () => {
		renderHistoryPage();
		await screen.findByText("历史响应");

		fireEvent.click(screen.getByRole("button", { name: /检查第一次性能/ }));
		await waitFor(() => {
			expect(apiMocks.getComparisonHistory).toHaveBeenCalledWith(1);
		});
		fireEvent.click(screen.getByRole("button", { name: /检查第二次性能/ }));
		await screen.findByText("历史响应");

		expect(
			apiMocks.getComparisonHistory.mock.calls.filter(([id]) => id === 2),
		).toHaveLength(1);
	});
});
