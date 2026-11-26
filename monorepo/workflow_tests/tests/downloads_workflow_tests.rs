//! Tests for nightly-downloads.yml, standards/scripts/downloads.sh and the
//! game build tasks in standards/tasks/bevy-game.toml.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use workflow_tests::*;

fn load() -> (String, serde_yaml::Value) {
    let path = get_workflow_path("nightly-downloads.yml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let parsed = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    (content, parsed)
}

fn script(name: &str) -> PathBuf {
    get_repo_root().join("standards/scripts").join(name)
}

#[test]
fn test_runs_nightly_by_hand_and_on_pull_requests() {
    let (_, parsed) = load();
    assert_eq!(parsed["name"].as_str(), Some("Nightly downloads"));
    let cron = parsed["on"]["schedule"][0]["cron"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(cron, "30 10 * * *", "2:30 AM Pacific, with Nightly CD");
    assert!(parsed["on"].get("workflow_dispatch").is_some());
    assert!(
        !parsed["on"]["pull_request"]["paths"].is_null(),
        "pull requests build only when the build changes"
    );
}

#[test]
fn test_only_the_publish_job_can_write_and_never_on_pull_requests() {
    let (_, parsed) = load();
    assert_eq!(parsed["permissions"]["contents"].as_str(), Some("read"));
    let jobs = parsed["jobs"]
        .as_mapping()
        .unwrap_or_else(|| panic!("workflow has no jobs"));
    for (name, job) in jobs {
        let name = name.as_str().unwrap_or_default();
        let write = job["permissions"]["contents"].as_str() == Some("write");
        assert_eq!(write, name == "publish", "only publish may write ({name})");
    }
    let publish = &parsed["jobs"]["publish"];
    let cond = publish["if"].as_str().unwrap_or_default();
    assert!(
        cond.contains("github.event_name != 'pull_request'") && cond.contains("refs/heads/main"),
        "publish runs only on main, never for a pull request: {cond}"
    );
}

#[test]
fn test_builds_after_ci_and_publishes_after_builds() {
    let (content, parsed) = load();
    let needs = |job: &str| -> Vec<String> {
        parsed["jobs"][job]["needs"]
            .as_sequence()
            .map(|s| {
                s.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    };
    assert!(needs("build").contains(&"ci".to_string()));
    assert!(needs("publish").contains(&"build".to_string()));
    assert!(content.contains("mise run ci"));
    assert!(content.contains("mise run release:${{ matrix.platform }}"));
    assert!(content.contains("mise run --quiet //:downloads"));
    assert!(content.contains("downloads.sh publish"));
}

#[test]
fn test_macos_builds_on_a_standard_runner() {
    let (_, parsed) = load();
    let runs_on = parsed["jobs"]["build"]["runs-on"]
        .as_str()
        .unwrap_or_default();
    assert!(
        runs_on.contains("'macos-15'") && runs_on.contains("'ubuntu-24.04'"),
        "standard runners only (free on a public repo): {runs_on}"
    );
    let platforms = &parsed["jobs"]["build"]["strategy"]["matrix"]["platform"];
    assert_eq!(
        serde_yaml::to_string(platforms).unwrap_or_default().trim(),
        "- macos\n- android"
    );
}

#[test]
fn test_signing_secrets_are_not_passed_to_pull_requests() {
    let (content, _) = load();
    for secret in ["ANDROID_KEYSTORE_BASE64", "ANDROID_KEYSTORE_PASSWORD"] {
        let guarded = format!("github.event_name != 'pull_request' && secrets.{secret}");
        assert!(
            content.contains(&guarded),
            "{secret} must only reach non-PR builds"
        );
    }
}

#[test]
fn test_downloads_list_names_controlled_chaos() {
    let out = Command::new("bash")
        .arg(script("downloads.sh"))
        .arg("list")
        .current_dir(get_repo_root())
        .output()
        .unwrap_or_else(|e| panic!("run downloads.sh: {e}"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(r#""project": "controlled_chaos""#)
            && stdout.contains(r#""tag": "controlled_chaos-nightly""#)
            && stdout.contains(r#""deploy_url": "https://controlled-chaos-5v4.pages.dev""#),
        "unexpected list: {stdout}"
    );
}

#[test]
fn test_publish_needs_a_tag_and_files() {
    let out = Command::new("bash")
        .arg(script("downloads.sh"))
        .arg("publish")
        .current_dir(get_repo_root())
        .output()
        .unwrap_or_else(|e| panic!("run downloads.sh: {e}"));
    assert!(!out.status.success(), "publish with no tag should fail");
}

#[test]
fn test_bevy_game_tasks_build_for_players() {
    let tasks = fs::read_to_string(get_repo_root().join("standards/tasks/bevy-game.toml"))
        .unwrap_or_else(|e| panic!("read bevy-game.toml: {e}"));
    for platform in ["web", "macos", "android"] {
        assert!(
            tasks.contains(&format!("[\"release:{platform}\"]"))
                && tasks.contains(&format!("bevy-release.sh\\\" {platform}")),
            "bevy-game.toml should have release:{platform}"
        );
    }
    assert!(
        tasks.contains("if [ ! -f wrangler.toml ]") && tasks.contains("free-tiers.sh"),
        "deploy should do nothing without a wrangler.toml and go through the free-tier guard"
    );
}

#[test]
fn test_bevy_release_rejects_unknown_platforms() {
    let out = Command::new("bash")
        .arg(script("bevy-release.sh"))
        .arg("windows")
        .current_dir(get_repo_root().join("controlled_chaos"))
        .output()
        .unwrap_or_else(|e| panic!("run bevy-release.sh: {e}"));
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
}

fn git(dir: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_tag_target_skips_commits_that_change_workflows() {
    let dir = std::env::temp_dir().join(format!("downloads-tag-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join(".github/workflows")).unwrap_or_else(|e| panic!("{e}"));
    git(&dir, &["init", "-q"]);
    fs::write(dir.join("game.txt"), "1").unwrap_or_else(|e| panic!("{e}"));
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "game"]);
    let plain = git(&dir, &["rev-parse", "HEAD"]);
    fs::write(dir.join(".github/workflows/a.yml"), "on: push").unwrap_or_else(|e| panic!("{e}"));
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "workflow"]);
    let workflow = git(&dir, &["rev-parse", "HEAD"]);

    let run = |sha: &str| -> String {
        let out = Command::new("bash")
            .arg(script("downloads.sh"))
            .args(["tag-target", sha])
            .current_dir(&dir)
            .output()
            .unwrap_or_else(|e| panic!("run downloads.sh: {e}"));
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    assert_eq!(
        run(&workflow),
        plain,
        "a commit that changes workflows is skipped"
    );
    assert_eq!(run(&plain), plain, "a commit that doesn't is used as is");
    let _ = fs::remove_dir_all(&dir);
}
