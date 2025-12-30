use crate::ci;
use crate::command::Command;
use std::path::Path;

/// Command to run CI checks
pub struct CiCommand;

impl Command for CiCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        ci::run(working_directory)?;
        Ok("CI passed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_ci_command_success() {
        let tmp_dir = TempDir::new("test_ci_command").unwrap();

        // Create a minimal Cargo.toml and src/lib.rs for CI to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let cmd = CiCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CI passed");
    }

    #[test]
    fn test_ci_command_failure() {
        let tmp_dir = TempDir::new("test_ci_command_fail").unwrap();

        // Create a Cargo.toml with invalid syntax to cause CI failure
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        // Write invalid Rust code to cause CI failure
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() { this is not valid rust code }",
        )
        .unwrap();

        let cmd = CiCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }
}
