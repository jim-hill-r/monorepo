//! Tests for telemetry-feedback-export.yml: the weekly export of fan
//! feedback from Analytics Engine to the feedback-data data branch.

use std::fs;
use workflow_tests::*;

fn load() -> (String, serde_yaml::Value) {
    let path = get_workflow_path("telemetry-feedback-export.yml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let parsed = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    (content, parsed)
}

#[test]
fn test_runs_weekly_and_by_hand_never_on_pull_requests() {
    let (_, parsed) = load();
    let on = &parsed["on"];
    assert!(!on["schedule"].is_null(), "should run on a schedule");
    assert!(on.get("workflow_dispatch").is_some(), "should run by hand");
    assert!(
        on["pull_request"].is_null(),
        "should not run on pull requests"
    );
    assert!(on["push"].is_null(), "should not run on pushes");
}

#[test]
fn test_reads_by_default_and_runs_only_on_main() {
    let (_, parsed) = load();
    assert_eq!(parsed["permissions"]["contents"].as_str(), Some("read"));
    assert_eq!(
        parsed["jobs"]["export"]["if"].as_str(),
        Some("github.ref == 'refs/heads/main'"),
        "only main's copy of the script may push"
    );
}

#[test]
fn test_uses_the_feedback_script_and_never_pushes_itself() {
    let (content, _) = load();
    assert!(content.contains("standards/scripts/feedback.sh query bear_down"));
    assert!(content.contains("standards/scripts/feedback.sh store bear_down"));
    assert!(
        !content.contains("git push"),
        "the workflow never pushes itself; feedback.sh store does (SEC-007)"
    );
}
