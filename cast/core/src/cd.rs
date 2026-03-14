use crate::config::CastConfig;
use crate::deploy;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CdError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("Deploy error: {0}")]
    DeployError(#[from] deploy::DeployError),
    #[error("Git error: {0}")]
    GitError(String),
}

/// Get the root directory of the git repository
fn get_git_root(working_directory: &Path) -> Result<PathBuf, CdError> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(CdError::GitError("Failed to find git root".to_string()));
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// Get the list of files changed in the most recent commit
fn get_last_commit_files(git_root: &Path) -> Result<Vec<String>, CdError> {
    let output = Command::new("git")
        .arg("diff-tree")
        .arg("--no-commit-id")
        .arg("-r")
        .arg("HEAD")
        .arg("--name-only")
        .current_dir(git_root)
        .output()?;

    if !output.status.success() {
        return Err(CdError::GitError(
            "Failed to get last commit files".to_string(),
        ));
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    Ok(files)
}

/// Find project directories that had artifacts committed
fn find_projects_with_artifacts(files: &[String], git_root: &Path) -> Vec<PathBuf> {
    let mut project_dirs: HashSet<PathBuf> = HashSet::new();

    for file in files {
        // Look for files that contain "/artifacts/" in their path
        if let Some(artifacts_pos) = file.find("/artifacts/") {
            let project_rel_path = &file[..artifacts_pos];
            let project_dir = git_root.join(project_rel_path);
            project_dirs.insert(project_dir);
        } else if file.starts_with("artifacts/") {
            // Handle case where the artifact is at the repo root
            project_dirs.insert(git_root.to_path_buf());
        }
    }

    let mut dirs: Vec<PathBuf> = project_dirs.into_iter().collect();
    dirs.sort();
    dirs
}

/// Run CD for projects with artifacts in the most recent git commit
fn run_last_commit(working_directory: &Path) -> Result<(), CdError> {
    let git_root = get_git_root(working_directory)?;
    let files = get_last_commit_files(&git_root)?;
    let project_dirs = find_projects_with_artifacts(&files, &git_root);

    if project_dirs.is_empty() {
        println!("No artifacts found in last commit");
        return Ok(());
    }

    for project_dir in &project_dirs {
        if project_dir.join("Cast.toml").exists() {
            println!("Deploying {}", project_dir.display());
            run_deploy(project_dir)?;
            println!("✓ {} deployed successfully", project_dir.display());
        } else {
            println!("Skipping {} - no Cast.toml found", project_dir.display());
        }
    }

    Ok(())
}

/// Run continuous deployment for a project
///
/// This command:
/// 1. Runs `cast deploy` on the current project if it's an IAC project
/// 2. Runs `cast deploy` on any projects listed in the `deploys` section of the Cast config
pub fn run(working_directory: impl AsRef<Path>, last_commit: bool) -> Result<(), CdError> {
    let working_directory = working_directory.as_ref();

    if last_commit {
        return run_last_commit(working_directory);
    }

    run_deploy(working_directory)
}

fn run_deploy(working_directory: &Path) -> Result<(), CdError> {
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
        let result = run(tmp_dir.path(), false);
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
        let result = run(tmp_dir.path(), false);
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
        let result = run(tmp_dir.path(), false);
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
        let result = run(tmp_dir.path(), false);
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
        let result = run(tmp_dir.path(), false);
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
        let result = run(&web_dir, false);
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
        let result = run(&web_dir, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_projects_with_artifacts_empty() {
        let tmp_dir = TempDir::new("test_find_empty").unwrap();
        let files = vec![];
        let result = find_projects_with_artifacts(&files, tmp_dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_projects_with_artifacts_detects_nested_artifacts() {
        let tmp_dir = TempDir::new("test_find_nested").unwrap();
        let files = vec![
            "cookbook/web/artifacts/wasm/0.1.0+2025-01-01.1.abc1234.zip".to_string(),
            "cast/cast_cli/artifacts/x86_64-unknown-linux-gnu/0.1.0+2025-01-01.1.abc1234.zip"
                .to_string(),
        ];
        let result = find_projects_with_artifacts(&files, tmp_dir.path());
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .any(|p| p == &tmp_dir.path().join("cookbook/web")));
        assert!(result
            .iter()
            .any(|p| p == &tmp_dir.path().join("cast/cast_cli")));
    }

    #[test]
    fn test_find_projects_with_artifacts_detects_root_artifacts() {
        let tmp_dir = TempDir::new("test_find_root").unwrap();
        let files = vec!["artifacts/x86_64-unknown-linux-gnu/0.1.0.zip".to_string()];
        let result = find_projects_with_artifacts(&files, tmp_dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], tmp_dir.path());
    }

    #[test]
    fn test_find_projects_with_artifacts_deduplicates() {
        let tmp_dir = TempDir::new("test_find_dedup").unwrap();
        let files = vec![
            "cookbook/web/artifacts/wasm/build1.zip".to_string(),
            "cookbook/web/artifacts/wasm/build2.zip".to_string(),
        ];
        let result = find_projects_with_artifacts(&files, tmp_dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], tmp_dir.path().join("cookbook/web"));
    }

    #[test]
    fn test_find_projects_ignores_non_artifact_files() {
        let tmp_dir = TempDir::new("test_find_ignore").unwrap();
        let files = vec![
            "src/main.rs".to_string(),
            "Cargo.toml".to_string(),
            "cookbook/web/src/lib.rs".to_string(),
        ];
        let result = find_projects_with_artifacts(&files, tmp_dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_run_last_commit_with_no_artifacts_in_last_commit() {
        use std::process::Command as StdCommand;

        let tmp_dir = TempDir::new("test_cd_last_commit_no_artifacts").unwrap();

        // Initialize git repo
        StdCommand::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Commit a non-artifact file
        fs::write(tmp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();
        StdCommand::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Should succeed with no deployments triggered
        let result = run(tmp_dir.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_last_commit_deploys_project_with_artifacts() {
        use std::process::Command as StdCommand;

        let tmp_dir = TempDir::new("test_cd_last_commit_artifacts").unwrap();

        // Initialize git repo
        StdCommand::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a subdirectory project with an artifact and a Cast.toml (non-IAC)
        let project_dir = tmp_dir.path().join("myproject");
        let artifact_dir = project_dir
            .join("artifacts")
            .join("x86_64-unknown-linux-gnu");
        fs::create_dir_all(&artifact_dir).unwrap();

        fs::write(project_dir.join("Cast.toml"), "framework = \"dioxus\"").unwrap();
        fs::write(artifact_dir.join("app.zip"), "fake zip").unwrap();

        // Commit the artifact
        StdCommand::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        StdCommand::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Add artifact")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Should succeed: project has Cast.toml but no deploys/iac, so cd does nothing
        let result = run(tmp_dir.path(), true);
        assert!(result.is_ok());
    }
}
