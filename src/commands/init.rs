use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::manifest::MANIFEST_FILE;

/// Default manifest.toml template (embedded at compile time)
const MANIFEST_TEMPLATE: &str = include_str!("../../examples/manifest.toml");

/// Initialize a new airis workspace
///
/// If manifest.toml doesn't exist, creates it from template.
/// If manifest.toml exists, shows guidance for next steps.
pub fn run(_force_snapshot: bool, _no_snapshot: bool, write: bool) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILE);

    if manifest_path.exists() {
        // manifest.toml already exists - show guidance
        println!(
            "{} {} already exists",
            "✓".green(),
            MANIFEST_FILE.bright_cyan()
        );
        println!();
        println!("{}", "Next steps:".bright_yellow());
        println!("  1. Edit {} to configure your workspace", MANIFEST_FILE);
        println!("  2. Run {} to generate workspace files", "airis generate files".bright_cyan());
        println!();
        println!("{}", "Or use Claude Code for intelligent configuration:".bright_yellow());
        println!("  Ask Claude to analyze your repo and update manifest.toml");
        return Ok(());
    }

    // manifest.toml doesn't exist - create from template
    if write {
        fs::write(manifest_path, MANIFEST_TEMPLATE)?;
        println!(
            "{} Created {}",
            "✓".green(),
            MANIFEST_FILE.bright_cyan()
        );
        println!();
        println!("{}", "Next steps:".bright_yellow());
        println!("  1. Edit {} to configure your workspace:", MANIFEST_FILE);
        println!("     - Set [workspace].name to your project name");
        println!("     - Add your apps under [apps.*]");
        println!("     - Add your libs under [libs.*]");
        println!("     - Configure [packages.catalog] for shared dependencies");
        println!();
        println!("  2. Run {} to generate workspace files", "airis generate files".bright_cyan());
        println!();
        println!("{}", "Pro tip:".bright_yellow());
        println!("  Use Claude Code to intelligently configure manifest.toml");
        println!("  based on your existing project structure.");
    } else {
        // Dry-run mode - show what would be created
        println!(
            "{} Would create {}",
            "→".bright_blue(),
            MANIFEST_FILE.bright_cyan()
        );
        println!();
        println!("{}", "Preview (first 50 lines):".bright_yellow());
        println!("{}", "─".repeat(60));
        for line in MANIFEST_TEMPLATE.lines().take(50) {
            println!("{}", line);
        }
        println!("{}", "─".repeat(60));
        println!("... ({} more lines)", MANIFEST_TEMPLATE.lines().count() - 50);
        println!();
        println!(
            "Run {} to actually create the file",
            "airis init --write".bright_cyan()
        );
    }

    Ok(())
}

/// Setup .npmrc symlinks for Docker-First enforcement
/// This creates symlinks in apps/* and libs/* pointing to root .npmrc
pub fn setup_npmrc() -> Result<()> {
    println!("{}", "🔗 Setting up .npmrc symlinks...".bright_blue());
    println!();

    let root_npmrc = Path::new(".npmrc");
    if !root_npmrc.exists() {
        anyhow::bail!("Root .npmrc not found. Create it first.");
    }

    let mut created = 0;
    let mut skipped = 0;

    // Process apps directory
    let apps_dir = Path::new("apps");
    if apps_dir.exists() {
        for entry in fs::read_dir(apps_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Check if package.json exists (valid app)
            if !path.join("package.json").exists() {
                continue;
            }

            let npmrc_path = path.join(".npmrc");
            let relative_root = "../../.npmrc";

            if npmrc_path.exists() {
                // Check if it's already a symlink to root
                if npmrc_path.is_symlink() {
                    println!(
                        "  {} {} (already linked)",
                        "⏭️".yellow(),
                        npmrc_path.display()
                    );
                    skipped += 1;
                } else {
                    // Remove existing file and create symlink
                    fs::remove_file(&npmrc_path)?;
                    symlink(relative_root, &npmrc_path)?;
                    println!("  {} {} (replaced)", "✓".green(), npmrc_path.display());
                    created += 1;
                }
            } else {
                // Create new symlink
                symlink(relative_root, &npmrc_path)?;
                println!("  {} {}", "✓".green(), npmrc_path.display());
                created += 1;
            }
        }
    }

    // Process libs directory
    let libs_dir = Path::new("libs");
    if libs_dir.exists() {
        for entry in fs::read_dir(libs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Check if package.json exists (valid lib)
            if !path.join("package.json").exists() {
                continue;
            }

            let npmrc_path = path.join(".npmrc");
            let relative_root = "../../.npmrc";

            if npmrc_path.exists() {
                if npmrc_path.is_symlink() {
                    println!(
                        "  {} {} (already linked)",
                        "⏭️".yellow(),
                        npmrc_path.display()
                    );
                    skipped += 1;
                } else {
                    fs::remove_file(&npmrc_path)?;
                    symlink(relative_root, &npmrc_path)?;
                    println!("  {} {} (replaced)", "✓".green(), npmrc_path.display());
                    created += 1;
                }
            } else {
                symlink(relative_root, &npmrc_path)?;
                println!("  {} {}", "✓".green(), npmrc_path.display());
                created += 1;
            }
        }
    }

    println!();
    println!(
        "{} Created {} symlinks, skipped {} existing",
        "✅".green(),
        created,
        skipped
    );
    println!();
    println!("{}", "🛡️  Triple-layer defense active:".bright_yellow());
    println!("  1. .npmrc symlinks (primary)");
    println!("  2. preinstall hooks (backup)");
    println!("  3. Root preinstall + monorepo check (fallback)");

    Ok(())
}
