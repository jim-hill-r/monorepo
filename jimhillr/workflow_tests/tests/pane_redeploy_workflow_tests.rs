//! Tests for .github/workflows/pane-redeploy.yml, which redeploys pane after
//! each run that stores eval results, so its Requirements section is current.

use serde_yaml::Value;
use std::fs;
use workflow_tests::*;

fn workflow() -> Value {
    let text =
        fs::read_to_string(get_workflow_path("pane-redeploy.yml")).expect("read pane-redeploy.yml");
    serde_yaml::from_str(&text).expect("parse pane-redeploy.yml")
}

fn names(v: &Value) -> Vec<String> {
    v.as_sequence()
        .expect("a list")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// The `name:` of a workflow file.
fn workflow_name(file: &str) -> String {
    let text = fs::read_to_string(get_workflow_path(file)).expect("read workflow");
    let wf: Value = serde_yaml::from_str(&text).expect("parse workflow");
    wf["name"].as_str().expect("workflow has a name").to_owned()
}

#[test]
fn runs_after_every_workflow_that_stores_eval_results_on_main() {
    let wf = workflow();
    let run = &wf["on"]["workflow_run"];
    let mut expected: Vec<String> = ["push-main.yml", "nightly-ci.yml", "nightly-cd.yml"]
        .iter()
        .map(|f| workflow_name(f))
        .collect();
    let mut found = names(&run["workflows"]);
    expected.sort();
    found.sort();
    assert_eq!(
        found, expected,
        "names must match the storing workflows' names"
    );
    for file in ["push-main.yml", "nightly-ci.yml", "nightly-cd.yml"] {
        let text = fs::read_to_string(get_workflow_path(file)).expect("read workflow");
        assert!(
            text.contains("./.github/workflows/evals-store.yml"),
            "{file} no longer stores eval results; update pane-redeploy.yml"
        );
    }
    assert_eq!(names(&run["types"]), ["completed"]);
    assert_eq!(names(&run["branches"]), ["main"]);
    assert!(wf["on"].get("pull_request").is_none(), "PRs never deploy");
}

#[test]
fn deploys_only_pane_with_the_newest_run_winning() {
    let wf = workflow();
    let job = &wf["jobs"]["redeploy"];
    assert_eq!(
        job["uses"].as_str(),
        Some("./.github/workflows/mise-projects.yml")
    );
    assert_eq!(job["with"]["project"].as_str(), Some("pane/cloudflare"));
    assert_eq!(job["with"]["deploy"].as_bool(), Some(true));
    assert_eq!(wf["concurrency"]["group"].as_str(), Some("pane-redeploy"));
    assert_eq!(
        wf["concurrency"]["cancel-in-progress"].as_bool(),
        Some(true)
    );
    assert_eq!(wf["permissions"]["contents"].as_str(), Some("read"));
}
