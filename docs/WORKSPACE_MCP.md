# Workspace MCP

The MCP server is intentionally thin. It exposes only read-oriented discovery,
workspace cleanup reporting, policy checks, validation, and a force-gated clean
operation. It does not scaffold repositories, write configuration schemas, or
generate AI adapters.

Tools:

- `workspace_discover`
- `workspace_cleanup`
- `workspace_validate_all`
- `workspace_policy_check`
- `workspace_clean`

The authoritative inputs are native project files and `.airis/policies.toml`.
AIris Code owns installation and distribution of agent rules, skills, hooks, and
tool-specific adapters.
