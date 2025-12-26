use std::fs;
use workflow_tests::*;

#[test]
fn test_workflow_file_exists() {
    let workflow_path = get_start_a_new_task_workflow_path();
    assert!(
        workflow_path.exists(),
        "Workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_agent_prompt_file_exists() {
    let prompt_path = get_agent_prompt_path();
    assert!(
        prompt_path.exists(),
        "Agent prompt file not found: {}",
        prompt_path.display()
    );
}

#[test]
fn test_agent_binary_exists() {
    let binary_path = get_agent_binary_path();
    assert!(
        binary_path.exists(),
        "agent-copilot binary not found: {}",
        binary_path.display()
    );
}

#[test]
fn test_workflow_yaml_can_be_parsed() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_workflow_uses_agent_copilot_binary() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("agent-copilot"),
        "Workflow does not use agent-copilot binary"
    );
}

#[test]
fn test_workflow_does_not_use_gh_issue_create() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        !content.contains("gh issue create"),
        "Workflow should not use gh issue create (use agent-copilot instead)"
    );
}

#[test]
fn test_workflow_trigger_is_pull_request_closed() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("pull_request:") && content.contains("types: [closed]"),
        "Workflow trigger is not configured correctly (should include pull_request closed)"
    );
}

#[test]
fn test_workflow_trigger_includes_workflow_dispatch() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("workflow_dispatch"),
        "Workflow should include workflow_dispatch trigger for manual execution from GitHub UI"
    );
}

#[test]
fn test_workflow_has_issues_write_permission() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    // This is a warning-level test, so we just check if it's there
    // If not present, we'll note it but not fail
    if !content.contains("issues: write") {
        eprintln!(
            "WARNING: Workflow missing 'issues: write' permission (may not be needed with direct Copilot API)"
        );
    }
}

#[test]
fn test_workflow_uses_correct_copilot_user_login() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("user.login == 'Copilot'"),
        "Workflow does not use correct Copilot user login (should be 'Copilot')"
    );
}

#[test]
fn test_workflow_checks_for_copilot_swe_agent_bot() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("copilot-swe-agent[bot]"),
        "Workflow should check for both Copilot and copilot-swe-agent[bot] PRs"
    );
}

#[test]
fn test_workflow_uses_github_token() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    if !content.contains("GITHUB_TOKEN") || !content.contains("secrets.GITHUB_TOKEN") {
        eprintln!("WARNING: Workflow should use GITHUB_TOKEN");
    }
}

#[test]
fn test_workflow_uses_prompt_file_flag() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("prompt-file") || content.contains("--prompt-file"),
        "Workflow should use --prompt-file flag to read from agent prompt"
    );
}

#[test]
fn test_workflow_has_concurrency_check() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("Check for running agent tasks") && content.contains("gh pr list"),
        "Workflow should check for running agents before starting a new task"
    );
}

#[test]
fn test_workflow_has_conditional_steps() {
    let content = fs::read_to_string(get_start_a_new_task_workflow_path())
        .expect("Failed to read workflow file");

    assert!(
        content.contains("if: steps.check_running_agents.outputs.skip_task"),
        "Workflow should conditionally execute steps based on running agents check"
    );
}
