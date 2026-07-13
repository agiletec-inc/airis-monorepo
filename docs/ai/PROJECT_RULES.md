# AIRIS Workspace project rules

- Native repository files are the source of truth. Do not introduce or restore `manifest.toml`.
- `.airis/policies.toml` is the only AIRIS Workspace policy configuration.
- AI rules, skills, hooks, and adapters belong to AIris Code and must not be generated here.
- Preserve user-owned source, knowledge, and policy files during cleanup and uninstall.
- Use host-native repository commands for build, test, lint, and deployment.
