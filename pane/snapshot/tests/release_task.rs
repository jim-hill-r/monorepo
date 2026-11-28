//! pane/web's release task must build the snapshot into the web app's
//! static assets before bundling, and the output must never be committed.

use std::path::Path;
use std::process::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn web_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../web")
}

#[test]
fn release_builds_the_snapshot_before_the_bundle() -> TestResult {
    let mise: toml::Table = toml::from_str(&std::fs::read_to_string(web_dir().join("mise.toml"))?)?;
    let steps: Vec<&str> = mise
        .get("tasks")
        .and_then(|t| t.get("release"))
        .and_then(|r| r.get("run"))
        .and_then(|r| r.as_array())
        .ok_or("pane/web/mise.toml has no [tasks.release] run list")?
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    let snapshot = steps
        .iter()
        .position(|s| s.contains("pane_snapshot") && s.contains("--out public/data/snapshot.json"))
        .ok_or("release doesn't run pane_snapshot into public/data/snapshot.json")?;
    let bundle = steps
        .iter()
        .position(|s| s.starts_with("dx bundle"))
        .ok_or("release doesn't run dx bundle")?;
    assert!(
        snapshot < bundle,
        "the snapshot must be built before the bundle"
    );
    Ok(())
}

#[test]
fn snapshot_output_is_gitignored() -> TestResult {
    let out = Command::new("git")
        .arg("-C")
        .arg(web_dir())
        .args(["check-ignore", "-q", "public/data/snapshot.json"])
        .status()?;
    assert!(
        out.success(),
        "public/data/snapshot.json must be gitignored"
    );
    Ok(())
}

#[test]
fn release_saves_the_audit_and_passes_it_to_the_snapshot() -> TestResult {
    let mise: toml::Table = toml::from_str(&std::fs::read_to_string(web_dir().join("mise.toml"))?)?;
    let steps: Vec<&str> = mise
        .get("tasks")
        .and_then(|t| t.get("release"))
        .and_then(|r| r.get("run"))
        .and_then(|r| r.as_array())
        .ok_or("pane/web/mise.toml has no [tasks.release] run list")?
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    let audit = steps
        .iter()
        .position(|s| s.contains("standards/scripts/audit.sh") && s.contains("> target/audit.txt"))
        .ok_or("release doesn't save the audit to target/audit.txt")?;
    let snapshot = steps
        .iter()
        .position(|s| s.contains("pane_snapshot") && s.contains("--audit target/audit.txt"))
        .ok_or("release doesn't pass --audit target/audit.txt to pane_snapshot")?;
    assert!(audit < snapshot, "the audit must run before the snapshot");
    assert!(
        steps[audit].ends_with("|| true"),
        "a failing audit is data for pane, not a failed release"
    );
    let ignored = Command::new("git")
        .arg("-C")
        .arg(web_dir())
        .args(["check-ignore", "-q", "target/audit.txt"])
        .status()?;
    assert!(ignored.success(), "target/audit.txt must be gitignored");
    Ok(())
}

#[test]
fn release_and_dev_pass_the_eval_results_to_the_snapshot() -> TestResult {
    let mise: toml::Table = toml::from_str(&std::fs::read_to_string(web_dir().join("mise.toml"))?)?;
    let tasks = mise.get("tasks").ok_or("pane/web/mise.toml has no tasks")?;
    let fetch = tasks
        .get("eval-results")
        .and_then(|t| t.get("run"))
        .and_then(|r| r.as_str())
        .ok_or("pane/web/mise.toml has no eval-results task")?;
    assert!(
        fetch.contains("evals-results") && fetch.contains("target/evals-results"),
        "eval-results must copy the evals-results branch into target/evals-results"
    );
    for task in ["release", "dev"] {
        let steps: Vec<&str> = tasks
            .get(task)
            .and_then(|r| r.get("run"))
            .and_then(|r| r.as_array())
            .ok_or(format!("no [tasks.{task}] run list"))?
            .iter()
            .filter_map(|s| s.as_str())
            .collect();
        let copy = steps
            .iter()
            .position(|s| s.starts_with("mise run eval-results"))
            .ok_or(format!("{task} doesn't run eval-results"))?;
        assert!(
            steps[copy].ends_with("|| true"),
            "missing eval results are data for pane, not a failed {task}"
        );
        let snapshot = steps
            .iter()
            .position(|s| s.contains("pane_snapshot") && s.contains("--evals target/evals-results"))
            .ok_or(format!("{task} doesn't pass --evals target/evals-results"))?;
        assert!(
            copy < snapshot,
            "{task} must copy the results before the snapshot"
        );
    }
    let ignored = Command::new("git")
        .arg("-C")
        .arg(web_dir())
        .args(["check-ignore", "-q", "target/evals-results/verdicts.json"])
        .status()?;
    assert!(ignored.success(), "target/evals-results must be gitignored");
    Ok(())
}
