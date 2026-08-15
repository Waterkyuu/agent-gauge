import "@testing-library/jest-dom/vitest";
import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import "@/i18n";

const apiMocks = vi.hoisted(() => ({
	checkAgentProcesses: vi.fn(),
	checkClaudeLogin: vi.fn(),
	checkCodexLogin: vi.fn(),
	checkWorkBuddyLogin: vi.fn(),
	onClaudeConfigChanged: vi.fn(),
	onCodexConfigChanged: vi.fn(),
	runClaudeTask: vi.fn(),
	runCodexTask: vi.fn(),
	runWorkBuddyTask: vi.fn(),
}));

vi.mock("@/api/agent", () => ({
	checkAgentProcesses: apiMocks.checkAgentProcesses,
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

// Covers process checks owned by the comparison page lifecycle.
describe("ComparisonPage process checks", () => {
	beforeEach(() => {
		vi.useFakeTimers();
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
	});

	afterEach(() => {
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
});
