//! Tests for the infra workflows: infra-plan.yml (PR plans), infra-apply.yml
//! (plan and apply on main) and infra-drift.yml (weekly drift), against
//! decisions 0007, 0009 and 0010.

use serde_yaml::Value;
use std::fs;
use workflow_tests::*;

fn load(name: &str) -> (String, Value) {
    let text = fs::read_to_string(get_workflow_path(name)).expect("read workflow");
    let value = serde_yaml::from_str(&text).expect("parse workflow");
    (text, value)
}

fn job_yaml(wf: &Value, job: &str) -> String {
    serde_yaml::to_string(&wf["jobs"][job]).expect("job yaml")
}

fn env_keys(wf: &Value, job: &str) -> Vec<String> {
    wf["jobs"][job]["env"]
        .as_mapping()
        .map(|m| {
            m.keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn pr_plans_use_only_read_credentials_and_never_lock() {
    let (_, wf) = load("infra-plan.yml");
    assert!(wf["on"].get("pull_request").is_some());
    assert!(
        wf["on"].get("pull_request_target").is_none(),
        "PR code must never run with the base repo's write token"
    );
    let plan = job_yaml(&wf, "plan");
    assert!(plan.contains("secrets.TOFU_STATE_READ_PASSWORD"));
    assert!(!plan.contains("WRITE_PASSWORD"));
    assert!(!plan.contains("_APPLY_") && !plan.contains("_CREDENTIALS_"));
    assert_eq!(
        wf["jobs"]["plan"]["env"]["TOFU_LOCK"].as_str(),
        Some("false")
    );
    assert!(wf["jobs"]["plan"].get("environment").is_none());
    assert!(plan.contains("scripts/tofu.sh plan"));
}

#[test]
fn pr_plan_labels_from_a_job_that_runs_no_pr_code() {
    let (_, wf) = load("infra-plan.yml");
    let label = job_yaml(&wf, "label");
    assert!(
        !label.contains("actions/checkout"),
        "the job with write permissions must not check out the PR"
    );
    assert!(label.contains("credential-change"));
    assert_eq!(wf["permissions"]["contents"].as_str(), Some("read"));
    assert!(wf["jobs"]["plan"].get("permissions").is_none());
}

#[test]
fn plans_skip_until_the_stack_is_set_up() {
    for name in ["infra-plan.yml", "infra-apply.yml", "infra-drift.yml"] {
        let (text, _) = load(name);
        assert!(text.contains("scripts/tofu.sh ready"), "{name}");
        assert!(
            text.contains("steps.ready.outputs.ready == 'true'"),
            "{name}"
        );
    }
}

#[test]
fn main_plans_with_read_credentials_in_infra() {
    let (_, wf) = load("infra-apply.yml");
    let branches = wf["on"]["push"]["branches"]
        .as_sequence()
        .expect("branches");
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].as_str(), Some("main"));
    assert_eq!(wf["jobs"]["plan"]["environment"].as_str(), Some("infra"));
    let keys = env_keys(&wf, "plan");
    assert!(keys.iter().any(|k| k == "AUTH0_INFRA_READ_CLIENT_SECRET"));
    assert!(
        !keys
            .iter()
            .any(|k| k.contains("_APPLY_") || k.contains("_CREDENTIALS_")),
        "the plan job uses read-only tokens (decision 0010): {keys:?}"
    );
    let plan = job_yaml(&wf, "plan");
    assert!(
        !plan.contains("plan.json"),
        "the JSON plan is never uploaded"
    );
}

#[test]
fn credential_changes_apply_only_in_infra_credentials() {
    let (_, wf) = load("infra-apply.yml");
    let apply = &wf["jobs"]["apply"];
    let creds = &wf["jobs"]["apply-credentials"];
    assert_eq!(apply["environment"].as_str(), Some("infra"));
    assert_eq!(creds["environment"].as_str(), Some("infra-credentials"));
    assert!(
        apply["strategy"]["matrix"]["stack"]
            .as_str()
            .expect("apply matrix")
            .contains("needs.route.outputs.apply")
    );
    assert!(
        creds["strategy"]["matrix"]["stack"]
            .as_str()
            .expect("credentials matrix")
            .contains("needs.route.outputs.credentials")
    );
    let apply_keys = env_keys(&wf, "apply");
    assert!(
        !apply_keys.iter().any(|k| k.contains("_CREDENTIALS_")),
        "credential-capable tokens live only in infra-credentials: {apply_keys:?}"
    );
    let cred_keys = env_keys(&wf, "apply-credentials");
    assert!(
        cred_keys
            .iter()
            .any(|k| k == "AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET")
    );
    assert!(!cred_keys.iter().any(|k| k.contains("_APPLY_")));
    // Both apply the saved plan, never a fresh one.
    for job in ["apply", "apply-credentials"] {
        let text = job_yaml(&wf, job);
        assert!(text.contains("scripts/tofu.sh apply"), "{job}");
        assert!(!text.contains("tofu.sh plan"), "{job} must not re-plan");
        assert!(text.contains("download-artifact"), "{job}");
    }
}

#[test]
fn credential_tokens_are_never_used_outside_infra_credentials() {
    let dir = get_repo_root().join(".github/workflows");
    for entry in fs::read_dir(&dir).expect("read workflows") {
        let path = entry.expect("entry").path();
        let text = fs::read_to_string(&path).expect("read workflow");
        if !text.contains("_INFRA_CREDENTIALS_") {
            continue;
        }
        let wf: Value = serde_yaml::from_str(&text).expect("parse workflow");
        for (name, job) in wf["jobs"].as_mapping().expect("jobs") {
            let job_text = serde_yaml::to_string(job).expect("job yaml");
            if job_text.contains("secrets.AUTH0_INFRA_CREDENTIALS_")
                || job_text.contains("secrets.CLOUDFLARE_INFRA_CREDENTIALS_")
                || job_text.contains("secrets.GH_INFRA_CREDENTIALS_")
            {
                assert_eq!(
                    job["environment"].as_str(),
                    Some("infra-credentials"),
                    "{}: job {:?}",
                    path.display(),
                    name
                );
            }
        }
    }
}

#[test]
fn state_jobs_share_the_infra_concurrency_group() {
    for name in [
        "infra-apply.yml",
        "infra-drift.yml",
        "tofu-state-deploy.yml",
    ] {
        let (_, wf) = load(name);
        assert_eq!(wf["concurrency"]["group"].as_str(), Some("infra"), "{name}");
        assert_eq!(
            wf["concurrency"]["cancel-in-progress"].as_bool(),
            Some(false),
            "{name}: an apply must never be cancelled halfway"
        );
    }
    let (_, plan) = load("infra-plan.yml");
    assert!(
        plan.get("concurrency").is_none(),
        "PR plans don't lock, and a shared group would cancel queued applies"
    );
}

#[test]
fn drift_is_weekly_read_only_and_unlocked() {
    let (_, wf) = load("infra-drift.yml");
    assert!(wf["on"]["schedule"].as_sequence().is_some());
    let drift = job_yaml(&wf, "drift");
    assert!(drift.contains("secrets.TOFU_STATE_READ_PASSWORD"));
    assert!(!drift.contains("WRITE_PASSWORD") && !drift.contains("_APPLY_"));
    assert!(drift.contains("scripts/tofu.sh drift"));
    assert!(wf["jobs"]["drift"].get("environment").is_none());
}

#[test]
fn mint_runs_by_hand_from_main_in_infra_credentials() {
    let (_, wf) = load("infra-mint.yml");
    let on = wf["on"].as_mapping().expect("on");
    assert_eq!(
        on.len(),
        2,
        "only a person or the quarterly schedule may start a mint"
    );
    assert!(wf["on"].get("workflow_dispatch").is_some());
    assert!(wf["on"]["schedule"].as_sequence().is_some());
    let mint = job_yaml(&wf, "mint");
    assert!(
        mint.contains("github.event_name == 'schedule' && 'auto'"),
        "the scheduled run rotates the quarterly set"
    );
    let job = &wf["jobs"]["mint"];
    assert_eq!(job["environment"].as_str(), Some("infra-credentials"));
    assert_eq!(job["if"].as_str(), Some("github.ref == 'refs/heads/main'"));
    assert_eq!(wf["concurrency"]["group"].as_str(), Some("infra"));
    let text = job_yaml(&wf, "mint");
    assert!(text.contains("actions/create-github-app-token"));
    assert!(text.contains("secrets.GH_INFRA_CREDENTIALS_APP_KEY"));
    assert!(text.contains("infra/scripts/mint.sh run"));
    // Inputs reach the script through the environment, never pasted into
    // the shell command.
    for step in job["steps"].as_sequence().expect("steps") {
        if let Some(run) = step["run"].as_str() {
            assert!(!run.contains("${{"), "expression in a run script: {run}");
        }
    }
}

#[test]
fn every_infra_credentials_job_first_checks_its_approval_gate() {
    // Decision 0010. GitHub runs a job in an environment with no saved
    // reviewer without asking anyone, as the first Infra Mint runs did, so
    // every such job stops itself unless env-gate.sh finds the gate.
    let dir = get_workflow_path("infra-mint.yml")
        .parent()
        .expect("workflows dir")
        .to_path_buf();
    let mut found = 0;
    for entry in fs::read_dir(&dir).expect("read workflows") {
        let path = entry.expect("entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }
        let (_, wf) = load(name);
        let Some(jobs) = wf["jobs"].as_mapping() else {
            continue;
        };
        for (id, job) in jobs {
            let env = job["environment"]
                .as_str()
                .or_else(|| job["environment"]["name"].as_str());
            if env != Some("infra-credentials") {
                continue;
            }
            found += 1;
            let steps = job["steps"].as_sequence().expect("steps");
            let first_run = steps.iter().find_map(|s| s["run"].as_str()).unwrap_or("");
            assert!(
                first_run.contains("infra/scripts/env-gate.sh infra-credentials"),
                "{name} job {id:?}: the first run step must be env-gate.sh, got {first_run:?}"
            );
        }
    }
    assert!(found >= 2, "expected the mint and apply-credentials jobs");
}

#[test]
fn the_github_stack_plans_with_the_read_app_and_applies_with_the_infra_app() {
    // Plans and drift mint a token from the read-only infra GitHub App; only
    // the infra-credentials apply may mint one from the infra GitHub App.
    for (name, job) in [
        ("infra-plan.yml", "plan"),
        ("infra-apply.yml", "plan"),
        ("infra-drift.yml", "drift"),
    ] {
        let (_, wf) = load(name);
        let text = job_yaml(&wf, job);
        assert!(text.contains("actions/create-github-app-token"), "{name}");
        assert!(text.contains("secrets.GH_INFRA_READ_APP_KEY"), "{name}");
        assert!(text.contains("GH_INFRA_READ_TOKEN"), "{name}");
        assert!(!text.contains("GH_INFRA_CREDENTIALS_"), "{name} {job}");
    }
    let (_, wf) = load("infra-apply.yml");
    let apply = job_yaml(&wf, "apply");
    assert!(
        !apply.contains("GH_INFRA_"),
        "the github stack has no everyday apply (decision 0010)"
    );
    let creds = job_yaml(&wf, "apply-credentials");
    assert!(creds.contains("secrets.GH_INFRA_CREDENTIALS_APP_KEY"));
    assert!(creds.contains("GH_INFRA_CREDENTIALS_TOKEN"));
    // PR plans from forks get no secrets, so they skip minting.
    let (_, pr) = load("infra-plan.yml");
    let step = pr["jobs"]["plan"]["steps"]
        .as_sequence()
        .expect("steps")
        .iter()
        .find(|s| s["id"].as_str() == Some("gh-read"))
        .expect("gh-read step");
    assert!(
        step["if"]
            .as_str()
            .unwrap_or_default()
            .contains("head.repo.full_name == github.repository")
    );
}
