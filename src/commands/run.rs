use anyhow::{bail, Context, Result};
use colored::Colorize;
use indexmap::IndexMap;
use std::path::Path;
use std::process::Command;

use crate::manifest::Manifest;

/// Execute a shell command and return success status
fn exec_command(cmd: &str) -> Result<bool> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
    }
    .with_context(|| format!("Failed to execute: {}", cmd))?;

    Ok(status.success())
}

/// Orchestrated startup: supabase -> workspace -> apps
fn orchestrated_up(manifest: &Manifest) -> Result<()> {
    let dev = &manifest.dev;

    // 1. Start Supabase (if configured)
    if let Some(supabase_files) = &dev.supabase {
        println!("{}", "📦 Starting Supabase...".cyan().bold());
        let files: Vec<String> = supabase_files.iter()
            .map(|f| format!("-f {}", f))
            .collect();
        let cmd = format!("docker compose {} up -d", files.join(" "));
        println!("   {}", cmd.dimmed());

        if !exec_command(&cmd)? {
            bail!("❌ Failed to start Supabase");
        }

        // Wait for Supabase DB to be healthy
        println!("   {} Waiting for Supabase DB to be healthy...", "⏳".dimmed());
        let health_check = "docker compose -f supabase/docker-compose.yml exec -T db pg_isready -U postgres -h localhost";
        let mut retries = 30;
        while retries > 0 {
            if exec_command(health_check).unwrap_or(false) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            retries -= 1;
        }
        if retries == 0 {
            println!("   {} Supabase DB health check timed out, continuing anyway...", "⚠️".yellow());
        } else {
            println!("   {} Supabase DB is healthy", "✅".green());
        }
    }

    // 2. Start Traefik (if configured)
    if let Some(traefik) = &dev.traefik {
        println!("{}", "🔀 Starting Traefik...".cyan().bold());
        let cmd = format!("docker compose -f {} up -d", traefik);
        println!("   {}", cmd.dimmed());

        if !exec_command(&cmd)? {
            println!("   {} Traefik failed to start, continuing anyway...", "⚠️".yellow());
        }
    }

    // 3. Start workspace container
    if let Some(workspace) = &dev.workspace {
        println!("{}", "🛠️  Starting workspace...".cyan().bold());
        let cmd = format!("docker compose -f {} up -d", workspace);
        println!("   {}", cmd.dimmed());

        if !exec_command(&cmd)? {
            bail!("❌ Failed to start workspace");
        }
    } else {
        // Fall back to default workspace location
        let default_workspace = Path::new("workspace/docker-compose.yml");
        if default_workspace.exists() {
            println!("{}", "🛠️  Starting workspace...".cyan().bold());
            let cmd = "docker compose -f workspace/docker-compose.yml up -d";
            println!("   {}", cmd.dimmed());

            if !exec_command(cmd)? {
                bail!("❌ Failed to start workspace");
            }
        }
    }

    // 4. Start apps from [dev].apps (auto-detect docker-compose.yml)
    if !dev.apps.is_empty() {
        println!("{}", "🚀 Starting apps...".cyan().bold());

        for app_name in &dev.apps {
            let compose_path = format!("apps/{}/docker-compose.yml", app_name);
            let compose_file = Path::new(&compose_path);

            if compose_file.exists() {
                println!("   {} Starting {}...", "→".dimmed(), app_name.bold());
                let cmd = format!("docker compose -f {} up -d", compose_path);

                if !exec_command(&cmd)? {
                    println!("   {} {} failed to start", "⚠️".yellow(), app_name);
                } else {
                    println!("   {} {} started", "✅".green(), app_name);
                }
            } else {
                println!("   {} {} (no docker-compose.yml)", "⏭️".dimmed(), app_name.dimmed());
            }
        }
    }

    println!("\n{}", "✅ All services started!".green().bold());
    Ok(())
}

/// Orchestrated shutdown: apps -> workspace -> supabase
fn orchestrated_down(manifest: &Manifest) -> Result<()> {
    let dev = &manifest.dev;

    // 1. Stop apps (reverse order)
    if !dev.apps.is_empty() {
        println!("{}", "🛑 Stopping apps...".cyan().bold());

        for app_name in dev.apps.iter().rev() {
            let compose_path = format!("apps/{}/docker-compose.yml", app_name);
            let compose_file = Path::new(&compose_path);

            if compose_file.exists() {
                let cmd = format!("docker compose -f {} down --remove-orphans", compose_path);
                let _ = exec_command(&cmd);
                println!("   {} {} stopped", "✅".green(), app_name);
            }
        }
    }

    // 2. Stop workspace
    if let Some(workspace) = &dev.workspace {
        println!("{}", "🛑 Stopping workspace...".cyan().bold());
        let cmd = format!("docker compose -f {} down --remove-orphans", workspace);
        let _ = exec_command(&cmd);
    } else {
        let default_workspace = Path::new("workspace/docker-compose.yml");
        if default_workspace.exists() {
            println!("{}", "🛑 Stopping workspace...".cyan().bold());
            let cmd = "docker compose -f workspace/docker-compose.yml down --remove-orphans";
            let _ = exec_command(cmd);
        }
    }

    // 3. Stop Traefik
    if let Some(traefik) = &dev.traefik {
        println!("{}", "🛑 Stopping Traefik...".cyan().bold());
        let cmd = format!("docker compose -f {} down --remove-orphans", traefik);
        let _ = exec_command(&cmd);
    }

    // 4. Stop Supabase
    if let Some(supabase_files) = &dev.supabase {
        println!("{}", "🛑 Stopping Supabase...".cyan().bold());
        let files: Vec<String> = supabase_files.iter()
            .map(|f| format!("-f {}", f))
            .collect();
        let cmd = format!("docker compose {} down --remove-orphans", files.join(" "));
        let _ = exec_command(&cmd);
    }

    println!("\n{}", "✅ All services stopped!".green().bold());
    Ok(())
}

/// Build docker compose command with orchestration files
fn build_compose_command(manifest: &Manifest, base_cmd: &str) -> String {
    // Check if orchestration.dev is configured
    if let Some(dev) = &manifest.orchestration.dev {
        let mut compose_files = Vec::new();

        // Add workspace compose file
        if let Some(workspace) = &dev.workspace {
            compose_files.push(format!("-f {}", workspace));
        }

        // Add supabase compose files
        if let Some(supabase) = &dev.supabase {
            for file in supabase {
                compose_files.push(format!("-f {}", file));
            }
        }

        // Add traefik compose file
        if let Some(traefik) = &dev.traefik {
            compose_files.push(format!("-f {}", traefik));
        }

        if !compose_files.is_empty() {
            return format!("docker compose {} {}", compose_files.join(" "), base_cmd);
        }
    }

    // Fall back to default (workspace/docker-compose.yml if exists)
    let workspace_compose = Path::new("workspace/docker-compose.yml");
    if workspace_compose.exists() {
        format!("docker compose -f workspace/docker-compose.yml {}", base_cmd)
    } else {
        format!("docker compose {}", base_cmd)
    }
}

/// Build clean command from manifest.toml [workspace.clean] section
fn build_clean_command(manifest: &Manifest) -> String {
    let clean = &manifest.workspace.clean;
    let mut parts = Vec::new();

    // Recursive patterns (e.g., node_modules)
    for pattern in &clean.recursive {
        parts.push(format!(
            "find . -name '{}' -type d -prune -exec rm -rf {{}} + 2>/dev/null",
            pattern
        ));
    }

    // Root directories
    if !clean.dirs.is_empty() {
        let dirs = clean.dirs.iter()
            .map(|d| format!("./{}", d))
            .collect::<Vec<_>>()
            .join(" ");
        parts.push(format!("rm -rf {}", dirs));
    }

    // Always clean .DS_Store
    parts.push("find . -name '.DS_Store' -delete 2>/dev/null || true".to_string());

    // Success message
    parts.push("echo '✅ Cleaned all build artifacts'".to_string());

    parts.join("; ")
}

/// Default commands when manifest.toml [commands] is empty
fn default_commands(manifest: &Manifest) -> IndexMap<String, String> {
    let mut cmds = IndexMap::new();
    cmds.insert("up".to_string(), build_compose_command(manifest, "up -d"));
    cmds.insert("down".to_string(), build_compose_command(manifest, "down --remove-orphans"));
    cmds.insert("shell".to_string(), build_compose_command(manifest, "exec -it workspace sh"));
    cmds.insert("install".to_string(), build_compose_command(manifest, "exec workspace pnpm install"));
    cmds.insert("dev".to_string(), build_compose_command(manifest, "exec workspace pnpm dev"));
    cmds.insert("build".to_string(), build_compose_command(manifest, "exec workspace pnpm build"));
    cmds.insert("test".to_string(), build_compose_command(manifest, "exec workspace pnpm test"));
    cmds.insert("lint".to_string(), build_compose_command(manifest, "exec workspace pnpm lint"));
    cmds.insert("clean".to_string(), build_clean_command(manifest));
    cmds.insert("logs".to_string(), build_compose_command(manifest, "logs -f"));
    cmds.insert("ps".to_string(), build_compose_command(manifest, "ps"));
    cmds
}

/// Check if orchestration is configured in manifest
fn has_orchestration(manifest: &Manifest) -> bool {
    let dev = &manifest.dev;
    dev.supabase.is_some() || !dev.apps.is_empty() || dev.traefik.is_some()
}

/// Execute a command defined in manifest.toml [commands] section
pub fn run(task: &str) -> Result<()> {
    let manifest_path = Path::new("manifest.toml");

    if !manifest_path.exists() {
        bail!(
            "❌ manifest.toml not found. Run {} first.",
            "airis init".bold()
        );
    }

    let manifest = Manifest::load(manifest_path)
        .with_context(|| "Failed to load manifest.toml")?;

    // Special handling for up/down with orchestration
    if has_orchestration(&manifest) {
        match task {
            "up" => return orchestrated_up(&manifest),
            "down" => return orchestrated_down(&manifest),
            _ => {}
        }
    }

    // Use manifest commands or fall back to defaults
    let commands = if manifest.commands.is_empty() {
        default_commands(&manifest)
    } else {
        manifest.commands.clone()
    };

    // Check if command exists
    let cmd = commands
        .get(task)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "❌ Command '{}' not found in manifest.toml [commands] section.\n\n\
                 Available commands:\n{}",
                task.bold(),
                commands
                    .keys()
                    .map(|k| format!("  - {}", k))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })?;

    println!("🚀 Running: {}", cmd.cyan());

    // Execute command
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
    }
    .with_context(|| format!("Failed to execute: {}", cmd))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Execute logs command with options
pub fn run_logs(service: Option<&str>, follow: bool, tail: Option<u32>) -> Result<()> {
    let manifest_path = Path::new("manifest.toml");

    if !manifest_path.exists() {
        bail!(
            "❌ manifest.toml not found. Run {} first.",
            "airis init".bold()
        );
    }

    let manifest = Manifest::load(manifest_path)
        .with_context(|| "Failed to load manifest.toml")?;

    let mut args = vec!["logs".to_string()];

    if follow {
        args.push("-f".to_string());
    }

    if let Some(n) = tail {
        args.push(format!("--tail={}", n));
    }

    if let Some(svc) = service {
        args.push(svc.to_string());
    }

    let cmd = build_compose_command(&manifest, &args.join(" "));

    println!("🚀 Running: {}", cmd.cyan());

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
    }
    .with_context(|| format!("Failed to execute: {}", cmd))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Execute command in a service container
pub fn run_exec(service: &str, cmd: &[String]) -> Result<()> {
    let manifest_path = Path::new("manifest.toml");

    if !manifest_path.exists() {
        bail!(
            "❌ manifest.toml not found. Run {} first.",
            "airis init".bold()
        );
    }

    let manifest = Manifest::load(manifest_path)
        .with_context(|| "Failed to load manifest.toml")?;

    if cmd.is_empty() {
        bail!("❌ No command specified. Usage: airis exec <service> <cmd>");
    }

    let exec_cmd = format!("exec {} {}", service, cmd.join(" "));
    let full_cmd = build_compose_command(&manifest, &exec_cmd);

    println!("🚀 Running: {}", full_cmd.cyan());

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &full_cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .status()
    }
    .with_context(|| format!("Failed to execute: {}", full_cmd))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Restart Docker services
pub fn run_restart(service: Option<&str>) -> Result<()> {
    let manifest_path = Path::new("manifest.toml");

    if !manifest_path.exists() {
        bail!(
            "❌ manifest.toml not found. Run {} first.",
            "airis init".bold()
        );
    }

    let manifest = Manifest::load(manifest_path)
        .with_context(|| "Failed to load manifest.toml")?;

    let restart_cmd = match service {
        Some(svc) => format!("restart {}", svc),
        None => "restart".to_string(),
    };

    let full_cmd = build_compose_command(&manifest, &restart_cmd);

    println!("🚀 Running: {}", full_cmd.cyan());

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &full_cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .status()
    }
    .with_context(|| format!("Failed to execute: {}", full_cmd))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_missing_manifest() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let result = run("test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("manifest.toml not found"));
    }

    #[test]
    fn test_run_missing_command() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        // Create minimal manifest
        let manifest_content = r#"
version = 1

[workspace]
name = "test"

[commands]
test = "echo 'test'"
"#;
        fs::write(dir.path().join("manifest.toml"), manifest_content).unwrap();

        let result = run("nonexistent");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("nonexistent") && err_msg.contains("not found"),
            "Expected error about 'nonexistent' not found, got: {}",
            err_msg
        );
    }
}
