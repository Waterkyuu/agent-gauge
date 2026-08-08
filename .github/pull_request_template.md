## Summary

Describe the user-visible outcome and the reason for this change.

## Validation

- [ ] `pnpm typecheck`
- [ ] `pnpm check`
- [ ] `pnpm test`
- [ ] `pnpm build`
- [ ] Rust checks from `src-tauri/AGENTS.md` when backend files changed

## Risk review

- [ ] No new system permission or capability is required
- [ ] Sensitive Agent data is not logged, persisted, or exported unexpectedly
- [ ] macOS and Windows behavior has been considered
- [ ] Tests cover new behavior and relevant failure paths

## Screenshots or logs

Include redacted evidence when the change affects UI or runtime behavior.
