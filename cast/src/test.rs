use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Cargo test failed")]
    TestFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Run cargo test for a Rust project
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), TestError> {
    let working_directory = working_directory.as_ref();

    let status = Command::new("cargo")
        .arg("test")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(TestError::TestFailed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_fails_without_cargo_project() {
        let tmp_dir = TempDir::new("test_run_no_project").unwrap();
        let result = run(tmp_dir.path());
        assert!(result.is_err());
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
}
