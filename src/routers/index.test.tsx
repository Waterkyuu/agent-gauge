import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppRouter } from ".";

vi.mock("@/pages/comparison", () => new Promise(() => {}));
vi.mock("@/pages/comparison-history", () => ({
	default: () => <main>comparison history route</main>,
}));

describe("AppRouter", () => {
	it("gives the loading description an explicit readable width", () => {
		window.history.pushState({}, "", "/");
		render(<AppRouter />);

		const loadingPage = screen.getByRole("status", { name: "正在加载页面" });
		expect(loadingPage.querySelector(".mt-4")).toHaveClass("max-w-[36rem]");
	});

	it("renders comparison detail ids through the history route", async () => {
		window.history.pushState({}, "", "/comparison-history/42");
		render(<AppRouter />);

		expect(
			await screen.findByText("comparison history route"),
		).toBeInTheDocument();
	});
});
