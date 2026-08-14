import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Profiler } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import "./i18n";

const { invokeMock, listenMock, tauriEventListeners } = vi.hoisted(() => ({
	invokeMock: vi.fn(),
	listenMock: vi.fn(),
	tauriEventListeners: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
	listen: listenMock,
}));

// Covers the user-visible local agent task workflow.
describe("App", () => {
	beforeEach(() => {
		window.history.replaceState({}, "", "/");
		invokeMock.mockReset();
		listenMock.mockReset();
		tauriEventListeners.clear();
		listenMock.mockImplementation(
			(eventName: string, listener: (event: { payload: unknown }) => void) => {
				tauriEventListeners.set(eventName, listener);
				return Promise.resolve(() => tauriEventListeners.delete(eventName));
			},
		);
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

	// Verifies that the desktop sidebar remains navigable after it is collapsed.
	it("collapses and expands the desktop sidebar", async () => {
		const user = userEvent.setup();
		render(<App />);

		const collapseButton = screen.getByRole("button", {
			name: /收起侧边栏|Collapse sidebar/,
		});
		expect(collapseButton).toHaveAttribute("aria-expanded", "true");

		await user.click(collapseButton);

		expect(
			screen.getByRole("button", { name: /展开侧边栏|Expand sidebar/ }),
		).toHaveAttribute("aria-expanded", "false");
		expect(
			screen.queryByText(/本地 Agent 实验室|Local agent lab/),
		).not.toBeInTheDocument();
		expect(
			screen.queryByText(/切换语言|Switch language/),
		).not.toBeInTheDocument();

		await user.click(
			screen.getByRole("button", { name: /展开侧边栏|Expand sidebar/ }),
		);

		expect(
			screen.getByText(/本地 Agent 实验室|Local agent lab/),
		).toBeInTheDocument();
		expect(screen.getByText(/切换语言|Switch language/)).toBeInTheDocument();
	});

	// Verifies that overlay title-bar controls do not cover desktop content.
	it("reserves title-bar space above desktop content", () => {
		render(<App />);

		expect(screen.getByRole("complementary").firstElementChild).toHaveClass(
			"pt-14",
		);
		expect(screen.getByRole("main").parentElement).toHaveClass("lg:pt-14");
	});

	// Verifies that routing updates the URL and redirects unsupported locations.
	it("navigates lazy routes and redirects unknown paths", async () => {
		const user = userEvent.setup();
		render(<App />);

		await user.click(
			within(screen.getByRole("complementary")).getByRole("button", {
				name: /运行看板|Run board/,
			}),
		);
		expect(window.location.pathname).toBe("/runs");
		expect(
			await screen.findByRole("heading", {
				name: /Agent 运行看板|Agent run board/,
			}),
		).toBeInTheDocument();
		for (const statusName of ["运行中", "等待用户", "已完成", "异常"]) {
			expect(
				screen.getByRole("heading", { name: statusName }),
			).toBeInTheDocument();
		}

		window.history.pushState({}, "", "/unsupported");
		window.dispatchEvent(new PopStateEvent("popstate"));

		await waitFor(() => expect(window.location.pathname).toBe("/"));
	});

	// Verifies that the run board can switch between vertical columns and horizontal rows.
	it("switches the run board between vertical and horizontal layouts", async () => {
		const user = userEvent.setup();
		render(<App />);

		await user.click(
			within(screen.getByRole("complementary")).getByRole("button", {
				name: /运行看板|Run board/,
			}),
		);

		const layoutGroup = await screen.findByRole("group", {
			name: /切换看板布局|Switch run board layout/,
		});
		const board = screen.getByTestId("run-board");
		const verticalButton = within(layoutGroup).getByRole("button", {
			name: /竖面板|Vertical panel/,
		});
		const horizontalButton = within(layoutGroup).getByRole("button", {
			name: /水平面板|Horizontal panel/,
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

	// Verifies that run cards keep a stable footprint and clamp overflowing copy.
	it("keeps run cards at fixed dimensions with clamped text", async () => {
		const user = userEvent.setup();
		render(<App />);

		await user.click(
			within(screen.getByRole("complementary")).getByRole("button", {
				name: /运行看板|Run board/,
			}),
		);

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

	// Verifies that an unchanged process snapshot does not commit another render.
	it("ignores unchanged process snapshots", async () => {
		let processProbeCount = 0;
		let renderCommitCount = 0;
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				processProbeCount += 1;
				return Promise.resolve({
					claude: false,
					codex: false,
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

		render(
			<Profiler id="comparison" onRender={() => renderCommitCount++}>
				<App />
			</Profiler>,
		);
		expect(
			await screen.findByText("Codex：已就绪，未运行"),
		).toBeInTheDocument();
		await waitFor(() => expect(processProbeCount).toBeGreaterThanOrEqual(1));
		await new Promise((resolve) => window.setTimeout(resolve, 50));
		const settledRenderCommitCount = renderCommitCount;

		await waitFor(() => expect(processProbeCount).toBeGreaterThanOrEqual(2), {
			timeout: 1500,
		});

		expect(renderCommitCount).toBe(settledRenderCommitCount);
	});

	// Verifies that unchanged authentication results keep the current UI state object.
	it("ignores unchanged authentication snapshots", async () => {
		let loginProbeCount = 0;
		let processProbeCount = 0;
		let renderCommitCount = 0;
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				processProbeCount += 1;
				return Promise.resolve({
					claude: false,
					codex: false,
					workbuddy: false,
				});
			}
			loginProbeCount += 1;
			return Promise.resolve({
				installed: true,
				loggedIn: true,
				authenticationMethod: "ChatGPT",
				model: "gpt-5.6-sol",
				reasoningEffort: "high",
			});
		});

		render(
			<Profiler id="comparison" onRender={() => renderCommitCount++}>
				<App />
			</Profiler>,
		);
		expect(
			await screen.findByText("Codex：已就绪，未运行"),
		).toBeInTheDocument();
		await waitFor(() => expect(loginProbeCount).toBeGreaterThanOrEqual(3));
		await waitFor(() => expect(processProbeCount).toBeGreaterThanOrEqual(1));
		await new Promise((resolve) => window.setTimeout(resolve, 50));
		const settledRenderCommitCount = renderCommitCount;

		window.dispatchEvent(new Event("focus"));
		await waitFor(() => expect(loginProbeCount).toBeGreaterThanOrEqual(6));
		await waitFor(() => expect(processProbeCount).toBeGreaterThanOrEqual(2));

		expect(renderCommitCount).toBe(settledRenderCommitCount);
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

	// Verifies that a native Codex configuration event refreshes model defaults immediately.
	it("refreshes Codex runtime defaults after its configuration changes", async () => {
		let codexModel = "gpt-5.6-sol";
		let codexProbeCount = 0;
		let finishPendingProbe = () => {};
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: false,
					codex: false,
					workbuddy: false,
				});
			}
			const status = {
				installed: true,
				loggedIn: true,
				authenticationMethod: "ChatGPT",
				model: command === "check_codex_login" ? codexModel : null,
				reasoningEffort: command === "check_codex_login" ? "high" : null,
			};
			if (command !== "check_codex_login") {
				return Promise.resolve(status);
			}

			codexProbeCount += 1;
			if (codexProbeCount !== 2) {
				return Promise.resolve(status);
			}

			return new Promise((resolve) => {
				finishPendingProbe = () => resolve(status);
			});
		});

		render(<App />);
		expect(await screen.findByText("gpt-5.6-sol")).toBeInTheDocument();

		codexModel = "gpt-5.6-terra";
		const listener = tauriEventListeners.get("codex-config-changed");
		expect(listener).toBeTypeOf("function");
		listener?.({ payload: null });
		await waitFor(() => expect(codexProbeCount).toBe(2));

		codexModel = "gpt-5.6-luna";
		listener?.({ payload: null });
		finishPendingProbe();

		expect(await screen.findByText("gpt-5.6-luna")).toBeInTheDocument();
		expect(codexProbeCount).toBe(3);
	});

	// Verifies that a native Claude configuration event refreshes model and effort immediately.
	it("refreshes Claude runtime settings after its configuration changes", async () => {
		let claudeModel = "claude-sonnet-4-6";
		let claudeEffort = "medium";
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
				loggedIn: true,
				authenticationMethod: "Claude account",
				model: command === "check_claude_login" ? claudeModel : "other-model",
				reasoningEffort:
					command === "check_claude_login" ? claudeEffort : "high",
			});
		});

		render(<App />);
		expect(await screen.findByText("claude-sonnet-4-6")).toBeInTheDocument();
		expect(screen.getByText("中 (medium)")).toBeInTheDocument();

		claudeModel = "claude-opus-4-6";
		claudeEffort = "high";
		const listener = tauriEventListeners.get("claude-config-changed");
		expect(listener).toBeTypeOf("function");
		listener?.({ payload: null });

		expect(await screen.findByText("claude-opus-4-6")).toBeInTheDocument();
		expect(
			within(screen.getByRole("button", { name: "Claude Code" })).getByText(
				"高 (high)",
			),
		).toBeInTheDocument();
	});

	// Verifies that native configuration subscriptions are removed when the page unmounts.
	it("removes agent configuration listeners on unmount", async () => {
		const { unmount } = render(<App />);
		await waitFor(() => {
			expect(tauriEventListeners.has("codex-config-changed")).toBe(true);
			expect(tauriEventListeners.has("claude-config-changed")).toBe(true);
		});

		unmount();

		await waitFor(() => {
			expect(tauriEventListeners.has("codex-config-changed")).toBe(false);
			expect(tauriEventListeners.has("claude-config-changed")).toBe(false);
		});
	});

	// Verifies that authentication probe processes do not masquerade as active Agent runs.
	it("keeps the idle status while authentication probes are running", async () => {
		let claudeLoginProbeCount = 0;
		let isClaudeLoginProbeRunning = false;
		let finishClaudeLoginProbe = () => {};
		const readyStatus = {
			installed: true,
			loggedIn: true,
			authenticationMethod: "Claude account",
			model: null,
			reasoningEffort: null,
		};
		invokeMock.mockImplementation((command: string) => {
			if (command === "check_agent_processes") {
				return Promise.resolve({
					claude: isClaudeLoginProbeRunning,
					codex: false,
					workbuddy: false,
				});
			}
			if (command === "check_claude_login") {
				claudeLoginProbeCount += 1;
				if (claudeLoginProbeCount === 1) {
					return Promise.resolve(readyStatus);
				}

				isClaudeLoginProbeRunning = true;
				return new Promise((resolve) => {
					finishClaudeLoginProbe = () => {
						isClaudeLoginProbeRunning = false;
						resolve(readyStatus);
					};
				});
			}
			return Promise.resolve(readyStatus);
		});

		render(<App />);
		expect(
			await screen.findByText("Claude Code：已就绪，未运行"),
		).toBeInTheDocument();

		window.dispatchEvent(new Event("focus"));
		await waitFor(() => expect(isClaudeLoginProbeRunning).toBe(true));
		await new Promise((resolve) => window.setTimeout(resolve, 1100));

		expect(screen.queryByText("Claude Code：运行中")).not.toBeInTheDocument();
		finishClaudeLoginProbe();
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
					thinkingDurationMs: 1200,
					toolCallCount: 2,
					toolCalls: [
						{ sequence: 1, name: "Read", durationMs: 120 },
						{ sequence: 2, name: "Bash", durationMs: 21_000 },
					],
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
					thinkingDurationMs: 900,
					toolCallCount: 1,
					toolCalls: [{ sequence: 1, name: "WebSearch", durationMs: 61_000 }],
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
					thinkingDurationMs: 400,
					toolCallCount: 0,
					toolCalls: [],
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
		expect(within(codexResult).getByText("思考时间")).toBeInTheDocument();
		expect(within(codexResult).getByText("1.20 s")).toBeInTheDocument();
		expect(within(codexResult).getByText("工具调用")).toBeInTheDocument();
		expect(within(codexResult).getByText("Read")).toBeInTheDocument();
		expect(within(codexResult).getByText("Bash")).toBeInTheDocument();
		expect(within(codexResult).getByText("21.00 s")).toHaveClass(
			"text-red-700",
		);
		expect(within(claudeResult).getByText("540 ms")).toBeInTheDocument();
		expect(within(claudeResult).getByText("61.00 s")).toHaveClass(
			"text-red-900",
		);
		expect(within(workbuddyResult).getByText("14.62 s")).toBeInTheDocument();
		expect(
			within(workbuddyResult).getByText("本次未调用工具"),
		).toBeInTheDocument();
		expect(within(codexResult).getByText("Codex 完成")).toBeInTheDocument();
		expect(within(claudeResult).getByText("Claude 完成")).toBeInTheDocument();
		expect(
			within(workbuddyResult).getByText("WorkBuddy 完成"),
		).toBeInTheDocument();
		for (const agentName of ["Codex", "Claude Code", "WorkBuddy"]) {
			expect(
				screen.getByText(`只回复任务完成 任务 ${agentName} 已结束`),
			).toBeInTheDocument();
		}
		expect(screen.getAllByText("请查看结果")).toHaveLength(3);
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
					thinkingDurationMs: 0,
					toolCallCount: 0,
					toolCalls: [],
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
					thinkingDurationMs: 0,
					toolCallCount: 0,
					toolCalls: [],
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
		expect(screen.getByText("执行对比 任务 Codex 已结束")).toBeInTheDocument();
		expect(screen.getAllByText("请查看结果")).toHaveLength(3);
	});
});
