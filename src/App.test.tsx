import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: invokeMock,
}));

// Covers the user-visible local Codex task workflow.
describe("App", () => {
	beforeEach(() => {
		invokeMock.mockReset();
	});

	// Verifies that authentication is checked before task submission is enabled.
	it("shows the local Codex login status", async () => {
		invokeMock.mockResolvedValueOnce({
			installed: true,
			loggedIn: true,
			authenticationMethod: "ChatGPT",
		});

		render(<App />);

		expect(await screen.findByText("已通过 ChatGPT 登录")).toBeInTheDocument();
		expect(screen.getByLabelText("任务内容")).toBeEnabled();
	});

	// Verifies the query submission and completed metric presentation.
	it("submits a query and displays the completed metrics", async () => {
		invokeMock
			.mockResolvedValueOnce({
				installed: true,
				loggedIn: true,
				authenticationMethod: "ChatGPT",
			})
			.mockResolvedValueOnce({
				response: "任务完成",
				totalDurationMs: 2450,
				timeToFirstTokenMs: 680,
				tokenUsage: {
					totalTokens: 1280,
					inputTokens: 900,
					cachedInputTokens: 400,
					cacheWriteInputTokens: 0,
					outputTokens: 300,
					reasoningOutputTokens: 80,
				},
			});
		const user = userEvent.setup();

		render(<App />);
		await screen.findByText("已通过 ChatGPT 登录");
		await user.type(screen.getByLabelText("任务内容"), "只回复任务完成");
		await user.click(screen.getByRole("button", { name: "发送任务" }));

		await waitFor(() => {
			expect(invokeMock).toHaveBeenLastCalledWith("run_codex_task", {
				request: { query: "只回复任务完成" },
			});
		});
		expect(await screen.findByText("任务完成")).toBeInTheDocument();
		expect(screen.getByText("680 ms")).toBeInTheDocument();
		expect(screen.getByText("2.45 s")).toBeInTheDocument();
		expect(screen.getByText("1,280")).toBeInTheDocument();
	});
});
