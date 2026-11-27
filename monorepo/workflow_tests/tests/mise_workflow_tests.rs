//! Tests for the mise-based CI/CD workflows: pull-request-ci.yml,
//! push-main.yml, nightly-ci.yml and nightly-cd.yml, which all call
//! mise-projects.yml, and evals-store.yml, which stores their eval results.

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
    let callers: Vec<&serde_yaml::Value> = jobs
        .values()
        .filter(|job| job["uses"].as_str() == Some("./.github/workflows/mise-projects.yml"))
        .collect();
    assert_eq!(
        callers.len(),
        1,
        "exactly one job should call mise-projects.yml"
    );
    for (name, job) in jobs {
        let uses = job["uses"].as_str();
        assert!(
            uses == Some("./.github/workflows/mise-projects.yml")
                || uses == Some("./.github/workflows/evals-store.yml"),
            "job {name:?} should call mise-projects.yml or evals-store.yml"
        );
    }
    callers[0].clone()
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
    let job = parsed["jobs"]["ci"].clone();
    assert_eq!(
        job["uses"].as_str(),
        Some("./.github/workflows/mise-projects.yml"),
        "the ci job should call mise-projects.yml"
    );
    let base = job["with"]["base"].as_str().unwrap_or_default();
    assert!(
        base.contains("github.base_ref"),
        "should diff against the PR base"
    );
    assert!(job["with"]["deploy"].is_null(), "PRs must never deploy");
}

#[test]
fn test_pull_request_ci_writes_the_cost_summary_and_labels() {
    let (_, parsed) = load("pull-request-ci.yml");
    let job = &parsed["jobs"]["cost"];
    assert_eq!(job["name"].as_str(), Some("Cost summary"));
    assert_eq!(
        job["permissions"]["pull-requests"].as_str(),
        Some("write"),
        "the job labels the PR"
    );
    let steps = serde_yaml::to_string(&job["steps"]).unwrap_or_default();
    assert!(
        steps.contains("summary --base") && steps.contains("GITHUB_STEP_SUMMARY"),
        "the job should write `cost summary` to the job summary"
    );
    assert!(
        steps.contains("needs-human") && steps.contains("cost"),
        "the job should set the cost and needs-human labels"
    );
}

#[test]
fn test_pull_request_ci_has_one_requirable_result_check() {
    let (_, parsed) = load("pull-request-ci.yml");
    let jobs = parsed["jobs"]
        .as_mapping()
        .unwrap_or_else(|| panic!("no jobs"));
    assert_eq!(
        jobs.len(),
        5,
        "expected the ci job, the secret scan, the dependency audit, the cost summary and the CI result gate"
    );
    let gate = &parsed["jobs"]["ci-result"];
    assert_eq!(
        gate["name"].as_str(),
        Some("CI result"),
        "the ruleset on main requires this check name; rename both together"
    );
    let needs: Vec<&str> = gate["needs"]
        .as_sequence()
        .unwrap_or_else(|| panic!("the gate should need a list of jobs"))
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        needs,
        ["ci", "secrets"],
        "the gate waits for CI and the scan"
    );
    assert_eq!(
        gate["if"].as_str(),
        Some("always()"),
        "the gate must report even when a project fails"
    );
    let steps = serde_yaml::to_string(&gate["steps"]).unwrap_or_default();
    assert!(
        steps.contains("needs.ci.result")
            && steps.contains("needs.secrets.result")
            && steps.contains("= success"),
        "the gate should fail unless the ci job and the secret scan succeeded"
    );
}

#[test]
fn test_pull_request_ci_scans_the_pr_commits_for_secrets() {
    let (_, parsed) = load("pull-request-ci.yml");
    let job = &parsed["jobs"]["secrets"];
    let steps = serde_yaml::to_string(&job["steps"]).unwrap_or_default();
    assert!(
        steps.contains("fetch-depth: 0"),
        "the scan needs the PR's commits, so a full clone"
    );
    assert!(
        steps.contains("secrets-check.sh range \"origin/$BASE_REF\""),
        "PRs scan only their own commits (SEC-001)"
    );
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
    assert_eq!(parsed["name"].as_str(), Some("Nightly CI"));
    assert!(!parsed["on"]["schedule"].is_null());
    let job = calls_mise_projects(&parsed);
    assert_eq!(
        job["with"]["base"].as_str(),
        Some(""),
        "empty base runs all"
    );
    assert!(
        job["with"]["deploy"].is_null(),
        "deploys belong to nightly-cd.yml"
    );
}

#[test]
fn test_nightly_cd_redeploys_every_deploying_project() {
    let (_, parsed) = load("nightly-cd.yml");
    assert_eq!(parsed["name"].as_str(), Some("Nightly CD"));
    assert!(!parsed["on"]["schedule"].is_null());
    let job = calls_mise_projects(&parsed);
    assert_eq!(job["with"]["base"].as_str(), Some(""));
    assert_eq!(job["with"]["deploy"].as_bool(), Some(true));
    assert_eq!(
        job["with"]["deploy_only"].as_bool(),
        Some(true),
        "only projects with DEPLOY_URL"
    );
    for secret in ["CLOUDFLARE_API_TOKEN", "CLOUDFLARE_ACCOUNT_ID"] {
        assert!(
            job["secrets"][secret]
                .as_str()
                .is_some_and(|v| v.contains(secret)),
            "nightly deploy needs the {secret} secret"
        );
    }

    let (content, projects) = load("mise-projects.yml");
    assert!(!projects["on"]["workflow_call"]["inputs"]["deploy_only"].is_null());
    assert!(
        content.contains("DEPLOY_ONLY"),
        "the plan job should narrow the list to deploying projects"
    );
}

#[test]
fn test_readme_has_nightly_ci_and_cd_badges() {
    let readme = fs::read_to_string(get_repo_root().join("README.md"))
        .unwrap_or_else(|e| panic!("read README.md: {e}"));
    for (label, file) in [
        ("Nightly CI", "nightly-ci.yml"),
        ("Nightly CD", "nightly-cd.yml"),
    ] {
        let badge = format!(
            "[![{label}](https://github.com/jimhillr/monorepo/actions/workflows/{file}/badge.svg)]"
        );
        assert!(
            readme.contains(&badge),
            "README should have the {label} badge"
        );
        assert!(
            get_workflow_path(file).exists(),
            "{file} should exist for its badge"
        );
    }
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
    let check = run
        .find("deploy-check.sh")
        .unwrap_or_else(|| panic!("deploy should first check it is the latest version of main"));
    let deploy = run.find("mise run deploy").unwrap_or_default();
    assert!(check < deploy, "the check must run before the deploy");
    let ci = steps
        .iter()
        .position(|s| s["run"].as_str() == Some("mise run ci"))
        .unwrap_or_else(|| panic!("no ci step"));
    let deploy_step = steps
        .iter()
        .position(|s| {
            s["run"]
                .as_str()
                .is_some_and(|r| r.contains("mise run deploy"))
        })
        .unwrap_or_default();
    assert!(ci < deploy_step, "deploy only runs after CI passed");
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

/// The `steps` of a job in mise-projects.yml, by job name.
fn mise_projects_steps(job: &str) -> Vec<serde_yaml::Value> {
    let (_, parsed) = load("mise-projects.yml");
    parsed["jobs"][job]["steps"]
        .as_sequence()
        .unwrap_or_else(|| panic!("mise-projects.yml job {job} has no steps"))
        .clone()
}

fn step_index(steps: &[serde_yaml::Value], name: &str) -> usize {
    steps
        .iter()
        .position(|s| s["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no step named {name:?}"))
}

#[test]
fn test_evals_run_after_ci_and_publish_their_results() {
    let steps = mise_projects_steps("project");
    let ci = step_index(&steps, "mise run ci");
    let evals = step_index(&steps, "mise run evals");
    let upload = step_index(&steps, "Upload eval results");
    assert!(ci < evals && evals < upload, "ci, then evals, then upload");

    let step = &steps[evals];
    assert_eq!(step["env"]["EVALS_STAGE"].as_str(), Some("ci"));
    assert_eq!(
        step["if"].as_str(),
        Some("${{ !cancelled() }}"),
        "evals should run after a failed ci, so failing evals show"
    );
    assert_eq!(
        step["continue-on-error"].as_bool(),
        Some(true),
        "the eval step must not fail the job"
    );
    let run = step["run"].as_str().unwrap_or_default();
    assert!(run.contains("mise run evals"), "should run the evals task");
    assert!(
        run.contains("standards/scripts/evals.sh\" publish"),
        "should write the job summary"
    );

    let upload = &steps[upload];
    assert_eq!(upload["with"]["retention-days"].as_u64(), Some(14));
    assert!(
        upload["uses"]
            .as_str()
            .unwrap_or_default()
            .starts_with("actions/upload-artifact@")
    );
}

#[test]
fn test_rust_caches_include_the_evals_cli() {
    for job in ["plan", "project"] {
        let steps = mise_projects_steps(job);
        let cache = steps
            .iter()
            .find(|s| {
                s["uses"]
                    .as_str()
                    .is_some_and(|u| u.starts_with("Swatinem/rust-cache@"))
            })
            .unwrap_or_else(|| panic!("job {job} has no rust-cache step"));
        let workspaces = cache["with"]["workspaces"].as_str().unwrap_or_default();
        assert!(
            workspaces.lines().any(|l| l.trim() == "evals"),
            "job {job} should cache evals/ (got {workspaces:?})"
        );
        assert_eq!(
            cache["with"]["save-if"].as_str(),
            Some("${{ github.ref == 'refs/heads/main' }}"),
            "job {job} should save its cache from main only"
        );
    }
}

#[test]
fn test_every_shared_task_file_has_an_evals_task_and_ci_saves_cargo_test() {
    let dir = get_repo_root().join("standards/tasks");
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{e}")) {
        let path = entry.unwrap_or_else(|e| panic!("{e}")).path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        let name = path.display();
        assert!(
            content.lines().any(|l| l == "[evals]"),
            "{name} has no evals task"
        );
        // The [ci] table runs up to the next table header.
        let ci = content
            .split("\n[ci]\n")
            .nth(1)
            .and_then(|rest| rest.split("\n[").next())
            .unwrap_or_else(|| panic!("{name} has no ci task"));
        if ci.contains("cargo test") {
            assert!(
                ci.contains("evals.sh\\\" report cargo-test"),
                "{name}: ci should save `cargo test` output with evals.sh report"
            );
        }
    }
}

#[test]
fn test_push_main_deploys_every_push_projects() {
    let (content, _) = load("mise-projects.yml");
    assert!(
        content.contains("AFFECTED_DEPLOY: ${{ inputs.deploy && '1' || '' }}"),
        "deploy runs should list projects marked `# deploy-on-every-push`"
    );
    let pane = fs::read_to_string(get_repo_root().join("pane/cloudflare/mise.toml"))
        .unwrap_or_else(|e| panic!("read pane/cloudflare/mise.toml: {e}"));
    assert!(
        pane.lines()
            .any(|l| l.starts_with("# deploy-on-every-push")),
        "pane's snapshot reads the whole repo, so it deploys on every push"
    );
}

/// Stores eval results after the projects run: push-main and nightly CI only.
fn stores_evals(name: &str, mise_job: &str) {
    let (_, parsed) = load(name);
    let job = &parsed["jobs"]["store-evals"];
    assert_eq!(
        job["uses"].as_str(),
        Some("./.github/workflows/evals-store.yml"),
        "{name} should store eval results"
    );
    assert_eq!(job["needs"].as_str(), Some(mise_job));
    assert!(
        job["if"]
            .as_str()
            .unwrap_or_default()
            .contains("!cancelled()"),
        "results are stored even when a project failed"
    );
    assert_eq!(job["permissions"]["contents"].as_str(), Some("write"));
    assert_eq!(
        parsed["permissions"]["contents"].as_str(),
        Some("read"),
        "{name}: only the store-evals job may write"
    );
    assert!(
        parsed["jobs"][mise_job]["permissions"].is_null(),
        "{name}: the mise-projects.yml call stays read-only"
    );
}

#[test]
fn test_push_main_and_nightly_ci_store_eval_results() {
    stores_evals("push-main.yml", "ci-cd");
    stores_evals("nightly-ci.yml", "ci");
    stores_evals("nightly-cd.yml", "cd");
    let (_, parsed) = load("nightly-cd.yml");
    assert_eq!(
        parsed["jobs"]["store-evals"]["with"]["pattern"].as_str(),
        Some("evals-results-cd-*"),
        "Nightly CD stores only its cd results; Nightly CI stores the ci ones"
    );
    let (_, parsed) = load("evals-store.yml");
    assert_eq!(
        parsed["on"]["workflow_call"]["inputs"]["pattern"]["default"].as_str(),
        Some("evals-results-*"),
        "by default every results artifact is stored, ci and cd"
    );
    let (content, _) = load("pull-request-ci.yml");
    assert!(
        !content.contains("evals-store.yml"),
        "pull requests must not store results"
    );
}

#[test]
fn test_cd_evals_run_right_after_a_deploy() {
    let steps = mise_projects_steps("project");
    let deploy = step_index(&steps, "mise run deploy");
    let cd = step_index(&steps, "cd evals");
    let upload = step_index(&steps, "Upload cd eval results");
    assert!(
        deploy < cd && cd < upload,
        "deploy, then cd evals, then upload"
    );
    assert_eq!(steps[deploy]["id"].as_str(), Some("deploy"));
    assert!(
        steps[deploy]["run"]
            .as_str()
            .unwrap_or_default()
            .contains("echo \"deployed=true\" >> \"$GITHUB_OUTPUT\""),
        "the deploy step says when it really deployed"
    );

    let step = &steps[cd];
    assert_eq!(step["env"]["EVALS_STAGE"].as_str(), Some("cd"));
    assert!(
        step["env"]["DEPLOY_URL"]
            .as_str()
            .unwrap_or_default()
            .contains("deploy_urls"),
        "http evals are joined to the project's DEPLOY_URL"
    );
    assert!(
        step["if"]
            .as_str()
            .unwrap_or_default()
            .contains("steps.deploy.outputs.deployed == 'true'"),
        "cd evals run only after a deploy that happened"
    );
    assert!(
        step["continue-on-error"].is_null(),
        "a failed must eval fails the deploy job"
    );
    let run = step["run"].as_str().unwrap_or_default();
    assert!(run.contains("evals.sh\" run"), "should run the evals");
    assert!(
        run.contains("evals.sh\" publish"),
        "should write the job summary"
    );
    assert!(
        run.contains("evals-results-cd-"),
        "cd results get their own artifact, so Nightly CD can store only them"
    );

    let script = fs::read_to_string(get_repo_root().join("standards/scripts/evals.sh"))
        .unwrap_or_else(|e| panic!("read evals.sh: {e}"));
    assert!(
        script.contains("args+=(--fail-on-must)"),
        "the cd stage fails on a failed must eval"
    );
}

#[test]
fn test_evals_store_pushes_only_to_the_evals_results_branch() {
    let (content, parsed) = load("evals-store.yml");
    assert!(!parsed["on"]["workflow_call"].is_null());
    assert_eq!(parsed["permissions"]["contents"].as_str(), Some("read"));
    let job = &parsed["jobs"]["store"];
    assert_eq!(job["permissions"]["contents"].as_str(), Some("write"));
    assert_eq!(
        job["continue-on-error"].as_bool(),
        Some(true),
        "storing must never fail the run"
    );
    assert!(content.contains("evals.sh store"));
    assert!(!content.contains("git push"), "the push lives in evals.sh");

    let (mise, _) = load("mise-projects.yml");
    assert!(
        !mise.contains("contents: write"),
        "a write job inside mise-projects.yml breaks callers that grant only read"
    );

    let script = fs::read_to_string(get_repo_root().join("standards/scripts/evals.sh"))
        .unwrap_or_else(|e| panic!("read evals.sh: {e}"));
    let pushes: Vec<&str> = script
        .lines()
        .filter(|l| l.contains("\" push ") && !l.trim_start().starts_with('#'))
        .collect();
    assert_eq!(pushes.len(), 1, "evals.sh pushes once: {pushes:?}");
    assert!(pushes[0].contains("HEAD:refs/heads/evals-results"));
    assert!(!pushes[0].contains("--force") && !pushes[0].contains(" -f "));
}
