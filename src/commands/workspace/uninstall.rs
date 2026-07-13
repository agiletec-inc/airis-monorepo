use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

/// Uninstall airis from the current workspace by removing hooks and generated files.
pub fn uninstall() -> Result<()> {
    println!(
        "{}",
        "🗑️  Uninstalling airis from current workspace...".bright_blue()
    );
    println!();

    // 1. Remove markers from Git hooks
    remove_git_hooks_markers()?;

    // Generated AI/tool files are owned by AIris Code and are not removed by
    // this workspace utility. This avoids deleting user-edited knowledge.

    // 2. Cleanup .airis directory if empty (excluding backups)
    cleanup_airis_dir()?;

    println!();
    println!("{}", "✅ Workspace uninstalled successfully.".green());
    println!("   Note: User-owned project files and .airis/policies.toml remain untouched.");

    Ok(())
}

fn remove_git_hooks_markers() -> Result<()> {
    let hooks_dir = Path::new(".git/hooks");
    if !hooks_dir.exists() {
        return Ok(());
    }

    println!("🧹 Cleaning Git hooks...");
    let mut cleaned = 0;

    for entry in fs::read_dir(hooks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && clean_file_markers(&path, "# >>> airis start", "# <<< airis end")? {
            println!("   {} {}", "✓".green(), path.display());
            cleaned += 1;
        }
    }

    if cleaned == 0 {
        println!("   No airis markers found in hooks.");
    }
    Ok(())
}

fn cleanup_airis_dir() -> Result<()> {
    let airis_dir = Path::new(".airis");
    if !airis_dir.exists() {
        return Ok(());
    }

    // Only remove if it contains no important files (backups are kept)
    let has_backups = airis_dir.join("backups").exists();
    if !has_backups {
        let entries = fs::read_dir(airis_dir)?;
        if entries.count() == 0 {
            fs::remove_dir(airis_dir)?;
            println!("   {} Removed empty .airis directory", "✓".green());
        }
    }
    Ok(())
}

/// Clean a file by removing blocks between start and end markers
fn clean_file_markers(path: &Path, start_marker: &str, end_marker: &str) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    if !content.contains(start_marker) {
        return Ok(false);
    }

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut new_lines = Vec::new();
    let mut inside_block = false;
    let mut found = false;

    for line in reader.lines() {
        let line = line?;
        if line.contains(start_marker) {
            inside_block = true;
            found = true;
            continue;
        }
        if line.contains(end_marker) {
            inside_block = false;
            continue;
        }
        if !inside_block {
            new_lines.push(line);
        }
    }

    if found {
        fs::write(path, new_lines.join("\n") + "\n")?;
    }
    Ok(found)
}
