use std::path::Path;
use thiserror::Error;

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
    _working_directory: impl AsRef<Path>,
) -> Result<Vec<Tool>, ToolchainError> {
    // Placeholder implementation - will be implemented in Phase 1
    Ok(vec![])
}

/// Check if a tool is installed and get its version
pub fn check_tool(tool: &Tool) -> Result<ToolStatus, ToolchainError> {
    // Placeholder implementation - will be implemented in Phase 1
    Ok(ToolStatus {
        tool: tool.clone(),
        installed: false,
        version: None,
    })
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
    fn test_detect_required_tools_placeholder() {
        let temp_dir = std::env::temp_dir();
        let result = detect_required_tools(&temp_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn test_check_tool_placeholder() {
        let result = check_tool(&Tool::Rustc);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.tool, Tool::Rustc);
        assert!(!status.installed);
        assert_eq!(status.version, None);
    }
}
