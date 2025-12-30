use std::path::Path;
use thiserror::Error;

/// Generic error type for command execution
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
}

/// Trait for executable commands
///
/// This trait defines a common interface for all cast commands,
/// following the Command/Executor pattern. Each command type implements
/// this trait to provide its specific execution logic.
///
/// # Examples
///
/// ```
/// use cast_core::command::Command;
/// use std::path::Path;
///
/// struct BuildCommand;
///
/// impl Command for BuildCommand {
///     fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
///         // Execute build logic
///         Ok("Build completed".to_string())
///     }
/// }
/// ```
pub trait Command {
    /// Execute the command in the given working directory
    ///
    /// # Arguments
    ///
    /// * `working_directory` - The directory in which to execute the command
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Success message to display to the user
    /// * `Err(Box<dyn std::error::Error>)` - Error that occurred during execution
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TestCommand {
        should_succeed: bool,
    }

    impl Command for TestCommand {
        fn execute(&self, _working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
            if self.should_succeed {
                Ok("Test command succeeded".to_string())
            } else {
                Err(Box::new(CommandError::ExecutionFailed(
                    "Test command failed".to_string(),
                )))
            }
        }
    }

    #[test]
    fn test_command_execute_success() {
        let cmd = TestCommand {
            should_succeed: true,
        };
        let result = cmd.execute(&PathBuf::from("/tmp"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test command succeeded");
    }

    #[test]
    fn test_command_execute_failure() {
        let cmd = TestCommand {
            should_succeed: false,
        };
        let result = cmd.execute(&PathBuf::from("/tmp"));
        assert!(result.is_err());
    }
}
