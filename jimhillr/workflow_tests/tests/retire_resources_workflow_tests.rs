//! all-retire-resources.yml: the weekly deletion of temporary resources
//! ("Temporary resources" in standards/docs/resources.md).

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
