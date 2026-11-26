//! Tests for the mise-based CI/CD workflows: pull-request-ci.yml,
//! push-main.yml and nightly-ci.yml, which all call mise-projects.yml.

use std::fs;
use workflow_tests::*;

fn load(name: &str) -> (String, serde_yaml::Value) {
    let path = get_workflow_path(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let parsed = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    (content, parsed)
}

fn calls_mise_projects(parsed: &serde_yaml::Value) -> serde_yaml::Value {
    let jobs = parsed["jobs"]
        .as_mapping()
        .unwrap_or_else(|| panic!("workflow has no jobs"));
    assert_eq!(jobs.len(), 1, "workflow should have exactly one job");
    let (_, job) = jobs.iter().next().unwrap_or_else(|| panic!("no job"));
    assert_eq!(
        job["uses"].as_str(),
        Some("./.github/workflows/mise-projects.yml"),
        "the job should call mise-projects.yml"
    );
    job.clone()
}

#[test]
fn test_mise_projects_is_reusable_and_runs_mise_ci() {
    let (content, parsed) = load("mise-projects.yml");
    assert!(
        !parsed["on"]["workflow_call"].is_null(),
        "mise-projects.yml should be a reusable workflow"
    );
    assert!(
        content.contains("mise run //:audit"),
        "should run the audit"
    );
    assert!(content.contains("mise run ci"), "should run `mise run ci`");
    assert!(
        content.contains("mise run deploy"),
        "should run `mise run deploy` when asked"
    );
    assert!(
        parsed["jobs"]["project"]["strategy"]["fail-fast"].as_bool() == Some(false),
        "one failing project should not cancel the others"
    );
}

#[test]
fn test_no_workflow_uses_cast() {
    let dir = get_repo_root().join(".github/workflows");
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir: {}", e)) {
        let path = entry.unwrap_or_else(|e| panic!("entry: {}", e)).path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read: {}", e));
        assert!(
            !content.contains("cast ci") && !content.contains("cast cd"),
            "{} still runs Cast",
            path.display()
        );
    }
}

#[test]
fn test_pull_request_ci_checks_affected_projects() {
    let (_, parsed) = load("pull-request-ci.yml");
    assert!(!parsed["on"]["pull_request"].is_null());
    let job = calls_mise_projects(&parsed);
    let base = job["with"]["base"].as_str().unwrap_or_default();
    assert!(
        base.contains("github.base_ref"),
        "should diff against the PR base"
    );
    assert!(job["with"]["deploy"].is_null(), "PRs must never deploy");
}

#[test]
fn test_push_main_runs_ci_and_deploy() {
    let (_, parsed) = load("push-main.yml");
    assert_eq!(
        parsed["on"]["push"]["branches"][0].as_str(),
        Some("main"),
        "should run on pushes to main"
    );
    assert_eq!(
        parsed["concurrency"]["cancel-in-progress"].as_bool(),
        Some(false),
        "a deploy must not be cancelled midway"
    );
    let job = calls_mise_projects(&parsed);
    assert_eq!(job["with"]["deploy"].as_bool(), Some(true));
    assert!(
        job["with"]["base"]
            .as_str()
            .unwrap_or_default()
            .contains("github.event.before"),
        "should only check projects the push changed"
    );
}

#[test]
fn test_nightly_ci_checks_every_project() {
    let (_, parsed) = load("nightly-ci.yml");
    assert!(!parsed["on"]["schedule"].is_null());
    let job = calls_mise_projects(&parsed);
    assert_eq!(
        job["with"]["base"].as_str(),
        Some(""),
        "empty base runs all"
    );
    assert!(job["with"]["deploy"].is_null(), "nightly must not deploy");
}

#[test]
fn test_no_workflow_commits_build_artifacts() {
    let dir = get_repo_root().join(".github/workflows");
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir: {}", e)) {
        let path = entry.unwrap_or_else(|e| panic!("entry: {}", e)).path();
        let content = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !content.contains("artifacts/*"),
            "{} handles committed build artifacts, which the repo no longer keeps",
            path.display()
        );
    }
}
