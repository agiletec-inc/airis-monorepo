# AIRIS Workspace

`airis-workspace` is a small, convention-based utility for polyglot repositories.
It discovers native project metadata, checks `.airis/policies.toml`, validates
workspace hygiene, and removes build artifacts safely.

## Source of truth

- Native project files remain authoritative: `package.json`, `pnpm-workspace.yaml`,
  `Cargo.toml`, `pyproject.toml`, `go.mod`, and Compose files where present.
- Workspace policy lives in `.airis/policies.toml`.
- AI rules, skills, hooks, and tool-specific adapters are owned and distributed by
  AIris Code (`agiletec/products/airis/code`).
- This repository does not read, write, generate, or migrate `manifest.toml`.

## Commands

```bash
airis workspace discover
airis workspace validate all
airis workspace policy check
airis workspace clean                 # dry-run
airis workspace clean --force
airis workspace new web my-app
airis workspace mcp
```

`clean` never deletes user-owned source, policy, or knowledge files. `workspace
uninstall` removes AIRIS hook markers but leaves user-managed files and policy
data intact.

## Runtime model

AIRIS does not replace pnpm, cargo, uv, Docker Compose, Nx, or Turborepo. Use the
native toolchain for builds and tests. The Rust binary is intentionally limited
to repository discovery, policy, validation, and safe hygiene operations.

## Development

```bash
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```
