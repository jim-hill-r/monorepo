//! The README's "Repo health" table has a badge for every workflow that runs
//! on `main` by itself (a push to `main` or a schedule), so a new check can't
//! go red without anyone seeing it. PR triage reads the same list
//! (standards/docs/repo-health.md). The table is written from
//! standards/repo-health.toml, which pane reads too.

use std::fs;
use workflow_tests::*;

/// Workflows that run on a push to `main` or on a schedule.
fn main_workflows() -> Vec<String> {
    let dir = get_repo_root().join(".github/workflows");
    let mut found = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir: {e}")) {
        let path = entry.unwrap_or_else(|e| panic!("entry: {e}")).path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !name.ends_with(".yml") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&content).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        let on = &parsed["on"];
        let scheduled = !on["schedule"].is_null();
        let on_main_push = on["push"]["branches"]
            .as_sequence()
            .is_some_and(|b| b.iter().any(|v| v.as_str() == Some("main")));
        if scheduled || on_main_push {
            found.push(name);
        }
    }
    found.sort();
    found
}

#[test]
fn test_main_workflows_are_found() {
    let workflows = main_workflows();
    for expected in ["push-main.yml", "nightly-ci.yml", "infra-drift.yml"] {
        assert!(
            workflows.iter().any(|w| w == expected),
            "{expected} should count as a workflow that runs on main, got {workflows:?}"
        );
    }
}

#[test]
fn test_readme_has_a_health_badge_for_every_main_workflow() {
    let readme = fs::read_to_string(get_repo_root().join("README.md"))
        .unwrap_or_else(|e| panic!("read README.md: {e}"));
    let health = readme
        .split("## Repo health")
        .nth(1)
        .unwrap_or_else(|| panic!("README.md should have a \"## Repo health\" section"));
    for file in main_workflows() {
        let badge =
            format!("https://github.com/jimhillr/monorepo/actions/workflows/{file}/badge.svg");
        assert!(
            health.contains(&badge),
            "README.md's Repo health table should have a badge for {file} ({badge}): add it to standards/repo-health.toml and run `chores repo-health --write`"
        );
    }
}

#[test]
fn test_health_badges_point_at_existing_workflows() {
    let readme = fs::read_to_string(get_repo_root().join("README.md"))
        .unwrap_or_else(|e| panic!("read README.md: {e}"));
    let marker = "https://github.com/jimhillr/monorepo/actions/workflows/";
    for part in readme.split(marker).skip(1) {
        let Some((file, _)) = part.split_once("/badge.svg") else {
            continue;
        };
        if file.starts_with("dynamic/") || file.contains(')') {
            continue;
        }
        assert!(
            get_workflow_path(file).exists(),
            "README.md has a badge for {file}, which doesn't exist"
        );
    }
}
