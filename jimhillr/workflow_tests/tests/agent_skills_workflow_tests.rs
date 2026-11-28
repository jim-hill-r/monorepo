use std::fs;
use workflow_tests::*;

fn workflow() -> (String, serde_yaml::Value) {
    let content = fs::read_to_string(get_agent_skills_workflow_path())
        .expect("Failed to read pull-request-agent-skills.yml");
    let parsed = serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
    (content, parsed)
}

#[test]
fn test_runs_on_pull_requests_touching_skills() {
    let (_, yaml) = workflow();
    let paths = yaml["on"]["pull_request"]["paths"]
        .as_sequence()
        .expect("pull_request.paths must be a list");
    assert!(paths.iter().any(|p| p == ".agents/**"));
    assert!(paths.iter().any(|p| p == "AGENTS.md"));
}

#[test]
fn test_permissions_are_read_only() {
    let (_, yaml) = workflow();
    let perms = yaml["permissions"]
        .as_mapping()
        .expect("permissions must be set");
    assert_eq!(perms.len(), 1, "Only contents permission expected");
    assert_eq!(yaml["permissions"]["contents"], "read");
}

#[test]
fn test_validator_is_pinned() {
    let (content, _) = workflow();
    assert!(
        content.contains("skills-ref=="),
        "skills-ref must be installed at a pinned version"
    );
}

#[test]
fn test_every_skill_has_skill_md() {
    let skills = get_repo_root().join(".agents/skills");
    let entries = fs::read_dir(&skills).expect("Failed to read .agents/skills");
    let mut count = 0;
    for entry in entries {
        let path = entry.expect("Failed to read entry").path();
        if path.is_dir() {
            assert!(
                path.join("SKILL.md").is_file(),
                "{} is missing SKILL.md",
                path.display()
            );
            count += 1;
        }
    }
    assert!(count > 0, "Expected at least one skill");
}
