import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { AgentSelectionCard } from "./agent-selection-card";

const AVAILABILITY_CASES = [
	["notInstalled", "未安装"],
	["notLoggedIn", "未登录"],
	["agentReady", "未启动"],
	["agentRunning", "已启动"],
] as const;

// Covers the user-visible availability state rendered by one Agent card.
describe("AgentSelectionCard", () => {
	beforeEach(async () => {
		await i18n.changeLanguage("zh-CN");
	});

	it.each(AVAILABILITY_CASES)(
		"renders the %s availability label",
		(statusKey, expectedLabel) => {
			render(
				<AgentSelectionCard
					agent="codex"
					isDisabled={false}
					isSelected={false}
					onToggle={vi.fn()}
					runtimeStatus={null}
					statusMessage={i18n.t(statusKey)}
					statusTone="bg-mute"
				/>,
			);

			expect(screen.getByRole("button", { name: "Codex" })).toHaveTextContent(
				expectedLabel,
			);
		},
	);
});
