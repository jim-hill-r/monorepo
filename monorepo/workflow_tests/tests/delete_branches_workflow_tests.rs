use std::fs;
use workflow_tests::*;

fn workflow() -> (String, serde_yaml::Value) {
    let content = fs::read_to_string(get_delete_branches_workflow_path())
        .expect("Failed to read all-delete-branches.yml");
    let parsed = serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
    (content, parsed)
}

#[test]
fn test_workflow_is_manual_only() {
    let (_, yaml) = workflow();
    let on = &yaml["on"];
    assert!(
        on.get("workflow_dispatch").is_some(),
        "Workflow must be triggered by workflow_dispatch"
    );
    let triggers = on.as_mapping().expect("'on' must be a mapping");
    assert_eq!(
        triggers.len(),
        1,
        "Workflow must only run on workflow_dispatch"
    );
}

#[test]
fn test_dry_run_defaults_to_true() {
    let (_, yaml) = workflow();
    let dry_run = &yaml["on"]["workflow_dispatch"]["inputs"]["dry_run"];
    assert_eq!(dry_run["default"], serde_yaml::Value::Bool(true));
    let branches = &yaml["on"]["workflow_dispatch"]["inputs"]["branches"];
    assert_eq!(branches["required"], serde_yaml::Value::Bool(true));
}

#[test]
fn test_permissions_are_minimal() {
    let (_, yaml) = workflow();
    let perms = yaml["permissions"]
        .as_mapping()
        .expect("permissions must be set");
    assert_eq!(
        perms.len(),
        2,
        "Only contents and pull-requests permissions expected"
    );
    assert_eq!(yaml["permissions"]["contents"], "write");
    assert_eq!(yaml["permissions"]["pull-requests"], "read");
}

#[test]
fn test_protects_default_branch_and_open_prs() {
    let (content, _) = workflow();
    assert!(
        content.contains("DEFAULT_BRANCH"),
        "Must guard the default branch"
    );
    assert!(
        content.contains("state=open"),
        "Must skip branches with open PRs"
    );
}

#[test]
fn test_inputs_passed_via_env_not_inline() {
    let (content, _) = workflow();
    let run_start = content.find("run: |").expect("run step missing");
    assert!(
        !content[run_start..].contains("${{"),
        "Inputs must reach the script through env, not inline expressions"
    );
}
