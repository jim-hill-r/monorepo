use crate::cd;
use crate::command::Command;
use std::path::Path;

/// Command to run continuous deployment
pub struct CdCommand;

impl Command for CdCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        cd::run(working_directory)?;
        Ok("CD completed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_cd_command_does_nothing_for_non_iac_project_without_deploys() {
        let tmp_dir = TempDir::new("test_cd_command_no_deploy").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create Cast.toml without IAC project type or deploys
        fs::write(tmp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let cmd = CdCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CD completed");
    }

    #[test]
    fn test_cd_command_fails_when_deploying_iac_project() {
        let tmp_dir = TempDir::new("test_cd_command_iac").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create Cast.toml with IAC project type but unsupported framework
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        let cmd = CdCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_command_skips_non_existent_deploy_projects() {
        let tmp_dir = TempDir::new("test_cd_command_skip_missing").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create main project with deploys list pointing to non-existent project
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"dioxus\"\ndeploys = [\"non-existent-project\"]",
        )
        .unwrap();

        let cmd = CdCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CD completed");
    }
}
