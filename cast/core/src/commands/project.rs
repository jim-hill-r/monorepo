use crate::command::Command;
use crate::projects;
use std::path::Path;

/// Command to create a new project
pub struct NewCommand {
    pub name: String,
}

impl Command for NewCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        projects::new(working_directory, &self.name)?;
        Ok("Creating project.".to_string())
    }
}

/// Command to list projects with changes between git refs
pub struct WithChangesCommand {
    pub base: String,
    pub head: String,
}

impl Command for WithChangesCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let changed_projects = projects::with_changes(working_directory, &self.base, &self.head)?;

        // Return newline-separated list of project paths
        let output = changed_projects
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_new_command_without_exemplars() {
        let tmp_dir = TempDir::new("test_new_project").unwrap();

        let cmd = NewCommand {
            name: "test-project".to_string(),
        };
        let result = cmd.execute(tmp_dir.path());
        // Should fail because no exemplar projects exist
        assert!(result.is_err());
    }

    #[test]
    fn test_new_command_with_exemplar() {
        let tmp_dir = TempDir::new("test_new_project_exemplar").unwrap();

        // Create an exemplar project
        let exemplar_dir = tmp_dir.path().join("exemplar");
        fs::create_dir_all(&exemplar_dir).unwrap();
        fs::write(
            exemplar_dir.join("Cast.toml"),
            "exemplar = true\nframework = \"dioxus\"",
        )
        .unwrap();
        fs::create_dir_all(exemplar_dir.join("src")).unwrap();
        fs::write(exemplar_dir.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(
            exemplar_dir.join("Cargo.toml"),
            "[package]\nname = \"TODO-CHANGE-ME\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();

        let cmd = NewCommand {
            name: "test-project".to_string(),
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Creating project.");

        // Verify the project was created
        assert!(tmp_dir.path().join("test-project").exists());
        assert!(tmp_dir.path().join("test-project/src/main.rs").exists());
        // Verify exemplar flag was removed
        let cast_toml_content =
            fs::read_to_string(tmp_dir.path().join("test-project/Cast.toml")).unwrap();
        assert!(!cast_toml_content.contains("exemplar = true"));
        // Verify package name was updated from TODO-CHANGE-ME
        let cargo_toml_content =
            fs::read_to_string(tmp_dir.path().join("test-project/Cargo.toml")).unwrap();
        assert!(cargo_toml_content.contains("name = \"test-project\""));
    }

    #[test]
    fn test_with_changes_command() {
        let tmp_dir = TempDir::new("test_with_changes").unwrap();

        // Initialize a git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create and commit initial file
        fs::write(tmp_dir.path().join("README.md"), "initial").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let base_commit = String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(tmp_dir.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        // Create a project directory with Cast.toml
        let project_dir = tmp_dir.path().join("test-project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("Cast.toml"), "").unwrap();
        fs::write(project_dir.join("README.md"), "changed").unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "add project"])
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let head_commit = String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(tmp_dir.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        let cmd = WithChangesCommand {
            base: base_commit,
            head: head_commit,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain the test-project path
        assert!(output.contains("test-project"));
    }
}
