use crate::command::Command;
use crate::toolchain::{self, CheckOptions, InstallOptions, ListOptions, Tool};
use std::path::Path;

/// Command to install toolchain dependencies
pub struct InstallCommand {
    pub tool: Option<String>,
    pub skip: Option<String>,
    pub dry_run: bool,
    pub force: bool,
}

impl Command for InstallCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        // Parse tool names from command line
        let specific_tools = if let Some(tool_name) = &self.tool {
            let tool =
                Tool::from_name(tool_name).ok_or_else(|| format!("Unknown tool: {}", tool_name))?;
            Some(vec![tool])
        } else {
            None
        };

        // Parse skip list
        let skip_tools = if let Some(skip_str) = &self.skip {
            skip_str
                .split(',')
                .filter_map(|s| Tool::from_name(s.trim()))
                .collect()
        } else {
            Vec::new()
        };

        let options = InstallOptions {
            specific_tools,
            skip_tools,
            dry_run: self.dry_run,
            force: self.force,
        };

        let results = toolchain::install_tools(working_directory, options)?;

        // Format output
        let mut output = String::new();
        let mut any_failed = false;

        for result in &results {
            if result.skipped {
                output.push_str(&format!("⊘ {}: {}\n", result.tool.name(), result.message));
            } else if result.success {
                output.push_str(&format!("✓ {}: {}\n", result.tool.name(), result.message));
            } else {
                output.push_str(&format!("✗ {}: {}\n", result.tool.name(), result.message));
                any_failed = true;
            }
        }

        if any_failed {
            output.push_str("\nSome tools failed to install. Please address the errors above.");
            Err(output.into())
        } else {
            Ok(output)
        }
    }
}

/// Command to check if required tools are installed
pub struct CheckCommand {
    pub verbose: bool,
    pub json: bool,
}

impl Command for CheckCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let options = CheckOptions {
            verbose: self.verbose,
            json: self.json,
        };

        let check_result = toolchain::check_tools(working_directory, options)?;

        // Format output based on options
        let output = if self.json {
            check_result.format_json()?
        } else {
            check_result.format_text(self.verbose)
        };

        // Return error if tools are missing (exit code 1)
        if !check_result.all_installed {
            Err(output.into())
        } else {
            Ok(output)
        }
    }
}

/// Command to list installed tools and their versions
pub struct ListCommand {
    pub required_only: bool,
    pub all: bool,
    pub json: bool,
}

impl Command for ListCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let options = ListOptions {
            required_only: self.required_only,
            all: self.all,
            json: self.json,
        };

        let list_result = toolchain::list_tools(working_directory, options)?;

        // Format output based on options
        let output = if self.json {
            list_result.format_json()?
        } else {
            list_result.format_text()
        };

        Ok(output)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_install_command_dry_run() {
        let tmp_dir = TempDir::new("test_install_toolchain").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = InstallCommand {
            tool: None,
            skip: None,
            dry_run: true,
            force: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_install_command_specific_tool() {
        let tmp_dir = TempDir::new("test_install_specific_tool").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = InstallCommand {
            tool: Some("rustc".to_string()),
            skip: None,
            dry_run: true,
            force: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("rustc"));
    }

    #[test]
    fn test_install_command_unknown_tool() {
        let tmp_dir = TempDir::new("test_install_unknown_tool").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = InstallCommand {
            tool: Some("unknown-tool".to_string()),
            skip: None,
            dry_run: true,
            force: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_check_command() {
        let tmp_dir = TempDir::new("test_check_toolchain").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = CheckCommand {
            verbose: false,
            json: false,
        };
        let result = cmd.execute(tmp_dir.path());
        // Check command should work now, but might fail if some tools are not installed
        match result {
            Ok(output) => {
                assert!(output.contains("All required tools are installed"));
            }
            Err(err) => {
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("tool") || error_msg.contains("missing"),
                    "Error message should contain tool information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_check_command_json() {
        let tmp_dir = TempDir::new("test_check_toolchain_json").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = CheckCommand {
            verbose: false,
            json: true,
        };
        let result = cmd.execute(tmp_dir.path());
        match result {
            Ok(output) => {
                assert!(output.contains("\"framework\""));
                assert!(output.contains("\"tools\""));
                assert!(output.contains("\"all_installed\""));
            }
            Err(err) => {
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("\"framework\"") || error_msg.contains("JSON"),
                    "Error message should contain JSON information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_list_command() {
        let tmp_dir = TempDir::new("test_list_toolchain").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = ListCommand {
            required_only: false,
            all: false,
            json: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("rustc") || output.contains("cargo"));
    }

    #[test]
    fn test_list_command_all() {
        let tmp_dir = TempDir::new("test_list_toolchain_all").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = ListCommand {
            required_only: false,
            all: true,
            json: false,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain all 11 tools
        assert!(output.contains("rustc"));
        assert!(output.contains("cargo"));
        assert!(output.contains("rustfmt"));
        assert!(output.contains("clippy"));
        assert!(output.contains("cast"));
        assert!(output.contains("dx"));
        assert!(output.contains("node"));
        assert!(output.contains("npm"));
        assert!(output.contains("playwright"));
        assert!(output.contains("wrangler"));
        assert!(output.contains("git-lfs"));
    }

    #[test]
    fn test_list_command_json() {
        let tmp_dir = TempDir::new("test_list_toolchain_json").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let cmd = ListCommand {
            required_only: false,
            all: false,
            json: true,
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("{"));
        assert!(output.contains("}"));
        assert!(output.contains("\"tools\""));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"installed\""));
    }
}
