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
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run CI checks for a Rust project
/// This runs:
/// 1. cargo fmt --check
/// 2. cargo clippy -- -D warnings
/// 3. cast build (cargo build)
/// 4. cast test (cargo test)
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

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
}
