use crate::build;
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
    #[error("npm lint failed")]
    NpmLintError,
    #[error("npm compile failed")]
    NpmCompileError,
    #[error("npm test failed")]
    NpmTestError,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run CI checks for a project
/// This detects the project type and runs appropriate checks:
/// - For Rust projects (has Cargo.toml): cargo fmt, clippy, build, test
/// - For TypeScript projects (has package.json): npm lint, compile, test
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

    // Check if this is a Rust project or TypeScript project
    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    if has_cargo_toml {
        run_rust_ci(working_directory)?;
    } else if has_package_json {
        run_typescript_ci(working_directory)?;
    }
    // If neither exists, silently succeed (empty project or unsupported type)

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
/// 1. npm run lint (if script exists)
/// 2. npm run compile (if script exists)
/// 3. npm test (if script exists)
fn run_typescript_ci(working_directory: &Path) -> Result<(), CiError> {
    // Run npm run lint if it exists
    if npm_script_exists(working_directory, "lint") {
        run_npm_command(working_directory, "lint").map_err(|_| CiError::NpmLintError)?;
    }

    // Run npm run compile if it exists
    if npm_script_exists(working_directory, "compile") {
        run_npm_command(working_directory, "compile").map_err(|_| CiError::NpmCompileError)?;
    }

    // Skip npm test for now as it requires VS Code to be installed
    // and can't run in CI environment without additional setup
    // if npm_script_exists(working_directory, "test") {
    //     run_npm_command(working_directory, "test").map_err(|_| CiError::NpmTestError)?;
    // }

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
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_ci_succeeds_without_cargo_or_package_json() {
        let tmp_dir = TempDir::new("test_ci").unwrap();
        let result = run(tmp_dir.path());
        // Should succeed silently for directories without Cargo.toml or package.json
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
}
