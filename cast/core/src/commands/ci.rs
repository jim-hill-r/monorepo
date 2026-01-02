use crate::ci;
use crate::command::Command;
use std::path::Path;

/// Command to run CI checks
pub struct CiCommand {
    pub mode: ci::CiMode,
    pub recursive_depth: Option<usize>,
    pub only_changed: bool,
}

impl Command for CiCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        ci::run(
            working_directory,
            self.mode,
            self.recursive_depth,
            self.only_changed,
        )?;
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

        // Initialize git repo (required by publish which CI now runs)
        std::process::Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a minimal binary project for CI to pass (publish requires a binary)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .unwrap();

        // Commit the project (required by publish)
        std::process::Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let cmd = CiCommand {
            mode: ci::CiMode::Check,
            recursive_depth: None,
            only_changed: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok(), "CI failed: {:?}", result.err());
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
        // Write properly formatted but semantically invalid Rust code to cause CI failure
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() {\n    undefined_function();\n}\n",
        )
        .unwrap();

        let cmd = CiCommand {
            mode: ci::CiMode::Check,
            recursive_depth: None,
            only_changed: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }
}
