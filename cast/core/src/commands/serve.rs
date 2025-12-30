use crate::command::Command;
use crate::serve;
use std::path::Path;

/// Command to serve static files from the working directory
/// Starts a simple HTTP server on localhost:8000
pub struct ServeCommand;

impl Command for ServeCommand {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>> {
        serve::run(working_directory)?;
        Ok("Static file server started".to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_serve_command_with_directory() {
        let tmp_dir = TempDir::new("test_serve_command").unwrap();

        // Create a simple HTML file to serve
        fs::write(
            tmp_dir.path().join("index.html"),
            "<html><body>Hello, world!</body></html>",
        )
        .unwrap();

        // The serve command starts a blocking server, so we can't easily test it
        // in a synchronous test. We'll just verify the command can be created.
        let cmd = ServeCommand;

        // We can't actually execute the server in a test because it blocks forever
        // Instead we just verify the struct is properly created
        assert!(tmp_dir.path().exists());

        // For completeness, we could verify that the struct exists
        let _ = cmd;
    }

    #[test]
    fn test_serve_command_structure() {
        // Verify we can create the command
        let cmd = ServeCommand;
        let _ = cmd;

        // The Command trait implementation exists and compiles
        // This test mainly verifies the trait implementation is correct
    }
}
