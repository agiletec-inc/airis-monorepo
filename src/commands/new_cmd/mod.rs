//! New command: scaffold new apps, services, and libraries from templates

mod api;
mod edge;
mod lib;
mod python;
mod rust;
mod supabase;
mod web;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::fs;
use std::path::Path;

use api::generate_api_project;
use edge::generate_edge_function;
use lib::generate_lib_project;
use python::{generate_py_api, generate_py_lib};
use rust::generate_rust_service;
use supabase::{generate_supabase_realtime, generate_supabase_trigger};
use web::generate_web_project;

/// Get the base directory for a template category
fn get_base_dir(category: &str) -> &str {
    match category {
        "api" | "web" | "worker" | "cli" => "apps",
        "lib" => "libs",
        "edge" | "supabase-trigger" | "supabase-realtime" => "supabase/functions",
        _ => "apps",
    }
}

/// Resolve runtime alias to full runtime name
/// Run the new command with runtime selection
pub fn run_with_runtime(category: &str, name: &str, runtime: &str) -> Result<()> {
    // Validate name
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Project name can only contain alphanumeric characters, hyphens, and underscores");
    }

    let resolved_runtime = runtime.to_string();

    let base_dir = get_base_dir(category);
    let project_dir = Path::new(base_dir).join(name);

    // Check if directory already exists
    if project_dir.exists() {
        bail!(
            "Directory {} already exists. Choose a different name.",
            project_dir.display()
        );
    }

    // Ensure base directory exists
    if !Path::new(base_dir).exists() {
        fs::create_dir_all(base_dir)
            .with_context(|| format!("Failed to create {} directory", base_dir))?;
    }

    let display_name = format!("{} ({})", category, resolved_runtime);
    println!(
        "{} {} at {}",
        "Creating".bright_blue(),
        display_name,
        project_dir.display().to_string().cyan()
    );

    // Generate project based on category and runtime
    match (category, resolved_runtime.as_str()) {
        ("api", "hono") => generate_api_project(&project_dir, name)?,
        ("api", "fastapi") => generate_py_api(&project_dir, name)?,
        ("api", "rust-axum") => generate_rust_service(&project_dir, name)?,
        ("web", "nextjs") => generate_web_project(&project_dir, name)?,
        ("lib", "ts") => generate_lib_project(&project_dir, name)?,
        ("lib", "python") => generate_py_lib(&project_dir, name)?,
        ("edge", "deno") => generate_edge_function(&project_dir, name)?,
        ("supabase-trigger", "plpgsql") => generate_supabase_trigger(&project_dir, name)?,
        ("supabase-realtime", "deno") => generate_supabase_realtime(&project_dir, name)?,
        _ => {
            bail!(
                "Unknown runtime '{}' for category '{}'. Available runtimes:\n  \
                api: hono, fastapi, rust-axum\n  \
                web: nextjs\n  \
                lib: ts, python\n  \
                edge: deno\n  \
                supabase-trigger: plpgsql\n  \
                supabase-realtime: deno",
                resolved_runtime,
                category
            );
        }
    }

    println!();
    println!("{}", "✅ Project created successfully!".green());
    println!();
    println!("{}", "Next steps:".bright_yellow());
    println!(
        "  1. Run your repository's native install/build command (for example {})",
        "pnpm install".cyan()
    );
    println!(
        "  2. Install dependencies and start services (e.g. {} / {})",
        "pnpm install".cyan(),
        "docker compose up -d".cyan()
    );

    Ok(())
}
