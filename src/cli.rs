use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "airis-workspace")]
#[command(about = "Convention engine for polyglot monorepos")]
#[command(long_about = "\
A workspace utility for convention-based monorepos.

Discovers native project metadata, enforces `.airis/policies.toml`, and keeps
workspace cleanup and validation safe. AI agent definitions are distributed by
AIris Code.

Invoked through the airis dispatcher as `airis workspace <cmd>`.")]
#[command(after_help = "\
QUICK REFERENCE:
  airis workspace discover      Inspect native project metadata
  airis workspace clean         Remove build artifacts (dry-run by default)
  airis workspace validate all  Validate workspace configuration

CONVENTIONS:
  airis-workspace automatically discovers projects in apps/* and libs/*.
  Native files such as package.json, Cargo.toml, pyproject.toml, and go.mod are
  the source of truth. Policy is stored in .airis/policies.toml.")]
pub struct Cli {
    /// Print version
    #[arg(short = 'V', long = "version")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Project-level cleanup and management
    Workspace(WorkspaceArgs),

    /// Discover projects from native repository metadata
    Discover,

    /// Validate workspace configuration
    Validate {
        #[command(subcommand)]
        action: ValidateCommands,
        /// Output results as JSON
        #[arg(long, global = true)]
        json: bool,
    },

    /// Clean build artifacts
    Clean {
        /// Preview only (default)
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Remove orphaned or legacy config files (e.g., docker-compose.yml).
        #[arg(long)]
        purge: bool,
        /// Actually execute deletions
        #[arg(long)]
        force: bool,
        /// Skip the project-root safety check (run even without
        /// package.json / Cargo.toml / pyproject.toml / go.mod
        /// in the current directory)
        #[arg(long)]
        allow_anywhere: bool,
        /// Extra arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },

    /// Create new app, service, or library
    New {
        #[command(subcommand)]
        template: NewCommands,
    },

    /// Bump version
    #[command(name = "bump-version")]
    BumpVersion {
        #[arg(long)]
        major: bool,
        #[arg(long)]
        minor: bool,
        #[arg(long)]
        patch: bool,
        #[arg(long)]
        auto: bool,
    },

    /// Generate database types
    Generate {
        #[command(subcommand)]
        action: GenerateCommands,
    },

    /// Policy gates
    Policy {
        #[command(subcommand)]
        action: PolicyCommands,
    },

    /// Dependency graph visualization
    Deps {
        #[command(subcommand)]
        action: DepsCommands,
    },

    /// Upgrade airis-workspace
    Upgrade {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        version: Option<String>,
    },

    /// Generate shell completion scripts
    Completion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Start the MCP server
    Mcp,
}

#[derive(Args)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub action: WorkspaceCommands,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// Uninstall airis from the current workspace (removes hooks and generated files)
    Uninstall,
}

#[derive(Subcommand)]
pub enum PolicyCommands {
    Init,
    Check { project: Option<String> },
    Enforce { project: Option<String> },
}

#[derive(Subcommand)]
pub enum DepsCommands {
    Tree,
    Json,
    Show { package: String },
    Check,
}

#[derive(Subcommand)]
pub enum ValidateCommands {
    Ports,
    Networks,
    Env,
    #[command(name = "deps")]
    Dependencies,
    #[command(name = "arch")]
    Architecture,
    All,
}

#[derive(Subcommand)]
pub enum GenerateCommands {
    Types {
        #[arg(long, default_value = "localhost")]
        host: String,
        #[arg(long, default_value = "54322")]
        port: String,
        #[arg(long, default_value = "postgres")]
        database: String,
        #[arg(short, long, default_value = "libs/types")]
        output: String,
    },
}

#[derive(Subcommand)]
pub enum NewCommands {
    Api {
        name: String,
        #[arg(short, long, default_value = "hono")]
        runtime: String,
    },
    Web {
        name: String,
        #[arg(short, long, default_value = "nextjs")]
        runtime: String,
    },
    Lib {
        name: String,
        #[arg(short, long, default_value = "ts")]
        runtime: String,
    },
    Edge {
        name: String,
    },
    #[command(name = "supabase-trigger")]
    SupabaseTrigger {
        name: String,
    },
    #[command(name = "supabase-realtime")]
    SupabaseRealtime {
        name: String,
    },
}
