use crate::command::Command;
use crate::test;
use std::path::Path;

/// Command to run tests for a Rust project
pub struct TestCommand;

impl Command for TestCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        test::run(working_directory)?;
        Ok("Tests passed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_test_command_success() {
        let tmp_dir = TempDir::new("test_test_command").unwrap();

        // Create a minimal Cargo.toml and src/lib.rs with a passing test
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        assert_eq!(2 + 2, 4);\n    }\n}",
        )
        .unwrap();

        let cmd = TestCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Tests passed");
    }

    #[test]
    fn test_test_command_failure() {
        let tmp_dir = TempDir::new("test_test_command_fail").unwrap();

        let cmd = TestCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }
}
