use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Cargo test failed")]
    TestFailed,
    #[error("npm ci failed")]
    NpmCiFailed,
    #[error("npm test failed")]
    NpmTestFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run tests for a project
/// This detects the project type and runs appropriate tests:
/// - For Rust projects (has Cargo.toml): cargo test
/// - For TypeScript/Node.js projects (has package.json with test script): npm ci (to ensure dependencies are installed), then npm test
/// - Projects can have both (e.g., Dioxus web apps with Playwright tests)
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), TestError> {
    let working_directory = working_directory.as_ref();

    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    // Run Rust tests if Cargo.toml exists
    if has_cargo_toml {
        run_cargo_test(working_directory)?;
    }

    // Run npm tests if package.json exists and has a test script
    // First ensure dependencies are installed via npm ci (only if package-lock.json exists)
    if has_package_json && npm_script_exists(working_directory, "test") {
        // Only run npm ci if package-lock.json exists (npm ci requires a lock file)
        if working_directory.join("package-lock.json").exists() {
            run_npm_ci(working_directory)?;
        }
        run_npm_test(working_directory)?;
    }

    Ok(())
}

/// Run cargo test for a Rust project
fn run_cargo_test(working_directory: &Path) -> Result<(), TestError> {
    let status = Command::new("cargo")
        .arg("test")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(TestError::TestFailed);
    }

    Ok(())
}

/// Run npm test for a Node.js project
fn run_npm_test(working_directory: &Path) -> Result<(), TestError> {
    let status = Command::new("npm")
        .arg("test")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(TestError::NpmTestFailed);
    }

    Ok(())
}

/// Run npm ci to install dependencies
fn run_npm_ci(working_directory: &Path) -> Result<(), TestError> {
    let status = Command::new("npm")
        .arg("ci")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(TestError::NpmCiFailed);
    }

    // After installing npm packages, check if Playwright is installed and install browsers
    // This is necessary because npm ci only installs the packages, not the browser binaries
    if working_directory.join("node_modules/@playwright/test").exists() {
        let status = Command::new("npx")
            .args(["playwright", "install", "--with-deps"])
            .current_dir(working_directory)
            .status()?;

        if !status.success() {
            eprintln!("Warning: Playwright browser installation failed, tests may fail");
            // Don't return error here, as tests might still work with cached browsers
        }
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
        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
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

        let result = run(tmp_dir.path());
        // Should succeed (only cargo test runs, npm test is skipped)
        assert!(result.is_ok());
    }
}
