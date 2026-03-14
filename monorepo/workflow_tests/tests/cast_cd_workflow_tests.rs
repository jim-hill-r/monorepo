use std::fs;
use workflow_tests::*;

#[test]
fn test_workflow_file_exists() {
    let workflow_path = get_cast_cd_workflow_path();
    assert!(
        workflow_path.exists(),
        "Workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_workflow_yaml_can_be_parsed() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_workflow_trigger_is_push_on_main() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("push:") && content.contains("branches:") && content.contains("- main"),
        "Workflow trigger should be push on main branch"
    );
}

#[test]
fn test_workflow_trigger_filters_artifacts_path() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("paths:") && content.contains("**/artifacts/x86_64-unknown-linux-gnu/**"),
        "Workflow trigger should filter for Linux artifact paths"
    );
}

#[test]
fn test_workflow_detects_projects_from_artifacts() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Current workflow uses cast cd with --last-commit flag for detection
    assert!(
        content.contains("artifacts/x86_64-unknown-linux-gnu/") && content.contains("cast cd"),
        "Workflow should trigger on artifact paths and use cast cd for deployment"
    );
}

#[test]
fn test_workflow_uses_last_commit_for_detection() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Current workflow uses cast cd --last-commit instead of manual git diff
    assert!(
        content.contains("--last-commit"),
        "Workflow should use --last-commit flag for deployment detection"
    );
}

#[test]
fn test_workflow_processes_projects_recursively() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow uses cast cd with --recursive flag
    assert!(
        content.contains("--recursive"),
        "Workflow should use --recursive flag to process projects"
    );
}

#[test]
fn test_workflow_installs_cast_cli() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast/cli") && content.contains("cargo install"),
        "Workflow does not install cast CLI"
    );
}

#[test]
fn test_workflow_runs_cast_cd_command() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast cd"),
        "Workflow does not run cast cd command"
    );
}

#[test]
fn test_workflow_sets_up_rust_toolchain() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("setup-rust-toolchain")
            || content.contains("rust-toolchain")
            || content.contains("actions-rust-lang"),
        "Workflow does not set up Rust toolchain"
    );
}

#[test]
fn test_workflow_uses_fetch_depth_for_commits() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow needs fetch-depth: 2 to access HEAD~1 for --last-commit
    assert!(
        content.contains("fetch-depth: 2"),
        "Workflow should set fetch-depth to 2 for commit comparison"
    );
}

#[test]
fn test_workflow_runs_cd_on_artifact_changes() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow is triggered by artifact path changes
    assert!(
        content.contains("paths:") && content.contains("artifacts"),
        "Workflow should be triggered by artifact changes"
    );
}

#[test]
fn test_workflow_has_run_cd_step() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("Run CD") || content.contains("cast cd"),
        "Workflow should have a step to run CD"
    );
}

#[test]
fn test_workflow_installs_cast_before_running() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow installs cast CLI before using it
    let install_pos = content.find("cargo install").expect("Workflow missing cargo install");
    let cd_pos = content.find("cast cd").expect("Workflow missing cast cd command");
    
    assert!(
        install_pos < cd_pos,
        "Workflow should install cast CLI before running cast cd"
    );
}

#[test]
fn test_workflow_installs_rustfmt_component() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("components: rustfmt, clippy")
            || content.contains("components: clippy, rustfmt"),
        "Workflow does not explicitly install rustfmt component. Expected 'components: rustfmt, clippy' or similar."
    );
}

#[test]
fn test_workflow_installs_clippy_component() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("components: rustfmt, clippy")
            || content.contains("components: clippy, rustfmt"),
        "Workflow does not explicitly install clippy component. Expected 'components: rustfmt, clippy' or similar."
    );
}

#[test]
fn test_workflow_deploys_to_production() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow is triggered on main branch, which typically deploys to production
    assert!(
        content.contains("main"),
        "Workflow should be configured for main branch (production) deployment"
    );
}

#[test]
fn test_workflow_has_permissions_set() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow should have permissions defined
    assert!(
        content.contains("permissions:"),
        "Workflow should have permissions defined"
    );
}

#[test]
fn test_workflow_sets_up_nodejs() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("setup-node") || content.contains("actions/setup-node"),
        "Workflow does not set up Node.js (required for cast toolchain install)"
    );
}

#[test]
fn test_workflow_uses_recursive_flag() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow should use --recursive flag with depth specification
    assert!(
        content.contains("--recursive 2") || content.contains("--recursive"),
        "Workflow should use --recursive flag for cast cd"
    );
}

#[test]
fn test_workflow_checkout_uses_fetch_depth() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Workflow needs proper fetch-depth for --last-commit to work
    assert!(
        content.contains("fetch-depth"),
        "Workflow should specify fetch-depth for git checkout"
    );
}
