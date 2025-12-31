use crate::config::CastConfig;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeployError {
    #[error("Deploy failed")]
    DeployFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("Not an IAC project - missing project_type = \"iac\" in Cast configuration")]
    NotIacProject,
    #[error("Unsupported framework: {0}")]
    UnsupportedFramework(String),
    #[error("wrangler.toml not found")]
    WranglerTomlNotFound,
    #[error("wrangler not installed - install it with: npm install -g wrangler")]
    WranglerNotInstalled,
    #[error("Failed to parse .env file: {0}")]
    EnvFileParseError(String),
}

/// Run deployment for an IAC project or a project with deploy dependencies
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), DeployError> {
    let working_directory = working_directory.as_ref();

    // Load config to determine project type and framework
    let config = CastConfig::load_from_dir(working_directory)?;

    // Check if this project has deploys configured
    if let Some(deploys) = &config.deploys {
        // Deploy all listed deploy projects
        for deploy_path in deploys {
            let deploy_dir = working_directory.join(deploy_path);
            println!("Deploying: {}", deploy_dir.display());

            // Recursively call run on each deploy project
            run(&deploy_dir)?;
        }
        return Ok(());
    }

    // If no deploys field, verify this is an IAC project
    if config.project_type != Some("iac".to_string()) {
        return Err(DeployError::NotIacProject);
    }

    // Determine deployment strategy based on framework
    match config.framework.as_deref() {
        Some("cloudflare-pages") => deploy_cloudflare_pages(working_directory),
        Some(framework) => Err(DeployError::UnsupportedFramework(framework.to_string())),
        None => Err(DeployError::UnsupportedFramework("none".to_string())),
    }
}

/// Deploy to Cloudflare Pages
fn deploy_cloudflare_pages(working_directory: &Path) -> Result<(), DeployError> {
    println!("Deploying to Cloudflare Pages...");

    // Check if wrangler is installed
    let wrangler_installed = Command::new("wrangler")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !wrangler_installed {
        return Err(DeployError::WranglerNotInstalled);
    }

    // Check for wrangler.toml
    let wrangler_toml_path = working_directory.join("wrangler.toml");
    if !wrangler_toml_path.exists() {
        return Err(DeployError::WranglerTomlNotFound);
    }

    // Load environment variables from .env if it exists
    let env_vars = load_env_file(working_directory)?;

    println!("Running: wrangler pages deploy");
    // Run wrangler pages deploy with inherited stdio and environment variables
    let mut cmd = Command::new("wrangler");
    cmd.arg("pages")
        .arg("deploy")
        .current_dir(working_directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Add environment variables from .env file to the command
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(DeployError::DeployFailed);
    }

    Ok(())
}

/// Load environment variables from a .env file using dotenvy
fn load_env_file(working_directory: &Path) -> Result<HashMap<String, String>, DeployError> {
    let env_file = working_directory.join(".env");

    if !env_file.exists() {
        return Ok(HashMap::new());
    }

    // Use dotenvy to parse the .env file properly
    let env_vars = dotenvy::from_path_iter(&env_file)
        .map_err(|e| DeployError::EnvFileParseError(e.to_string()))?
        .collect::<Result<HashMap<String, String>, _>>()
        .map_err(|e| DeployError::EnvFileParseError(e.to_string()))?;

    Ok(env_vars)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_deploy_fails_without_iac_project_type() {
        let tmp_dir = TempDir::new("test_deploy_not_iac").unwrap();

        // Create Cast.toml without project_type = "iac"
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"cloudflare-pages\"",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        match result {
            Err(DeployError::NotIacProject) => {
                // Expected error
            }
            _ => panic!("Expected NotIacProject error"),
        }
    }

    #[test]
    fn test_deploy_fails_with_unsupported_framework() {
        let tmp_dir = TempDir::new("test_deploy_unsupported").unwrap();

        // Create Cast.toml with unsupported framework
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"unsupported\"",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        match result {
            Err(DeployError::UnsupportedFramework(_)) => {
                // Expected error
            }
            _ => panic!("Expected UnsupportedFramework error"),
        }
    }

    #[test]
    fn test_deploy_cloudflare_fails_without_wrangler_toml() {
        let tmp_dir = TempDir::new("test_deploy_no_wrangler").unwrap();

        // Create Cast.toml with cloudflare-pages
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        // Could fail with either WranglerNotInstalled or WranglerTomlNotFound
        // depending on whether wrangler is installed
    }

    #[test]
    fn test_load_env_file_with_valid_content() {
        let tmp_dir = TempDir::new("test_env").unwrap();
        let env_file = tmp_dir.path().join(".env");

        fs::write(
            &env_file,
            "TEST_VAR=test_value\nANOTHER_VAR=\"quoted value\"\nSINGLE_QUOTED='single value'\n# Comment\n\nEMPTY_LINE=after",
        )
        .unwrap();

        let result = load_env_file(tmp_dir.path());
        assert!(result.is_ok());
        let env_vars = result.unwrap();
        assert_eq!(env_vars.get("TEST_VAR").unwrap(), "test_value");
        assert_eq!(env_vars.get("ANOTHER_VAR").unwrap(), "quoted value");
        assert_eq!(env_vars.get("SINGLE_QUOTED").unwrap(), "single value");
        assert_eq!(env_vars.get("EMPTY_LINE").unwrap(), "after");
    }

    #[test]
    fn test_load_env_file_with_escaped_characters() {
        let tmp_dir = TempDir::new("test_env_escaped").unwrap();
        let env_file = tmp_dir.path().join(".env");

        // Test with properly escaped content that dotenvy can handle
        fs::write(
            &env_file,
            "ESCAPED_VALUE=\"value with spaces\"\nMULTI_WORD='hello world'\n",
        )
        .unwrap();

        let result = load_env_file(tmp_dir.path());
        assert!(result.is_ok());
        let env_vars = result.unwrap();

        // dotenvy properly handles quoted values
        assert_eq!(env_vars.get("ESCAPED_VALUE").unwrap(), "value with spaces");
        assert_eq!(env_vars.get("MULTI_WORD").unwrap(), "hello world");
    }

    #[test]
    fn test_load_env_file_returns_empty_when_file_missing() {
        let tmp_dir = TempDir::new("test_no_env").unwrap();

        let result = load_env_file(tmp_dir.path());
        assert!(result.is_ok());
        let env_vars = result.unwrap();
        assert!(env_vars.is_empty());
    }

    #[test]
    fn test_env_vars_not_set_globally() {
        let tmp_dir = TempDir::new("test_env_isolation").unwrap();
        let env_file = tmp_dir.path().join(".env");

        let test_key = "CAST_DEPLOY_TEST_VAR_ISOLATION";
        fs::write(&env_file, format!("{}=test_value", test_key)).unwrap();

        // Load env vars (should not set them globally)
        let result = load_env_file(tmp_dir.path());
        assert!(result.is_ok());

        // Verify the variable is NOT set globally
        assert!(std::env::var(test_key).is_err());
    }

    #[test]
    fn test_deploy_succeeds_with_deploys_field_single_project() {
        let tmp_dir = TempDir::new("test_deploy_with_deploys").unwrap();

        // Create a non-IAC project with deploys field
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"static_website\"\nframework = \"dioxus\"\ndeploys = [\"../iac\"]",
        )
        .unwrap();

        // Create the IAC project directory
        let iac_dir = tmp_dir.path().join("../iac");
        fs::create_dir_all(&iac_dir).unwrap();
        fs::write(
            iac_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();
        fs::write(iac_dir.join("wrangler.toml"), "name = \"test\"\n").unwrap();

        // This should succeed if wrangler is installed, or fail with WranglerNotInstalled
        let result = run(tmp_dir.path());
        // We can't guarantee wrangler is installed in test environment,
        // so we just verify it attempts to deploy the IAC project
        // by checking it doesn't error with NotIacProject
        if let Err(e) = result {
            // These are acceptable errors in test environment
            assert!(
                matches!(e, DeployError::WranglerNotInstalled)
                    || matches!(e, DeployError::DeployFailed),
                "Expected WranglerNotInstalled or DeployFailed, got: {:?}",
                e
            );
        }
    }

    #[test]
    fn test_deploy_succeeds_with_deploys_field_multiple_projects() {
        let tmp_dir = TempDir::new("test_deploy_multiple").unwrap();

        // Create a non-IAC project with multiple deploys
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"static_website\"\nframework = \"dioxus\"\ndeploys = [\"../iac1\", \"../iac2\"]",
        )
        .unwrap();

        // Create the first IAC project directory
        let iac1_dir = tmp_dir.path().join("../iac1");
        fs::create_dir_all(&iac1_dir).unwrap();
        fs::write(
            iac1_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();
        fs::write(iac1_dir.join("wrangler.toml"), "name = \"test1\"\n").unwrap();

        // Create the second IAC project directory
        let iac2_dir = tmp_dir.path().join("../iac2");
        fs::create_dir_all(&iac2_dir).unwrap();
        fs::write(
            iac2_dir.join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();
        fs::write(iac2_dir.join("wrangler.toml"), "name = \"test2\"\n").unwrap();

        // This should attempt to deploy both IAC projects
        let result = run(tmp_dir.path());
        // Similar to above, verify it doesn't fail with NotIacProject
        if let Err(e) = result {
            assert!(
                matches!(e, DeployError::WranglerNotInstalled)
                    || matches!(e, DeployError::DeployFailed),
                "Expected WranglerNotInstalled or DeployFailed, got: {:?}",
                e
            );
        }
    }

    #[test]
    fn test_deploy_without_iac_project_and_without_deploys_fails() {
        let tmp_dir = TempDir::new("test_deploy_no_deploys").unwrap();

        // Create a non-IAC project without deploys field
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"static_website\"\nframework = \"dioxus\"",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        match result {
            Err(DeployError::NotIacProject) => {
                // Expected error
            }
            _ => panic!("Expected NotIacProject error"),
        }
    }
}
