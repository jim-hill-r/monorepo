use crate::config::CastConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    #[error("Failed to parse wrangler.toml: {0}")]
    WranglerTomlParseError(String),
    #[error("Missing 'name' field in wrangler.toml")]
    MissingProjectName,
    #[error("Failed to find deploy source directory")]
    DeploySourceNotFound,
}

/// Run deployment for an IAC project
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), DeployError> {
    let working_directory = working_directory.as_ref();

    // Load config to determine project type and framework
    let config = CastConfig::load_from_dir(working_directory)?;

    // Verify this is an IAC project
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
    // Check if wrangler is installed
    let wrangler_check = Command::new("wrangler")
        .arg("--version")
        .output()
        .ok();

    if wrangler_check.is_none() || !wrangler_check.unwrap().status.success() {
        return Err(DeployError::WranglerNotInstalled);
    }

    // Check for wrangler.toml
    let wrangler_toml_path = working_directory.join("wrangler.toml");
    if !wrangler_toml_path.exists() {
        return Err(DeployError::WranglerTomlNotFound);
    }

    // Parse wrangler.toml to get project name
    let wrangler_content = fs::read_to_string(&wrangler_toml_path)?;
    let wrangler_toml: toml::Value = toml::from_str(&wrangler_content)
        .map_err(|e| DeployError::WranglerTomlParseError(e.to_string()))?;

    let project_name = wrangler_toml
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(DeployError::MissingProjectName)?;

    // Find the dist directory to deploy
    let dist_dir = find_deploy_source(working_directory)?;

    // Load environment variables from .env if it exists
    let env_file = working_directory.join(".env");
    if env_file.exists() {
        load_env_file(&env_file)?;
    }

    // Run wrangler pages deploy
    let status = Command::new("wrangler")
        .arg("pages")
        .arg("deploy")
        .arg(&dist_dir)
        .arg("--project-name")
        .arg(project_name)
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(DeployError::DeployFailed);
    }

    Ok(())
}

/// Find the source directory to deploy (typically a dist directory)
fn find_deploy_source(working_directory: &Path) -> Result<PathBuf, DeployError> {
    // Look for common build output directories
    let candidate_paths = vec![
        // Check parent directory first (common pattern: iac project alongside web project)
        working_directory.parent().and_then(|p| {
            // Look for sibling projects with dist directories
            if let Ok(entries) = fs::read_dir(p) {
                for entry in entries.flatten() {
                    let dist_path = entry.path().join("dist");
                    if dist_path.exists() && dist_path.is_dir() {
                        return Some(dist_path);
                    }
                }
            }
            None
        }),
        // Check for dist in current directory
        Some(working_directory.join("dist")),
        // Check for public directory
        Some(working_directory.join("public")),
        // Check for build directory
        Some(working_directory.join("build")),
    ];

    for candidate in candidate_paths.into_iter().flatten() {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err(DeployError::DeploySourceNotFound)
}

/// Load environment variables from a .env file
fn load_env_file(env_file: &Path) -> Result<(), DeployError> {
    let content = fs::read_to_string(env_file)?;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE format
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            // Remove surrounding quotes if present
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            
            std::env::set_var(key, value);
        }
    }
    
    Ok(())
}

#[cfg(test)]
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
    fn test_find_deploy_source_finds_dist() {
        let tmp_dir = TempDir::new("test_find_dist").unwrap();
        
        // Create a dist directory
        let dist_dir = tmp_dir.path().join("dist");
        fs::create_dir_all(&dist_dir).unwrap();
        fs::write(dist_dir.join("index.html"), "<html></html>").unwrap();

        let result = find_deploy_source(tmp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("dist"));
    }

    #[test]
    fn test_find_deploy_source_finds_sibling_dist() {
        let tmp_dir = TempDir::new("test_find_sibling").unwrap();
        
        // Create sibling web project with dist
        let web_dir = tmp_dir.path().join("web");
        fs::create_dir_all(&web_dir).unwrap();
        let dist_dir = web_dir.join("dist");
        fs::create_dir_all(&dist_dir).unwrap();
        fs::write(dist_dir.join("index.html"), "<html></html>").unwrap();

        // Create iac project directory
        let iac_dir = tmp_dir.path().join("cloudflare");
        fs::create_dir_all(&iac_dir).unwrap();

        let result = find_deploy_source(&iac_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("dist"));
    }

    #[test]
    fn test_find_deploy_source_fails_without_source() {
        let tmp_dir = TempDir::new("test_no_source").unwrap();

        let result = find_deploy_source(tmp_dir.path());
        assert!(result.is_err());
        match result {
            Err(DeployError::DeploySourceNotFound) => {
                // Expected error
            }
            _ => panic!("Expected DeploySourceNotFound error"),
        }
    }

    #[test]
    fn test_load_env_file() {
        let tmp_dir = TempDir::new("test_env").unwrap();
        let env_file = tmp_dir.path().join(".env");

        fs::write(
            &env_file,
            "TEST_VAR=test_value\nANOTHER_VAR=\"quoted value\"\n# Comment\n\nEMPTY_LINE=after",
        )
        .unwrap();

        let result = load_env_file(&env_file);
        assert!(result.is_ok());
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
        assert_eq!(std::env::var("ANOTHER_VAR").unwrap(), "quoted value");
        assert_eq!(std::env::var("EMPTY_LINE").unwrap(), "after");
    }
}
