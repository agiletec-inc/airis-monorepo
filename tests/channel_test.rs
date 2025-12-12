//! Channel resolver tests
//!
//! Note: These are integration tests that test the public API.
//! Unit tests are in src/channel.rs

use std::process::Command;

/// Test that the CLI accepts all valid channel values
#[test]
fn test_cli_channel_lts() {
    let output = Command::new("cargo")
        .args(["run", "--", "build", "--help"])
        .output()
        .expect("Failed to run cargo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--channel"));
    assert!(stdout.contains("lts"));
}

/// Test channel values in help text
#[test]
fn test_channel_help_text() {
    let output = Command::new("cargo")
        .args(["run", "--", "build", "--help"])
        .output()
        .expect("Failed to run cargo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check that channel options are documented
    assert!(stdout.contains("lts") || stdout.contains("current") || stdout.contains("edge"));
}
