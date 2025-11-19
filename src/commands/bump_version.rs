use anyhow::{bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::manifest::{Manifest, VersioningStrategy, MANIFEST_FILE};

#[derive(Debug, Clone)]
pub enum BumpMode {
    Auto,     // Detect from commit message
    Major,    // x.0.0
    Minor,    // x.y.0
    Patch,    // x.y.z
}

/// Bump version in manifest.toml [meta].version and sync to Cargo.toml
pub fn run(mode: BumpMode) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILE);

    if !manifest_path.exists() {
        bail!(
            "❌ {} not found. Run {} first.",
            MANIFEST_FILE.bold(),
            "airis init".bold()
        );
    }

    let mut manifest = Manifest::load(manifest_path)
        .with_context(|| format!("Failed to load {}", MANIFEST_FILE))?;

    // Use [meta].version as SoT, fallback to versioning.source for backward compatibility
    let current_version = if !manifest.meta.version.is_empty() {
        manifest.meta.version.clone()
    } else {
        manifest.versioning.source.clone()
    };

    if current_version.is_empty() {
        bail!("❌ No version found in manifest.toml. Add [meta].version or [versioning].source.");
    }

    // Determine bump type
    let new_version = match mode {
        BumpMode::Auto => {
            // Detect from last commit message or versioning strategy
            match manifest.versioning.strategy {
                VersioningStrategy::Manual => {
                    bail!("❌ Versioning strategy is 'manual'. Use --major, --minor, or --patch.");
                }
                VersioningStrategy::Auto => {
                    // Default to minor bump
                    bump_version_string(&current_version, "minor")?
                }
                VersioningStrategy::ConventionalCommits => {
                    // Get last commit message
                    let commit_msg = get_last_commit_message()?;
                    detect_bump_type_from_conventional_commit(&commit_msg, &current_version)?
                }
            }
        }
        BumpMode::Major => bump_version_string(&current_version, "major")?,
        BumpMode::Minor => bump_version_string(&current_version, "minor")?,
        BumpMode::Patch => bump_version_string(&current_version, "patch")?,
    };

    println!(
        "🚀 Bumping version: {} → {}",
        current_version.yellow(),
        new_version.green().bold()
    );

    // Update manifest.toml [meta].version (SoT)
    manifest.meta.version = new_version.clone();
    // Also update versioning.source for backward compatibility
    manifest.versioning.source = new_version.clone();
    manifest.save(manifest_path)?;

    // Sync to Cargo.toml
    update_cargo_toml(&new_version)?;

    println!("✅ Version bumped successfully!");
    println!("   manifest.toml [meta].version: {}", new_version.green());
    println!("   Cargo.toml: {}", new_version.green());

    Ok(())
}

/// Bump version string by type
fn bump_version_string(current: &str, bump_type: &str) -> Result<String> {
    let parts: Vec<u32> = current
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    if parts.len() < 3 {
        bail!("Invalid version format: {}", current);
    }

    let (major, minor, patch) = (parts[0], parts[1], parts[2]);

    let new_version = match bump_type {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => bail!("Unknown bump type: {}", bump_type),
    };

    Ok(new_version)
}

/// Get the last commit message
fn get_last_commit_message() -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output()
        .with_context(|| "Failed to get git commit message")?;

    if !output.status.success() {
        bail!("Failed to get git commit message");
    }

    let msg = String::from_utf8(output.stdout)
        .with_context(|| "Invalid UTF-8 in commit message")?
        .trim()
        .to_string();

    Ok(msg)
}

/// Detect version bump type from Conventional Commits message
fn detect_bump_type_from_conventional_commit(
    commit_msg: &str,
    current_version: &str,
) -> Result<String> {
    // BREAKING CHANGE or feat!: → major
    if commit_msg.contains("BREAKING CHANGE") || commit_msg.contains("!:") {
        return bump_version_string(current_version, "major");
    }

    // feat: → minor
    if commit_msg.starts_with("feat:") || commit_msg.starts_with("feat(") {
        return bump_version_string(current_version, "minor");
    }

    // fix: → patch
    if commit_msg.starts_with("fix:") || commit_msg.starts_with("fix(") {
        return bump_version_string(current_version, "patch");
    }

    // chore:, docs:, style:, refactor:, test: → patch
    if commit_msg.starts_with("chore:")
        || commit_msg.starts_with("docs:")
        || commit_msg.starts_with("style:")
        || commit_msg.starts_with("refactor:")
        || commit_msg.starts_with("test:")
    {
        return bump_version_string(current_version, "patch");
    }

    // Default: patch
    bump_version_string(current_version, "patch")
}

/// Update version in Cargo.toml
fn update_cargo_toml(new_version: &str) -> Result<()> {
    let cargo_path = Path::new("Cargo.toml");

    if !cargo_path.exists() {
        // Cargo.toml not found (maybe not a Rust project)
        return Ok(());
    }

    let content = fs::read_to_string(cargo_path)
        .with_context(|| "Failed to read Cargo.toml")?;

    // Replace version line
    let updated = Regex::new(r#"version = "[\d.]+""#)?
        .replace(&content, format!(r#"version = "{}""#, new_version));

    fs::write(cargo_path, updated.as_ref())
        .with_context(|| "Failed to write Cargo.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_version_string() {
        // patch bump
        let result = bump_version_string("1.0.0", "patch");
        assert_eq!(result.unwrap(), "1.0.1");

        // minor bump
        let result = bump_version_string("1.0.0", "minor");
        assert_eq!(result.unwrap(), "1.1.0");

        // major bump
        let result = bump_version_string("1.0.0", "major");
        assert_eq!(result.unwrap(), "2.0.0");
    }

    #[test]
    fn test_conventional_commits_detection() {
        // feat: → minor
        let result = detect_bump_type_from_conventional_commit("feat: add new feature", "1.0.0");
        assert_eq!(result.unwrap(), "1.1.0");

        // fix: → patch
        let result = detect_bump_type_from_conventional_commit("fix: bug fix", "1.1.0");
        assert_eq!(result.unwrap(), "1.1.1");

        // BREAKING CHANGE → major
        let result = detect_bump_type_from_conventional_commit(
            "feat!: BREAKING CHANGE: api change",
            "1.1.1",
        );
        assert_eq!(result.unwrap(), "2.0.0");
    }
}
