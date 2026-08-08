import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
	plugins: [react()],
	test: {
		css: true,
		environment: "jsdom",
		setupFiles: ["./src/test-setup.ts"],
		coverage: {
			provider: "v8",
			reporter: ["text", "html"],
			include: ["src/**/*.{ts,tsx}"],
			exclude: ["src/main.tsx", "src/test-setup.ts", "src/vite-env.d.ts"],
		},
	},
});
