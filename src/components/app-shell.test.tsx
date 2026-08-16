import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppShell } from "./app-shell";

describe("AppShell", () => {
	it("navigates to comparison history from the shared navigation", () => {
		const onNavigate = vi.fn();
		render(
			<AppShell currentPath="/" onNavigate={onNavigate}>
				<main>content</main>
			</AppShell>,
		);

		fireEvent.click(screen.getAllByRole("button", { name: "历史对比" })[0]);

		expect(onNavigate).toHaveBeenCalledWith("/comparison-history");
	});
});
