use std::fs;
use workflow_tests::*;

fn workflow() -> (String, serde_yaml::Value) {
    let content = fs::read_to_string(get_workflow_path("pull-request-closed-cleanup.yml"))
        .expect("Failed to read pull-request-closed-cleanup.yml");
    let parsed = serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
    (content, parsed)
}

#[test]
fn test_runs_only_when_a_pull_request_closes() {
    let (_, yaml) = workflow();
    let on = yaml["on"].as_mapping().expect("'on' must be a mapping");
    assert_eq!(on.len(), 1, "Workflow must only run on pull_request");
    let types = yaml["on"]["pull_request"]["types"]
        .as_sequence()
        .expect("pull_request must list its types");
    assert_eq!(types, &vec![serde_yaml::Value::from("closed")]);
}

#[test]
fn test_never_uses_pull_request_target() {
    let (content, _) = workflow();
    assert!(
        !content.contains("pull_request_target"),
        "pull_request_target would give fork PRs a write token"
    );
}

#[test]
fn test_permissions_are_actions_write_only() {
    let (_, yaml) = workflow();
    let perms = yaml["permissions"]
        .as_mapping()
        .expect("permissions must be set");
    assert_eq!(perms.len(), 1, "Only the actions permission is expected");
    assert_eq!(yaml["permissions"]["actions"], "write");
    for (_, job) in yaml["jobs"].as_mapping().expect("jobs must be a mapping") {
        assert!(
            job.get("permissions").is_none(),
            "Jobs must not widen the workflow permissions"
        );
    }
}

#[test]
fn test_deletes_only_this_pull_requests_merge_ref_caches() {
    let (content, _) = workflow();
    assert!(
        content.contains(r#"ref="refs/pull/$PR_NUMBER/merge""#),
        "Must target only refs/pull/N/merge"
    );
    assert!(
        content.contains(r#"select(.ref == \"$ref\")"#),
        "Must re-check each cache's ref exactly before deleting it"
    );
    assert!(content.contains("gh cache delete"));
    assert!(
        !content.contains("--all"),
        "Must never delete every cache in the repo"
    );
}

#[test]
fn test_does_not_check_out_code() {
    let (content, _) = workflow();
    assert!(
        !content.contains("actions/checkout"),
        "The job needs no code, so it must not check any out"
    );
}

#[test]
fn test_inputs_passed_via_env_not_inline() {
    let (content, _) = workflow();
    let run_start = content.find("run: |").expect("run step missing");
    assert!(
        !content[run_start..].contains("${{"),
        "Event values must reach the script through env, not inline expressions"
    );
}

#[test]
fn test_writes_a_run_summary() {
    let (content, _) = workflow();
    assert!(content.contains("GITHUB_STEP_SUMMARY"));
}
