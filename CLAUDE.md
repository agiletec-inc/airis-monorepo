# AIRIS Workspace

`airis-workspace` is the Rust implementation of the AIRIS Workspace utility:
native workspace discovery, policy, validation, and safe cleanup. It
deliberately has no `manifest.toml` dependency.

## Source of truth

- Native files (`package.json`, `pnpm-workspace.yaml`, `Cargo.toml`,
  `pyproject.toml`, `go.mod`, and Compose files) own project configuration.
- `.airis/policies.toml` owns AIRIS policy.
- AI rules, skills, hooks, and tool adapters are distributed by AIris Code.
- `manifest.toml` is removed. Never reintroduce manifest-driven generation,
  migration, or adapter syncing, and do not add a compatibility read/write path.

## Safety

- Cleanup is dry-run by default and must protect user source and knowledge.
- Do not remove `.airis/policies.toml` during uninstall.
- Keep cleanup and uninstall conservative because user-edited knowledge is valuable.
- Use native host commands for build, test, lint, and release.

## Verification

```bash
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```
