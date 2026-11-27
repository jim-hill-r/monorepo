//! Tests for .github/workflows/tofu-state-deploy.yml, which deploys the
//! OpenTofu state Worker (tofu_state, decision 0009).

use serde_yaml::Value;
use std::fs;
use workflow_tests::*;

fn workflow() -> Value {
    let text = fs::read_to_string(get_workflow_path("tofu-state-deploy.yml"))
        .expect("read tofu-state-deploy.yml");
    serde_yaml::from_str(&text).expect("parse tofu-state-deploy.yml")
}

fn steps(wf: &Value) -> Vec<Value> {
    wf["jobs"]["deploy"]["steps"]
        .as_sequence()
        .expect("deploy job has steps")
        .clone()
}

fn step_index(wf: &Value, name: &str) -> usize {
    steps(wf)
        .iter()
        .position(|s| s["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no step named {name}"))
}

#[test]
fn deploys_only_from_main_or_by_hand() {
    let wf = workflow();
    let on = &wf["on"];
    let branches: Vec<&str> = on["push"]["branches"]
        .as_sequence()
        .expect("push branches")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(branches, ["main"]);
    assert!(
        on["push"]["paths"]
            .as_sequence()
            .expect("push paths")
            .iter()
            .any(|p| p.as_str() == Some("tofu_state/**"))
    );
    assert!(on.get("workflow_dispatch").is_some());
    assert!(
        on.get("pull_request").is_none(),
        "PRs never deploy or see the write password"
    );
}

#[test]
fn runs_in_the_infra_environment_and_concurrency_group() {
    let wf = workflow();
    let job = &wf["jobs"]["deploy"];
    assert_eq!(job["environment"].as_str(), Some("infra"));
    assert_eq!(wf["concurrency"]["group"].as_str(), Some("infra"));
    assert_eq!(
        wf["concurrency"]["cancel-in-progress"].as_bool(),
        Some(false)
    );
    assert_eq!(wf["permissions"]["contents"].as_str(), Some("read"));
}

#[test]
fn checks_secrets_then_deploys_then_smoke_checks() {
    let wf = workflow();
    let check = step_index(&wf, "Stop when the setup isn't done");
    let deploy = step_index(&wf, "mise run deploy:worker");
    let smoke = step_index(&wf, "Smoke check");
    assert!(check < deploy && deploy < smoke);
    let steps = steps(&wf);
    assert_eq!(
        steps[deploy]["run"].as_str(),
        Some("mise run deploy:worker")
    );
    let smoke_run = steps[smoke]["run"].as_str().expect("smoke run");
    assert!(smoke_run.contains("scripts/origin.sh") && smoke_run.contains("scripts/smoke.sh"));
    // The smoke check only needs to read.
    assert!(
        steps[smoke]["env"]
            .get("TOFU_STATE_WRITE_PASSWORD")
            .is_none()
    );
}

/// The write password may only reach jobs in the `infra` environment, or
/// `infra-credentials` for credential applies (decisions 0009 and 0010), in
/// any workflow.
#[test]
fn write_password_is_only_used_in_the_infra_environment() {
    let dir = get_repo_root().join(".github/workflows");
    for entry in fs::read_dir(&dir).expect("read workflows") {
        let path = entry.expect("entry").path();
        let text = fs::read_to_string(&path).expect("read workflow");
        if !text.contains("TOFU_STATE_WRITE_PASSWORD") {
            continue;
        }
        let wf: Value = serde_yaml::from_str(&text).expect("parse workflow");
        for (name, job) in wf["jobs"].as_mapping().expect("jobs") {
            let job_text = serde_yaml::to_string(job).expect("job yaml");
            if job_text.contains("TOFU_STATE_WRITE_PASSWORD") {
                let env = job["environment"]
                    .as_str()
                    .or_else(|| job["environment"]["name"].as_str());
                assert!(
                    matches!(env, Some("infra" | "infra-credentials")),
                    "{}: job {:?} uses the write password outside infra",
                    path.display(),
                    name
                );
            }
        }
    }
}

#[test]
fn tofu_state_is_not_deployed_by_the_shared_pipeline() {
    // The shared pipeline deploys projects with a DEPLOY_URL in the
    // `production` environment, which must never hold the write password.
    let mise = fs::read_to_string(get_repo_root().join("tofu_state/mise.toml"))
        .expect("read tofu_state/mise.toml");
    assert!(!mise.contains("DEPLOY_URL"));
    // push-main.yml runs `deploy` in every changed project, so tofu_state's
    // must do nothing; the real one is `deploy:worker`.
    let deploy = mise
        .split("[tasks.deploy]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .expect("tofu_state has a deploy task");
    assert!(!deploy.contains("wrangler"), "{deploy}");
    assert!(mise.contains("[tasks.\"deploy:worker\"]"));
}
