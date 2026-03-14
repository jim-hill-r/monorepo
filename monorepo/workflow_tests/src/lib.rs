// Workflow tests library
// This library provides utilities for testing GitHub Actions workflows

use std::env;
use std::path::PathBuf;

/// Get the repository root directory by walking up from current directory
/// until we find the .github directory
pub fn get_repo_root() -> PathBuf {
    let mut current =
        env::current_dir().unwrap_or_else(|e| panic!("Failed to get current directory: {}", e));

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

/// Get the path to the cast-ci.yml workflow file (legacy - now split into PR and Trunk CI)
pub fn get_cast_ci_workflow_path() -> PathBuf {
    // For backwards compatibility, return pull-request-ci.yml path
    get_pull_request_ci_workflow_path()
}

/// Get the path to the pull-request-ci.yml workflow file
pub fn get_pull_request_ci_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/pull-request-ci.yml")
}

/// Get the path to the trunk-ci.yml workflow file
pub fn get_trunk_ci_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/trunk-ci.yml")
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
    get_repo_root().join("cast/cli/Cargo.toml")
}

/// Get the path to the cast-cd.yml workflow file
pub fn get_cast_cd_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/cast-cd.yml")
}

/// Get the path to the codeql.yml workflow file
pub fn get_codeql_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/codeql.yml")
}

/// Get the path to the standards-audit.yml workflow file
pub fn get_standards_audit_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/standards-audit.yml")
}
