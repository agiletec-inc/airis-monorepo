# Review checklist

- Does the change preserve native project files as the source of truth?
- Does policy remain in `.airis/policies.toml`?
- Does cleanup avoid user source, knowledge, and policy files?
- Are AI rules and adapters left to AIris Code?
- Run `cargo test --all-targets` and `cargo clippy --all-targets --all-features -- -D warnings`.
