use crate::command::Command;
use crate::deploy;
use std::path::Path;

/// Command to deploy an IAC project
pub struct DeployCommand;

impl Command for DeployCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        deploy::run(working_directory)?;
        Ok("Deploy completed".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_deploy_command_fails_without_iac_project_type() {
        let tmp_dir = TempDir::new("test_deploy_command_not_iac").unwrap();

        // Create Cast.toml without project_type = "iac"
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"cloudflare-pages\"",
        )
        .unwrap();

        let cmd = DeployCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_deploy_command_fails_with_unsupported_framework() {
        let tmp_dir = TempDir::new("test_deploy_command_unsupported").unwrap();

        // Create Cast.toml with unsupported framework
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        let cmd = DeployCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_deploy_command_fails_without_wrangler_toml() {
        let tmp_dir = TempDir::new("test_deploy_command_no_wrangler").unwrap();

        // Create Cast.toml with cloudflare-pages
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();

        let cmd = DeployCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
        // Could fail with either WranglerNotInstalled or WranglerTomlNotFound
        // depending on whether wrangler is installed
    }
}
