use std::process::Command;

/// Test that the standards binary can be invoked and shows help
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "standards", "--", "--help"])
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
    let output = Command::new("cargo")
        .args(["run", "--bin", "standards", "--", "audit", "--help"])
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
    let output = Command::new("cargo")
        .args(["run", "--bin", "standards", "--", "audit"])
        .output()
        .expect("Failed to execute standards audit");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Standards audit completed"));
}

/// Test that audit command accepts path parameter
#[test]
fn test_cli_audit_with_path() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "standards", "--", "audit", "--path", "."])
        .output()
        .expect("Failed to execute standards audit with path");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Standards audit completed"));
}
