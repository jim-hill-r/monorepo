use std::fs;
use workflow_tests::*;

#[test]
fn test_workflow_file_exists() {
    let workflow_path = get_standards_audit_workflow_path();
    assert!(
        workflow_path.exists(),
        "Standards audit workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_workflow_yaml_can_be_parsed() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_workflow_trigger_includes_schedule() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("schedule:") && content.contains("cron:"),
        "Workflow should have a schedule trigger with a cron expression"
    );
}

#[test]
fn test_workflow_trigger_includes_workflow_dispatch() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("workflow_dispatch"),
        "Workflow should include workflow_dispatch trigger for manual execution"
    );
}

#[test]
fn test_workflow_has_contents_write_permission() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("contents: write"),
        "Workflow must have 'contents: write' permission to push branches"
    );
}

#[test]
fn test_workflow_has_pull_requests_write_permission() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("pull-requests: write"),
        "Workflow must have 'pull-requests: write' permission to create PRs"
    );
}

#[test]
fn test_workflow_builds_standards_cli() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("standards/Cargo.toml") || content.contains("standards"),
        "Workflow should build the standards CLI"
    );
}

#[test]
fn test_workflow_runs_audit_to_issues() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("audit-to-issues"),
        "Workflow should run the 'audit-to-issues' subcommand"
    );
}

#[test]
fn test_workflow_creates_pr() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("gh pr create"),
        "Workflow should create a pull request with audit results"
    );
}

#[test]
fn test_workflow_assigns_pr_to_jim_hill_r() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("--assignee jim-hill-r"),
        "Workflow should assign the PR to jim-hill-r"
    );
}

#[test]
fn test_workflow_sets_up_rust_toolchain() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("actions-rust-lang/setup-rust-toolchain")
            || content.contains("setup-rust-toolchain"),
        "Workflow should set up the Rust toolchain"
    );
}

#[test]
fn test_workflow_installs_rustfmt_and_clippy() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("components: rustfmt, clippy")
            || content.contains("components: clippy, rustfmt"),
        "Workflow should explicitly install rustfmt and clippy components"
    );
}

#[test]
fn test_workflow_skips_pr_creation_when_no_changes() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("git diff"),
        "Workflow should check for changes before creating a PR"
    );
}

#[test]
fn test_workflow_uses_github_token() {
    let content = fs::read_to_string(get_standards_audit_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("GH_TOKEN"),
        "Workflow should use GH_TOKEN for the GitHub CLI (gh) PR creation"
    );
}
