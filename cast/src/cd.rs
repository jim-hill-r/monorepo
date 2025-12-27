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
    #[error("Failed to find monorepo root")]
    MonorepoRootNotFound,
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
    if config.project_type == Some("iac".to_string()) {
        deploy::run(working_directory)?;
    }

    // Deploy any projects listed in the deploys section
    if let Some(deploys) = config.deploys {
        let monorepo_root = find_monorepo_root(working_directory)?;

        for deploy_project in deploys {
            let deploy_project_path = monorepo_root.join(&deploy_project);

            // Only deploy if the project directory exists
            if deploy_project_path.exists() {
                deploy::run(&deploy_project_path)?;
            }
        }
    }

    Ok(())
}

/// Find the monorepo root by walking up the directory tree looking for a .git directory
fn find_monorepo_root(working_directory: &Path) -> Result<&Path, CdError> {
    let mut current = Some(working_directory);

    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Ok(dir);
        }
        current = dir.parent();
    }

    Err(CdError::MonorepoRootNotFound)
}

#[cfg(test)]
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
    fn test_find_monorepo_root_finds_git_directory() {
        let tmp_dir = TempDir::new("test_find_root").unwrap();

        // Create .git directory
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        // Create nested directories
        let nested_dir = tmp_dir.path().join("level1").join("level2").join("level3");
        fs::create_dir_all(&nested_dir).unwrap();

        // Should find the root from nested directory
        let root = find_monorepo_root(&nested_dir).unwrap();
        assert_eq!(root, tmp_dir.path());
    }

    #[test]
    fn test_find_monorepo_root_fails_without_git() {
        let tmp_dir = TempDir::new("test_no_git").unwrap();

        // Don't create .git directory
        let result = find_monorepo_root(tmp_dir.path());
        assert!(result.is_err());
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
}
