use anyhow::{Context, Result};
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::manifest::{CatalogEntry, Manifest};

pub fn run() -> Result<()> {
    println!("🔄 Syncing dependencies from manifest.toml...");

    // Load manifest
    let manifest = Manifest::load(Path::new("manifest.toml"))
        .context("Failed to load manifest.toml")?;

    // Get catalog from manifest
    let catalog = &manifest.packages.catalog;

    if catalog.is_empty() {
        println!("⚠️  No catalog entries found in manifest.toml");
        return Ok(());
    }

    println!("📦 Found {} catalog entries", catalog.len());

    // Resolve versions
    let mut resolved_catalog: IndexMap<String, String> = IndexMap::new();

    for (package, entry) in catalog {
        let policy_str = entry.as_str();
        let version = resolve_version(package, policy_str)?;

        // Only show resolution if it changed
        if entry.needs_resolution() {
            println!("  {} {} → {}", package, policy_str, version);
        } else {
            println!("  {} {}", package, version);
        }

        resolved_catalog.insert(package.clone(), version);
    }

    // Update pnpm-workspace.yaml
    update_pnpm_workspace(&resolved_catalog)?;

    println!("✅ Dependency sync complete!");
    println!("   Run 'pnpm install' to apply changes");

    Ok(())
}

fn resolve_version(package: &str, policy: &str) -> Result<String> {
    match policy {
        "latest" => get_npm_latest(package),
        "lts" => get_npm_lts(package),
        version if version.starts_with('^') || version.starts_with('~') => {
            // Already a specific version
            Ok(version.to_string())
        }
        _ => {
            // Treat as specific version
            Ok(policy.to_string())
        }
    }
}

fn get_npm_latest(package: &str) -> Result<String> {
    let output = Command::new("npm")
        .args(&["view", package, "version"])
        .output()
        .context(format!("Failed to query npm for {}", package))?;

    if !output.status.success() {
        anyhow::bail!("npm view failed for {}", package);
    }

    let version = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 from npm")?
        .trim()
        .to_string();

    Ok(format!("^{}", version))
}

fn get_npm_lts(package: &str) -> Result<String> {
    // For LTS, we use the "dist-tags.latest" approach
    // In the future, could check for actual LTS tags
    get_npm_latest(package)
}

fn update_pnpm_workspace(catalog: &IndexMap<String, String>) -> Result<()> {
    let workspace_path = Path::new("pnpm-workspace.yaml");

    if !workspace_path.exists() {
        anyhow::bail!("pnpm-workspace.yaml not found");
    }

    // Read existing content
    let content = fs::read_to_string(workspace_path)
        .context("Failed to read pnpm-workspace.yaml")?;

    // Parse YAML
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .context("Failed to parse pnpm-workspace.yaml")?;

    // Update catalog section
    if let Some(catalog_section) = yaml.get_mut("catalog") {
        if let Some(catalog_map) = catalog_section.as_mapping_mut() {
            for (package, version) in catalog {
                let key = serde_yaml::Value::String(package.clone());
                let value = serde_yaml::Value::String(version.clone());
                catalog_map.insert(key, value);
            }
        }
    } else {
        // Create catalog section if it doesn't exist
        let mut catalog_map = serde_yaml::Mapping::new();
        for (package, version) in catalog {
            let key = serde_yaml::Value::String(package.clone());
            let value = serde_yaml::Value::String(version.clone());
            catalog_map.insert(key, value);
        }
        if let Some(root_map) = yaml.as_mapping_mut() {
            root_map.insert(
                serde_yaml::Value::String("catalog".to_string()),
                serde_yaml::Value::Mapping(catalog_map),
            );
        }
    }

    // Write back to file
    let updated_content = serde_yaml::to_string(&yaml)
        .context("Failed to serialize YAML")?;

    fs::write(workspace_path, updated_content)
        .context("Failed to write pnpm-workspace.yaml")?;

    println!("📝 Updated pnpm-workspace.yaml");

    Ok(())
}
