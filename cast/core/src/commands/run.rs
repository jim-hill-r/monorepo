use crate::command::Command;
use crate::run;
use std::path::Path;

/// Command to run a development server
/// - For dioxus projects: runs `dx serve`
/// - For other projects: runs `cargo run`
pub struct RunCommand;

impl Command for RunCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        run::run(working_directory)?;
        Ok("Server started".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_command_success() {
        let tmp_dir = TempDir::new("test_run_command").unwrap();

        // Create a minimal Cargo.toml and src/main.rs for run to pass
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

        let cmd = RunCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Server started");
    }

    #[test]
    fn test_run_command_failure() {
        let tmp_dir = TempDir::new("test_run_command_fail").unwrap();

        let cmd = RunCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_command_with_dioxus_framework() {
        let tmp_dir = TempDir::new("test_run_command_dioxus").unwrap();

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
        let cmd = RunCommand;
        let result = cmd.execute(tmp_dir.path());

        // We expect an error because dx is likely not installed
        assert!(result.is_err());
    }
}
