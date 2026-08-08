import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Releases the rendered DOM after every test so cases cannot leak state into each other.
afterEach(() => {
	cleanup();
});
