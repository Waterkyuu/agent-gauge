# AgentGauge

Desktop scaffold for measuring and comparing local AI agent performance.

## Stack

- Tauri 2
- React 19 and TypeScript
- Vite
- HeroUI 3
- Tailwind CSS 4
- Biome 2

## Development

```bash
pnpm install
pnpm tauri dev
```

## Quality checks

```bash
pnpm check
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
```
