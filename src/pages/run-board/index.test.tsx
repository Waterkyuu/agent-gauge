import "@testing-library/jest-dom/vitest";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import "@/i18n";
import RunBoardPage from ".";

const apiMocks = vi.hoisted(() => ({
	checkAgentActivities: vi.fn(),
	onAgentActivitiesChanged: vi.fn(),
}));

vi.mock("@/api/agent", () => apiMocks);

const INITIAL_ACTIVITIES = {
	activities: [
		{
			id: "codex-running",
			agent: "codex",
			status: "running",
			updatedAtMs: Date.parse("2026-08-17T01:30:00Z"),
		},
		{
			id: "claude-waiting",
			agent: "claude",
			status: "waiting",
			updatedAtMs: Date.parse("2026-08-17T01:20:00Z"),
		},
		{
			id: "workbuddy-finish",
			agent: "workbuddy",
			status: "finish",
			updatedAtMs: Date.parse("2026-08-17T01:10:00Z"),
		},
		{
			id: "codex-error",
			agent: "codex",
			status: "error",
			updatedAtMs: Date.parse("2026-08-17T01:00:00Z"),
		},
	],
};

// Covers the user-visible run board workflow.
describe("RunBoardPage", () => {
	beforeEach(() => {
		apiMocks.checkAgentActivities.mockResolvedValue(INITIAL_ACTIVITIES);
		apiMocks.onAgentActivitiesChanged.mockResolvedValue(vi.fn());
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.clearAllMocks();
	});

	// Verifies that only backend Agent activities are rendered instead of bundled demo records.
	it("renders the real activity snapshot and applies changed events", async () => {
		let activityListener:
			| ((response: typeof INITIAL_ACTIVITIES) => void)
			| undefined;
		apiMocks.onAgentActivitiesChanged.mockImplementation((listener) => {
			activityListener = listener;
			return Promise.resolve(vi.fn());
		});
		render(<RunBoardPage />);

		expect(await screen.findAllByRole("article")).toHaveLength(4);
		expect(screen.queryByText("仓库架构审计")).not.toBeInTheDocument();
		expect(apiMocks.checkAgentActivities).toHaveBeenCalledOnce();

		act(() => {
			activityListener?.({
				activities: [
					{
						id: "claude-completed",
						agent: "claude",
						status: "finish",
						updatedAtMs: Date.parse("2026-08-17T01:40:00Z"),
					},
				],
			});
		});

		expect(screen.getAllByRole("article")).toHaveLength(1);
		expect(screen.getByText("Claude Code")).toBeInTheDocument();
	});

	// Verifies that rapid input only applies the latest agent product name after the delay.
	it("debounces agent product filtering", async () => {
		vi.useFakeTimers();
		render(<RunBoardPage />);
		await act(async () => Promise.resolve());

		const searchInput = screen.getByRole("searchbox", {
			name: "搜索 Agent 产品",
		});
		fireEvent.change(searchInput, { target: { value: "Claude" } });
		act(() => vi.advanceTimersByTime(200));
		fireEvent.change(searchInput, { target: { value: "  CODEX  " } });
		act(() => vi.advanceTimersByTime(299));

		expect(screen.getAllByRole("article")).toHaveLength(4);

		act(() => vi.advanceTimersByTime(1));

		expect(screen.getAllByRole("article")).toHaveLength(2);
		expect(screen.getAllByText("Codex")).toHaveLength(2);
		expect(screen.queryByText("Claude Code")).not.toBeInTheDocument();
		expect(screen.queryByText("WorkBuddy")).not.toBeInTheDocument();
	});

	// Verifies that the board can switch between vertical columns and horizontal rows.
	it("switches between vertical and horizontal layouts", async () => {
		const user = userEvent.setup();
		render(<RunBoardPage />);

		const layoutGroup = screen.getByRole("group", {
			name: "切换看板布局",
		});
		const board = screen.getByTestId("run-board");
		const verticalButton = within(layoutGroup).getByRole("button", {
			name: "竖面板",
		});
		const horizontalButton = within(layoutGroup).getByRole("button", {
			name: "水平面板",
		});

		expect(verticalButton).toHaveAttribute("aria-pressed", "true");
		expect(horizontalButton).toHaveAttribute("aria-pressed", "false");
		expect(verticalButton).not.toHaveTextContent("竖面板");
		expect(horizontalButton).not.toHaveTextContent("水平面板");
		expect(board).toHaveAttribute("data-layout", "vertical");

		await user.click(horizontalButton);

		expect(verticalButton).toHaveAttribute("aria-pressed", "false");
		expect(horizontalButton).toHaveAttribute("aria-pressed", "true");
		expect(board).toHaveAttribute("data-layout", "horizontal");
	});

	// Verifies that a vertical status column scrolls instead of growing with every task.
	it("bounds vertical status columns with internal scrolling", () => {
		render(<RunBoardPage />);

		const runningList = screen.getByTestId("run-board-list-running");

		expect(runningList).toHaveClass(
			"max-h-[60vh]",
			"overflow-y-auto",
			"overscroll-contain",
		);
	});

	// Verifies that horizontal rows keep one line and scroll sideways for extra cards.
	it("scrolls horizontal status rows sideways", async () => {
		const user = userEvent.setup();
		render(<RunBoardPage />);

		await user.click(
			screen.getByRole("button", {
				name: "水平面板",
			}),
		);
		const runningList = screen.getByTestId("run-board-list-running");

		expect(runningList).toHaveClass(
			"lg:flex-nowrap",
			"lg:overflow-x-auto",
			"lg:overflow-y-hidden",
		);
		expect((await screen.findAllByRole("article"))[0]).toHaveClass(
			"lg:shrink-0",
		);
	});

	// Verifies that both icon-only board layout controls expose their meaning.
	it("describes both layout controls on hover", async () => {
		const user = userEvent.setup();
		render(<RunBoardPage />);

		const layoutGroup = screen.getByRole("group", {
			name: "切换看板布局",
		});
		const verticalButton = within(layoutGroup).getByRole("button", {
			name: "竖面板",
		});
		const horizontalButton = within(layoutGroup).getByRole("button", {
			name: "水平面板",
		});

		await user.hover(verticalButton);
		const verticalTooltip = await screen.findByRole("tooltip");
		expect(verticalTooltip).toHaveTextContent("竖面板");
		expect(verticalTooltip).toHaveClass("whitespace-nowrap");
		expect(verticalTooltip).toHaveClass("max-w-none");

		await user.unhover(verticalButton);
		await user.hover(horizontalButton);
		const horizontalTooltip = await screen.findByRole("tooltip");
		expect(horizontalTooltip).toHaveTextContent("水平面板");
		expect(horizontalTooltip).toHaveClass("whitespace-nowrap");
		expect(horizontalTooltip).toHaveClass("max-w-none");
	});

	// Verifies that run cards keep a stable footprint and clamp overflowing copy.
	it("keeps run cards at fixed dimensions with clamped text", async () => {
		render(<RunBoardPage />);

		const card = (await screen.findAllByRole("article"))[0];

		expect(card).toHaveClass(
			"h-48",
			"w-[18rem]",
			"max-w-full",
			"overflow-hidden",
		);
		expect(card.querySelector("h3")).toHaveClass(
			"line-clamp-2",
			"overflow-hidden",
		);
		expect(card.querySelector("p")).toHaveClass(
			"line-clamp-2",
			"overflow-hidden",
		);
	});
});
