import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
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

	// Verifies that every product is presented as a selectable comparison target.
	it("shows every local agent status and selects ready products", async () => {
		render(<App />);

		expect(await screen.findByText("Codex：运行中")).toBeInTheDocument();
		expect(screen.getByText("Claude Code：已就绪，未运行")).toBeInTheDocument();
		expect(screen.getByText("WorkBuddy：已就绪，未运行")).toBeInTheDocument();
		expect(screen.getByText("gpt-5.6-sol")).toBeInTheDocument();
		expect(screen.getByText("高 (high)")).toBeInTheDocument();
		expect(screen.getByText("fast-model")).toBeInTheDocument();
		expect(screen.getByLabelText("任务内容")).toBeEnabled();
		expect(screen.getByRole("button", { name: "Codex" })).toHaveAttribute(
			"aria-pressed",
			"true",
		);
		expect(screen.getByRole("button", { name: "Claude Code" })).toHaveAttribute(
			"aria-pressed",
			"true",
		);
		expect(screen.getByRole("button", { name: "WorkBuddy" })).toHaveAttribute(
			"aria-pressed",
			"true",
		);

		await userEvent.click(screen.getByRole("button", { name: "English" }));
		expect(
			screen.getByText("Run one task across agents. Compare what matters."),
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

	// Verifies that one query runs across all selected products for comparison.
	it("runs selected agents in parallel and compares their metrics", async () => {
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
					response: "Codex 完成",
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
			if (command === "run_claude_task") {
				return Promise.resolve({
					response: "Claude 完成",
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
			if (command === "run_workbuddy_task") {
				return Promise.resolve({
					response: "WorkBuddy 完成",
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
			if (command === "check_claude_login") {
				return Promise.resolve({
					installed: true,
					loggedIn: true,
					authenticationMethod: "Claude account",
					model: "claude-sonnet",
					reasoningEffort: null,
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
			if (command === "check_codex_login") {
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
		await user.click(
			screen.getByRole("button", { name: "运行 3 个 Agent 对比" }),
		);

		await waitFor(() => {
			expect(invokeMock).toHaveBeenCalledWith("run_codex_task", {
				request: { query: "只回复任务完成" },
			});
			expect(invokeMock).toHaveBeenCalledWith("run_claude_task", {
				request: { query: "只回复任务完成" },
			});
			expect(invokeMock).toHaveBeenCalledWith("run_workbuddy_task", {
				request: { query: "只回复任务完成" },
			});
		});

		const codexResult = await screen.findByRole("article", {
			name: "Codex 对比结果",
		});
		const claudeResult = screen.getByRole("article", {
			name: "Claude Code 对比结果",
		});
		const workbuddyResult = screen.getByRole("article", {
			name: "WorkBuddy 对比结果",
		});
		expect(within(codexResult).getByText("680 ms")).toBeInTheDocument();
		expect(within(codexResult).getByText("1,280")).toBeInTheDocument();
		expect(within(claudeResult).getByText("540 ms")).toBeInTheDocument();
		expect(within(workbuddyResult).getByText("14.62 s")).toBeInTheDocument();
		expect(within(codexResult).getByText("Codex 完成")).toBeInTheDocument();
		expect(within(claudeResult).getByText("Claude 完成")).toBeInTheDocument();
		expect(
			within(workbuddyResult).getByText("WorkBuddy 完成"),
		).toBeInTheDocument();
	});

	// Verifies that deselected products are excluded from a comparison run.
	it("runs only the products selected by the user", async () => {
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: true,
					workbuddy: true,
				});
			}
			if (command.startsWith("run_")) {
				return Promise.resolve({
					response: "OK",
					totalDurationMs: 1000,
					timeToFirstTokenMs: 500,
					tokenUsage: null,
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "account",
				model: "model",
				reasoningEffort: null,
			});
		});
		const user = userEvent.setup();

		render(<App />);
		await user.click(await screen.findByRole("button", { name: "WorkBuddy" }));
		await user.type(screen.getByLabelText("任务内容"), "只回复 OK");
		await user.click(
			screen.getByRole("button", { name: "运行 2 个 Agent 对比" }),
		);

		await waitFor(() => {
			expect(invokeMock).toHaveBeenCalledWith("run_codex_task", {
				request: { query: "只回复 OK" },
			});
			expect(invokeMock).toHaveBeenCalledWith("run_claude_task", {
				request: { query: "只回复 OK" },
			});
		});
		expect(invokeMock).not.toHaveBeenCalledWith("run_workbuddy_task", {
			request: { query: "只回复 OK" },
		});
	});

	// Verifies that one product failure does not discard successful comparison results.
	it("keeps successful results when another selected agent fails", async () => {
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: true,
					workbuddy: false,
				});
			}
			if (command === "run_codex_task") {
				return Promise.reject(new Error("Codex benchmark failed"));
			}
			if (command.startsWith("run_")) {
				return Promise.resolve({
					response: "成功结果",
					totalDurationMs: 1000,
					timeToFirstTokenMs: 500,
					tokenUsage: null,
				});
			}
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "account",
				model: "model",
				reasoningEffort: null,
			});
		});
		const user = userEvent.setup();

		render(<App />);
		await screen.findByText("Codex：运行中");
		await user.type(screen.getByLabelText("任务内容"), "执行对比");
		await user.click(
			screen.getByRole("button", { name: "运行 3 个 Agent 对比" }),
		);

		const codexResult = await screen.findByRole("article", {
			name: "Codex 对比结果",
		});
		expect(
			within(codexResult).getByText("Codex benchmark failed"),
		).toBeInTheDocument();
		expect(screen.getAllByText("成功结果")).toHaveLength(2);
	});
});
