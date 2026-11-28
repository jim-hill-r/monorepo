//! all-retire-resources.yml: the weekly deletion of temporary resources and
//! approved retired ones ("Temporary resources" and "Safe deletion order" in
//! standards/docs/resources.md).

use std::fs;
use workflow_tests::*;

fn workflow() -> (String, serde_yaml::Value) {
    let content = fs::read_to_string(get_retire_resources_workflow_path())
        .expect("Failed to read all-retire-resources.yml");
    let parsed = serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
    (content, parsed)
}

#[test]
fn runs_weekly_and_by_hand_as_a_dry_run() {
    let (_, yaml) = workflow();
    let on = &yaml["on"];
    assert!(on["schedule"].is_sequence(), "Must run on a schedule");
    let dry_run = &on["workflow_dispatch"]["inputs"]["dry_run"];
    assert_eq!(dry_run["default"], serde_yaml::Value::Bool(true));
}

#[test]
fn only_a_manual_run_can_be_a_dry_run() {
    let (content, _) = workflow();
    assert!(
        content.contains("github.event_name == 'workflow_dispatch' && inputs.dry_run || false"),
        "A scheduled run deletes; a manual run follows its dry_run input"
    );
}

#[test]
fn the_rules_are_chores_retire_temporary() {
    let (content, _) = workflow();
    assert!(content.contains("retire-temporary --live"));
}

#[test]
fn branches_go_through_the_delete_branches_workflow() {
    let (_, yaml) = workflow();
    let job = &yaml["jobs"]["branches"];
    assert_eq!(job["uses"], "./.github/workflows/all-delete-branches.yml");
    assert_eq!(job["permissions"]["contents"], "write");
    assert_eq!(job["permissions"]["pull-requests"], "read");
}

#[test]
fn top_level_permissions_are_read_only() {
    let (_, yaml) = workflow();
    let perms = yaml["permissions"]
        .as_mapping()
        .expect("permissions must be set");
    assert_eq!(perms.len(), 1);
    assert_eq!(yaml["permissions"]["contents"], "read");
}

#[test]
fn inputs_and_secrets_reach_scripts_through_env() {
    let (content, _) = workflow();
    for block in content.split("run: |").skip(1) {
        let script: String = block
            .lines()
            .take_while(|l| l.is_empty() || l.starts_with("          "))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !script.contains("${{"),
            "Expressions must reach run scripts through env:\n{script}"
        );
    }
}

fn step<'a>(yaml: &'a serde_yaml::Value, name: &str) -> &'a serde_yaml::Value {
    yaml["jobs"]["temporary"]["steps"]
        .as_sequence()
        .expect("temporary job has steps")
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no step named {name}"))
}

#[test]
fn approved_deletions_are_planned_by_chores_retire_approved() {
    let (content, yaml) = workflow();
    assert!(content.contains("retire-approved --live"));
    assert!(content.contains("resources.sh claims"));
    // An empty Pages list must not read as "every retired project is gone".
    assert!(content.contains("pages_listed: $pages_listed"));
    let run = step(&yaml, "Plan approved deletions")["run"]
        .as_str()
        .expect("run script");
    assert!(run.contains("--claims"));
}

#[test]
fn approved_resources_are_deleted_only_from_the_default_branch() {
    let (_, yaml) = workflow();
    let s = step(&yaml, "Delete approved Pages projects");
    assert_eq!(
        s["env"]["ON_DEFAULT"],
        "${{ github.ref_name == github.event.repository.default_branch }}"
    );
    assert_eq!(s["env"]["DRY_RUN"], "${{ steps.plan.outputs.dry-run }}");
    let run = s["run"].as_str().expect("run script");
    assert!(run.contains(r#"[ "$ON_DEFAULT" != "true" ]"#));
    // Only Pages projects, and the deletion is verified.
    assert!(run.contains(r#"[ "$kind" = "pages" ] || continue"#));
    assert!(run.contains("404"));
}
