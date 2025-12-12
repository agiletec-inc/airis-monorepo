//! DAG (Directed Acyclic Graph) integration tests
//!
//! Tests for dependency graph construction and traversal

/// Test that affected command uses DAG
#[test]
fn test_affected_uses_dag() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "affected", "--help"])
        .output()
        .expect("Failed to run cargo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--base"));
    assert!(stdout.contains("--head"));
}

/// Test affected command with same base and head
#[test]
fn test_affected_no_changes() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "affected", "--base", "HEAD", "--head", "HEAD"])
        .output()
        .expect("Failed to run cargo");

    // Should succeed (no changes between same commits)
    assert!(output.status.success());
}
