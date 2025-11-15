use anyhow::{Context, Result};
use colored::Colorize;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;

use crate::config::{DockerConfig, JustConfig, Mode, Rules, WorkspaceApp, WorkspaceConfig, Workspaces};

/// Generate workspace.yaml from existing project structure
pub fn run_generate_config(force: bool) -> Result<()> {
    let workspace_file = Path::new("workspace.yaml");

    if workspace_file.exists() && !force {
        anyhow::bail!(
            "❌ workspace.yaml already exists. Use --force to regenerate."
        );
    }

    println!("{}", "🔍 Detecting project structure...".bright_blue());

    // Detect existing files
    let makefile_exists = Path::new("Makefile").exists();
    let package_json_exists = Path::new("package.json").exists();
    let cargo_toml_exists = Path::new("Cargo.toml").exists();

    if makefile_exists {
        println!("{}", "✅ Found Makefile".green());
    }
    if package_json_exists {
        println!("{}", "✅ Found package.json".green());
    }
    if cargo_toml_exists {
        println!("{}", "✅ Found Cargo.toml".green());
    }

    // Create workspace.yaml
    let config = create_config_from_project()?;
    config
        .save(workspace_file)
        .context("Failed to save workspace.yaml")?;

    println!();
    println!("{}", "✅ Generated workspace.yaml".green());
    println!();
    println!("{}", "Next steps:".bright_yellow());
    println!("  1. Edit workspace.yaml to customize your workspace");
    println!("  2. Run: airis-workspace init");

    // Remove Makefile if exists
    if makefile_exists {
        fs::remove_file("Makefile").context("Failed to remove Makefile")?;
        println!();
        println!("{}", "🗑️  Removed Makefile".yellow());
        println!("{}", "   (Git history preserved - use 'git log -p Makefile' to view)".dimmed());
    }

    Ok(())
}

fn create_config_from_project() -> Result<WorkspaceConfig> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-monorepo")
        .to_string();

    let mut catalog = IndexMap::new();
    catalog.insert("react".to_string(), "19.0.0".to_string());
    catalog.insert("next".to_string(), "15.4.0".to_string());
    catalog.insert("typescript".to_string(), "5.8.0".to_string());
    catalog.insert("vitest".to_string(), "2.0.0".to_string());

    let workspaces = Workspaces {
        apps: vec![
            WorkspaceApp::Detailed {
                name: "dashboard".to_string(),
                app_type: "nextjs".to_string(),
            },
            WorkspaceApp::Detailed {
                name: "api".to_string(),
                app_type: "node".to_string(),
            },
        ],
        libs: vec![],
    };

    Ok(WorkspaceConfig {
        version: 1,
        name: project_name,
        mode: Mode::DockerFirst,
        catalog,
        package_manager: "pnpm@10.22.0".to_string(),
        workspaces,
        apps: IndexMap::new(),
        docker: DockerConfig::default(),
        rules: Rules::default(),
        just: JustConfig::default(),
        types: IndexMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config_from_project() {
        let config = create_config_from_project().unwrap();
        assert_eq!(config.version, 1);
        assert!(config.catalog.contains_key("react"));
    }
}
