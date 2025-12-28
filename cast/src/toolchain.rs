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
    /// Rust compiler
    Rustc,
    /// Cargo package manager
    Cargo,
    /// Rust formatter
    Rustfmt,
    /// Rust linter
    Clippy,
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
}

impl Tool {
    /// Get the display name for a tool
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Rustc => "rustc",
            Tool::Cargo => "cargo",
            Tool::Rustfmt => "rustfmt",
            Tool::Clippy => "clippy",
            Tool::Dx => "dx",
            Tool::Node => "node",
            Tool::Npm => "npm",
            Tool::Playwright => "playwright",
            Tool::Wrangler => "wrangler",
        }
    }

    /// Parse a tool name from a string
    pub fn from_name(name: &str) -> Option<Tool> {
        match name.to_lowercase().as_str() {
            "rustc" => Some(Tool::Rustc),
            "cargo" => Some(Tool::Cargo),
            "rustfmt" => Some(Tool::Rustfmt),
            "clippy" => Some(Tool::Clippy),
            "dx" | "dioxus" => Some(Tool::Dx),
            "node" | "nodejs" => Some(Tool::Node),
            "npm" => Some(Tool::Npm),
            "playwright" => Some(Tool::Playwright),
            "wrangler" => Some(Tool::Wrangler),
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

    // Always include base Rust tools
    tools.insert(Tool::Rustc);
    tools.insert(Tool::Cargo);
    tools.insert(Tool::Rustfmt);
    tools.insert(Tool::Clippy);

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

    let (command, args) = match tool {
        Tool::Rustc => ("rustc", vec!["--version"]),
        Tool::Cargo => ("cargo", vec!["--version"]),
        Tool::Rustfmt => ("rustfmt", vec!["--version"]),
        Tool::Clippy => ("cargo", vec!["clippy", "--version"]),
        Tool::Dx => ("dx", vec!["--version"]),
        Tool::Node => ("node", vec!["--version"]),
        Tool::Npm => ("npm", vec!["--version"]),
        Tool::Playwright => ("npx", vec!["playwright", "--version"]),
        Tool::Wrangler => ("wrangler", vec!["--version"]),
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
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        assert_eq!(Tool::Rustc.name(), "rustc");
        assert_eq!(Tool::Cargo.name(), "cargo");
        assert_eq!(Tool::Rustfmt.name(), "rustfmt");
        assert_eq!(Tool::Clippy.name(), "clippy");
        assert_eq!(Tool::Dx.name(), "dx");
        assert_eq!(Tool::Node.name(), "node");
        assert_eq!(Tool::Npm.name(), "npm");
        assert_eq!(Tool::Playwright.name(), "playwright");
        assert_eq!(Tool::Wrangler.name(), "wrangler");
    }

    #[test]
    fn test_tool_from_name() {
        assert_eq!(Tool::from_name("rustc"), Some(Tool::Rustc));
        assert_eq!(Tool::from_name("cargo"), Some(Tool::Cargo));
        assert_eq!(Tool::from_name("rustfmt"), Some(Tool::Rustfmt));
        assert_eq!(Tool::from_name("clippy"), Some(Tool::Clippy));
        assert_eq!(Tool::from_name("dx"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("dioxus"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("node"), Some(Tool::Node));
        assert_eq!(Tool::from_name("nodejs"), Some(Tool::Node));
        assert_eq!(Tool::from_name("npm"), Some(Tool::Npm));
        assert_eq!(Tool::from_name("playwright"), Some(Tool::Playwright));
        assert_eq!(Tool::from_name("wrangler"), Some(Tool::Wrangler));
        assert_eq!(Tool::from_name("unknown"), None);
    }

    #[test]
    fn test_tool_from_name_case_insensitive() {
        assert_eq!(Tool::from_name("RUSTC"), Some(Tool::Rustc));
        assert_eq!(Tool::from_name("Cargo"), Some(Tool::Cargo));
        assert_eq!(Tool::from_name("DX"), Some(Tool::Dx));
        assert_eq!(Tool::from_name("NODE"), Some(Tool::Node));
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
        // Dioxus requires: rustc, cargo, rustfmt, clippy, dx, node, npm, playwright
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::Dx));
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
        assert!(tools.contains(&Tool::Playwright));
        assert_eq!(tools.len(), 8);
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
        // Cloudflare Pages requires: rustc, cargo, rustfmt, clippy, wrangler, node, npm
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert!(tools.contains(&Tool::Wrangler));
        assert!(tools.contains(&Tool::Node));
        assert!(tools.contains(&Tool::Npm));
        assert_eq!(tools.len(), 7);
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
        // Pure Rust requires: rustc, cargo, rustfmt, clippy
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert_eq!(tools.len(), 4);
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
        assert!(tools.contains(&Tool::Rustc));
        assert!(tools.contains(&Tool::Cargo));
        assert!(tools.contains(&Tool::Rustfmt));
        assert!(tools.contains(&Tool::Clippy));
        assert_eq!(tools.len(), 4);
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
        Tool::Rustc | Tool::Cargo | Tool::Rustfmt | Tool::Clippy => {
            // Rust tools should be installed via rustup, not via this command
            Ok(InstallResult {
                tool: tool.clone(),
                success: false,
                message: "Rust tools must be installed via rustup".to_string(),
                skipped: false,
            })
        }
        Tool::Dx => install_dx(dry_run),
        Tool::Node | Tool::Npm => install_node(tool, dry_run),
        Tool::Playwright => install_playwright(working_directory, dry_run),
        Tool::Wrangler => install_wrangler(dry_run),
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

#[cfg(test)]
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
        use std::fs;
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
    fn test_install_rust_tools_returns_error() {
        let result = install_single_tool(&Tool::Rustc, std::path::Path::new("."), true);
        assert!(result.is_ok());
        let install_result = result.unwrap();
        assert!(!install_result.success);
        assert!(install_result.message.contains("rustup"));
    }
}
