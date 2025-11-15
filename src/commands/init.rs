use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::config::WorkspaceConfig;
use crate::templates::TemplateEngine;

/// Initialize workspace files from workspace.yaml
pub fn run() -> Result<()> {
    let workspace_file = Path::new("workspace.yaml");

    if !workspace_file.exists() {
        anyhow::bail!(
            "❌ workspace.yaml not found. Run 'airis-workspace generate' first."
        );
    }

    println!("{}", "🔨 Loading workspace.yaml...".bright_blue());
    let config = WorkspaceConfig::load(workspace_file)?;

    let engine = TemplateEngine::new()?;

    println!();
    generate_docker_compose(&config, &engine)?;
    generate_justfile(&config, &engine)?;
    generate_package_json(&config, &engine)?;
    generate_pnpm_workspace(&config, &engine)?;

    println!();
    println!("{}", "✅ Generated all files:".green());
    println!("   - docker-compose.yml");
    println!("   - justfile");
    println!("   - package.json");
    println!("   - pnpm-workspace.yaml");

    println!();
    println!("{}", "Next steps:".bright_yellow());
    println!("  1. Review generated files");
    println!("  2. Run: just up");

    println!();
    println!("{}", "🎉 Setup complete!".bright_green());
    println!("   Run: just up");

    Ok(())
}

fn generate_justfile(config: &WorkspaceConfig, engine: &TemplateEngine) -> Result<()> {
    let output = config
        .just
        .output
        .clone()
        .unwrap_or_else(|| "justfile".to_string());

    let content = engine.render_justfile(config)?;

    fs::write(&output, content)
        .with_context(|| format!("Failed to write {}", output))?;

    Ok(())
}

fn generate_package_json(config: &WorkspaceConfig, engine: &TemplateEngine) -> Result<()> {
    let content = engine.render_package_json(config)?;

    fs::write("package.json", content)
        .context("Failed to write package.json")?;

    Ok(())
}

fn generate_pnpm_workspace(config: &WorkspaceConfig, engine: &TemplateEngine) -> Result<()> {
    let content = engine.render_pnpm_workspace(config)?;

    fs::write("pnpm-workspace.yaml", content)
        .context("Failed to write pnpm-workspace.yaml")?;

    Ok(())
}

fn generate_docker_compose(config: &WorkspaceConfig, engine: &TemplateEngine) -> Result<()> {
    let content = engine.render_docker_compose(config)?;

    fs::write("docker-compose.yml", content)
        .context("Failed to write docker-compose.yml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Mode;
    use indexmap::IndexMap;

    #[test]
    fn test_generate_justfile() {
        let mut catalog = IndexMap::new();
        catalog.insert("react".to_string(), "19.0.0".to_string());

        let config = WorkspaceConfig {
            version: 1,
            name: "test".to_string(),
            mode: Mode::DockerFirst,
            catalog,
            ..Default::default()
        };

        let engine = TemplateEngine::new().unwrap();
        let result = engine.render_justfile(&config);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("project := \"test\""));
    }
}
