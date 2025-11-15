mod commands;
mod config;
mod templates;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "airis")]
#[command(version = "0.1.0")]
#[command(about = "Docker-first monorepo workspace manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate workspace.yaml from existing project (Makefile, package.json, etc.)
    Generate {
        /// Force regeneration even if workspace.yaml exists
        #[arg(short, long)]
        force: bool,
    },

    /// Initialize workspace files from workspace.yaml (justfile, docker-compose.yml, etc.)
    Init,

    /// Validate workspace configuration
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { force } => commands::generate::run_generate_config(force)?,
        Commands::Init => commands::init::run()?,
        Commands::Validate => {
            println!("⚠️  Validate command not yet implemented");
        }
    }

    Ok(())
}
