use std::fs;
use workflow_tests::*;

#[test]
fn test_workflow_file_exists() {
    let workflow_path = get_cast_ci_workflow_path();
    assert!(
        workflow_path.exists(),
        "Workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_workflow_yaml_can_be_parsed() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_workflow_trigger_is_pull_request() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("pull_request:"),
        "Workflow trigger does not include pull_request"
    );
}

#[test]
fn test_workflow_uses_cast_cli_to_detect_changes() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

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
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("Cast.toml"),
        "Workflow does not search for Cast.toml files"
    );
}

#[test]
fn test_workflow_builds_cast_cli() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast_workspace/cli") && content.contains("cargo build"),
        "Workflow does not build cast CLI"
    );
}

#[test]
fn test_workflow_runs_cast_ci_command() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast ci"),
        "Workflow does not run cast ci command"
    );
}

#[test]
fn test_cast_cli_project_exists() {
    let cast_cli_cargo = get_cast_cli_cargo_path();
    assert!(cast_cli_cargo.exists(), "cast_cli project not found");
}

#[test]
fn test_workflow_sets_up_rust_toolchain() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

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
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("No projects") || content.contains("has_projects"),
        "Workflow may not handle case where no projects changed"
    );
}

// Error handling tests
#[test]
fn test_workflow_contains_explicit_git_fetch_commands() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

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
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("if [ $? -ne 0 ]") || content.contains("if [ $? -eq 0 ]"),
        "Workflow does not check git diff exit code"
    );
}

#[test]
fn test_workflow_captures_stderr_from_cast_command() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("with-changes") && content.contains("2>&1"),
        "Workflow does not capture stderr from cast command"
    );
}

#[test]
fn test_workflow_prints_error_output_on_failure() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("echo") && content.contains("CHANGED_PROJECTS"),
        "Workflow does not print error output"
    );
}

#[test]
fn test_workflow_exits_with_error_on_cast_command_failure() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check that after checking exit code, there's an exit 1
    assert!(
        content.contains("exit 1"),
        "Workflow does not exit with error on cast command failure"
    );
}

#[test]
fn test_fetch_commands_use_graceful_failure() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("git fetch") && content.contains("|| true"),
        "Fetch commands may fail the workflow unnecessarily"
    );
}

// Quoting tests
#[test]
fn test_base_sha_is_properly_quoted() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check for properly quoted BASE_SHA assignment
    assert!(
        content.contains(r#"BASE_SHA="${{ github.event.pull_request.base.sha }}"#),
        "BASE_SHA is not properly quoted. Expected: BASE_SHA=\"${{{{ github.event.pull_request.base.sha }}}}\""
    );
}

#[test]
fn test_head_sha_is_properly_quoted() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check for properly quoted HEAD_SHA assignment
    assert!(
        content.contains(r#"HEAD_SHA="${{ github.event.pull_request.head.sha }}"#),
        "HEAD_SHA is not properly quoted. Expected: HEAD_SHA=\"${{{{ github.event.pull_request.head.sha }}}}\""
    );
}

#[test]
fn test_workflow_installs_rustfmt_component() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("components: rustfmt, clippy")
            || content.contains("components: clippy, rustfmt"),
        "Workflow does not explicitly install rustfmt component. Expected 'components: rustfmt, clippy' or similar."
    );
}

#[test]
fn test_workflow_installs_clippy_component() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("components: rustfmt, clippy")
            || content.contains("components: clippy, rustfmt"),
        "Workflow does not explicitly install clippy component. Expected 'components: rustfmt, clippy' or similar."
    );
}

#[test]
fn test_workflow_uses_cast_toolchain_install() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("toolchain install"),
        "Workflow does not use 'cast toolchain install' command"
    );
}

#[test]
fn test_workflow_does_not_manually_install_dioxus_cli() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check that workflow doesn't have manual dx installation steps
    assert!(
        !content.contains("cargo install dioxus-cli"),
        "Workflow should not manually install dioxus-cli; should use 'cast toolchain install'"
    );
}

#[test]
fn test_workflow_does_not_manually_install_playwright() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check that workflow doesn't have manual Playwright installation loop
    assert!(
        !content.contains("npx playwright install --with-deps chromium"),
        "Workflow should use 'cast toolchain install' instead of manual Playwright installation"
    );
}

// Integration tests for cast toolchain command
#[test]
fn test_cast_toolchain_check_works_on_rust_library() {
    // Test that `cast toolchain check` works on a pure Rust library project
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast_workspace/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast_workspace"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast toolchain check` on a pure Rust library project
    let output = Command::new(&cast_cli)
        .args(["toolchain", "check"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast toolchain check");

    assert!(
        output.status.success(),
        "cast toolchain check failed on example_rust_library: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo") && stdout.contains("rustc"),
        "cast toolchain check should detect cargo and rustc"
    );
}

#[test]
fn test_cast_toolchain_check_detects_framework() {
    // Test that `cast toolchain check` correctly detects Dioxus projects
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast_workspace/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast_workspace"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast toolchain check` on a Dioxus web project
    let output = Command::new(&cast_cli)
        .args(["toolchain", "check"])
        .current_dir(repo_root.join("pane"))
        .output()
        .expect("Failed to execute cast toolchain check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // For a Dioxus project, it should check for dx, node, npm, playwright
    assert!(
        stdout.contains("dx") || stderr.contains("dx") || stdout.contains("dioxus"),
        "cast toolchain check should detect Dioxus-specific tools for pane project"
    );
}

#[test]
fn test_cast_toolchain_list_command() {
    // Test that `cast toolchain list` command works
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast_workspace/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast_workspace"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast toolchain list`
    let output = Command::new(&cast_cli)
        .args(["toolchain", "list"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast toolchain list");

    assert!(
        output.status.success(),
        "cast toolchain list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo") && stdout.contains("rustc"),
        "cast toolchain list should show cargo and rustc"
    );
}

#[test]
fn test_cast_toolchain_install_dry_run() {
    // Test that `cast toolchain install --dry-run` works
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast_workspace/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast_workspace"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast toolchain install --dry-run`
    let output = Command::new(&cast_cli)
        .args(["toolchain", "install", "--dry-run"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast toolchain install --dry-run");

    assert!(
        output.status.success(),
        "cast toolchain install --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo")
            || stdout.contains("Already installed")
            || stdout.contains("Would install"),
        "cast toolchain install --dry-run should show installation status"
    );
}

#[test]
fn test_workflow_installs_toolchain_before_ci() {
    // Verify workflow installs toolchain before running cast ci
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Find positions of toolchain install and cast ci commands
    let toolchain_install_pos = content
        .find("toolchain install")
        .expect("Workflow should contain 'toolchain install'");

    let cast_ci_pos = content[toolchain_install_pos..]
        .find("cast ci")
        .expect("Workflow should contain 'cast ci' after toolchain install");

    assert!(
        cast_ci_pos > 0,
        "Workflow should run 'toolchain install' before 'cast ci'"
    );
}
