//! Integration tests for the toolchain command on Linux/GitHub Actions environment

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;

/// Get the path to the cast binary for testing
fn get_cast_binary_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut path = PathBuf::from(manifest_dir);
    // CARGO_MANIFEST_DIR is now cast/core, so go up one level to reach cast
    path.pop(); // Go up to cast
    path.push("target");

    // Try release first, then debug
    let mut release_path = path.clone();
    release_path.push("release");
    release_path.push("cast");

    if release_path.exists() {
        return release_path;
    }

    let mut debug_path = path;
    debug_path.push("debug");
    debug_path.push("cast");

    if debug_path.exists() {
        return debug_path;
    }

    panic!("Cast binary not found. Please build cast_cli first with: cd cast && cargo build -p cast_cli");
}

#[test]
fn test_toolchain_check_pure_rust_linux() {
    let temp_dir = TempDir::new("test_toolchain_check_rust").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "check"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    // Check command may return error if tools are missing, which is expected behavior
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("rustc"), "Output should mention rustc");
    assert!(combined.contains("cargo"), "Output should mention cargo");
    assert!(
        combined.contains("rustfmt"),
        "Output should mention rustfmt"
    );
    assert!(combined.contains("clippy"), "Output should mention clippy");
}

#[test]
fn test_toolchain_check_dioxus_detects_requirements_linux() {
    let temp_dir = TempDir::new("test_toolchain_check_dioxus").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "check"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("dioxus"),
        "Output should mention dioxus framework"
    );
    assert!(combined.contains("dx"), "Output should mention dx tool");
    assert!(combined.contains("node"), "Output should mention node tool");
    assert!(combined.contains("npm"), "Output should mention npm tool");
    assert!(
        combined.contains("playwright"),
        "Output should mention playwright tool"
    );
}

#[test]
fn test_toolchain_check_json_output_linux() {
    let temp_dir = TempDir::new("test_toolchain_check_json").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "check", "--json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains('{') && combined.contains('}'),
        "Output should contain JSON"
    );
    assert!(
        combined.contains("\"tools\""),
        "JSON output should have tools field"
    );
    assert!(
        combined.contains("\"all_installed\""),
        "JSON output should have all_installed field"
    );
}

#[test]
fn test_toolchain_list_linux() {
    let temp_dir = TempDir::new("test_toolchain_list").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "list"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "install list should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rustc:"), "Output should list rustc");
    assert!(stdout.contains("cargo:"), "Output should list cargo");
}

#[test]
fn test_toolchain_install_dry_run_linux() {
    let temp_dir =
        TempDir::new("test_toolchain_install_dry_run").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "--dry-run"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Already installed") || stdout.contains("Would install"),
        "Output should indicate dry-run actions"
    );
}

#[test]
fn test_toolchain_install_node_guidance_linux() {
    let temp_dir = TempDir::new("test_toolchain_install_node").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "--tool", "node", "--dry-run"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Node is already installed in GitHub Actions, or should provide guidance
    assert!(
        combined.contains("node")
            && (combined.contains("Already installed")
                || combined.contains("Linux")
                || combined.contains("apt")
                || combined.contains("system")),
        "Output should mention node and either show it's installed or provide guidance"
    );
}

#[test]
fn test_toolchain_list_json_output_linux() {
    let temp_dir = TempDir::new("test_toolchain_list_json").expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true")
        .expect("Failed to write Cast.toml");

    let cast_bin = get_cast_binary_path();
    let output = Command::new(&cast_bin)
        .args(["install", "list", "--json"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "install list --json should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains('{') && stdout.contains('}'),
        "Output should contain JSON"
    );
    assert!(
        stdout.contains("\"tools\""),
        "JSON output should have tools field"
    );
    assert!(
        stdout.contains("\"name\""),
        "JSON output should have name field for tools"
    );
    assert!(
        stdout.contains("\"installed\""),
        "JSON output should have installed field"
    );
}
