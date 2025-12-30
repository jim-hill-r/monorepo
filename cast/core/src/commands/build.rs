use crate::build;
use crate::command::Command;
use std::path::Path;

/// Command to build a Rust project
pub struct BuildCommand;

impl Command for BuildCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        build::run(working_directory)?;
        Ok("Build passed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_build_command_success() {
        let tmp_dir = TempDir::new("test_build_command").unwrap();

        // Create a minimal Cargo.toml and src/lib.rs for build to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let cmd = BuildCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Build passed");
    }

    #[test]
    fn test_build_command_failure() {
        let tmp_dir = TempDir::new("test_build_command_fail").unwrap();

        let cmd = BuildCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }
}
