# Architecture invariants

- Runtime application configuration belongs to each application repository.
- Native project files remain the source of truth for build and dependency state.
- `.airis/policies.toml` contains only workspace policy and quality gates.
- AI knowledge and adapters are distributed by AIris Code and remain separate
  from this runtime utility.
- Cleanup and uninstall must preserve user-owned source, knowledge, and policy.
