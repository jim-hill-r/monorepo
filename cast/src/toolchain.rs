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
