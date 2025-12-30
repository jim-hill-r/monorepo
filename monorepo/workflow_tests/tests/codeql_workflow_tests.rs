use std::fs;
use workflow_tests::*;

#[test]
fn test_workflow_file_exists() {
    let workflow_path = get_codeql_workflow_path();
    assert!(
        workflow_path.exists(),
        "Workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_workflow_yaml_can_be_parsed() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_workflow_trigger_is_pull_request() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("pull_request:"),
        "Workflow trigger does not include pull_request"
    );
}

#[test]
fn test_workflow_has_schedule_trigger() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("schedule:"),
        "Workflow should have schedule trigger for periodic scanning"
    );
}

#[test]
fn test_workflow_uses_cast_cli_to_detect_changes() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("CAST_BIN")
            && content.contains("project")
            && content.contains("with-changes"),
        "Workflow does not use cast CLI to detect changes"
    );
}

#[test]
fn test_workflow_filters_for_typescript_javascript() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("package.json"),
        "Workflow should filter for projects with TypeScript/JavaScript (package.json)"
    );
}

#[test]
fn test_workflow_builds_cast_cli() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast/cli") && content.contains("cargo build"),
        "Workflow does not build cast CLI"
    );
}

#[test]
fn test_workflow_uses_codeql_actions() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("github/codeql-action/init")
            && content.contains("github/codeql-action/analyze"),
        "Workflow does not use CodeQL actions"
    );
}

#[test]
fn test_workflow_scans_javascript_typescript() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("javascript-typescript"),
        "Workflow should scan JavaScript/TypeScript languages"
    );
}

#[test]
fn test_workflow_sets_up_rust_toolchain() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("setup-rust-toolchain")
            || content.contains("rust-toolchain")
            || content.contains("actions-rust-lang"),
        "Workflow does not set up Rust toolchain"
    );
}

#[test]
fn test_workflow_has_security_events_permission() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("security-events: write"),
        "Workflow needs security-events: write permission for CodeQL"
    );
}

#[test]
fn test_workflow_handles_merged_prs() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("github.event.pull_request.merged"),
        "Workflow should handle merged PRs"
    );
}

#[test]
fn test_workflow_validates_commit_shas() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("Invalid SHA format"),
        "Workflow should validate commit SHA format"
    );
}

#[test]
fn test_workflow_fetches_commits() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("git fetch"),
        "Workflow should explicitly fetch commits"
    );
}

#[test]
fn test_workflow_quotes_github_expressions() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Check for proper quoting of GitHub expressions in bash
    // Should have patterns like BASE_SHA="${{ ... }}"
    assert!(
        content.contains("BASE_SHA=\"${{") || content.contains("BASE_SHA=\"${"),
        "GitHub expressions should be quoted in bash variable assignments"
    );
}
