use std::fs;
use workflow_tests::*;

// Tests for Pull Request CI workflow
#[test]
fn test_pull_request_workflow_file_exists() {
    let workflow_path = get_pull_request_ci_workflow_path();
    assert!(
        workflow_path.exists(),
        "Pull Request CI workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_pull_request_workflow_yaml_can_be_parsed() {
    let content = fs::read_to_string(get_pull_request_ci_workflow_path())
        .expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_pull_request_workflow_trigger_is_pull_request() {
    let content = fs::read_to_string(get_pull_request_ci_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("pull_request:"),
        "Pull Request CI workflow trigger does not include pull_request"
    );
}

#[test]
fn test_pull_request_workflow_uses_check_flag() {
    let content = fs::read_to_string(get_pull_request_ci_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("--check"),
        "Pull Request CI workflow should use --check flag for validation only"
    );
}

// Tests for Trunk CI workflow
#[test]
fn test_trunk_workflow_file_exists() {
    let workflow_path = get_trunk_ci_workflow_path();
    assert!(
        workflow_path.exists(),
        "Trunk CI workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_trunk_workflow_yaml_can_be_parsed() {
    let content =
        fs::read_to_string(get_trunk_ci_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_trunk_workflow_trigger_is_push_to_main() {
    let content =
        fs::read_to_string(get_trunk_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("push:") && content.contains("main"),
        "Trunk CI workflow should trigger on push to main"
    );
}

#[test]
fn test_trunk_workflow_uses_release_flag() {
    let content =
        fs::read_to_string(get_trunk_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("--release"),
        "Trunk CI workflow should use --release flag to build artifacts"
    );
}

#[test]
fn test_trunk_workflow_ignores_artifacts() {
    let content =
        fs::read_to_string(get_trunk_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("paths-ignore") && content.contains("artifacts"),
        "Trunk CI workflow should ignore changes to artifacts directories"
    );
}

// Shared tests for both workflows
#[test]
fn test_workflow_file_exists() {
    // Test backwards compatibility - at least one workflow exists
    let pr_path = get_pull_request_ci_workflow_path();
    assert!(
        pr_path.exists(),
        "Workflow file not found: {}",
        pr_path.display()
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
fn test_workflow_uses_only_changed_flag() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("--only-changed"),
        "Workflow should use --only-changed flag to check only projects with changes"
    );
}

#[test]
fn test_workflow_uses_recursive_flag() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("--recursive"),
        "Workflow should use --recursive flag to check all Cast projects"
    );
}

#[test]
fn test_workflow_searches_for_cast_toml() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // With --recursive flag, cast ci searches for Cast.toml files automatically
    assert!(
        content.contains("--recursive"),
        "Workflow should use --recursive flag which searches for Cast.toml files"
    );
}

#[test]
fn test_workflow_builds_cast_cli() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    assert!(
        content.contains("cast/cast_cli") && content.contains("cargo build"),
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

    // With --only-changed flag, cast ci automatically skips projects with no changes
    assert!(
        content.contains("--only-changed"),
        "Workflow should use --only-changed flag to handle projects with no changes"
    );
}

// Error handling is now simplified - cast ci --only-changed handles git operations internally

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
fn test_workflow_uses_cast_ci_with_install() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // cast ci automatically runs install, so no separate install step is needed
    assert!(
        content.contains("cast ci"),
        "Workflow should use 'cast ci' command which includes automatic tool installation"
    );
}

#[test]
fn test_workflow_does_not_manually_install_dioxus_cli() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check that workflow doesn't have manual dx installation steps
    // cast ci handles installation automatically
    assert!(
        !content.contains("cargo install dioxus-cli"),
        "Workflow should not manually install dioxus-cli; cast ci handles installation automatically"
    );
}

#[test]
fn test_workflow_does_not_manually_install_playwright() {
    let content =
        fs::read_to_string(get_cast_ci_workflow_path()).expect("Failed to read workflow file");

    // Check that workflow doesn't have manual Playwright installation loop
    // cast ci handles installation automatically
    assert!(
        !content.contains("npx playwright install --with-deps chromium"),
        "Workflow should let cast ci handle Playwright installation automatically"
    );
}

// Integration tests for cast install command
#[test]
fn test_cast_install_check_works_on_rust_library() {
    // Test that `cast install check` works on a pure Rust library project
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast install check` on a pure Rust library project
    let output = Command::new(&cast_cli)
        .args(["install", "check"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast install check");

    assert!(
        output.status.success(),
        "cast install check failed on example_rust_library: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo") && stdout.contains("rustc"),
        "cast install check should detect cargo and rustc"
    );
}

#[test]
fn test_cast_install_check_detects_framework() {
    // Test that `cast install check` correctly detects Dioxus projects
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast install check` on a Dioxus web project
    let output = Command::new(&cast_cli)
        .args(["install", "check"])
        .current_dir(repo_root.join("pane"))
        .output()
        .expect("Failed to execute cast install check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // For a Dioxus project, it should check for dx, node, npm, playwright
    assert!(
        stdout.contains("dx") || stderr.contains("dx") || stdout.contains("dioxus"),
        "cast install check should detect Dioxus-specific tools for pane project"
    );
}

#[test]
fn test_cast_install_list_command() {
    // Test that `cast install list` command works
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast install list`
    let output = Command::new(&cast_cli)
        .args(["install", "list"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast install list");

    assert!(
        output.status.success(),
        "cast install list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo") && stdout.contains("rustc"),
        "cast install list should show cargo and rustc"
    );
}

#[test]
fn test_cast_install_dry_run() {
    // Test that `cast install --dry-run` works
    use std::process::Command;
    use workflow_tests::get_repo_root;

    let repo_root = get_repo_root();
    let cast_cli = repo_root.join("cast/target/release/cast");

    // Build cast CLI if not already built
    if !cast_cli.exists() {
        let build_output = Command::new("cargo")
            .args(["build", "--release", "-p", "cast_cli"])
            .current_dir(repo_root.join("cast"))
            .output()
            .expect("Failed to build cast CLI");

        assert!(
            build_output.status.success(),
            "Failed to build cast CLI: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Run `cast install --dry-run`
    let output = Command::new(&cast_cli)
        .args(["install", "--dry-run"])
        .current_dir(repo_root.join("example/example_rust_library"))
        .output()
        .expect("Failed to execute cast install --dry-run");

    assert!(
        output.status.success(),
        "cast install --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo")
            || stdout.contains("Already installed")
            || stdout.contains("Would install"),
        "cast install --dry-run should show installation status"
    );
}

// cast ci now handles toolchain installation internally, no separate install step needed
