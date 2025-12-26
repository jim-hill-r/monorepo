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
fn test_workflow_trigger_is_pull_request_closed() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("pull_request:") && content.contains("types: [closed]"),
        "Workflow trigger does not include pull_request with closed type"
    );
}

#[test]
fn test_workflow_checks_merged_condition() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("github.event.pull_request.merged"),
        "Workflow does not check if PR was merged"
    );
}

#[test]
fn test_workflow_uses_cast_cli_to_detect_changes() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("CAST_BIN")
            && content.contains("project")
            && content.contains("with-changes"),
        "Workflow does not use cast CLI to detect changes"
    );
}

#[test]
fn test_workflow_searches_for_cast_toml() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("Cast.toml"),
        "Workflow does not search for Cast.toml files"
    );
}

#[test]
fn test_workflow_builds_cast_cli() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast_cli") && content.contains("cargo build"),
        "Workflow does not build cast CLI"
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
fn test_workflow_handles_no_projects_changed() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("No projects") || content.contains("has_projects"),
        "Workflow may not handle case where no projects changed"
    );
}

// Error handling tests
#[test]
fn test_workflow_contains_explicit_git_fetch_commands() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("git fetch origin")
            && content.contains("BASE_SHA")
            && content.contains("HEAD_SHA"),
        "Workflow missing explicit git fetch commands"
    );
}

#[test]
fn test_workflow_checks_git_diff_exit_code() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("if [ $? -ne 0 ]") || content.contains("if [ $? -eq 0 ]"),
        "Workflow does not check git diff exit code"
    );
}

#[test]
fn test_workflow_captures_stderr_from_cast_command() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("with-changes") && content.contains("2>&1"),
        "Workflow does not capture stderr from cast command"
    );
}

#[test]
fn test_workflow_prints_error_output_on_failure() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("echo") && content.contains("CHANGED_PROJECTS"),
        "Workflow does not print error output"
    );
}

#[test]
fn test_workflow_exits_with_error_on_cast_command_failure() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Check that after checking exit code, there's an exit 1
    assert!(
        content.contains("exit 1"),
        "Workflow does not exit with error on cast command failure"
    );
}

#[test]
fn test_fetch_commands_use_graceful_failure() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("git fetch") && content.contains("|| true"),
        "Fetch commands may fail the workflow unnecessarily"
    );
}

// Quoting tests
#[test]
fn test_base_sha_is_properly_quoted() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Check for properly quoted BASE_SHA assignment
    assert!(
        content.contains(r#"BASE_SHA="${{ github.event.pull_request.base.sha }}"#),
        "BASE_SHA is not properly quoted. Expected: BASE_SHA=\"${{{{ github.event.pull_request.base.sha }}}}\""
    );
}

#[test]
fn test_head_sha_is_properly_quoted() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Check for properly quoted HEAD_SHA assignment
    assert!(
        content.contains(r#"HEAD_SHA="${{ github.event.pull_request.head.sha }}"#),
        "HEAD_SHA is not properly quoted. Expected: HEAD_SHA=\"${{{{ github.event.pull_request.head.sha }}}}\""
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
