import "@testing-library/jest-dom/vitest";
import { act, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import type { AgentProcessStates } from "@/types/agent";

const apiMocks = vi.hoisted(() => ({
	checkAgentProcesses: vi.fn(),
	checkClaudeLogin: vi.fn(),
	checkCodexLogin: vi.fn(),
	checkWorkBuddyLogin: vi.fn(),
	onClaudeConfigChanged: vi.fn(),
	onCodexConfigChanged: vi.fn(),
	onAgentProcessStatesChanged: vi.fn(),
	runClaudeTask: vi.fn(),
	runCodexTask: vi.fn(),
	runWorkBuddyTask: vi.fn(),
}));

vi.mock("@/api/agent", () => ({
	checkAgentProcesses: apiMocks.checkAgentProcesses,
	onAgentProcessStatesChanged: apiMocks.onAgentProcessStatesChanged,
}));

vi.mock("@/api/claude", () => ({
	checkClaudeLogin: apiMocks.checkClaudeLogin,
	onClaudeConfigChanged: apiMocks.onClaudeConfigChanged,
	runClaudeTask: apiMocks.runClaudeTask,
}));

vi.mock("@/api/codex", () => ({
	checkCodexLogin: apiMocks.checkCodexLogin,
	onCodexConfigChanged: apiMocks.onCodexConfigChanged,
	runCodexTask: apiMocks.runCodexTask,
}));

vi.mock("@/api/workbuddy", () => ({
	checkWorkBuddyLogin: apiMocks.checkWorkBuddyLogin,
	runWorkBuddyTask: apiMocks.runWorkBuddyTask,
}));

import ComparisonPage from ".";

const RUNTIME_STATUS = {
	installed: true,
	loggedIn: true,
	authenticationMethod: "test",
	model: "test-model",
	reasoningEffort: "medium",
};

let processStateListener: ((states: AgentProcessStates) => void) | null = null;
const stopProcessStateListener = vi.fn();

// Covers process checks owned by the comparison page lifecycle.
describe("ComparisonPage process checks", () => {
	beforeEach(async () => {
		vi.useFakeTimers();
		await i18n.changeLanguage("zh-CN");
		stopProcessStateListener.mockClear();
		apiMocks.checkAgentProcesses.mockResolvedValue({
			claude: false,
			codex: false,
			workbuddy: false,
		});
		apiMocks.checkClaudeLogin.mockResolvedValue(RUNTIME_STATUS);
		apiMocks.checkCodexLogin.mockResolvedValue(RUNTIME_STATUS);
		apiMocks.checkWorkBuddyLogin.mockResolvedValue(RUNTIME_STATUS);
		apiMocks.onClaudeConfigChanged.mockResolvedValue(vi.fn());
		apiMocks.onCodexConfigChanged.mockResolvedValue(vi.fn());
		apiMocks.onAgentProcessStatesChanged.mockImplementation((listener) => {
			processStateListener = listener;
			return Promise.resolve(stopProcessStateListener);
		});
	});

	afterEach(() => {
		processStateListener = null;
		vi.useRealTimers();
		vi.clearAllMocks();
	});

	it("does not repeat process checks on a one-second timer", async () => {
		render(<ComparisonPage />);

		await act(async () => {
			await Promise.resolve();
			await Promise.resolve();
		});
		expect(apiMocks.checkAgentProcesses).toHaveBeenCalledTimes(1);

		await act(async () => {
			await vi.advanceTimersByTimeAsync(3_100);
		});

		expect(apiMocks.checkAgentProcesses).toHaveBeenCalledTimes(1);
	});

	it("applies native process state changes to the Agent card", async () => {
		render(<ComparisonPage />);

		await act(async () => {
			await Promise.resolve();
			await Promise.resolve();
		});

		expect(apiMocks.onAgentProcessStatesChanged).toHaveBeenCalledTimes(1);

		act(() => {
			processStateListener?.({
				claude: false,
				codex: true,
				workbuddy: false,
			});
		});

		expect(
			within(screen.getByRole("button", { name: "Codex" })).getByText("已启动"),
		).toBeInTheDocument();
	});

	it("removes the native process listener when the page unmounts", async () => {
		const { unmount } = render(<ComparisonPage />);

		await act(async () => {
			await Promise.resolve();
			await Promise.resolve();
		});
		unmount();
		await act(async () => {
			await Promise.resolve();
		});

		expect(stopProcessStateListener).toHaveBeenCalledTimes(1);
	});
});
