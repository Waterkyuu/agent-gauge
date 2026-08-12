import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import "./i18n";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: invokeMock,
}));

// Covers the user-visible local agent task workflow.
describe("App", () => {
	beforeEach(() => {
		invokeMock.mockReset();
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: true,
					workbuddy: false,
				});
			}
			if (command === "check_claude_login") {
				return Promise.resolve({
					installed: true,
					loggedIn: true,
					authenticationMethod: "Claude account",
					model: null,
					reasoningEffort: null,
				});
			}
			if (command === "check_codex_login") {
				return Promise.resolve({
					installed: true,
					loggedIn: true,
					authenticationMethod: "ChatGPT",
					model: "gpt-5.6-sol",
					reasoningEffort: "high",
				});
			}
			if (command === "check_workbuddy_login") {
				return Promise.resolve({
					installed: true,
					loggedIn: true,
					authenticationMethod: "WorkBuddy account",
					model: "fast-model",
					reasoningEffort: "enabled",
				});
			}
			return Promise.reject(new Error(`Unexpected command: ${command}`));
		});
	});

	// Verifies that authentication is checked before task submission is enabled.
	it("shows the local Codex login status", async () => {
		render(<App />);

		expect(await screen.findByText("Codex：运行中")).toBeInTheDocument();
		expect(screen.getByText("gpt-5.6-sol")).toBeInTheDocument();
		expect(screen.getByText("高 (high)")).toBeInTheDocument();
		expect(screen.getByLabelText("任务内容")).toBeEnabled();

		await userEvent.click(screen.getByRole("button", { name: "WorkBuddy" }));
		expect(screen.getByText("fast-model")).toBeInTheDocument();
		expect(screen.getByText("默认 (enabled)")).toBeInTheDocument();

		await userEvent.click(screen.getByRole("button", { name: "English" }));
		expect(screen.getByText("Current model")).toBeInTheDocument();
		expect(
			screen.getByText("Send one task and get real runtime metrics."),
		).toBeInTheDocument();
		expect(localStorage.getItem("language")).toBe("en-US");
	});

	// Verifies that the running process state refreshes without reloading the UI.
	it("refreshes the selected agent process state", async () => {
		let processProbeCount = 0;
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				processProbeCount += 1;
				return Promise.resolve({
					claude: false,
					codex: processProbeCount > 1,
					workbuddy: false,
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "ChatGPT",
				model: "gpt-5.6-sol",
				reasoningEffort: "high",
			});
		});

		render(<App />);

		expect(
			await screen.findByText("Codex：已就绪，未运行"),
		).toBeInTheDocument();
		expect(
			await screen.findByText("Codex：运行中", {}, { timeout: 1500 }),
		).toBeInTheDocument();
		expect(processProbeCount).toBeGreaterThanOrEqual(2);
	});

	// Verifies that authentication changes refresh while the UI remains open.
	it("refreshes the selected agent authentication state", async () => {
		let codexLoggedIn = false;
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: false,
					workbuddy: false,
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: command === "check_codex_login" ? codexLoggedIn : true,
				authenticationMethod: "ChatGPT",
				model: "gpt-5.6-sol",
				reasoningEffort: "high",
			});
		});

		render(<App />);
		expect(
			await screen.findByText("本地 Codex 尚未登录。"),
		).toBeInTheDocument();

		codexLoggedIn = true;
		window.dispatchEvent(new Event("focus"));

		expect(
			await screen.findByText("Codex：已就绪，未运行"),
		).toBeInTheDocument();
	});

	// Verifies the query submission and completed metric presentation.
	it("submits a query and displays the completed metrics", async () => {
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: true,
					workbuddy: false,
				});
			}
			if (command === "run_codex_task") {
				return Promise.resolve({
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
			}
			if (
				command === "check_codex_login" ||
				command === "check_workbuddy_login"
			) {
				return Promise.resolve({
					installed: true,
					loggedIn: true,
					authenticationMethod: "ChatGPT",
					model: "gpt-5.6-sol",
					reasoningEffort: "high",
				});
			}
			return Promise.reject(new Error(`Unexpected command: ${command}`));
		});
		const user = userEvent.setup();

		render(<App />);
		await screen.findByText("Codex：运行中");
		await user.type(screen.getByLabelText("任务内容"), "只回复任务完成");
		await user.click(screen.getByRole("button", { name: "发送给 Codex" }));

		await waitFor(() => {
			expect(invokeMock).toHaveBeenCalledWith("run_codex_task", {
				request: { query: "只回复任务完成" },
			});
		});
		expect(await screen.findByText("任务完成")).toBeInTheDocument();
		expect(screen.getByText("680 ms")).toBeInTheDocument();
		expect(screen.getByText("2.45 s")).toBeInTheDocument();
		expect(screen.getByText("1,280")).toBeInTheDocument();
	});

	// Verifies that selecting WorkBuddy routes the query to its local runtime.
	it("submits a query to WorkBuddy", async () => {
		invokeMock.mockImplementation((command: string) => {
			if (command === "run_workbuddy_task") {
				return Promise.resolve({
					response: "OK",
					totalDurationMs: 14_619,
					timeToFirstTokenMs: 12_089,
					tokenUsage: {
						totalTokens: 26_509,
						inputTokens: 26_484,
						cachedInputTokens: 728,
						cacheWriteInputTokens: 25_756,
						outputTokens: 25,
						reasoningOutputTokens: null,
					},
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "WorkBuddy account",
				model: "fast-model",
				reasoningEffort: "enabled",
			});
		});
		const user = userEvent.setup();

		render(<App />);
		await user.click(await screen.findByRole("button", { name: "WorkBuddy" }));
		await waitFor(() =>
			expect(screen.getByLabelText("任务内容")).toBeEnabled(),
		);
		await user.type(screen.getByLabelText("任务内容"), "只回复 OK");
		await user.click(screen.getByRole("button", { name: "发送给 WorkBuddy" }));

		await waitFor(() => {
			expect(invokeMock).toHaveBeenCalledWith("run_workbuddy_task", {
				request: { query: "只回复 OK" },
			});
		});
		expect(await screen.findByText("26,509")).toBeInTheDocument();
	});

	// Verifies that selecting Claude Code routes the query to its local runtime.
	it("submits a query to Claude Code", async () => {
		invokeMock.mockImplementation((command: string) => {
			if (command === "run_claude_task") {
				return Promise.resolve({
					response: "OK",
					totalDurationMs: 3100,
					timeToFirstTokenMs: 540,
					tokenUsage: {
						totalTokens: 2950,
						inputTokens: 2920,
						cachedInputTokens: 800,
						cacheWriteInputTokens: 2000,
						outputTokens: 30,
						reasoningOutputTokens: null,
					},
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "Claude account",
				model: null,
				reasoningEffort: null,
			});
		});
		const user = userEvent.setup();

		render(<App />);
		await user.click(
			await screen.findByRole("button", { name: "Claude Code" }),
		);
		await waitFor(() =>
			expect(screen.getByLabelText("任务内容")).toBeEnabled(),
		);
		await user.type(screen.getByLabelText("任务内容"), "只回复 OK");
		await user.click(
			screen.getByRole("button", { name: "发送给 Claude Code" }),
		);

		await waitFor(() => {
			expect(invokeMock).toHaveBeenCalledWith("run_claude_task", {
				request: { query: "只回复 OK" },
			});
		});
		expect(await screen.findByText("2,950")).toBeInTheDocument();
	});
});
