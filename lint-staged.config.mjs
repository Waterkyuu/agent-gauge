/** @type {import("lint-staged").Configuration} */
const config = {
	"src/**/*.{ts,tsx}": [
		"biome check --write --no-errors-on-unmatched",
		() => "pnpm typecheck",
	],
	"src/**/*.{js,jsx,json,jsonc,css}":
		"biome check --write --no-errors-on-unmatched",
	"src-tauri/**/*.rs": [
		() => "cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check",
		() =>
			"cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings",
	],
	"src-tauri/**/*.{json,jsonc}": "biome check --write --no-errors-on-unmatched",
	"src-tauri/{Cargo.toml,Cargo.lock,**/*.toml}": () =>
		"cargo check --manifest-path src-tauri/Cargo.toml --all-targets --all-features",
};

export default config;
