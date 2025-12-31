use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

use crate::config::CastConfig;

#[derive(Error, Debug)]
pub enum ToolchainError {
    #[error("Tool detection failed: {0}")]
    DetectionError(String),
    #[error("Tool installation failed: {0}")]
    InstallationError(String),
    #[error("Cast configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Represents a tool that can be managed by the toolchain command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tool {
    /// Rustup toolchain installer
    Rustup,
    /// Rust compiler
    Rustc,
    /// Cargo package manager
    Cargo,
    /// Rust formatter
    Rustfmt,
    /// Rust linter
    Clippy,
    /// Cast CLI tool
    Cast,
    /// Dioxus CLI tool
    Dx,
    /// Node.js runtime
    Node,
    /// Node package manager
    Npm,
    /// Playwright testing framework
    Playwright,
    /// Cloudflare Wrangler CLI
    Wrangler,
    /// Git Large File Storage
    GitLfs,
}

impl Tool {
    /// Get the display name for a tool
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Rustup => "rustup",
            Tool::Rustc => "rustc",
            Tool::Cargo => "cargo",
            Tool::Rustfmt => "rustfmt",
            Tool::Clippy => "clippy",
            Tool::Cast => "cast",
            Tool::Dx => "dx",
            Tool::Node => "node",
            Tool::Npm => "npm",
            Tool::Playwright => "playwright",
            Tool::Wrangler => "wrangler",
            Tool::GitLfs => "git-lfs",
        }
    }

    /// Parse a tool name from a string
    pub fn from_name(name: &str) -> Option<Tool> {
        match name.to_lowercase().as_str() {
            "rustup" => Some(Tool::Rustup),
            "rustc" => Some(Tool::Rustc),
            "cargo" => Some(Tool::Cargo),
            "rustfmt" => Some(Tool::Rustfmt),
            "clippy" => Some(Tool::Clippy),
            "cast" => Some(Tool::Cast),
            "dx" | "dioxus" => Some(Tool::Dx),
            "node" | "nodejs" => Some(Tool::Node),
            "npm" => Some(Tool::Npm),
            "playwright" => Some(Tool::Playwright),
            "wrangler" => Some(Tool::Wrangler),
            "git-lfs" | "gitlfs" | "lfs" => Some(Tool::GitLfs),
            _ => None,
        }
    }
}

/// Represents the installation status and version of a tool
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub tool: Tool,
    pub installed: bool,
    pub version: Option<String>,
}

/// Determine required tools based on project configuration
pub fn detect_required_tools(
    working_directory: impl AsRef<Path>,
) -> Result<Vec<Tool>, ToolchainError> {
    let working_directory = working_directory.as_ref();

    // Load Cast configuration
    let config = CastConfig::load_from_dir(working_directory).map_err(|e| {
        ToolchainError::ConfigError(format!("Failed to load Cast configuration: {}", e))
    })?;

    // Use a HashSet to ensure tools are unique
    let mut tools = HashSet::new();

    // Always include rustup (required to install other Rust tools)
    tools.insert(Tool::Rustup);

    // Always include base Rust tools
    tools.insert(Tool::Rustc);
    tools.insert(Tool::Cargo);
    tools.insert(Tool::Rustfmt);
    tools.insert(Tool::Clippy);

    // Always include git-lfs (required for repository operations with large files)
    tools.insert(Tool::GitLfs);

    // Always include cast (required for running dev servers and builds)
    tools.insert(Tool::Cast);

    // Add framework-specific tools
    if let Some(framework) = &config.framework {
        match framework.as_str() {
            "dioxus" => {
                tools.insert(Tool::Dx);
                tools.insert(Tool::Node);
                tools.insert(Tool::Npm);
                tools.insert(Tool::Playwright);
            }
            "cloudflare-pages" => {
                tools.insert(Tool::Wrangler);
                tools.insert(Tool::Node);
                tools.insert(Tool::Npm);
            }
            _ => {
                // Unknown framework - log but continue with defaults
            }
        }
    }

    // Check for additional files that indicate tool requirements

    // Check for package.json - indicates Node.js/npm requirement
    if working_directory.join("package.json").exists() {
        tools.insert(Tool::Node);
        tools.insert(Tool::Npm);
    }

    // Check for playwright config files - indicates Playwright requirement
    if working_directory.join("playwright.config.ts").exists()
        || working_directory.join("playwright.config.js").exists()
    {
        tools.insert(Tool::Playwright);
    }

    // Check for wrangler.toml - indicates Wrangler requirement
    if working_directory.join("wrangler.toml").exists() {
        tools.insert(Tool::Wrangler);
    }

    // Convert HashSet to Vec and sort for consistent ordering
    let mut tools_vec: Vec<Tool> = tools.into_iter().collect();
    tools_vec.sort_by_key(|tool| tool.name());

    Ok(tools_vec)
}

/// Check if a tool is installed and get its version
pub fn check_tool(tool: &Tool) -> Result<ToolStatus, ToolchainError> {
    use std::process::Command;

    // Playwright needs special handling to verify chromium browsers are installed
    if matches!(tool, Tool::Playwright) {
        return check_playwright_with_browsers();
    }

    let (command, args) = match tool {
        Tool::Rustup => ("rustup", vec!["--version"]),
        Tool::Rustc => ("rustc", vec!["--version"]),
        Tool::Cargo => ("cargo", vec!["--version"]),
        Tool::Rustfmt => ("rustfmt", vec!["--version"]),
        Tool::Clippy => ("cargo", vec!["clippy", "--version"]),
        Tool::Cast => ("cast", vec!["--version"]),
        Tool::Dx => ("dx", vec!["--version"]),
        Tool::Node => ("node", vec!["--version"]),
        Tool::Npm => ("npm", vec!["--version"]),
        Tool::Playwright => ("npx", vec!["playwright", "--version"]),
        Tool::Wrangler => ("wrangler", vec!["--version"]),
        Tool::GitLfs => ("git", vec!["lfs", "version"]),
    };

    // Try to run the command
    let output_result = Command::new(command).args(&args).output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                // Parse version from output
                let version_output = String::from_utf8_lossy(&output.stdout);
                let version_str = version_output.trim();

                // Extract just the version number if possible
                let version = parse_version_string(version_str);

                Ok(ToolStatus {
                    tool: tool.clone(),
                    installed: true,
                    version: Some(version),
                })
            } else {
                // Command ran but failed (tool might be installed but broken)
                Ok(ToolStatus {
                    tool: tool.clone(),
                    installed: false,
                    version: None,
                })
            }
        }
        Err(_) => {
            // Command not found
            Ok(ToolStatus {
                tool: tool.clone(),
                installed: false,
                version: None,
            })
        }
    }
}

/// Check if Playwright is installed with chromium browsers
fn check_playwright_with_browsers() -> Result<ToolStatus, ToolchainError> {
    use std::process::Command;

    // First check if playwright npm package is available
    let version_check = Command::new("npx")
        .args(["playwright", "--version"])
        .output();

    let version = match version_check {
        Ok(output) if output.status.success() => {
            let version_output = String::from_utf8_lossy(&output.stdout);
            Some(parse_version_string(version_output.trim()))
        }
        _ => {
            // Playwright npm package not installed
            return Ok(ToolStatus {
                tool: Tool::Playwright,
                installed: false,
                version: None,
            });
        }
    };

    // Now check if chromium browsers are installed
    let browser_check = Command::new("npx")
        .args(["playwright", "install", "--list"])
        .output();

    match browser_check {
        Ok(output) if output.status.success() => {
            // Parse the output to check if chromium is installed
            let list_output = String::from_utf8_lossy(&output.stdout);
            let has_chromium = list_output.contains("chromium");

            if has_chromium {
                Ok(ToolStatus {
                    tool: Tool::Playwright,
                    installed: true,
                    version,
                })
            } else {
                // Playwright installed but chromium browser not installed
                Ok(ToolStatus {
                    tool: Tool::Playwright,
                    installed: false,
                    version,
                })
            }
        }
        _ => {
            // Failed to check browser list - treat as not installed
            Ok(ToolStatus {
                tool: Tool::Playwright,
                installed: false,
                version,
            })
        }
    }
}

/// Parse version string from tool output
/// This extracts the version number from various tool output formats
fn parse_version_string(output: &str) -> String {
    // Try to extract version number patterns like "1.2.3" or "v1.2.3"
    // Handle different tool output formats

    let output = output.trim();

    // For tools that output just the version (like npm, node)
    if output.starts_with('v')
        || output
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        // If it's just "v1.2.3" or "1.2.3", return as is
        if output.split_whitespace().count() == 1 {
            return output.to_string();
        }
    }

    // For tools with more complex output (like rustc, cargo)
    // Look for version-like patterns after the tool name
    for word in output.split_whitespace() {
        // Check if this word looks like a version number
        if word.contains('.')
            && (word.starts_with('v')
                || word
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false))
        {
            return word.to_string();
        }
    }

    // If we can't parse it, return the full output
    output.to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        assert_eq!(Tool::Rustup.name(), "rustup");
        assert_eq!(Tool::Rustc.name(), "rustc");
        assert_eq!(Tool::Cargo.name(), "cargo");
        assert_eq!(Tool::Rustfmt.name(), "rustfmt");
        assert_eq!(Tool::Clippy.name(), "clippy");
        assert_eq!(Tool::Cast.name(), "cast");
        assert_eq!(Tool::Dx.name(), "dx");
        assert_eq!(Tool::Node.name(), "node");
        assert_eq!(Tool::Npm.name(), "npm");
        assert_eq!(Tool::Playwright.name(), "playwright");
        assert_eq!(Tool::Wrangler.name(), "wrangler");
        assert_eq!(Tool::GitLfs.name(), "git-lfs");
    }

    #[test]
    fn test_tool_from_name() {
        assert_eq!(Tool::from_name("rustup"), Some(Tool::Rustup));
        assert_eq!(Tool::from_name("rustc"), Some(Tool::Rustc));
        assert_eq!(Tool::from_name("cargo"), Some(Tool::Cargo));
        assert_eq!(Tool::from_name("rustfmt"), Some(Tool::Rustfmt));
        assert_eq!(Tool::from_name("clippy"), Some(Tool::Clippy));
        assert_eq!(Tool::from_name("cast"), Some(Tool::Cast));
        assert_eq!(Tool::from_name("dx"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("dioxus"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("node"), Some(Tool::Node));
        assert_eq!(Tool::from_name("nodejs"), Some(Tool::Node));
        assert_eq!(Tool::from_name("npm"), Some(Tool::Npm));
        assert_eq!(Tool::from_name("playwright"), Some(Tool::Playwright));
        assert_eq!(Tool::from_name("wrangler"), Some(Tool::Wrangler));
        assert_eq!(Tool::from_name("git-lfs"), Some(Tool::GitLfs));
        assert_eq!(Tool::from_name("gitlfs"), Some(Tool::GitLfs));
        assert_eq!(Tool::from_name("lfs"), Some(Tool::GitLfs));
        assert_eq!(Tool::from_name("unknown"), None);
    }

    #[test]
    fn test_tool_from_name_case_insensitive() {
        assert_eq!(Tool::from_name("RUSTUP"), Some(Tool::Rustup));
        assert_eq!(Tool::from_name("RUSTC"), Some(Tool::Rustc));
        assert_eq!(Tool::from_name("Cargo"), Some(Tool::Cargo));
        assert_eq!(Tool::from_name("CAST"), Some(Tool::Cast));
        assert_eq!(Tool::from_name("DX"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("NODE"), Some(Tool::Node));
        assert_eq!(Tool::from_name("GIT-LFS"), Some(Tool::GitLfs));
    }

    #[test]
    fn test_detect_required_tools_dioxus_framework() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_dioxus").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Dioxus requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast, dx, node, npm, playwright
        assert!(tools.contains(&Tool::Rustup));
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::GitLfs));
        assert!(tools.contains(&Tool::Cast));
        assert!(tools.contains(&Tool::Dx));
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
        assert!(tools.contains(&Tool::Playwright));
        assert_eq!(tools.len(), 11);
    }

    #[test]
    fn test_detect_required_tools_cloudflare_pages_framework() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_cloudflare").unwrap();

        // Create a Cast.toml with cloudflare-pages framework
        fs::write(
            temp_dir.path().join("Cast.toml"),
            "framework = \"cloudflare-pages\"",
        )
        .unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Cloudflare Pages requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast, wrangler, node, npm
        assert!(tools.contains(&Tool::Rustup));
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::GitLfs));
        assert!(tools.contains(&Tool::Cast));
        assert!(tools.contains(&Tool::Wrangler));
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn test_detect_required_tools_pure_rust_no_framework() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_rust").unwrap();

        // Create a Cast.toml without framework (pure Rust)
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Pure Rust requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast
        assert!(tools.contains(&Tool::Rustup));
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::GitLfs));
        assert!(tools.contains(&Tool::Cast));
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_detect_required_tools_no_config_file() {
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_no_config").unwrap();

        // No Cast.toml or Cargo.toml - defaults to pure Rust
        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should default to pure Rust requirements
        assert!(tools.contains(&Tool::Rustup));
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::GitLfs));
        assert!(tools.contains(&Tool::Cast));
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_detect_required_tools_with_package_json() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_package_json").unwrap();

        // Create a Cast.toml without framework, but with package.json
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )
        .unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should include Node and npm due to package.json
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
    }

    #[test]
    fn test_detect_required_tools_with_playwright_config() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_playwright").unwrap();

        // Create a Cast.toml without framework, but with playwright.config.ts
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(
            temp_dir.path().join("playwright.config.ts"),
            "// Playwright config",
        )
        .unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should include Playwright due to config file
        assert!(tools.contains(&Tool::Playwright));
    }

    #[test]
    fn test_detect_required_tools_with_playwright_config_js() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_playwright_js").unwrap();

        // Create a Cast.toml without framework, but with playwright.config.js
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(
            temp_dir.path().join("playwright.config.js"),
            "// Playwright config",
        )
        .unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should include Playwright due to config file
        assert!(tools.contains(&Tool::Playwright));
    }

    #[test]
    fn test_detect_required_tools_with_wrangler_toml() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_wrangler").unwrap();

        // Create a Cast.toml without framework, but with wrangler.toml
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(temp_dir.path().join("wrangler.toml"), "name = \"test\"").unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should include Wrangler due to wrangler.toml
        assert!(tools.contains(&Tool::Wrangler));
    }

    #[test]
    fn test_detect_required_tools_from_cargo_toml_metadata() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_cargo_metadata").unwrap();

        // Create a Cargo.toml with cast metadata
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.cast]
framework = "dioxus"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let result = detect_required_tools(temp_dir.path());
        assert!(result.is_ok());

        let tools = result.unwrap();
        // Should detect dioxus framework from Cargo.toml
        assert!(tools.contains(&Tool::Dx));
    }

    #[test]
    fn test_check_tool_rustc() {
        // This test checks if rustc is available (it should be in the CI environment)
        let result = check_tool(&Tool::Rustc);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.tool, Tool::Rustc);
        // In CI environment, rustc should be installed
        // We can't guarantee version, so just check it's detected
    }

    #[test]
    fn test_check_tool_cargo() {
        // This test checks if cargo is available (it should be in the CI environment)
        let result = check_tool(&Tool::Cargo);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.tool, Tool::Cargo);
        // In CI environment, cargo should be installed
    }

    #[test]
    fn test_check_tool_nonexistent() {
        // Test with a tool that likely doesn't exist
        // We can't be 100% sure but dx is unlikely in standard environments
        // This test just ensures the function doesn't crash
        let result = check_tool(&Tool::Dx);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.tool, Tool::Dx);
        // installed status will vary by environment
    }
}

/// Options for installing tools
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Install only specific tools (None means install all required)
    pub specific_tools: Option<Vec<Tool>>,
    /// Skip these tools during installation
    pub skip_tools: Vec<Tool>,
    /// Don't actually install, just show what would be done
    pub dry_run: bool,
    /// Force reinstall even if already installed
    pub force: bool,
}

/// Result of an installation attempt
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub tool: Tool,
    pub success: bool,
    pub message: String,
    pub skipped: bool,
}

/// Install tools for the current project
pub fn install_tools(
    working_directory: impl AsRef<Path>,
    options: InstallOptions,
) -> Result<Vec<InstallResult>, ToolchainError> {
    let working_directory = working_directory.as_ref();

    // Determine which tools to install
    let required_tools = detect_required_tools(working_directory)?;

    let tools_to_install = if let Some(specific) = options.specific_tools {
        specific
    } else {
        required_tools.clone()
    };

    let mut results = Vec::new();

    for tool in tools_to_install {
        // Check if tool should be skipped
        if options.skip_tools.contains(&tool) {
            results.push(InstallResult {
                tool: tool.clone(),
                success: true,
                message: "Skipped".to_string(),
                skipped: true,
            });
            continue;
        }

        // Check if tool is already installed (unless force is set)
        if !options.force {
            let status = check_tool(&tool)?;
            if status.installed {
                results.push(InstallResult {
                    tool: tool.clone(),
                    success: true,
                    message: format!(
                        "Already installed ({})",
                        status
                            .version
                            .unwrap_or_else(|| "unknown version".to_string())
                    ),
                    skipped: false,
                });
                continue;
            }
        }

        // Install the tool
        let result = install_single_tool(&tool, working_directory, options.dry_run)?;
        results.push(result);
    }

    Ok(results)
}

/// Install a single tool
fn install_single_tool(
    tool: &Tool,
    working_directory: &Path,
    dry_run: bool,
) -> Result<InstallResult, ToolchainError> {
    match tool {
        Tool::Rustup => install_rustup(dry_run),
        Tool::Rustc | Tool::Cargo | Tool::Rustfmt | Tool::Clippy => {
            // Rust tools should be installed via rustup, not via this command
            Ok(InstallResult {
                tool: tool.clone(),
                success: false,
                message: "Rust tools must be installed via rustup".to_string(),
                skipped: false,
            })
        }
        Tool::Cast => install_cast(working_directory, dry_run),
        Tool::Dx => install_dx(dry_run),
        Tool::Node | Tool::Npm => install_node(tool, dry_run),
        Tool::Playwright => install_playwright(working_directory, dry_run),
        Tool::Wrangler => install_wrangler(dry_run),
        Tool::GitLfs => install_git_lfs(dry_run),
    }
}

/// Install Rustup toolchain installer
fn install_rustup(dry_run: bool) -> Result<InstallResult, ToolchainError> {
    use std::process::Command;

    let install_cmd = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y";

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::Rustup,
            success: true,
            message: format!("Would install: {}", install_cmd),
            skipped: false,
        });
    }

    println!("Installing Rustup...");
    println!("This will download and install the Rust toolchain installer.");

    // Execute the rustup installation script
    let output = Command::new("sh")
        .arg("-c")
        .arg(install_cmd)
        .output()
        .map_err(|e| {
            ToolchainError::InstallationError(format!("Failed to run rustup installer: {}", e))
        })?;

    if output.status.success() {
        println!("Rustup installed successfully!");
        println!("Note: You may need to restart your shell or run 'source $HOME/.cargo/env' to use Rust tools.");
        Ok(InstallResult {
            tool: Tool::Rustup,
            success: true,
            message: "Installed successfully. Restart your shell or run 'source $HOME/.cargo/env'"
                .to_string(),
            skipped: false,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolchainError::InstallationError(format!(
            "Failed to install rustup: {}",
            stderr
        )))
    }
}

/// Install Cast CLI
fn install_cast(working_directory: &Path, dry_run: bool) -> Result<InstallResult, ToolchainError> {
    use std::process::Command;

    // Find the monorepo root by looking for cast/cli directory
    let mut current_dir = working_directory;
    let mut cast_cli_path = None;
    
    // Walk up the directory tree to find cast/cli
    loop {
        let potential_path = current_dir.join("cast/cli");
        if potential_path.exists() && potential_path.is_dir() {
            cast_cli_path = Some(potential_path);
            break;
        }
        
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }
    
    let cast_cli_path = match cast_cli_path {
        Some(path) => path,
        None => {
            // In test environments or when not in the monorepo, skip installation
            return Ok(InstallResult {
                tool: Tool::Cast,
                success: false,
                message: "Skipped: Could not find cast/cli directory. Run from within the monorepo.".to_string(),
                skipped: true,
            });
        }
    };

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::Cast,
            success: true,
            message: format!(
                "Would install: cargo install --path {}",
                cast_cli_path.display()
            ),
            skipped: false,
        });
    }

    println!("Installing Cast CLI from {}...", cast_cli_path.display());

    let output = Command::new("cargo")
        .args(["install", "--path", cast_cli_path.to_str().unwrap()])
        .output()
        .map_err(|e| ToolchainError::InstallationError(format!("Failed to run cargo: {}", e)))?;

    if output.status.success() {
        Ok(InstallResult {
            tool: Tool::Cast,
            success: true,
            message: "Installed successfully".to_string(),
            skipped: false,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolchainError::InstallationError(format!(
            "Failed to install cast: {}",
            stderr
        )))
    }
}

/// Install Dioxus CLI
fn install_dx(dry_run: bool) -> Result<InstallResult, ToolchainError> {
    use std::process::Command;

    let version = "0.7.2";

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::Dx,
            success: true,
            message: format!(
                "Would install: cargo install dioxus-cli --version {}",
                version
            ),
            skipped: false,
        });
    }

    println!("Installing Dioxus CLI (dx) version {}...", version);

    let output = Command::new("cargo")
        .args(["install", "dioxus-cli", "--version", version])
        .output()
        .map_err(|e| ToolchainError::InstallationError(format!("Failed to run cargo: {}", e)))?;

    if output.status.success() {
        Ok(InstallResult {
            tool: Tool::Dx,
            success: true,
            message: format!("Installed successfully (version {})", version),
            skipped: false,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolchainError::InstallationError(format!(
            "Failed to install dx: {}",
            stderr
        )))
    }
}

/// Provide guidance for Node.js installation
fn install_node(tool: &Tool, dry_run: bool) -> Result<InstallResult, ToolchainError> {
    // Node.js and npm should be installed via system package manager
    // We provide guidance instead of trying to install

    let (os_name, install_cmd) = if cfg!(target_os = "linux") {
        ("Linux", "sudo apt install nodejs npm")
    } else if cfg!(target_os = "macos") {
        ("macOS", "brew install node")
    } else if cfg!(target_os = "windows") {
        ("Windows", "winget install OpenJS.NodeJS")
    } else {
        ("your OS", "your system package manager")
    };

    let message = format!(
        "{} must be installed via your system package manager.\n\
         For {}, use: {}",
        tool.name(),
        os_name,
        install_cmd
    );

    if dry_run {
        return Ok(InstallResult {
            tool: tool.clone(),
            success: false,
            message: format!("Would provide guidance: {}", message),
            skipped: false,
        });
    }

    Ok(InstallResult {
        tool: tool.clone(),
        success: false,
        message,
        skipped: false,
    })
}

/// Install Playwright
fn install_playwright(
    working_directory: &Path,
    dry_run: bool,
) -> Result<InstallResult, ToolchainError> {
    use std::process::Command;

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::Playwright,
            success: true,
            message: "Would install: npm ci && npx playwright install --with-deps chromium"
                .to_string(),
            skipped: false,
        });
    }

    println!("Installing Playwright...");

    // First, run npm ci if package.json exists
    if working_directory.join("package.json").exists() {
        println!("Running npm ci to install dependencies...");
        let output = Command::new("npm")
            .args(["ci"])
            .current_dir(working_directory)
            .output()
            .map_err(|e| {
                ToolchainError::InstallationError(format!("Failed to run npm ci: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolchainError::InstallationError(format!(
                "npm ci failed: {}",
                stderr
            )));
        }
    }

    // Then install Playwright browsers
    println!("Installing Playwright browsers...");
    let output = Command::new("npx")
        .args(["playwright", "install", "--with-deps", "chromium"])
        .current_dir(working_directory)
        .output()
        .map_err(|e| {
            ToolchainError::InstallationError(format!("Failed to run npx playwright: {}", e))
        })?;

    if output.status.success() {
        Ok(InstallResult {
            tool: Tool::Playwright,
            success: true,
            message: "Installed successfully".to_string(),
            skipped: false,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolchainError::InstallationError(format!(
            "Failed to install Playwright: {}",
            stderr
        )))
    }
}

/// Install Wrangler CLI
fn install_wrangler(dry_run: bool) -> Result<InstallResult, ToolchainError> {
    use std::process::Command;

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::Wrangler,
            success: true,
            message: "Would install: npm install -g wrangler".to_string(),
            skipped: false,
        });
    }

    println!("Installing Wrangler CLI...");

    // Try npm first (primary method)
    let output = Command::new("npm")
        .args(["install", "-g", "wrangler"])
        .output()
        .map_err(|e| ToolchainError::InstallationError(format!("Failed to run npm: {}", e)))?;

    if output.status.success() {
        Ok(InstallResult {
            tool: Tool::Wrangler,
            success: true,
            message: "Installed successfully via npm".to_string(),
            skipped: false,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolchainError::InstallationError(format!(
            "Failed to install wrangler: {}",
            stderr
        )))
    }
}

/// Provide guidance for Git LFS installation
fn install_git_lfs(dry_run: bool) -> Result<InstallResult, ToolchainError> {
    // Git LFS should be installed via system package manager
    // We provide guidance instead of trying to install

    let (os_name, install_cmd) = if cfg!(target_os = "linux") {
        ("Linux", "sudo apt install git-lfs && git lfs install")
    } else if cfg!(target_os = "macos") {
        ("macOS", "brew install git-lfs && git lfs install")
    } else if cfg!(target_os = "windows") {
        ("Windows", "winget install GitHub.GitLFS")
    } else {
        ("your OS", "your system package manager")
    };

    let message = format!(
        "git-lfs must be installed via your system package manager.\n\
         For {}, use: {}",
        os_name, install_cmd
    );

    if dry_run {
        return Ok(InstallResult {
            tool: Tool::GitLfs,
            success: false,
            message: format!("Would provide guidance: {}", message),
            skipped: false,
        });
    }

    Ok(InstallResult {
        tool: Tool::GitLfs,
        success: false,
        message,
        skipped: false,
    })
}

/// Options for checking tools
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// Show detailed version information
    pub verbose: bool,
    /// Output results in JSON format
    pub json: bool,
}

/// Result of checking toolchain
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub framework: Option<String>,
    pub tool_statuses: Vec<ToolStatus>,
    pub all_installed: bool,
    pub missing_count: usize,
}

impl CheckResult {
    /// Format check result as text output
    pub fn format_text(&self, verbose: bool) -> String {
        let mut output = String::new();

        // Show framework if known
        if let Some(framework) = &self.framework {
            output.push_str(&format!(
                "Checking toolchain for {} project...\n",
                framework
            ));
        } else {
            output.push_str("Checking toolchain for pure Rust project...\n");
        }

        // Show tool status
        for status in &self.tool_statuses {
            if status.installed {
                let version_str = if verbose {
                    if let Some(version) = &status.version {
                        format!(" ({})", version)
                    } else {
                        " (version unknown)".to_string()
                    }
                } else {
                    String::new()
                };
                output.push_str(&format!("✓ {}{}\n", status.tool.name(), version_str));
            } else {
                output.push_str(&format!("✗ {} (not installed)\n", status.tool.name()));
            }
        }

        // Show summary
        output.push('\n');
        if self.all_installed {
            output.push_str("Status: All required tools are installed\n");
        } else {
            output.push_str(&format!(
                "Status: {} tool{} missing\n",
                self.missing_count,
                if self.missing_count == 1 { "" } else { "s" }
            ));
        }

        output
    }

    /// Format check result as JSON
    pub fn format_json(&self) -> Result<String, ToolchainError> {
        use serde_json::json;

        let tools_json: Vec<serde_json::Value> = self
            .tool_statuses
            .iter()
            .map(|status| {
                json!({
                    "name": status.tool.name(),
                    "required": true,
                    "installed": status.installed,
                    "version": status.version,
                })
            })
            .collect();

        let result = json!({
            "framework": self.framework,
            "tools": tools_json,
            "all_installed": self.all_installed,
            "missing_count": self.missing_count,
        });

        serde_json::to_string_pretty(&result).map_err(|e| {
            ToolchainError::DetectionError(format!("JSON serialization failed: {}", e))
        })
    }
}

/// Check if all required tools are installed
pub fn check_tools(
    working_directory: impl AsRef<Path>,
    _options: CheckOptions,
) -> Result<CheckResult, ToolchainError> {
    let working_directory = working_directory.as_ref();

    // Get framework from config
    let config = CastConfig::load_from_dir(working_directory).ok();
    let framework = config.and_then(|c| c.framework);

    // Detect required tools
    let required_tools = detect_required_tools(working_directory)?;

    // Check each tool
    let mut tool_statuses = Vec::new();
    for tool in required_tools {
        let status = check_tool(&tool)?;
        tool_statuses.push(status);
    }

    // Calculate summary
    let missing_count = tool_statuses.iter().filter(|s| !s.installed).count();
    let all_installed = missing_count == 0;

    Ok(CheckResult {
        framework,
        tool_statuses,
        all_installed,
        missing_count,
    })
}

/// Options for listing tools
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    /// Only show tools required by the current project
    pub required_only: bool,
    /// Show all tools that Cast can manage
    pub all: bool,
    /// Output results in JSON format
    pub json: bool,
}

/// Result of listing tools
#[derive(Debug, Clone)]
pub struct ListResult {
    pub tool_statuses: Vec<ToolStatus>,
}

impl ListResult {
    /// Format list result as text output
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        // Show tool status
        for status in &self.tool_statuses {
            if status.installed {
                if let Some(version) = &status.version {
                    output.push_str(&format!(
                        "{}: {} (installed)\n",
                        status.tool.name(),
                        version
                    ));
                } else {
                    output.push_str(&format!(
                        "{}: installed (version unknown)\n",
                        status.tool.name()
                    ));
                }
            } else {
                output.push_str(&format!("{}: not installed\n", status.tool.name()));
            }
        }

        output
    }

    /// Format list result as JSON
    pub fn format_json(&self) -> Result<String, ToolchainError> {
        use serde_json::json;

        let tools_json: Vec<serde_json::Value> = self
            .tool_statuses
            .iter()
            .map(|status| {
                json!({
                    "name": status.tool.name(),
                    "installed": status.installed,
                    "version": status.version,
                })
            })
            .collect();

        let result = json!({
            "tools": tools_json,
        });

        serde_json::to_string_pretty(&result).map_err(|e| {
            ToolchainError::DetectionError(format!("JSON serialization failed: {}", e))
        })
    }
}

/// Get all tools that Cast can manage
fn get_all_tools() -> Vec<Tool> {
    vec![
        Tool::Rustup,
        Tool::Rustc,
        Tool::Cargo,
        Tool::Rustfmt,
        Tool::Clippy,
        Tool::Cast,
        Tool::Dx,
        Tool::Node,
        Tool::Npm,
        Tool::Playwright,
        Tool::Wrangler,
        Tool::GitLfs,
    ]
}

/// List tools and their installation status
pub fn list_tools(
    working_directory: impl AsRef<Path>,
    options: ListOptions,
) -> Result<ListResult, ToolchainError> {
    let working_directory = working_directory.as_ref();

    // Determine which tools to list
    let tools_to_list = if options.all {
        // Show all tools that Cast can manage
        get_all_tools()
    } else if options.required_only {
        // Show only tools required by the current project
        detect_required_tools(working_directory)?
    } else {
        // Default behavior: show required tools only
        detect_required_tools(working_directory)?
    };

    // Check each tool
    let mut tool_statuses = Vec::new();
    for tool in tools_to_list {
        let status = check_tool(&tool)?;
        tool_statuses.push(status);
    }

    Ok(ListResult { tool_statuses })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod install_tests {
    use super::*;

    #[test]
    fn test_install_options_default() {
        let options = InstallOptions::default();
        assert!(options.specific_tools.is_none());
        assert!(options.skip_tools.is_empty());
        assert!(!options.dry_run);
        assert!(!options.force);
    }

    #[test]
    fn test_install_tools_dry_run() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_install_dry_run").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let options = InstallOptions {
            specific_tools: None,
            skip_tools: Vec::new(),
            dry_run: true,
            force: false,
        };

        let result = install_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let results = result.unwrap();
        // All tools should report what would be installed
        assert!(!results.is_empty());
    }

    #[test]
    fn test_install_tools_skip() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_install_skip").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let options = InstallOptions {
            specific_tools: None,
            skip_tools: vec![Tool::Dx, Tool::Playwright],
            dry_run: true,
            force: false,
        };

        let result = install_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let results = result.unwrap();

        // Check that skipped tools are marked as skipped
        let dx_result = results.iter().find(|r| r.tool == Tool::Dx);
        assert!(dx_result.is_some());
        assert!(dx_result.unwrap().skipped);

        let playwright_result = results.iter().find(|r| r.tool == Tool::Playwright);
        assert!(playwright_result.is_some());
        assert!(playwright_result.unwrap().skipped);
    }

    #[test]
    fn test_install_tools_specific_tool() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_install_specific").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let options = InstallOptions {
            specific_tools: Some(vec![Tool::Dx]),
            skip_tools: Vec::new(),
            dry_run: true,
            force: false,
        };

        let result = install_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let results = result.unwrap();

        // Should only try to install the specific tool
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool, Tool::Dx);
    }

    #[test]
    fn test_install_dx_dry_run() {
        let result = install_dx(true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert_eq!(install_result.tool, Tool::Dx);
        assert!(install_result.success);
        assert!(install_result.message.contains("Would install"));
        assert!(install_result.message.contains("0.7.2"));
    }

    #[test]
    fn test_install_node_provides_guidance() {
        let result = install_node(&Tool::Node, true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert_eq!(install_result.tool, Tool::Node);
        assert!(!install_result.success); // Should indicate it needs manual installation
        assert!(install_result.message.contains("system package manager"));
    }

    #[test]
    fn test_install_playwright_dry_run() {
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_playwright_dry_run").unwrap();

        let result = install_playwright(temp_dir.path(), true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert_eq!(install_result.tool, Tool::Playwright);
        assert!(install_result.success);
        assert!(install_result.message.contains("Would install"));
        assert!(install_result.message.contains("npx playwright install"));
    }

    #[test]
    fn test_install_wrangler_dry_run() {
        let result = install_wrangler(true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert_eq!(install_result.tool, Tool::Wrangler);
        assert!(install_result.success);
        assert!(install_result.message.contains("Would install"));
        assert!(install_result.message.contains("npm install -g wrangler"));
    }

    #[test]
    fn test_install_rustup_dry_run() {
        let result = install_rustup(true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert_eq!(install_result.tool, Tool::Rustup);
        assert!(install_result.success);
        assert!(install_result.message.contains("Would install"));
        assert!(install_result.message.contains("curl"));
        assert!(install_result.message.contains("https://sh.rustup.rs"));
    }

    #[test]
    fn test_install_rust_tools_returns_error() {
        let result = install_single_tool(&Tool::Rustc, std::path::Path::new("."), true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert!(!install_result.success);
        assert!(install_result.message.contains("rustup"));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod check_tests {
    use super::*;

    #[test]
    fn test_check_options_default() {
        let options = CheckOptions::default();
        assert!(!options.verbose);
        assert!(!options.json);
    }

    #[test]
    fn test_check_tools_pure_rust() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_check_pure_rust").unwrap();

        // Create a Cast.toml without framework (pure Rust)
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        let options = CheckOptions {
            verbose: false,
            json: false,
        };

        let result = check_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let check_result = result.unwrap();
        assert!(check_result.framework.is_none());
        // Pure Rust requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast
        assert_eq!(check_result.tool_statuses.len(), 7);

        // In CI environment, Rust tools should be installed
        let rustc_status = check_result
            .tool_statuses
            .iter()
            .find(|s| s.tool == Tool::Rustc);
        assert!(rustc_status.is_some());
    }

    #[test]
    fn test_check_tools_dioxus_framework() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_check_dioxus").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let options = CheckOptions {
            verbose: false,
            json: false,
        };

        let result = check_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let check_result = result.unwrap();
        assert_eq!(check_result.framework, Some("dioxus".to_string()));
        // Dioxus requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast, dx, node, npm, playwright
        assert_eq!(check_result.tool_statuses.len(), 11);
    }

    #[test]
    fn test_check_result_format_text_basic() {
        let check_result = CheckResult {
            framework: Some("dioxus".to_string()),
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
            all_installed: false,
            missing_count: 1,
        };

        let output = check_result.format_text(false);
        assert!(output.contains("Checking toolchain for dioxus project"));
        assert!(output.contains("✓ rustc"));
        assert!(output.contains("✗ dx (not installed)"));
        assert!(output.contains("1 tool missing"));
        // In non-verbose mode, version should not be shown
        assert!(!output.contains("1.75.0"));
    }

    #[test]
    fn test_check_result_format_text_verbose() {
        let check_result = CheckResult {
            framework: Some("dioxus".to_string()),
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
            all_installed: false,
            missing_count: 1,
        };

        let output = check_result.format_text(true);
        assert!(output.contains("Checking toolchain for dioxus project"));
        assert!(output.contains("✓ rustc"));
        assert!(output.contains("1.75.0")); // Version should be shown in verbose mode
        assert!(output.contains("✗ dx (not installed)"));
        assert!(output.contains("1 tool missing"));
    }

    #[test]
    fn test_check_result_format_text_all_installed() {
        let check_result = CheckResult {
            framework: None,
            tool_statuses: vec![ToolStatus {
                tool: Tool::Rustc,
                installed: true,
                version: Some("1.75.0".to_string()),
            }],
            all_installed: true,
            missing_count: 0,
        };

        let output = check_result.format_text(false);
        assert!(output.contains("pure Rust project"));
        assert!(output.contains("✓ rustc"));
        assert!(output.contains("All required tools are installed"));
    }

    #[test]
    fn test_check_result_format_json() {
        let check_result = CheckResult {
            framework: Some("dioxus".to_string()),
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
            all_installed: false,
            missing_count: 1,
        };

        let result = check_result.format_json();
        assert!(result.is_ok());

        let json_output = result.unwrap();
        assert!(json_output.contains("\"framework\": \"dioxus\""));
        assert!(json_output.contains("\"name\": \"rustc\""));
        assert!(json_output.contains("\"installed\": true"));
        assert!(json_output.contains("\"version\": \"1.75.0\""));
        assert!(json_output.contains("\"name\": \"dx\""));
        assert!(json_output.contains("\"installed\": false"));
        assert!(json_output.contains("\"all_installed\": false"));
        assert!(json_output.contains("\"missing_count\": 1"));
    }

    #[test]
    fn test_check_result_format_json_pure_rust() {
        let check_result = CheckResult {
            framework: None,
            tool_statuses: vec![ToolStatus {
                tool: Tool::Rustc,
                installed: true,
                version: Some("1.75.0".to_string()),
            }],
            all_installed: true,
            missing_count: 0,
        };

        let result = check_result.format_json();
        assert!(result.is_ok());

        let json_output = result.unwrap();
        assert!(json_output.contains("\"framework\": null"));
        assert!(json_output.contains("\"all_installed\": true"));
        assert!(json_output.contains("\"missing_count\": 0"));
    }

    #[test]
    fn test_check_result_missing_count_plural() {
        let check_result = CheckResult {
            framework: Some("dioxus".to_string()),
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: false,
                    version: None,
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
            all_installed: false,
            missing_count: 2,
        };

        let output = check_result.format_text(false);
        assert!(output.contains("2 tools missing")); // "tools" not "tool"
    }

    #[test]
    fn test_check_result_missing_count_singular() {
        let check_result = CheckResult {
            framework: Some("dioxus".to_string()),
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
            all_installed: false,
            missing_count: 1,
        };

        let output = check_result.format_text(false);
        assert!(output.contains("1 tool missing")); // "tool" not "tools"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod list_tests {
    use super::*;

    #[test]
    fn test_list_options_default() {
        let options = ListOptions::default();
        assert!(!options.required_only);
        assert!(!options.all);
        assert!(!options.json);
    }

    #[test]
    fn test_get_all_tools() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 12);
        assert!(tools.contains(&Tool::Rustup));
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::Cast));
        assert!(tools.contains(&Tool::Dx));
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
        assert!(tools.contains(&Tool::Playwright));
        assert!(tools.contains(&Tool::Wrangler));
        assert!(tools.contains(&Tool::GitLfs));
    }

    #[test]
    fn test_list_tools_required_only() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_list_required").unwrap();

        // Create a Cast.toml with dioxus framework
        fs::write(temp_dir.path().join("Cast.toml"), "framework = \"dioxus\"").unwrap();

        let options = ListOptions {
            required_only: true,
            all: false,
            json: false,
        };

        let result = list_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let list_result = result.unwrap();
        // Dioxus requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast, dx, node, npm, playwright
        assert_eq!(list_result.tool_statuses.len(), 11);

        let tool_names: Vec<&str> = list_result
            .tool_statuses
            .iter()
            .map(|s| s.tool.name())
            .collect();
        assert!(tool_names.contains(&"rustup"));
        assert!(tool_names.contains(&"rustc"));
        assert!(tool_names.contains(&"cargo"));
        assert!(tool_names.contains(&"rustfmt"));
        assert!(tool_names.contains(&"clippy"));
        assert!(tool_names.contains(&"git-lfs"));
        assert!(tool_names.contains(&"dx"));
        assert!(tool_names.contains(&"node"));
        assert!(tool_names.contains(&"npm"));
        assert!(tool_names.contains(&"playwright"));
    }

    #[test]
    fn test_list_tools_all() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_list_all").unwrap();

        // Create a Cast.toml (framework doesn't matter for --all)
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        let options = ListOptions {
            required_only: false,
            all: true,
            json: false,
        };

        let result = list_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let list_result = result.unwrap();
        // Should list all 12 tools
        assert_eq!(list_result.tool_statuses.len(), 12);
    }

    #[test]
    fn test_list_tools_default_shows_required() {
        use std::fs;
        use tempdir::TempDir;

        let temp_dir = TempDir::new("test_list_default").unwrap();

        // Create a Cast.toml without framework (pure Rust)
        fs::write(temp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        let options = ListOptions {
            required_only: false,
            all: false,
            json: false,
        };

        let result = list_tools(temp_dir.path(), options);
        assert!(result.is_ok());

        let list_result = result.unwrap();
        // Pure Rust requires: rustup, rustc, cargo, rustfmt, clippy, git-lfs, cast
        assert_eq!(list_result.tool_statuses.len(), 7);
    }

    #[test]
    fn test_list_result_format_text() {
        let list_result = ListResult {
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
                ToolStatus {
                    tool: Tool::Node,
                    installed: true,
                    version: Some("v20.10.0".to_string()),
                },
            ],
        };

        let output = list_result.format_text();
        assert!(output.contains("rustc: 1.75.0 (installed)"));
        assert!(output.contains("dx: not installed"));
        assert!(output.contains("node: v20.10.0 (installed)"));
    }

    #[test]
    fn test_list_result_format_text_unknown_version() {
        let list_result = ListResult {
            tool_statuses: vec![ToolStatus {
                tool: Tool::Rustc,
                installed: true,
                version: None,
            }],
        };

        let output = list_result.format_text();
        assert!(output.contains("rustc: installed (version unknown)"));
    }

    #[test]
    fn test_list_result_format_json() {
        let list_result = ListResult {
            tool_statuses: vec![
                ToolStatus {
                    tool: Tool::Rustc,
                    installed: true,
                    version: Some("1.75.0".to_string()),
                },
                ToolStatus {
                    tool: Tool::Dx,
                    installed: false,
                    version: None,
                },
            ],
        };

        let result = list_result.format_json();
        assert!(result.is_ok());

        let json_output = result.unwrap();
        assert!(json_output.contains("\"name\": \"rustc\""));
        assert!(json_output.contains("\"installed\": true"));
        assert!(json_output.contains("\"version\": \"1.75.0\""));
        assert!(json_output.contains("\"name\": \"dx\""));
        assert!(json_output.contains("\"installed\": false"));
        assert!(json_output.contains("\"version\": null"));
    }
}
