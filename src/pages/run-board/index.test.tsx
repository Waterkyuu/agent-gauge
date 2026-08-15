import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
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
});
