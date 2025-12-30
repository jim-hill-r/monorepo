use crate::build;
use crate::publish;
use crate::test;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CiError {
    #[error("Cargo fmt check failed")]
    FmtError,
    #[error("Cargo clippy check failed")]
    ClippyError,
    #[error("Cargo build failed: {0}")]
    BuildError(#[from] build::BuildError),
    #[error("Cargo test failed: {0}")]
    TestError(#[from] test::TestError),
    #[error("npm install failed")]
    NpmInstallError,
    #[error("npm lint failed")]
    NpmLintError,
    #[error("npm compile failed")]
    NpmCompileError,
    #[error("npm test failed")]
    NpmTestError,
    #[error("Publish failed: {0}")]
    PublishError(#[from] publish::PublishError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run CI checks for a project
/// This detects the project type and runs appropriate checks:
/// - For Rust projects (has Cargo.toml): cargo fmt, clippy, build, test
/// - For TypeScript projects (has package.json): npm lint, compile, test
/// - Projects can have both Cargo.toml and package.json (e.g., Dioxus web apps with Playwright tests)
/// - If all checks pass, runs publish to create release artifacts
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

    // Check if this is a Rust project or TypeScript project
    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    // Track if we ran any CI checks
    let mut ran_ci_checks = false;

    // Run Rust CI if Cargo.toml exists
    if has_cargo_toml {
        run_rust_ci(working_directory)?;
        ran_ci_checks = true;
    }

    // Run TypeScript CI if package.json exists (can run in addition to Rust CI)
    if has_package_json {
        run_typescript_ci(working_directory)?;
        ran_ci_checks = true;
    }

    // If we ran CI checks and they all passed, run publish to create release artifacts
    if ran_ci_checks {
        publish::run(working_directory)?;
    }
    // If no CI checks were run, silently succeed (empty project or unsupported type)

    Ok(())
}

/// Run CI checks for a Rust project
/// This runs:
/// 1. cargo fmt --check
/// 2. cargo clippy -- -D warnings
/// 3. cast build (cargo build)
/// 4. cast test (cargo test)
fn run_rust_ci(working_directory: &Path) -> Result<(), CiError> {
    // Run cargo fmt --check
    run_fmt_check(working_directory)?;

    // Run cargo clippy
    run_clippy(working_directory)?;

    // Run cast build
    build::run(working_directory)?;

    // Run cast test
    test::run(working_directory)?;

    Ok(())
}

/// Run CI checks for a TypeScript/Node.js project
/// This runs:
/// 1. npm ci (to install dependencies from lockfile)
/// 2. npm run lint (if script exists)
/// 3. npm run compile (if script exists)
/// 4. npm test (if script exists)
fn run_typescript_ci(working_directory: &Path) -> Result<(), CiError> {
    // Run npm ci to ensure dependencies are installed from lockfile
    run_npm_install(working_directory).map_err(|_| CiError::NpmInstallError)?;

    // Run npm run lint if it exists
    if npm_script_exists(working_directory, "lint") {
        run_npm_command(working_directory, "lint").map_err(|_| CiError::NpmLintError)?;
    }

    // Run npm run compile if it exists
    if npm_script_exists(working_directory, "compile") {
        run_npm_command(working_directory, "compile").map_err(|_| CiError::NpmCompileError)?;
    }

    // Run npm test if it exists (e.g., Playwright tests)
    if npm_script_exists(working_directory, "test") {
        run_npm_command(working_directory, "test").map_err(|_| CiError::NpmTestError)?;
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

/// Run an npm command
fn run_npm_command(working_directory: &Path, command: &str) -> Result<(), std::io::Error> {
    let status = Command::new("npm")
        .arg("run")
        .arg(command)
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other(format!("npm run {} failed", command)));
    }

    Ok(())
}

/// Run npm ci to install dependencies from lockfile
/// Uses 'npm ci' for faster, more reliable installs in CI environments
fn run_npm_install(working_directory: &Path) -> Result<(), std::io::Error> {
    let status = Command::new("npm")
        .arg("ci")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("npm ci failed"));
    }

    Ok(())
}

fn run_fmt_check(working_directory: &Path) -> Result<(), CiError> {
    let status = Command::new("cargo")
        .arg("fmt")
        .arg("--check")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::FmtError);
    }

    Ok(())
}

fn run_clippy(working_directory: &Path) -> Result<(), CiError> {
    let status = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::ClippyError);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_ci_succeeds_without_cargo_or_package_json() {
        let tmp_dir = TempDir::new("test_ci").unwrap();

        let result = run(tmp_dir.path());
        // Should succeed silently for directories without Cargo.toml or package.json
        // Publish should not run since no CI checks were performed
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_fmt_check_passes_with_formatted_code() {
        let tmp_dir = TempDir::new("test_fmt").unwrap();

        // Create a simple Cargo project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = run_fmt_check(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_ci_runs_publish_after_successful_checks() {
        let tmp_dir = TempDir::new("test_ci_publish").unwrap();

        // Initialize git repo (required by publish)
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a minimal binary project with properly formatted code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .unwrap();

        // Commit the project
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let result = run(tmp_dir.path());
        assert!(
            result.is_ok(),
            "CI should succeed and run publish: {:?}",
            result.err()
        );

        // Verify that artifacts directory was created by publish
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            artifacts_dir.exists(),
            "Artifacts directory should exist after CI runs publish"
        );
    }

    #[test]
    fn test_ci_does_not_run_publish_when_build_fails() {
        let tmp_dir = TempDir::new("test_ci_no_publish").unwrap();

        // Initialize git repo (required by publish)
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a project with invalid code (will fail build)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        // Write properly formatted but semantically invalid code
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    undefined_function();\n}\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err(), "CI should fail when build fails");

        // Verify that artifacts directory was NOT created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should not exist when CI fails"
        );
    }
}
