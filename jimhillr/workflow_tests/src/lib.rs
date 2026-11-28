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

/// Get the path to a workflow file by name, e.g. `pull-request-ci.yml`
pub fn get_workflow_path(name: &str) -> PathBuf {
    get_repo_root().join(".github/workflows").join(name)
}

/// Get the path to the all-codeql.yml workflow file
pub fn get_codeql_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/all-codeql.yml")
}

/// Get the path to the all-delete-branches.yml workflow file
pub fn get_delete_branches_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/all-delete-branches.yml")
}

/// Get the path to the all-retire-resources.yml workflow file
pub fn get_retire_resources_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/all-retire-resources.yml")
}

/// Get the path to the pull-request-agent-skills.yml workflow file
pub fn get_agent_skills_workflow_path() -> PathBuf {
    get_repo_root().join(".github/workflows/pull-request-agent-skills.yml")
}
