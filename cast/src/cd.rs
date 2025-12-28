use crate::config::CastConfig;
use crate::deploy;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CdError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("Deploy error: {0}")]
    DeployError(#[from] deploy::DeployError),
}

/// Run continuous deployment for a project
///
/// This command:
/// 1. Runs `cast deploy` on the current project if it's an IAC project
/// 2. Runs `cast deploy` on any projects listed in the `deploys` section of the Cast config
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CdError> {
    let working_directory = working_directory.as_ref();

    // Load config to check if current project is IAC and get deploys list
    let config = CastConfig::load_from_dir(working_directory)?;

    // If current project is IAC, deploy it
    if config.project_type.as_deref() == Some("iac") {
        println!("Deploying current project: {}", working_directory.display());
        deploy::run(working_directory)?;
        println!("✓ Current project deployed successfully");
    }

    // Deploy any projects listed in the deploys section
    if let Some(deploys) = config.deploys {
        for deploy_project in deploys {
            // Resolve deploy paths relative to the working directory
            let deploy_project_path = working_directory.join(&deploy_project);

            // Only deploy if the project directory exists
            if deploy_project_path.exists() {
                println!("Deploying {}", deploy_project_path.display());
                deploy::run(&deploy_project_path)?;
                println!("✓ {} deployed successfully", deploy_project_path.display());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_cd_does_nothing_for_non_iac_project_without_deploys() {
        let tmp_dir = TempDir::new("test_cd_no_deploy").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create Cast.toml without IAC project type or deploys
        fs::write(tmp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        // Should succeed without doing anything
        let result = run(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cd_deploys_iac_project() {
        let tmp_dir = TempDir::new("test_cd_iac").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create Cast.toml with IAC project type but unsupported framework
        // (This will cause deploy to fail, which is expected)
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        // Should fail because it tries to deploy unsupported framework
        let result = run(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_deploys_projects_in_deploys_list() {
        let tmp_dir = TempDir::new("test_cd_deploys_list").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create main project with deploys list
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"dioxus\"\ndeploys = [\"deploy-project\"]",
        )
        .unwrap();

        // Create deploy project directory with IAC config
        let deploy_dir = tmp_dir.path().join("deploy-project");
        fs::create_dir(&deploy_dir).unwrap();
        fs::write(
            deploy_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        // Should fail because deploy project has unsupported framework
        let result = run(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_skips_non_existent_deploy_projects() {
        let tmp_dir = TempDir::new("test_cd_skip_missing").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create main project with deploys list pointing to non-existent project
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"dioxus\"\ndeploys = [\"non-existent-project\"]",
        )
        .unwrap();

        // Should succeed by skipping the non-existent project
        let result = run(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cd_deploys_both_current_and_deploys_list() {
        let tmp_dir = TempDir::new("test_cd_both").unwrap();

        // Create a .git directory to mark as monorepo root
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create IAC project with deploys list
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"\ndeploys = [\"another-deploy\"]",
        )
        .unwrap();

        // Create another deploy project
        let deploy_dir = tmp_dir.path().join("another-deploy");
        fs::create_dir(&deploy_dir).unwrap();
        fs::write(
            deploy_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        // Should fail when trying to deploy the current project (first)
        let result = run(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_deploys_relative_path_with_parent_directory() {
        let tmp_dir = TempDir::new("test_cd_relative").unwrap();

        // Create structure: root/project/web and root/project/deploy
        let project_dir = tmp_dir.path().join("project");
        let web_dir = project_dir.join("web");
        let deploy_dir = project_dir.join("deploy");

        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&deploy_dir).unwrap();

        // Create web project with relative deploy path using ..
        fs::write(
            web_dir.join("Cast.toml"),
            "framework = \"dioxus\"\ndeploys = [\"../deploy\"]",
        )
        .unwrap();

        // Create deploy project with unsupported framework (will fail)
        fs::write(
            deploy_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        // Should fail when trying to deploy the '../deploy' directory (proves path resolution works)
        let result = run(&web_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_deploys_relative_subdirectory() {
        let tmp_dir = TempDir::new("test_cd_subdir").unwrap();

        // Create structure: root/web and root/web/deploy
        let web_dir = tmp_dir.path().join("web");
        let deploy_dir = web_dir.join("deploy");

        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&deploy_dir).unwrap();

        // Create web project with relative deploy path to subdirectory
        fs::write(
            web_dir.join("Cast.toml"),
            "framework = \"dioxus\"\ndeploys = [\"deploy\"]",
        )
        .unwrap();

        // Create deploy project with unsupported framework (will fail)
        fs::write(
            deploy_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        // Should fail when trying to deploy the 'deploy' subdirectory (proves path resolution works)
        let result = run(&web_dir);
        assert!(result.is_err());
    }
}
