# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**AIris Workspace** is a Docker-first monorepo workspace manager built in Rust. It enforces Docker-first development by auto-generating `package.json`, and `pnpm-workspace.yaml` from a single `manifest.toml`. `workspace.yaml` is generated metadata, not the user-editable manifest.

**Core Philosophy**: Prevent host pollution by blocking direct `pnpm`/`npm`/`yarn` execution and forcing Docker-first workflow via `airis` CLI commands. Special exception for Rust projects (local builds for GPU support).

**Current Version**: v1.1.0
- v1.0.2: Command unification (`[commands]`, `[guards]`, `[remap]`) - justfile now optional
- v1.1.0: Version automation (`[versioning]`, `airis bump-version`, Git hooks)

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
# Test init command (creates manifest.toml + workspace metadata)
cargo run -- init

# Validate MANIFEST + workspace metadata
cargo run -- validate
```

## ⚠️ airis init の仕様（絶対厳守）

**manifest.toml は神聖不可侵。**

`airis init` コマンドは manifest.toml を絶対に書き換えてはならない。

### 動作仕様

1. **manifest.toml が存在する場合**
   - 読み込み専用として扱う
   - package.json, pnpm-workspace.yaml, workspace.yaml を再生成
   - manifest.toml 自体は一切書き換えない

2. **manifest.toml が存在しない場合（初回のみ）**
   - 初回テンプレートを生成
   - 他のファイルも併せて生成

3. **manifest.toml を上書きする機能は存在しない**
   - `--force` フラグは存在しない
   - `--reset` コマンドは存在しない
   - manifest.toml を削除・上書きする手段は CLI に存在しない

### 実装ガード

```rust
pub fn run() -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILE);

    if manifest_path.exists() {
        // ✅ READ-ONLY MODE: Never modify existing manifest.toml
        let manifest = Manifest::load(manifest_path)?;
        Generator::from(&manifest).write_all()?;
        return Ok(());
    }

    // ✅ INITIAL CREATION MODE: Only when manifest.toml doesn't exist
    let template = Manifest::bootstrap_from_repo()?;
    template.save(manifest_path)?;
    Generator::from(&template).write_all()?;

    Ok(())
}
```

**この仕様に違反する実装は全てバグとして扱う。**

## Architecture & Code Structure

### Configuration Flow (manifest.toml → Generated Files)

1. **manifest.toml** (user-editable)
   - Parsed via `toml`
   - Describes dev apps, infra services, lint/test rules, package config (`src/manifest.rs`)

2. **workspace.yaml** (auto-generated metadata)
   - Derived from manifest.toml for IDE/tooling compatibility (`src/config/mod.rs`)

3. **Template Engine** (`src/templates/mod.rs`)
   - Uses Handlebars for templating with MANIFEST-driven data
   - Generates `package.json`, `pnpm-workspace.yaml`, `docker-compose.yml`
   - Optional: `justfile` (if `[just]` section is present)

4. **Generation Pipeline**
   - `init` command → creates or loads manifest.toml, then triggers template sync (src/commands/init.rs)
   - `commands::generate` module → helper invoked by `init` that syncs workspace.yaml + templates (src/commands/generate.rs)

### Key Design Patterns

**Docker-First Guards (v1.0.2+)**: `[guards]` section in manifest.toml defines:
- `deny`: Block commands for all users (e.g., `["npm", "yarn", "pnpm"]`)
- `forbid`: LLM-specific blocking (via MCP integration)
- `danger`: Prevent catastrophic commands (e.g., `["rm -rf /"]`)

**Command Unification (v1.0.2+)**: All operations via `airis` CLI:
- `[commands]` section defines user commands (install, up, down, dev, test, build, clean)
- `airis run <task>` executes commands from manifest.toml
- Built-in shorthands: `airis up`, `airis dev`, `airis shell`, etc.
- `[remap]` auto-translates banned commands to safe alternatives

**Version Automation (v1.1.0+)**: Automatic version bumping:
- `[versioning]` section with `strategy` (conventional-commits, auto, manual)
- `airis bump-version` command (--major, --minor, --patch, --auto)
- Git pre-commit hook for auto-bump on commit
- Syncs manifest.toml ↔ Cargo.toml

**Runtime Exceptions**: `apps[].runtime` field allows "local" builds (e.g., Rust with GPU support). Default is "docker".

**Catalog System (NEW DESIGN)**:
- `catalog` セクションでバージョンポリシー（`policy = "latest"` | `"lts"` | `"^X.Y.Z"`）を定義
- `airis workspace sync-deps` コマンドで npm registry から実際のバージョンを解決
- package.json の `pnpm.catalog` に数字を書き込む
- Dependencies は `"dep": "catalog:"` で catalog を参照
- **人間が編集するのは manifest.toml だけ、package.json は生成物**

**Design Philosophy**:
- **Avoid hardcoded version numbers** in manifest.toml
- Use version policies (`latest`, `lts`) instead
- Auto-resolve to actual versions at `sync-deps` time
- Lock files maintain reproducibility

**Auto-Generation Markers**: All generated files include `DO NOT EDIT` warnings and `_generated` metadata to prevent manual edits.

### Module Responsibilities

- **src/main.rs**: CLI entry point using `clap` derive macros
- **src/config/mod.rs**: Workspace YAML schema + helpers (generated metadata)
- **src/manifest.rs**: manifest.toml schema/helpers
- **src/commands/init.rs**: Creates or reloads manifest.toml, then re-syncs derived files
- **src/commands/generate.rs**: Helper that syncs workspace.yaml + templates from an in-memory Manifest
- **src/commands/manifest_cmd.rs**: Implements `airis manifest ...` plumbing
- **src/commands/run.rs**: Executes commands from `[commands]` section (v1.0.2+)
- **src/commands/bump_version.rs**: Version bumping with Conventional Commits (v1.1.0+)
- **src/commands/hooks.rs**: Git hooks installation (v1.1.0+)
- **src/templates/mod.rs**: Handlebars engine driven by MANIFEST data

## Important Constraints

### DO NOT violate these rules when making changes:

1. **Generated files must remain read-only**: Never encourage users to edit `package.json`, or `pnpm-workspace.yaml` directly. All changes go through `manifest.toml`. (justfile is optional in v1.0.2+)

2. **Docker-first is non-negotiable**: Do not weaken guard recipes or suggest host-level package manager usage (except for Rust projects with `runtime: local`).

3. **Rust edition is 2024**: Cargo.toml specifies `edition = "2024"` (line 4). Maintain compatibility.

4. **Template consistency**: When modifying templates, ensure:
   - Handlebars syntax is valid
   - Generated files include auto-generation warnings
   - Commands follow naming convention: `<action>` (e.g., `dev`, `build`, `test`)

## Configuration Schema Notes

**Mode types** (src/config/mod.rs:30-34):
- `docker-first`: Default. Allows local builds with explicit `runtime: local`
- `hybrid`: (not yet implemented)
- `strict`: (not yet implemented)

**WorkspaceApp variants** (src/config/mod.rs:52-59):
- `Simple(String)`: App name (type inferred from `apps` section)
- `Detailed`: Inline type specification

**App runtime resolution**: Keep Docker-first semantics. Runtime overrides (e.g., Rust local builds) should be modeled in MANIFEST extensions rather than reintroducing host-level exceptions elsewhere.

## Testing Strategy

When adding features:
1. Add unit tests in `#[cfg(test)]` modules (see examples in config/mod.rs:242-270, commands/generate.rs:92-118, commands/bump_version.rs:154-182)
2. Use `tempfile` crate for filesystem tests (already in dev-dependencies)
3. Test YAML parsing/serialization roundtrips
4. Verify template rendering produces valid output (valid JSON, TOML)

## Future Implementation Notes

**Planned but not yet implemented** (README.md:162-171):
- Environment variable validation
- LLM context generation
- MCP server integration
- Migration from existing projects

**New Features (Catalog Version Policy)**:
- [ ] Add `catalog` section to Manifest struct (src/manifest.rs)
  - `catalog.<package>.policy = "latest" | "lts" | "^X.Y.Z"`
- [ ] Implement `airis workspace sync-deps` command
  - Query npm registry for latest/lts versions
  - Resolve policy → actual version number
  - Write to package.json `pnpm.catalog`
- [ ] Update template generation
  - Generate package.json with `pnpm.catalog` from resolved versions
  - Add `_generated.from = "manifest.toml"` marker

**Implementation Priority**:
1. Schema addition (manifest.rs) - Define CatalogSection struct
2. npm registry client - Query API for version info
3. sync-deps command - Main logic for version resolution
4. Template updates - package.json generation with catalog

**New Features (Auto-Migration)**:
- [ ] Project discovery module (src/commands/discover.rs)
  - Scan apps/ directory → detect Next.js/Node/Rust apps
  - Scan libs/ directory → detect TypeScript libraries
  - Find docker-compose.yml locations (root, supabase/, traefik/, etc.)
  - Parse existing package.json → extract catalog info
- [ ] Safe migration module (src/commands/migrate.rs)
  - Move docker-compose.yml to correct locations (NEVER overwrite)
  - Create workspace/ directory if missing
  - Warn user if file already exists at target location
- [ ] Enhanced init command
  - Run discovery → migration → generation flow
  - Generate manifest.toml from detected project structure
  - Display changes and ask for confirmation (unless --force)
  - User runs `airis init` and everything is optimized

**Auto-Migration Workflow**:
```
airis init
  ↓
1. Discovery Phase
   - Scan apps/, libs/
   - Detect docker-compose.yml locations
   - Parse package.json catalog
  ↓
2. Migration Phase (safe, no overwrites)
   - Create workspace/ if missing
   - Move root/docker-compose.yml → workspace/docker-compose.yml
   - Validate supabase/docker-compose.yml, traefik/docker-compose.yml
  ↓
3. Generation Phase
   - Generate manifest.toml with:
     - Detected apps/libs
     - Detected compose file paths in orchestration.dev
     - Extracted catalog from package.json
   - Generate workspace.yaml, package.json, etc.
  ↓
4. Verification Phase
   - Show diff/changes
   - Ask confirmation (unless --force)
   - Save files
```

**Safety Rules**:
- NEVER overwrite existing files without user confirmation
- ALWAYS create backups before migration (.bak suffix)
- ALWAYS warn user if target file exists
- Prefer moving files over copying (preserve git history)

Do not implement these features without checking the current project roadmap.
