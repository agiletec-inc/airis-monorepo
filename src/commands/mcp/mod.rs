//! Small MCP facade for native workspace discovery, policy, validation, and cleanup.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<Value>,
}

pub fn run() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let request: McpRequest = match serde_json::from_str(&line?) {
            Ok(request) => request,
            Err(_) => continue,
        };
        let response = handle_request(request)?;
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle_request(request: McpRequest) -> Result<McpResponse> {
    let result = match request.method.as_str() {
        "initialize" => Some(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": { "tools": { "listChanged": false }, "resources": { "listChanged": false } },
            "serverInfo": { "name": "airis-workspace-mcp", "version": env!("CARGO_PKG_VERSION") }
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(json!({ "tools": tools() })),
        "tools/call" => {
            let params = request.params.as_ref().context("Missing params")?;
            Some(call_tool(
                params["name"].as_str().context("Missing tool name")?,
                &params["arguments"],
            )?)
        }
        "resources/list" => Some(resources_list()?),
        "resources/read" => {
            let params = request.params.as_ref().context("Missing params")?;
            Some(resources_read(
                params["uri"].as_str().context("Missing uri")?,
            )?)
        }
        _ => None,
    };
    Ok(McpResponse {
        jsonrpc: "2.0".into(),
        result,
        error: None,
        id: request.id,
    })
}

fn tools() -> Value {
    let names = [
        (
            "workspace_discover",
            "Inspect native project metadata without writing files.",
        ),
        (
            "workspace_cleanup",
            "List legacy workspace artifacts for review.",
        ),
        (
            "workspace_validate_all",
            "Run native ports, network, environment, and dependency checks.",
        ),
        (
            "workspace_policy_check",
            "Check .airis/policies.toml against the workspace.",
        ),
        (
            "workspace_clean",
            "Preview or remove build artifacts; force is required to delete.",
        ),
    ];
    Value::Array(names.into_iter().map(|(name, description)| json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": { "force": { "type": "boolean" } } }
    })).collect())
}

fn call_tool(name: &str, args: &Value) -> Result<Value> {
    let value = match name {
        "workspace_discover" => serde_json::to_string_pretty(&crate::commands::discover::run()?)?,
        "workspace_cleanup" => cleanup_report()?,
        "workspace_validate_all" => run_cli(&["validate", "all"])?,
        "workspace_policy_check" => run_cli(&["policy", "check"])?,
        "workspace_clean" => {
            if args["force"].as_bool().unwrap_or(false) {
                run_cli(&["clean", "--force"])?
            } else {
                run_cli(&["clean"])?
            }
        }
        _ => {
            return Ok(
                json!({ "isError": true, "content": [{ "type": "text", "text": format!("Unknown tool: {name}") }] }),
            );
        }
    };
    Ok(json!({ "content": [{ "type": "text", "text": value }] }))
}

fn run_cli(args: &[&str]) -> Result<String> {
    let bin = std::env::current_exe().context("Failed to resolve airis binary")?;
    let output = std::process::Command::new(bin).args(args).output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(text.trim().to_string())
}

fn cleanup_report() -> Result<String> {
    let mut found = Vec::new();
    for pattern in [
        "docker-compose.yml",
        "docker-compose.yaml",
        "docker-compose.override.yml",
        "compose.override.yml",
    ] {
        for path in glob::glob(pattern)?.flatten() {
            found.push(path.display().to_string());
        }
    }
    found.sort();
    found.dedup();
    Ok(if found.is_empty() {
        "Workspace is already clean.".into()
    } else {
        format!("Legacy artifacts:\n{}", found.join("\n"))
    })
}

const RESOURCES: &[(&str, &str, &str)] = &[
    (
        ".airis/policies.toml",
        "Workspace policy",
        "application/toml",
    ),
    ("AGENTS.md", "Agent instructions", "text/markdown"),
    ("CLAUDE.md", "Claude instructions", "text/markdown"),
    ("Cargo.toml", "Rust project manifest", "application/toml"),
    ("package.json", "Node project manifest", "application/json"),
];

fn resources_list() -> Result<Value> {
    Ok(
        json!({ "resources": RESOURCES.iter().filter(|(path, _, _)| Path::new(path).exists()).map(|(path, name, mime)| json!({ "uri": format!("file:///{path}"), "name": path, "description": name, "mimeType": mime })).collect::<Vec<_>>() }),
    )
}

fn resources_read(uri: &str) -> Result<Value> {
    let path = uri.strip_prefix("file:///").context("Invalid file URI")?;
    anyhow::ensure!(
        !path.starts_with('/') && !path.split('/').any(|part| part == ".."),
        "Invalid resource path"
    );
    let (_, _, mime) = RESOURCES
        .iter()
        .find(|(candidate, _, _)| *candidate == path)
        .context("Resource not advertised")?;
    Ok(
        json!({ "contents": [{ "uri": uri, "mimeType": mime, "text": std::fs::read_to_string(path)? }] }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resources_reject_traversal() {
        assert!(resources_read("file:///../etc/passwd").is_err());
    }
}
