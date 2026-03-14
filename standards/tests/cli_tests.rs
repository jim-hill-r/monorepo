#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

/// Get the path to the compiled standards binary
fn standards_bin() -> &'static str {
    env!("CARGO_BIN_EXE_standards")
}

/// Test that the standards binary can be invoked and shows help
#[test]
fn test_cli_help() {
    let output = Command::new(standards_bin())
        .arg("--help")
        .output()
        .expect("Failed to execute standards binary");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("audit and enforce coding standards"));
    assert!(stdout.contains("audit"));
}

/// Test that the audit subcommand can be invoked
#[test]
fn test_cli_audit_subcommand() {
    let output = Command::new(standards_bin())
        .args(["audit", "--help"])
        .output()
        .expect("Failed to execute standards audit");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Audit projects for standards compliance"));
    assert!(stdout.contains("--path"));
}

/// Test that audit command runs without errors (even if no audits implemented yet)
#[test]
fn test_cli_audit_runs() {
    let output = Command::new(standards_bin())
        .arg("audit")
        .output()
        .expect("Failed to execute standards audit");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Standards audit for path"));
    assert!(stdout.contains("Discovered"));
}

/// Test that audit command accepts path parameter
#[test]
fn test_cli_audit_with_path() {
    let output = Command::new(standards_bin())
        .args(["audit", "--path", "."])
        .output()
        .expect("Failed to execute standards audit with path");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Standards audit for path"));
    assert!(stdout.contains("Discovered"));
}
