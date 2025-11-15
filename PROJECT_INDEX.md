# Project Index: airis-workspace

**Generated**: 2025-11-14
**Version**: 0.1.0
**Language**: Rust (edition 2024)

---

## 📁 Project Structure

```
airis-workspace/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/
│   │   ├── mod.rs           # Command module exports
│   │   ├── init.rs          # Initialize workspace
│   │   └── generate.rs      # Generate files from config
│   ├── config/
│   │   └── mod.rs           # WorkspaceConfig schema
│   └── templates/
│       └── mod.rs           # Handlebars template engine
├── templates/
│   └── justfile-global      # Justfile template
├── examples/
│   └── workspace.yaml       # Example configuration
├── Cargo.toml               # Rust package manifest
├── README.md                # User documentation
└── CLAUDE.md                # Claude Code guidance
```

---

## 🚀 Entry Points

### CLI Binary
- **Path**: `src/main.rs`
- **Binary name**: `airis-workspace`
- **Commands**:
  - `init [--force]` - Create workspace.yaml + auto-generate files
  - `validate` - Validate workspace configuration (not implemented)

### Key Dependencies
- `clap` 4.5 - CLI argument parsing (derive macros)
- `serde` 1.0 + `serde_yaml` 0.9 - Configuration parsing
- `handlebars` 6.2 - Template rendering
- `colored` 2.1 - Terminal color output
- `anyhow` 1.0 - Error handling

---

## 📦 Core Modules

### `src/config/mod.rs` (240 lines)
**Purpose**: Configuration schema and YAML serialization

**Key Types**:
- `WorkspaceConfig` - Root configuration struct
  - `version: u8` - Config format version
  - `name: String` - Project name
  - `mode: Mode` - Docker-first | hybrid | strict
  - `catalog: IndexMap<String, String>` - Dependency versions (pnpm catalog)
  - `workspaces: Workspaces` - Apps and libs
  - `apps: IndexMap<String, AppConfig>` - App-specific config
  - `docker: DockerConfig` - Docker settings
  - `rules: Rules` - Enforcement rules
  - `just: JustConfig` - Justfile generation settings
  - `types: IndexMap<String, TypeConfig>` - Type-specific configs

- `AppConfig` - Per-app configuration
  - `app_type: String` - nextjs | hono | rust | node
  - `port: Option<u16>` - Port binding
  - `runtime: Option<Runtime>` - docker | local (for GPU support)
  - `reason: Option<String>` - Explanation for local runtime

**Methods**:
- `load(path)` - Parse workspace.yaml
- `save(path)` - Write workspace.yaml
- `workspace_service()` - Get Docker service name
- `get_app_name(app)` - Extract app name from WorkspaceApp
- `get_app_type(app)` - Resolve app type

**Tests**: 1 unit test for YAML parsing

---

### `src/commands/init.rs` (111 lines)
**Purpose**: Initialize workspace with default configuration

**Exported Functions**:
- `run(force: bool) -> Result<()>`
  1. Check if workspace.yaml exists (bail unless --force)
  2. Detect Makefile presence
  3. Create default WorkspaceConfig with:
     - React 19.0.0, Next.js 15.4.0, TypeScript 5.8.0, Vitest 2.0.0
     - Apps: dashboard (nextjs), api (node)
  4. Auto-call `generate::run("all")`
  5. Remove Makefile if present

**Internal Functions**:
- `create_default_config()` - Generate default workspace.yaml structure

**Tests**: 1 unit test for default config creation

---

### `src/commands/generate.rs` (119 lines)
**Purpose**: Generate justfile, package.json, pnpm-workspace.yaml from workspace.yaml

**Exported Functions**:
- `run(target: &str) -> Result<()>`
  - Targets: `"all"` | `"justfile"` | `"package"` | `"pnpm"`
  - Loads workspace.yaml
  - Calls TemplateEngine to render files

**Internal Functions**:
- `generate_justfile(config, engine)` - Render justfile
- `generate_package_json(config, engine)` - Render package.json
- `generate_pnpm_workspace(config, engine)` - Render pnpm-workspace.yaml

**Tests**: 1 unit test for justfile generation

---

### `src/templates/mod.rs` (296 lines)
**Purpose**: Handlebars template engine and embedded templates

**Main Struct**:
- `TemplateEngine`
  - `new()` - Register embedded templates
  - `render_justfile(config)` - Generate justfile
  - `render_package_json(config)` - Generate package.json
  - `render_pnpm_workspace(config)` - Generate pnpm-workspace.yaml

**Data Preparation**:
- `prepare_justfile_data()` - Extract apps/libs for justfile
- `prepare_package_json_data()` - Extract catalog for package.json
- `prepare_pnpm_workspace_data()` - Extract packages for pnpm

**Embedded Templates** (const strings):
- `JUSTFILE_TEMPLATE` (145 lines)
  - Docker management recipes (up, down, install, workspace)
  - Type-specific commands (dev-ts, build-next, run-rust)
  - Docker-first guards (block pnpm/npm/yarn on host)
  - Per-app shortcuts (dev-dashboard, dev-api)

- `PACKAGE_JSON_TEMPLATE` (22 lines)
  - Root package.json with pnpm catalog
  - PackageManager pinning: pnpm@10.12.0

- `PNPM_WORKSPACE_TEMPLATE` (15 lines)
  - Workspace packages list (apps/*, libs/*)
  - Catalog references

---

## 🔧 Configuration

### `Cargo.toml`
- **Package**: `airis` 0.1.0
- **Edition**: 2024 (latest Rust edition)
- **Binary**: `airis-workspace` (from src/main.rs)
- **License**: MIT
- **Keywords**: monorepo, docker, workspace, cli, prototyping

### `workspace.yaml` (examples/)
Example configuration with:
- React 19, Next.js 15.4, Hono 4.0, Drizzle ORM
- Apps: dashboard (nextjs), api (hono), duplicate-finder (rust/local)
- Rules: no-host-install, catalog-only, no-env-local
- Docker: node:22-alpine base image

---

## 📚 Documentation

### `README.md`
- Problem statement (Docker-first enforcement)
- Quick start guide
- Installation instructions (cargo install)
- Command reference
- File structure explanation
- Core concepts (catalog, guards, Just vs Make)
- Roadmap (implemented ✅, in progress 🚧, planned 📋)

### `CLAUDE.md`
- Architecture overview for Claude Code
- Build commands (cargo build, test, run)
- Configuration flow diagram
- Design patterns (guards, runtime exceptions, catalog)
- Module responsibilities
- Testing strategy
- Important constraints (DO NOT edit generated files)

---

## 🧪 Test Coverage

**Unit Tests**: 3 test modules
1. `config/mod.rs::tests` - YAML parsing
2. `commands/init.rs::tests` - Default config creation
3. `commands/generate.rs::tests` - Justfile generation

**Test Dependencies**:
- `assert_cmd` 2.0 - CLI testing
- `predicates` 3.1 - Assertions
- `tempfile` 3.13 - Temporary file handling

**Coverage**: Basic unit tests for core functionality (parsing, generation)

---

## 🔗 Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| clap | 4.5 | CLI parsing with derive macros |
| serde | 1.0 | Serialization framework |
| serde_yaml | 0.9 | YAML parsing for workspace.yaml |
| handlebars | 6.2 | Template rendering engine |
| anyhow | 1.0 | Ergonomic error handling |
| colored | 2.1 | Terminal color output |
| tokio | 1.40 | Async runtime (full features) |
| indexmap | 2.7 | Ordered map for catalog |

---

## 📝 Quick Start

### Development Setup
```bash
# Clone repository
git clone https://github.com/agiletec-inc/airis-workspace.git
cd airis-workspace

# Build debug binary
cargo build

# Run tests
cargo test

# Install locally
cargo install --path .
```

### Testing the CLI
```bash
# Create test directory
mkdir test-workspace && cd test-workspace

# Initialize workspace
airis-workspace init

# Verify generated files
ls -la  # Should see: workspace.yaml, justfile, package.json, pnpm-workspace.yaml

# Test Docker workflow
just up
just workspace  # Enter container shell
```

### Making Changes
```bash
# Edit workspace.yaml (e.g., add new app)
vim workspace.yaml

# Regenerate files
airis-workspace generate all

# Or regenerate specific file
airis-workspace generate justfile
```

---

## 🎯 Design Philosophy

**Single Source of Truth**: All configuration lives in `workspace.yaml`. Generated files (`justfile`, `package.json`, `pnpm-workspace.yaml`) are read-only artifacts.

**Docker-First Enforcement**: Justfile guards block host-level `pnpm`/`npm`/`yarn` with helpful error messages. Forces developers to use `just workspace` → Docker shell workflow.

**Special Cases**: Rust projects can opt into `runtime: local` for GPU support (Apple Silicon Metal acceleration). All other apps default to Docker.

**LLM-Friendly**: Clear error messages, MCP server integration (planned), auto-generated context files for AI assistants.

---

## 📊 Code Statistics

- **Total Rust files**: 6
- **Lines of code**: ~900 (estimated, excluding tests)
- **Embedded templates**: 182 lines (justfile + package.json + pnpm)
- **Test coverage**: 3 unit test modules

---

## 🚧 Current Status

**Implemented**:
- ✅ CLI skeleton (clap)
- ✅ Configuration schema (WorkspaceConfig)
- ✅ `init` command (create workspace.yaml + auto-generate)
- ✅ `generate` command (justfile, package.json, pnpm-workspace.yaml)
- ✅ Template engine (Handlebars)
- ✅ Docker-first guards in justfile

**In Progress**:
- 🚧 `validate` command (stub exists in main.rs:35-37)

**Planned**:
- 📋 Environment variable validation
- 📋 LLM context generation
- 📋 MCP server integration
- 📋 Migration tool (Makefile → justfile)

---

## 🔍 Next Steps for Development

1. **Implement `validate` command**:
   - Parse workspace.yaml
   - Check for missing required fields
   - Validate app types exist in `types` config
   - Check port conflicts

2. **Add integration tests**:
   - Use `assert_cmd` to test full CLI flow
   - Create temporary workspace, run init, verify outputs

3. **Extract templates to files**:
   - Move `JUSTFILE_TEMPLATE`, etc. to `templates/` directory
   - Load at runtime instead of embedding as const strings

4. **Improve error messages**:
   - Add context to anyhow errors
   - Suggest fixes for common issues (e.g., missing Docker, invalid YAML syntax)

---

**Last Updated**: 2025-11-14
**Maintainer**: Agile Technology <hello@agiletec.jp>
**Repository**: https://github.com/agiletec-inc/airis-workspace
