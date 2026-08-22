import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BrowserRouter, Route, Routes } from "react-router";
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
	window.history.pushState({}, "", "/comparison-history");
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	});

	return render(
		<QueryClientProvider client={queryClient}>
			<BrowserRouter>
				<Routes>
					<Route
						element={<ComparisonHistoryPage />}
						path="/comparison-history"
					/>
					<Route
						element={<ComparisonHistoryPage />}
						path="/comparison-history/:comparisonId"
					/>
				</Routes>
			</BrowserRouter>
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

	it("waits for a history row selection before loading its detail", async () => {
		renderHistoryPage();
		const newestRow = await screen.findByRole("button", {
			name: "检查第二次性能",
		});

		expect(apiMocks.getComparisonHistory).not.toHaveBeenCalled();

		fireEvent.click(newestRow);

		expect(await screen.findByText("历史响应")).toBeInTheDocument();
		expect(apiMocks.getComparisonHistory).toHaveBeenCalledWith(2);
		expect(screen.getByText("gpt-5")).toBeInTheDocument();
	});

	it("replaces the history list with detail and returns from its back button", async () => {
		renderHistoryPage();
		const newestRow = await screen.findByRole("button", {
			name: "检查第二次性能",
		});

		fireEvent.click(newestRow);
		await screen.findByText("历史响应");

		expect(window.location.pathname).toBe("/comparison-history/2");
		expect(
			screen.queryByRole("heading", { level: 1, name: "历史对比" }),
		).not.toBeInTheDocument();
		expect(
			screen.queryByRole("region", { name: "对比记录" }),
		).not.toBeInTheDocument();
		fireEvent.click(screen.getByRole("button", { name: "返回对比记录" }));

		expect(
			screen.getByRole("region", { name: "对比记录" }),
		).toBeInTheDocument();
		expect(window.location.pathname).toBe("/comparison-history");
		expect(screen.queryByText("历史响应")).not.toBeInTheDocument();
	});

	it("loads another detail when its history row is selected", async () => {
		renderHistoryPage();
		const previousRow = await screen.findByRole("button", {
			name: "检查第一次性能",
		});

		fireEvent.click(previousRow);

		await waitFor(() => {
			expect(apiMocks.getComparisonHistory).toHaveBeenLastCalledWith(1);
		});
	});

	it("opens the rename modal from a record action menu without navigating", async () => {
		const user = userEvent.setup();
		renderHistoryPage();
		const actionsButton = await screen.findByRole("button", {
			name: "检查第二次性能的更多操作",
		});

		await user.click(actionsButton);
		await user.click(await screen.findByRole("menuitem", { name: "重命名" }));

		expect(
			await screen.findByRole("dialog", { name: "重命名记录" }),
		).toBeInTheDocument();
		expect(window.location.pathname).toBe("/comparison-history");
	});

	it("opens the delete alert from a record action menu", async () => {
		const user = userEvent.setup();
		renderHistoryPage();
		const actionsButton = await screen.findByRole("button", {
			name: "检查第二次性能的更多操作",
		});

		await user.click(actionsButton);
		await user.click(await screen.findByRole("menuitem", { name: "删除" }));

		expect(
			await screen.findByRole("alertdialog", { name: "删除记录？" }),
		).toBeInTheDocument();
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
		const newestRow = await screen.findByRole("button", {
			name: "检查第二次性能",
		});

		fireEvent.click(newestRow);
		await screen.findByText("历史响应");

		fireEvent.click(screen.getByRole("button", { name: "返回对比记录" }));
		fireEvent.click(screen.getByRole("button", { name: "检查第一次性能" }));
		await waitFor(() => {
			expect(apiMocks.getComparisonHistory).toHaveBeenCalledWith(1);
		});
		fireEvent.click(screen.getByRole("button", { name: "返回对比记录" }));
		fireEvent.click(screen.getByRole("button", { name: "检查第二次性能" }));
		await screen.findByText("历史响应");

		expect(
			apiMocks.getComparisonHistory.mock.calls.filter(([id]) => id === 2),
		).toHaveLength(1);
	});
});
