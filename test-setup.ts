import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach } from "vitest";
import i18n from "./src/i18n";

// Starts every UI test from the Chinese locale without leaking persisted state.
beforeEach(async () => {
	localStorage.clear();
	await i18n.changeLanguage("zh-CN");
});

// Releases the rendered DOM after every test so cases cannot leak state into each other.
afterEach(() => {
	cleanup();
});
