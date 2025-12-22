use crate::utils::format_cargo_output;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Cargo build failed: {0}")]
    BuildFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run cargo build for a Rust project
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), BuildError> {
    let working_directory = working_directory.as_ref();

    let output = Command::new("cargo")
        .arg("build")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(BuildError::BuildFailed(format_cargo_output(&output)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_build_fails_without_cargo_project() {
        let tmp_dir = TempDir::new("test_build_no_project").unwrap();
        let result = run(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_build_passes_with_valid_code() {
        let tmp_dir = TempDir::new("test_build_valid").unwrap();

        // Create a simple Cargo project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_fails_with_invalid_code() {
        let tmp_dir = TempDir::new("test_build_invalid").unwrap();

        // Create a Cargo project with invalid code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() { this_does_not_compile }\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        if let Err(BuildError::BuildFailed(_)) = result {
            // Expected error type
        } else {
            panic!("Expected BuildFailed error");
        }
    }
}
