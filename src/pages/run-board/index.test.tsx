import "@testing-library/jest-dom/vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import "@/i18n";
import RunBoardPage from ".";

// Covers the user-visible run board workflow.
describe("RunBoardPage", () => {
	// Verifies that users can narrow run cards by the visible agent product name.
	it("filters run cards by agent product name", async () => {
		const user = userEvent.setup();
		render(<RunBoardPage />);

		const searchInput = screen.getByRole("searchbox", {
			name: "搜索 Agent 产品",
		});
		await user.type(searchInput, "  CODEX  ");

		expect(screen.getAllByRole("article")).toHaveLength(2);
		expect(screen.getAllByText("Codex")).toHaveLength(2);
		expect(screen.queryByText("Claude Code")).not.toBeInTheDocument();
		expect(screen.queryByText("WorkBuddy")).not.toBeInTheDocument();

		await user.clear(searchInput);

		expect(screen.getAllByRole("article")).toHaveLength(6);
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
	it("keeps run cards at fixed dimensions with clamped text", () => {
		render(<RunBoardPage />);

		const card = screen.getAllByRole("article")[0];

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
