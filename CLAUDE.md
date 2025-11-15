# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**AIris Workspace** is a Docker-first monorepo workspace manager built in Rust. It enforces Docker-first development by auto-generating `justfile`, `package.json`, and `pnpm-workspace.yaml` from a single `workspace.yaml` manifest.

**Core Philosophy**: Prevent host pollution by blocking direct `pnpm`/`npm`/`yarn` execution and forcing Docker-first workflow. Special exception for Rust projects (local builds for GPU support).

## Build & Development Commands

### Building the CLI
```bash
# Build debug binary
cargo build

# Build release binary
cargo build --release

# Install locally (for testing)
cargo install --path .

# Run tests
cargo test
```

### Testing the CLI
```bash
# Test init command (creates workspace.yaml + generates files)
cargo run -- init

# Test with force flag
cargo run -- init --force

# Validate workspace.yaml
cargo run -- validate
```

## Architecture & Code Structure

### Configuration Flow (workspace.yaml → Generated Files)

1. **workspace.yaml** (user-editable manifest)
   - `WorkspaceConfig` struct in `src/config/mod.rs:8-26`
   - Supports `catalog` (dependency versions), `workspaces` (apps/libs), `apps` (app-specific config)
   - Parsed via `serde_yaml`

2. **Template Engine** (`src/templates/mod.rs`)
   - Uses Handlebars for templating
   - Embeds templates as const strings (lines 112-295)
   - Three main templates: `justfile`, `package.json`, `pnpm-workspace.yaml`

3. **Generation Pipeline**
   - `init` command → creates default `workspace.yaml` → auto-calls `generate` (src/commands/init.rs:40)
   - `generate` command → loads config → renders templates → writes files (src/commands/generate.rs)

### Key Design Patterns

**Docker-First Guards**: Generated justfile contains guard recipes that block host-level `pnpm`/`npm`/`yarn` with helpful error messages (templates/mod.rs:227-247). This is the project's core enforcement mechanism.

**Runtime Exceptions**: `apps[].runtime` field allows "local" builds (e.g., Rust with GPU support). Default is "docker".

**Catalog System**: `catalog` field maps to pnpm's catalog feature. Dependencies in workspace packages reference versions via `"dep": "catalog:"`.

**Auto-Generation Markers**: All generated files include `DO NOT EDIT` warnings and `_generated` metadata to prevent manual edits.

### Module Responsibilities

- **src/main.rs**: CLI entry point using `clap` derive macros
- **src/config/mod.rs**: Configuration schema and YAML parsing/saving
- **src/commands/init.rs**: Creates default workspace.yaml + calls generate
- **src/commands/generate.rs**: Orchestrates template rendering for all outputs
- **src/templates/mod.rs**: Handlebars engine and embedded templates

## Important Constraints

### DO NOT violate these rules when making changes:

1. **Generated files must remain read-only**: Never encourage users to edit `justfile`, `package.json`, or `pnpm-workspace.yaml` directly. All changes go through `workspace.yaml`.

2. **Docker-first is non-negotiable**: Do not weaken guard recipes or suggest host-level package manager usage (except for Rust projects with `runtime: local`).

3. **Rust edition is 2024**: Cargo.toml specifies `edition = "2024"` (line 4). Maintain compatibility.

4. **Template consistency**: When modifying templates, ensure:
   - Handlebars syntax is valid
   - Generated files include auto-generation warnings
   - Just recipes follow naming convention: `<action>-<type>` (e.g., `dev-next`, `build-rust`)

## Configuration Schema Notes

**Mode types** (src/config/mod.rs:30-34):
- `docker-first`: Default. Allows local builds with explicit `runtime: local`
- `hybrid`: (not yet implemented)
- `strict`: (not yet implemented)

**WorkspaceApp variants** (src/config/mod.rs:52-59):
- `Simple(String)`: Just app name (type inferred from `apps` section)
- `Detailed`: Inline type specification

**App runtime resolution** (examples/workspace.yaml:58-65):
```yaml
duplicate-finder:
  type: rust
  runtime: local  # Exception to Docker-first
  reason: "Apple Silicon GPU (Metal) acceleration"
```

## Testing Strategy

When adding features:
1. Add unit tests in `#[cfg(test)]` modules (see examples in config/mod.rs:242-270, commands/generate.rs:92-118)
2. Use `tempfile` crate for filesystem tests (already in dev-dependencies)
3. Test YAML parsing/serialization roundtrips
4. Verify template rendering produces valid output (justfile syntax, valid JSON)

## Future Implementation Notes

**Planned but not yet implemented** (README.md:162-171):
- Environment variable validation
- LLM context generation
- MCP server integration
- Migration from existing projects

Do not implement these features without checking the current project roadmap.
