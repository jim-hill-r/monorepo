use crate::command::Command;
use crate::publish;
use std::path::Path;

/// Command to publish release artifacts
pub struct PublishCommand;

impl Command for PublishCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        publish::run(working_directory)?;
        Ok("Publish completed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_publish_command_fails_without_cargo_project() {
        let tmp_dir = TempDir::new("test_publish_command_no_project").unwrap();

        let cmd = PublishCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_command_creates_artifacts_directory() {
        use std::process::Command as StdCommand;

        let tmp_dir = TempDir::new("test_publish_command_artifacts").unwrap();

        // Initialize git repository (required for generate_bundle_filename)
        StdCommand::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a minimal binary project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test_binary"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        // Commit to have a valid git SHA
        StdCommand::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let cmd = PublishCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok(), "Publish failed: {:?}", result.err());
        assert_eq!(result.unwrap(), "Publish completed");

        // Check that artifacts directory was created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(artifacts_dir.exists());

        // Get target triple
        let target_triple = publish::get_target_triple().unwrap();
        let target_artifacts_dir = artifacts_dir.join(&target_triple);
        assert!(
            target_artifacts_dir.exists(),
            "Target triple subdirectory should exist"
        );

        // Check that a zip file was created in the target directory
        let entries = fs::read_dir(&target_artifacts_dir).unwrap();
        let zip_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "zip")
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(
            zip_files.len(),
            1,
            "Expected exactly one zip file in artifacts target directory"
        );
    }

    #[test]
    fn test_publish_command_fails_with_invalid_code() {
        let tmp_dir = TempDir::new("test_publish_command_invalid").unwrap();

        // Create a project with invalid code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test_invalid"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { this_does_not_compile }\n",
        )
        .unwrap();

        let cmd = PublishCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }
}
