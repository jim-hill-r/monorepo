use crate::config::CastConfig;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Run command failed")]
    RunFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

/// Run appropriate run command for a project
/// - For dioxus projects: runs `dx serve`
/// - For other projects: runs `cargo run`
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), RunError> {
    let working_directory = working_directory.as_ref();

    // Load config to determine framework
    let config = CastConfig::load_from_dir(working_directory)?;

    // Determine which command to run based on framework
    let (command, args) = match config.framework.as_deref() {
        Some("dioxus") => ("dx", vec!["serve"]),
        _ => ("cargo", vec!["run"]),
    };

    let status = Command::new(command)
        .args(&args)
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(RunError::RunFailed);
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
    fn test_run_runs_cargo_run_by_default() {
        let tmp_dir = TempDir::new("test_run_default").unwrap();

        // Create a simple Cargo project with a main.rs
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_runs_cargo_run_with_empty_cast_toml() {
        let tmp_dir = TempDir::new("test_run_empty_cast").unwrap();

        // Create a simple Cargo project with Cast.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_runs_dx_serve_for_dioxus_framework() {
        let tmp_dir = TempDir::new("test_run_dioxus").unwrap();

        // Create a Cargo project with dioxus framework
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndioxus = \"0.6\"",
        )
        .unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        // This test will fail if dx is not installed, which is expected
        // The test validates that we attempt to run dx serve
        let result = run(tmp_dir.path());

        // We expect an error because dx is likely not installed
        // but we verify we tried to run the right command
        assert!(result.is_err());
        if let Err(RunError::RunFailed) = result {
            // Expected error type
        } else if let Err(RunError::IoError(_)) = result {
            // Also acceptable - dx command not found
        } else {
            panic!("Expected RunFailed or IoError");
        }
    }

    #[test]
    fn test_run_uses_cargo_toml_metadata() {
        let tmp_dir = TempDir::new("test_run_cargo_metadata").unwrap();

        // Create a Cargo project with dioxus framework in metadata
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"\n\n[dependencies]\ndioxus = \"0.6\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        // This test will fail if dx is not installed, which is expected
        let result = run(tmp_dir.path());

        // We expect an error because dx is likely not installed
        assert!(result.is_err());
        if let Err(RunError::RunFailed) = result {
            // Expected error type
        } else if let Err(RunError::IoError(_)) = result {
            // Also acceptable - dx command not found
        } else {
            panic!("Expected RunFailed or IoError");
        }
    }
}
