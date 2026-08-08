import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

// Groups the user-visible expectations for the AgentGauge starter screen.
describe("App", () => {
	// Verifies that the starter screen exposes its product purpose and primary action.
	it("renders the AgentGauge product message", () => {
		render(<App />);

		expect(
			screen.getByRole("heading", {
				name: "Measure where your agents spend their time.",
			}),
		).toBeInTheDocument();
		expect(
			screen.getByRole("button", { name: "Start building" }),
		).toBeInTheDocument();
	});

	// Verifies that the scaffold communicates every configured technology.
	it("lists the configured application stack", () => {
		render(<App />);

		for (const technology of [
			"Tauri 2",
			"React 19",
			"TypeScript",
			"Biome",
			"HeroUI 3",
			"Tailwind CSS 4",
		]) {
			expect(screen.getByText(technology)).toBeInTheDocument();
		}
	});
});
