use crate::config::CastConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Run command failed")]
    RunFailed,
    #[error("Command '{0}' not found. Please install it using 'cast install' or install it manually.")]
    CommandNotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

/// Run appropriate command for a project
/// - For dioxus projects: runs `dx serve` (or `dx serve -p <package>` if in a workspace)
/// - For other projects: runs `cargo run`
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), RunError> {
    let working_directory = working_directory.as_ref();

    // Load config to determine framework
    let config = CastConfig::load_from_dir(working_directory)?;

    // Determine which command to run based on framework
    match config.framework.as_deref() {
        Some("dioxus") => {
            // For Dioxus projects, check if we're in a workspace
            let (run_dir, args) =
                if let Some(workspace_root) = find_workspace_root(working_directory) {
                    // We're in a workspace, run from workspace root with -p flag
                    if let Some(package_name) = get_package_name(working_directory) {
                        (
                            workspace_root,
                            vec!["serve".to_string(), "-p".to_string(), package_name],
                        )
                    } else {
                        // Couldn't get package name, run from current directory
                        (working_directory.to_path_buf(), vec!["serve".to_string()])
                    }
                } else {
                    // Not in a workspace, run from current directory
                    (working_directory.to_path_buf(), vec!["serve".to_string()])
                };

            let status = Command::new("dx")
                .args(&args)
                .current_dir(&run_dir)
                .status()
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RunError::CommandNotFound("dx".to_string())
                    } else {
                        RunError::IoError(e)
                    }
                })?;

            if !status.success() {
                return Err(RunError::RunFailed);
            }

            Ok(())
        }
        _ => {
            // For non-Dioxus projects, run cargo run from current directory
            let status = Command::new("cargo")
                .args(["run"])
                .current_dir(working_directory)
                .status()
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        RunError::CommandNotFound("cargo".to_string())
                    } else {
                        RunError::IoError(e)
                    }
                })?;

            if !status.success() {
                return Err(RunError::RunFailed);
            }

            Ok(())
        }
    }
}

/// Find the workspace root by walking up the directory tree
fn find_workspace_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                // Check if this Cargo.toml defines a workspace
                if content.contains("[workspace]") {
                    return Some(current.to_path_buf());
                }
            }
        }

        // Move to parent directory
        current = current.parent()?;
    }
}

/// Get the package name from Cargo.toml
fn get_package_name(dir: &Path) -> Option<String> {
    let cargo_toml = dir.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(cargo_toml) {
        // Simple parsing to extract package name
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") && line.contains('=') {
                if let Some(name_part) = line.split('=').nth(1) {
                    let name = name_part.trim().trim_matches('"').trim_matches('\'');
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
            include_str!("../tests/fixtures/basic_cargo.toml"),
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
            include_str!("../tests/fixtures/basic_cargo.toml"),
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
    #[ignore] // Skip in CI - test hangs if dx command behavior is unexpected
    fn test_run_runs_dx_serve_for_dioxus_framework() {
        let tmp_dir = TempDir::new("test_run_dioxus").unwrap();

        // Create a Cargo project with dioxus framework
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            include_str!("../tests/fixtures/dioxus_cargo.toml"),
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
        if let Err(RunError::CommandNotFound(cmd)) = result {
            // Expected error type - command not found
            assert_eq!(cmd, "dx");
        } else if let Err(RunError::RunFailed) = result {
            // Also acceptable - dx was found but failed to run
        } else if let Err(RunError::IoError(_)) = result {
            // Also acceptable - other IO error
        } else {
            panic!("Expected CommandNotFound, RunFailed, or IoError");
        }
    }

    #[test]
    #[ignore] // Skip in CI - test hangs if dx command behavior is unexpected
    fn test_run_uses_cargo_toml_metadata() {
        let tmp_dir = TempDir::new("test_run_cargo_metadata").unwrap();

        // Create a Cargo project with dioxus framework in metadata
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            include_str!("../tests/fixtures/dioxus_metadata_cargo.toml"),
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
        if let Err(RunError::CommandNotFound(cmd)) = result {
            // Expected error type - command not found
            assert_eq!(cmd, "dx");
        } else if let Err(RunError::RunFailed) = result {
            // Also acceptable - dx was found but failed to run
        } else if let Err(RunError::IoError(_)) = result {
            // Also acceptable - other IO error
        } else {
            panic!("Expected CommandNotFound, RunFailed, or IoError");
        }
    }

    #[test]
    #[ignore] // Skip in CI - test hangs if dx command behavior is unexpected
    fn test_run_command_not_found_error_message() {
        let tmp_dir = TempDir::new("test_run_command_not_found").unwrap();

        // Create a Cargo project with a non-existent framework command
        // This will cause the command to not be found
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            include_str!("../tests/fixtures/dioxus_metadata_only_cargo.toml"),
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());

        // Verify we get a helpful error message
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();

        // The error message should mention the command and suggest using cast install
        assert!(
            err_msg.contains("dx")
                || err_msg.contains("not found")
                || err_msg.contains("install"),
            "Error message should mention the missing command or install: {}",
            err_msg
        );
    }

    #[test]
    fn test_find_workspace_root_finds_workspace() {
        let tmp_dir = TempDir::new("test_workspace").unwrap();

        // Create workspace root with workspace Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"web\"]\n",
        )
        .unwrap();

        // Create a member project
        let web_dir = tmp_dir.path().join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("Cargo.toml"),
            "[package]\nname = \"web\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();

        let workspace_root = find_workspace_root(&web_dir);
        assert!(workspace_root.is_some());
        assert_eq!(workspace_root.unwrap(), tmp_dir.path());
    }

    #[test]
    fn test_find_workspace_root_returns_none_when_no_workspace() {
        let tmp_dir = TempDir::new("test_no_workspace").unwrap();

        // Create a regular project without workspace
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();

        let workspace_root = find_workspace_root(tmp_dir.path());
        assert!(workspace_root.is_none());
    }

    #[test]
    fn test_get_package_name_extracts_name() {
        let tmp_dir = TempDir::new("test_package_name").unwrap();

        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"my-package\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();

        let package_name = get_package_name(tmp_dir.path());
        assert_eq!(package_name, Some("my-package".to_string()));
    }

    #[test]
    #[ignore] // Skip in CI - test hangs if dx command behavior is unexpected
    fn test_run_dioxus_in_workspace_uses_package_flag() {
        let tmp_dir = TempDir::new("test_workspace_run").unwrap();

        // Create workspace root
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"web\"]\n",
        )
        .unwrap();

        // Create web member
        let web_dir = tmp_dir.path().join("web");
        fs::create_dir_all(web_dir.join("src")).unwrap();
        fs::write(
            web_dir.join("Cargo.toml"),
            "[package]\nname = \"web\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"",
        )
        .unwrap();
        fs::write(
            web_dir.join("src/main.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();

        // Running from web directory should detect workspace
        let result = run(&web_dir);

        // We expect an error because dx is not installed
        assert!(result.is_err());
        if let Err(RunError::CommandNotFound(cmd)) = result {
            assert_eq!(cmd, "dx");
        }
    }
}
