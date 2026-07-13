# AGENTS.md

`airis-workspace` is a Rust utility for native workspace discovery, policy,
validation, and safe cleanup.

## Source of truth

- Native files (`package.json`, `pnpm-workspace.yaml`, `Cargo.toml`,
  `pyproject.toml`, `go.mod`, and Compose files) own project configuration.
- `.airis/policies.toml` owns AIRIS policy.
- AI rules, skills, hooks, and adapters are owned by AIris Code.
- `manifest.toml` is removed. Do not add a compatibility read/write path.

## Safety

- Cleanup is dry-run by default and must protect user source and knowledge.
- Do not remove `.airis/policies.toml` during uninstall.
- Use native host commands for build, test, lint, and release.

## Verification

```bash
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```
