import "@testing-library/jest-dom/vitest";
import { Toast } from "@heroui/react";
import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach } from "vitest";
import i18n from "./src/i18n";

// Provides the element resize contract used by stacked HeroUI toasts.
class ResizeObserverMock implements ResizeObserver {
	observe() {}
	unobserve() {}
	disconnect() {}
}

globalThis.ResizeObserver = ResizeObserverMock;

// Provides the browser media-query contract used by responsive HeroUI components.
Object.defineProperty(window, "matchMedia", {
	writable: true,
	value: (query: string): MediaQueryList => ({
		matches: false,
		media: query,
		onchange: null,
		addListener: () => {},
		removeListener: () => {},
		addEventListener: () => {},
		removeEventListener: () => {},
		dispatchEvent: () => false,
	}),
});

// Starts every UI test from the Chinese locale without leaking persisted state.
beforeEach(async () => {
	localStorage.clear();
	await i18n.changeLanguage("zh-CN");
});

// Releases the rendered DOM after every test so cases cannot leak state into each other.
afterEach(() => {
	Toast.toast.clear();
	cleanup();
});
