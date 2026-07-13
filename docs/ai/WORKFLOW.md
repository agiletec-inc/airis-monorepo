# Workflow

1. Inspect native project metadata with `airis workspace discover`.
2. Run `airis workspace policy check` and `airis workspace validate all`.
3. Use the repository's native package/build/test commands.
4. Use `airis workspace clean` as a dry-run first; pass `--force` only with approval.
5. Manage AI instructions through AIris Code, not this repository.
