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
    assert!(
        parsed["concurrency"].is_null(),
        "a concurrency group would cancel queued pushes and skip their CI and deploys"
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

/// Projects that publish a site: they include the shared cloudflare-pages tasks.
fn deploying_projects() -> Vec<std::path::PathBuf> {
    let root = get_repo_root();
    let mut found = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir: {}", e)) {
            let path = entry.unwrap_or_else(|e| panic!("entry: {}", e)).path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    stack.push(path);
                }
            } else if name == "mise.toml" {
                let content = fs::read_to_string(&path).unwrap_or_default();
                if content.contains("standards/tasks/cloudflare-pages.toml") {
                    found.push(path);
                }
            }
        }
    }
    found.sort();
    found
}

/// The string value of `key` in the `[env]` table of a mise.toml.
fn env_value(content: &str, key: &str) -> Option<String> {
    let mut in_env = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_env = line == "[env]";
            continue;
        }
        if !in_env {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

#[test]
fn test_every_deploying_project_records_its_production_url() {
    let projects = deploying_projects();
    assert!(
        !projects.is_empty(),
        "expected at least one cloudflare project"
    );
    for path in projects {
        let content = fs::read_to_string(&path).unwrap_or_default();
        let url = env_value(&content, "DEPLOY_URL")
            .unwrap_or_else(|| panic!("{} should set [env] DEPLOY_URL", path.display()));
        assert!(
            url.starts_with("https://") && !url.ends_with('/'),
            "{}: DEPLOY_URL should be an https URL without a trailing slash, got {url}",
            path.display()
        );
    }
}

#[test]
fn test_deploys_use_the_production_environment_with_its_url() {
    let (content, parsed) = load("mise-projects.yml");
    let env = &parsed["jobs"]["project"]["environment"];
    let name = env["name"].as_str().unwrap_or_default();
    assert!(
        name.contains("'production'") && name.contains("inputs.deploy"),
        "deploying jobs should use the production environment, got {name:?}"
    );
    let url = env["url"].as_str().unwrap_or_default();
    assert!(
        url.contains("deploy_urls"),
        "the environment should link the project's DEPLOY_URL, got {url:?}"
    );
    assert!(
        content.contains("DEPLOY_URL"),
        "the plan job should read each project's DEPLOY_URL"
    );
}

#[test]
fn test_deploy_writes_the_url_to_the_job_summary() {
    let (_, parsed) = load("mise-projects.yml");
    let steps = parsed["jobs"]["project"]["steps"]
        .as_sequence()
        .unwrap_or_else(|| panic!("project job has no steps"));
    let deploy = steps
        .iter()
        .find(|s| {
            s["run"]
                .as_str()
                .is_some_and(|r| r.contains("mise run deploy"))
        })
        .unwrap_or_else(|| panic!("no deploy step"));
    let run = deploy["run"].as_str().unwrap_or_default();
    assert!(
        run.contains("GITHUB_STEP_SUMMARY"),
        "deploy should write a job summary"
    );
    assert!(
        run.contains("pipefail"),
        "a failed deploy must still fail the step when its output is piped"
    );
}

#[test]
fn test_push_main_can_redeploy_one_project_at_a_ref() {
    let (_, push) = load("push-main.yml");
    let inputs = &push["on"]["workflow_dispatch"]["inputs"];
    assert!(
        !inputs["project"].is_null(),
        "dispatch needs a project input"
    );
    assert!(!inputs["ref"].is_null(), "dispatch needs a ref input");
    let job = calls_mise_projects(&push);
    assert!(
        job["with"]["project"]
            .as_str()
            .unwrap_or_default()
            .contains("inputs.project"),
        "the project input should be passed on"
    );
    assert!(
        job["with"]["ref"]
            .as_str()
            .unwrap_or_default()
            .contains("inputs.ref"),
        "the ref input should be passed on"
    );

    let (content, projects) = load("mise-projects.yml");
    let call_inputs = &projects["on"]["workflow_call"]["inputs"];
    assert!(!call_inputs["project"].is_null());
    assert!(!call_inputs["ref"].is_null());
    let checkouts = content.matches("ref: ${{ inputs.ref }}").count();
    assert_eq!(
        checkouts, 2,
        "both the plan and project jobs should check out the requested ref"
    );
}
