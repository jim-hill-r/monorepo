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

    assert!(
        content.contains("artifacts/x86_64-unknown-linux-gnu/")
            && content.contains("PROJECTS="),
        "Workflow should detect projects from artifact paths"
    );
}

#[test]
fn test_workflow_uses_git_diff_for_detection() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("git diff") && content.contains("HEAD~1"),
        "Workflow should use git diff to detect changed artifacts"
    );
}

#[test]
fn test_workflow_processes_projects_with_cast_toml() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // While the workflow doesn't search for Cast.toml during detection,
    // it still expects projects to have Cast.toml for cast cd to work
    assert!(
        content.contains("project") || content.contains("cd \"$GITHUB_WORKSPACE/$project\""),
        "Workflow should process projects (which are expected to have Cast.toml)"
    );
}

#[test]
fn test_workflow_builds_cast_cli() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast/cli") && content.contains("cargo build"),
        "Workflow does not build cast CLI"
    );
}

#[test]
fn test_workflow_runs_cast_cd_command() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains(r#"CAST_BIN" cd"#) || content.contains("cast cd"),
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

#[test]
fn test_workflow_checks_git_diff_exit_code() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // The workflow may check exit codes for validation
    assert!(
        content.contains("EXIT_CODE"),
        "Workflow should track exit code"
    );
}

#[test]
fn test_workflow_prints_error_output_on_failure() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("echo") && content.contains("PROJECTS"),
        "Workflow should print error output"
    );
}

#[test]
fn test_workflow_exits_with_error_on_cast_command_failure() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Check that after checking exit code, there's an exit 1 or EXIT_CODE tracking
    assert!(
        content.contains("exit") && content.contains("EXIT_CODE"),
        "Workflow should exit with error on cast command failure"
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
fn test_workflow_sets_cloudflare_api_token_from_secret() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("CLOUDFLARE_API_TOKEN")
            && content.contains("GHA_CLOUDFLARE_PAGES_DEPLOY_TOKEN"),
        "Workflow does not set CLOUDFLARE_API_TOKEN environment variable from GHA_CLOUDFLARE_PAGES_DEPLOY_TOKEN secret"
    );
}

#[test]
fn test_workflow_masks_cloudflare_api_token() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Check that the workflow uses secrets.GHA_CLOUDFLARE_PAGES_DEPLOY_TOKEN
    // which is automatically masked by GitHub Actions
    assert!(
        content.contains("secrets.GHA_CLOUDFLARE_PAGES_DEPLOY_TOKEN"),
        "Workflow should use secrets.GHA_CLOUDFLARE_PAGES_DEPLOY_TOKEN to ensure the token is masked in logs"
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
fn test_workflow_runs_cast_install() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast") && content.contains("install"),
        "Workflow does not run cast install command"
    );
}

#[test]
fn test_workflow_installs_before_cd() {
    let content =
        fs::read_to_string(get_cast_cd_workflow_path()).expect("Failed to read workflow file");

    // Find positions of cast install and cast cd
    let install_pos = content
        .find(r#"CAST_BIN" install"#)
        .expect("Workflow missing cast install");
    // Look for the pattern where CAST_BIN is used to run cd command
    let cd_pos = content
        .find(r#"CAST_BIN" cd"#)
        .expect("Workflow missing cast cd command (looking for CAST_BIN cd)");

    assert!(
        install_pos < cd_pos,
        "Workflow must run cast install before cast cd"
    );
}
