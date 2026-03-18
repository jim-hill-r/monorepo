/// Utilities for running commands with optional headless (xvfb) support
use std::path::Path;
use std::process::Command;

/// Check if xvfb-run is available on the system
pub fn is_xvfb_available() -> bool {
    Command::new("which")
        .arg("xvfb-run")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Wrap a command to run with xvfb-run if in headless mode and xvfb is available
/// Returns a new Command that will execute the original command with xvfb-run
pub fn wrap_with_xvfb_if_headless(
    cmd: &str,
    args: &[&str],
    working_directory: &Path,
    headless: bool,
) -> Command {
    if headless && is_xvfb_available() {
        let mut xvfb_cmd = Command::new("xvfb-run");
        xvfb_cmd.args([
            "--auto-servernum",
            "--server-args=-screen 0 1920x1080x24",
            cmd,
        ]);
        xvfb_cmd.args(args);
        xvfb_cmd.current_dir(working_directory);
        xvfb_cmd
    } else {
        let mut regular_cmd = Command::new(cmd);
        regular_cmd.args(args);
        regular_cmd.current_dir(working_directory);
        regular_cmd
    }
}
