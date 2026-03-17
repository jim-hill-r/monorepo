use crate::install;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Cargo test failed")]
    TestFailed,
    #[error("npm test failed")]
    NpmTestFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run tests for a project
/// This detects the project type and runs appropriate tests:
/// - For Rust projects (has Cargo.toml): cargo test (or cargo llvm-cov if coverage is requested)
/// - For TypeScript/Node.js projects (has package.json with test script): npm test (with --coverage if supported)
/// - Projects can have both (e.g., Dioxus web apps with Playwright tests)
pub fn run(working_directory: impl AsRef<Path>, coverage: bool) -> Result<(), TestError> {
    let working_directory = working_directory.as_ref();

    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    // Run Rust tests if Cargo.toml exists
    if has_cargo_toml {
        run_cargo_test(working_directory, coverage)?;
    }

    // Run npm tests if package.json exists and has a test script
    if has_package_json && npm_script_exists(working_directory, "test") {
        run_npm_test(working_directory, coverage)?;
    }

    Ok(())
}

/// Run cargo test for a Rust project
fn run_cargo_test(working_directory: &Path, coverage: bool) -> Result<(), TestError> {
    run_cargo_test_with_options(working_directory, coverage, true, None)
}

/// Returns true if cargo-llvm-cov is available.
fn is_llvm_cov_installed() -> bool {
    match Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Attempts to install cargo-llvm-cov. Returns true if installation succeeded.
fn try_install_llvm_cov() -> bool {
    eprintln!("Warning: cargo-llvm-cov is not installed. Installing it now...");
    eprintln!("Run: cargo install cargo-llvm-cov");
    match Command::new("cargo")
        .args(["install", "cargo-llvm-cov"])
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Run regular `cargo test` without coverage for a Rust project.
///
/// Used both when coverage is not requested and as a fallback when
/// cargo-llvm-cov is unavailable or installation fails.
fn run_regular_cargo_test(working_directory: &Path) -> Result<(), TestError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test").current_dir(working_directory);

    for (var_name, var_value) in install::detect_llvm_env() {
        cmd.env(var_name, var_value);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(TestError::TestFailed);
    }
    Ok(())
}

/// Run cargo test for a Rust project with optional coverage support.
///
/// When `coverage` is true, the function uses `cargo llvm-cov` if it is
/// available.  When `attempt_install` is true and `cargo-llvm-cov` is not
/// found, the function will first attempt to install it before falling back
/// to regular `cargo test`.  When `attempt_install` is false the function
/// skips the install step and falls back to regular `cargo test` immediately
/// if `cargo-llvm-cov` is not already available.  This is primarily useful
/// for tests that want to exercise the fallback path without triggering a
/// potentially long-running installation.
///
/// The `is_installed_override` parameter allows tests to simulate whether
/// cargo-llvm-cov is installed without actually checking the system.
fn run_cargo_test_with_options(
    working_directory: &Path,
    coverage: bool,
    attempt_install: bool,
    is_installed_override: Option<bool>,
) -> Result<(), TestError> {
    if coverage {
        // Use cargo-llvm-cov for code coverage if available.
        let is_installed = is_installed_override.unwrap_or_else(is_llvm_cov_installed);

        if !is_installed {
            let install_succeeded = if attempt_install {
                try_install_llvm_cov()
            } else {
                false
            };

            if !install_succeeded {
                eprintln!("Failed to install cargo-llvm-cov. Falling back to regular cargo test.");
                return run_regular_cargo_test(working_directory);
            }
        }

        // Run cargo llvm-cov to generate coverage
        let mut cmd = Command::new("cargo");
        cmd.args([
            "llvm-cov",
            "--all-features",
            "--workspace",
            "--lcov",
            "--output-path",
            "lcov.info",
        ])
        .current_dir(working_directory);

        // Configure LLVM environment if needed
        for (var_name, var_value) in install::detect_llvm_env() {
            cmd.env(var_name, var_value);
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err(TestError::TestFailed);
        }

        println!("Coverage report generated: lcov.info");
    } else {
        run_regular_cargo_test(working_directory)?;
    }

    Ok(())
}

/// Run npm test for a Node.js project
fn run_npm_test(working_directory: &Path, coverage: bool) -> Result<(), TestError> {
    let mut args = vec!["test"];

    // Add --coverage flag if coverage is requested
    // Most modern test frameworks (Jest, Vitest, etc.) support this flag
    if coverage {
        args.push("--");
        args.push("--coverage");
    }

    let status = Command::new("npm")
        .args(&args)
        .current_dir(working_directory)
        .stdin(std::process::Stdio::null()) // Prevent blocking on user input (e.g., Playwright HTML reporter)
        .status()?;

    if !status.success() {
        return Err(TestError::NpmTestFailed);
    }

    if coverage {
        println!("Coverage report generated in coverage/ directory");
    }

    Ok(())
}

/// Check if an npm script exists in package.json
fn npm_script_exists(working_directory: &Path, script: &str) -> bool {
    let package_json_path = working_directory.join("package.json");
    if let Ok(content) = std::fs::read_to_string(package_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(scripts) = json.get("scripts") {
                return scripts.get(script).is_some();
            }
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_succeeds_without_cargo_or_package_json() {
        let tmp_dir = TempDir::new("test_run_no_project").unwrap();
        let result = run(tmp_dir.path(), false);
        // Should succeed silently for directories without Cargo.toml or package.json
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_passes_with_valid_tests() {
        let tmp_dir = TempDir::new("test_run_valid").unwrap();

        // Create a simple Cargo project with a passing test
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn test_add() {\n        assert_eq!(add(1, 2), 3);\n    }\n}",
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_fails_with_failing_tests() {
        let tmp_dir = TempDir::new("test_run_failing").unwrap();

        // Create a Cargo project with a failing test
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn test_add_fails() {\n        assert_eq!(add(1, 2), 4); // This will fail\n    }\n}",
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        assert!(result.is_err());
        if let Err(TestError::TestFailed) = result {
            // Expected error type
        } else {
            panic!("Expected TestFailed error");
        }
    }

    #[test]
    fn test_npm_script_exists_returns_true_when_script_exists() {
        let tmp_dir = TempDir::new("test_npm_script").unwrap();

        // Create a package.json with a test script
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"test": "echo test"}}"#,
        )
        .unwrap();

        assert!(npm_script_exists(tmp_dir.path(), "test"));
    }

    #[test]
    fn test_npm_script_exists_returns_false_when_script_missing() {
        let tmp_dir = TempDir::new("test_npm_no_script").unwrap();

        // Create a package.json without a test script
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"build": "echo build"}}"#,
        )
        .unwrap();

        assert!(!npm_script_exists(tmp_dir.path(), "test"));
    }

    #[test]
    fn test_npm_script_exists_returns_false_when_no_package_json() {
        let tmp_dir = TempDir::new("test_no_package_json").unwrap();

        assert!(!npm_script_exists(tmp_dir.path(), "test"));
    }

    #[test]
    fn test_run_npm_test_passes_with_valid_test() {
        let tmp_dir = TempDir::new("test_npm_valid").unwrap();

        // Create a package.json with a passing test
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"test": "exit 0"}}"#,
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_npm_test_fails_with_failing_test() {
        let tmp_dir = TempDir::new("test_npm_failing").unwrap();

        // Create a package.json with a failing test
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"test": "exit 1"}}"#,
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        assert!(result.is_err());
        if let Err(TestError::NpmTestFailed) = result {
            // Expected error type
        } else {
            panic!("Expected NpmTestFailed error");
        }
    }

    #[test]
    fn test_run_with_both_cargo_and_npm_tests() {
        let tmp_dir = TempDir::new("test_both").unwrap();

        // Create a Cargo project with tests
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn test_add() {\n        assert_eq!(add(1, 2), 3);\n    }\n}",
        )
        .unwrap();

        // Create a package.json with a test script
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"test": "exit 0"}}"#,
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_package_json_without_test_script_only_runs_cargo() {
        let tmp_dir = TempDir::new("test_no_npm_script").unwrap();

        // Create a Cargo project with tests
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn test_add() {\n        assert_eq!(add(1, 2), 3);\n    }\n}",
        )
        .unwrap();

        // Create a package.json without a test script
        fs::write(
            tmp_dir.path().join("package.json"),
            r#"{"scripts": {"build": "echo build"}}"#,
        )
        .unwrap();

        let result = run(tmp_dir.path(), false);
        // Should succeed (only cargo test runs, npm test is skipped)
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cargo_coverage_falls_back_when_llvm_cov_unavailable() {
        let tmp_dir = TempDir::new("test_coverage_fallback").unwrap();

        // Create a simple Cargo project with a passing test
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn test_add() {\n        assert_eq!(add(1, 2), 3);\n    }\n}",
        )
        .unwrap();

        // Simulate cargo-llvm-cov being unavailable by passing Some(false) as is_installed_override.
        // This exercises the fallback path without actually checking the system or attempting
        // to install cargo-llvm-cov (which can take several minutes).
        let result = run_cargo_test_with_options(tmp_dir.path(), true, false, Some(false));
        // Should succeed by falling back to regular cargo test.
        assert!(result.is_ok());
    }
}
