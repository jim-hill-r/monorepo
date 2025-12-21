use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CiError {
    #[error("Cargo fmt check failed: {0}")]
    FmtError(String),
    #[error("Cargo clippy check failed: {0}")]
    ClippyError(String),
    #[error("Cargo build failed: {0}")]
    BuildError(String),
    #[error("Cargo test failed: {0}")]
    TestError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run CI checks for a Rust project
/// This runs:
/// 1. cargo fmt --check
/// 2. cargo clippy -- -D warnings
/// 3. cargo build
/// 4. cargo test
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

    // Run cargo fmt --check
    run_fmt_check(working_directory)?;

    // Run cargo clippy
    run_clippy(working_directory)?;

    // Run cargo build
    run_build(working_directory)?;

    // Run cargo test
    run_test(working_directory)?;

    Ok(())
}

/// Format the output from a cargo command failure
fn format_cargo_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&stderr);
    }
    result
}

fn run_fmt_check(working_directory: &Path) -> Result<(), CiError> {
    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--check")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(CiError::FmtError(format_cargo_output(&output)));
    }

    Ok(())
}

fn run_clippy(working_directory: &Path) -> Result<(), CiError> {
    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(CiError::ClippyError(format_cargo_output(&output)));
    }

    Ok(())
}

fn run_build(working_directory: &Path) -> Result<(), CiError> {
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(CiError::BuildError(format_cargo_output(&output)));
    }

    Ok(())
}

fn run_test(working_directory: &Path) -> Result<(), CiError> {
    let output = Command::new("cargo")
        .arg("test")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(CiError::TestError(format_cargo_output(&output)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_ci_fails_without_cargo_project() {
        let tmp_dir = TempDir::new("test_ci").unwrap();
        let result = run(tmp_dir.path());
        assert!(result.is_err());
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
    fn test_run_build_passes_with_valid_code() {
        let tmp_dir = TempDir::new("test_build").unwrap();

        // Create a simple Cargo project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = run_build(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_test_passes_with_valid_tests() {
        let tmp_dir = TempDir::new("test_test").unwrap();

        // Create a simple Cargo project with a test
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

        let result = run_test(tmp_dir.path());
        assert!(result.is_ok());
    }
}
