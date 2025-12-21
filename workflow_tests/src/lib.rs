// Workflow tests library
// This library provides utilities for testing GitHub Actions workflows

use std::env;
use std::path::PathBuf;

/// Get the repository root directory by walking up from current directory
/// until we find the .github directory
pub fn get_repo_root() -> PathBuf {
    let mut current = env::current_dir().expect("Failed to get current directory");

    // Walk up until we find .github directory
    loop {
        if current.join(".github").exists() {
            return current;
        }
        if !current.pop() {
            panic!("Could not find repository root");
        }
    }
}

/// Get the path to the cast-ci.yml workflow file
pub fn get_cast_ci_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/cast-ci.yml")
}

/// Get the path to the start-a-new-task.yml workflow file
pub fn get_start_a_new_task_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/start-a-new-task.yml")
}

/// Get the path to the agent prompt file
pub fn get_agent_prompt_path() -> PathBuf {
    get_repo_root().join("agent-copilot/prompts/start-a-new-task.md")
}

/// Get the path to the agent-copilot binary
pub fn get_agent_binary_path() -> PathBuf {
    get_repo_root().join("agent-copilot/artifacts/x86_64-unknown-linux-gnu/agent-copilot")
}

/// Get the path to the cast_cli Cargo.toml
pub fn get_cast_cli_cargo_path() -> PathBuf {
    get_repo_root().join("cast_cli/Cargo.toml")
}

/// Get the path to the cast-cd.yml workflow file
pub fn get_cast_cd_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/cast-cd.yml")
}
