use crate::command::Command;
use crate::test;
use std::path::Path;

/// Command to run tests for a project (Rust and/or npm tests)
pub struct TestCommand {
    pub coverage: bool,
}

impl Command for TestCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        test::run(working_directory, self.coverage, false)?;
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

        let cmd = TestCommand { coverage: false };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Tests passed");
    }

    #[test]
    fn test_test_command_failure() {
        let tmp_dir = TempDir::new("test_test_command_fail").unwrap();

        // Create a Cargo project with a failing test
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_fails() {\n        assert_eq!(2 + 2, 5); // This will fail\n    }\n}",
        )
        .unwrap();

        let cmd = TestCommand { coverage: false };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_test_command_succeeds_without_project() {
        let tmp_dir = TempDir::new("test_test_command_empty").unwrap();

        let cmd = TestCommand { coverage: false };
        let result = cmd.execute(tmp_dir.path());
        // Should succeed silently when there are no tests to run
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Tests passed");
    }
}
