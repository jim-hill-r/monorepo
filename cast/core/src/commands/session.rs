use crate::command::Command;
use crate::sessions::{self, SessionStartOptions};
use std::path::Path;

/// Command to start a session
pub struct StartCommand {
    pub name: Option<String>,
}

impl Command for StartCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        sessions::start(
            working_directory,
            Some(SessionStartOptions {
                name: self.name.clone(),
            }),
        )?;
        Ok("Starting session.".to_string())
    }
}

/// Command to pause a session
pub struct PauseCommand;

impl Command for PauseCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        sessions::pause(working_directory)?;
        Ok("Pausing session.".to_string())
    }
}

/// Command to stop a session
pub struct StopCommand;

impl Command for StopCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        sessions::stop(working_directory)?;
        Ok("Stopping session.".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_start_command() {
        let tmp_dir = TempDir::new("test_start_session").unwrap();

        let cmd = StartCommand { name: None };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Starting session.");
    }

    #[test]
    fn test_start_command_with_name() {
        let tmp_dir = TempDir::new("test_start_session_named").unwrap();

        let cmd = StartCommand {
            name: Some("my-session".to_string()),
        };
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Starting session.");
    }

    #[test]
    fn test_pause_command_without_active_session() {
        let tmp_dir = TempDir::new("test_pause_session").unwrap();

        let cmd = PauseCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_command_with_active_session() {
        let tmp_dir = TempDir::new("test_pause_session_active").unwrap();

        // Start a session first
        let start_cmd = StartCommand { name: None };
        start_cmd.execute(tmp_dir.path()).unwrap();

        // Then pause it
        let pause_cmd = PauseCommand;
        let result = pause_cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Pausing session.");
    }

    #[test]
    fn test_stop_command_without_active_session() {
        let tmp_dir = TempDir::new("test_stop_session").unwrap();

        let cmd = StopCommand;
        let result = cmd.execute(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_command_with_active_session() {
        let tmp_dir = TempDir::new("test_stop_session_active").unwrap();

        // Start a session first
        let start_cmd = StartCommand { name: None };
        start_cmd.execute(tmp_dir.path()).unwrap();

        // Then stop it
        let stop_cmd = StopCommand;
        let result = stop_cmd.execute(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Stopping session.");
    }
}
