use std::fs;
use workflow_tests::*;

#[test]
fn test_codeql_workflow_file_exists() {
    let workflow_path = get_codeql_workflow_path();
    assert!(
        workflow_path.exists(),
        "CodeQL workflow file not found: {}",
        workflow_path.display()
    );
}

#[test]
fn test_codeql_workflow_yaml_can_be_parsed() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML to ensure it's valid
    let _parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");
}

#[test]
fn test_codeql_workflow_triggers() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Should trigger on push to main
    assert!(
        content.contains("push:") && content.contains("branches: [main]"),
        "CodeQL workflow should trigger on push to main branch"
    );

    // Should trigger on pull requests to main
    assert!(
        content.contains("pull_request:"),
        "CodeQL workflow should trigger on pull requests"
    );

    // Should have a schedule
    assert!(
        content.contains("schedule:"),
        "CodeQL workflow should have a scheduled run"
    );
}

#[test]
fn test_codeql_workflow_excludes_rust() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Parse YAML
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");

    // Get the matrix languages
    let languages = parsed["jobs"]["analyze"]["strategy"]["matrix"]["language"]
        .as_sequence()
        .expect("matrix.language should be a sequence");

    // Should include javascript-typescript
    let has_js = languages
        .iter()
        .any(|lang| lang.as_str() == Some("javascript-typescript"));
    assert!(
        has_js,
        "CodeQL workflow should include javascript-typescript"
    );

    // Should include actions
    let has_actions = languages
        .iter()
        .any(|lang| lang.as_str() == Some("actions"));
    assert!(has_actions, "CodeQL workflow should include actions");

    // Should NOT include rust
    let has_rust = languages.iter().any(|lang| lang.as_str() == Some("rust"));
    assert!(
        !has_rust,
        "CodeQL workflow should NOT include rust (takes ~26 minutes)"
    );
}

#[test]
fn test_codeql_workflow_uses_codeql_actions() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Should use github/codeql-action/init
    assert!(
        content.contains("github/codeql-action/init@"),
        "CodeQL workflow should use codeql-action/init"
    );

    // Should use github/codeql-action/autobuild
    assert!(
        content.contains("github/codeql-action/autobuild@"),
        "CodeQL workflow should use codeql-action/autobuild"
    );

    // Should use github/codeql-action/analyze
    assert!(
        content.contains("github/codeql-action/analyze@"),
        "CodeQL workflow should use codeql-action/analyze"
    );
}

#[test]
fn test_codeql_workflow_has_required_permissions() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse workflow YAML");

    // Check for required permissions
    let permissions = parsed["permissions"]
        .as_mapping()
        .expect("permissions should be a mapping");

    // Should have contents: read
    assert_eq!(
        permissions.get("contents").and_then(|v| v.as_str()),
        Some("read"),
        "CodeQL workflow should have contents: read permission"
    );

    // Should have security-events: write
    assert_eq!(
        permissions.get("security-events").and_then(|v| v.as_str()),
        Some("write"),
        "CodeQL workflow should have security-events: write permission"
    );

    // Should have actions: read
    assert_eq!(
        permissions.get("actions").and_then(|v| v.as_str()),
        Some("read"),
        "CodeQL workflow should have actions: read permission"
    );
}

#[test]
fn test_codeql_workflow_has_documentation() {
    let content =
        fs::read_to_string(get_codeql_workflow_path()).expect("Failed to read workflow file");

    // Should have comments explaining why Rust is excluded
    assert!(
        content.contains("26 minutes") || content.contains("Rust"),
        "CodeQL workflow should document why Rust is excluded"
    );
}
