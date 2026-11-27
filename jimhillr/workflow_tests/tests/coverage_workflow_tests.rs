//! Tests for nightly-coverage.yml: nightly line coverage of every Rust
//! project (COV-001), stored on the coverage-results data branch.

use std::fs;
use workflow_tests::*;

fn load() -> (String, serde_yaml::Value) {
    let path = get_workflow_path("nightly-coverage.yml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let parsed = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    (content, parsed)
}

#[test]
fn test_runs_nightly_and_by_hand_never_on_pull_requests() {
    let (_, parsed) = load();
    let on = &parsed["on"];
    assert!(!on["schedule"].is_null(), "should run on a schedule");
    assert!(on.get("workflow_dispatch").is_some(), "should run by hand");
    assert!(
        on["pull_request"].is_null(),
        "should not run on pull requests"
    );
    assert!(on["push"].is_null(), "should not run on pushes");
}

#[test]
fn test_reads_by_default_and_only_store_can_write() {
    let (_, parsed) = load();
    assert_eq!(parsed["permissions"]["contents"].as_str(), Some("read"));
    let jobs = parsed["jobs"]
        .as_mapping()
        .unwrap_or_else(|| panic!("workflow has no jobs"));
    for (name, job) in jobs {
        let writes = job["permissions"]["contents"].as_str() == Some("write");
        assert_eq!(
            writes,
            name.as_str() == Some("store"),
            "only the store job may have contents: write (job {name:?})"
        );
    }
}

#[test]
fn test_uses_the_coverage_script_for_every_step_with_logic() {
    let (content, parsed) = load();
    assert!(content.contains("standards/scripts/coverage.sh projects"));
    assert!(content.contains("standards/scripts/coverage.sh\" run"));
    assert!(content.contains("standards/scripts/coverage.sh store"));
    assert!(
        !content.contains("git push"),
        "the workflow never pushes itself; coverage.sh store does (SEC-007)"
    );
    assert_eq!(
        parsed["jobs"]["store"]["if"].as_str(),
        Some("${{ !cancelled() }}"),
        "results are stored even when one project failed"
    );
    assert_eq!(
        parsed["jobs"]["coverage"]["strategy"]["fail-fast"].as_bool(),
        Some(false),
        "one failing project should not cancel the others"
    );
}

#[test]
fn test_does_not_use_the_rust_cache() {
    let (content, _) = load();
    assert!(
        !content.contains("Swatinem/rust-cache"),
        "coverage builds would fill the Actions cache budget"
    );
}
