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
