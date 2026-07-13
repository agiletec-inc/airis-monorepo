# AIRIS Workspace

This repository contains the Rust implementation of the AIRIS Workspace
utility. It deliberately has no `manifest.toml` dependency.

- Native repository files are authoritative.
- `.airis/policies.toml` is the policy source of truth.
- AI rules, skills, hooks, and tool adapters are distributed by AIris Code.
- Never reintroduce manifest-driven generation, migration, or adapter syncing.
- Keep cleanup and uninstall conservative because user-edited knowledge is valuable.

Run `cargo test --all-targets` and `cargo clippy --all-targets --all-features -- -D warnings` before handoff.
