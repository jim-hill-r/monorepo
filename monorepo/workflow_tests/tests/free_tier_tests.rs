//! Tests for standards/scripts/free-tiers.sh and the All Free Tier Usage
//! workflow. Cloudflare and wrangler are replaced by stub scripts on PATH, so
//! nothing here talks to a real service.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use workflow_tests::*;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn script() -> PathBuf {
    get_repo_root().join("standards/scripts/free-tiers.sh")
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "free-tiers-{}-{}-{}",
        label,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write file");
}

fn write_exe(path: &Path, content: &str) {
    write(path, content);
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod");
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("run git");
    assert!(status.success(), "git {:?} failed", args);
}

/// A throwaway git repo holding the given files, for `check`.
fn fixture_repo(files: &[(&str, &str)]) -> PathBuf {
    let dir = temp_dir("repo");
    for (path, content) in files {
        write(&dir.join(path), content);
    }
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "-A"]);
    dir
}

fn run_check(root: &Path) -> Output {
    Command::new("bash")
        .arg(script())
        .arg("check")
        .env("FREE_TIERS_ROOT", root)
        .output()
        .expect("run free-tiers.sh")
}

fn stdout(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const OK_WORKFLOW: &str = "jobs:\n  a:\n    runs-on: ubuntu-24.04  # comment\n  b:\n    runs-on: ubuntu-latest\n  c:\n    runs-on: ${{ matrix.os }}\n";

#[test]
fn test_check_passes_on_the_repo() {
    let out = Command::new("bash")
        .arg(script())
        .arg("check")
        .current_dir(get_repo_root())
        .output()
        .expect("run free-tiers.sh");
    assert!(out.status.success(), "{}", stdout(&out));
}

#[test]
fn test_check_passes_on_a_clean_fixture() {
    let repo = fixture_repo(&[
        (".github/workflows/ci.yml", OK_WORKFLOW),
        (
            "site/wrangler.toml",
            "name = \"jimhillr-site\"\npages_build_output_dir = \"dist\"\n",
        ),
    ]);
    let out = run_check(&repo);
    assert!(out.status.success(), "{}", stdout(&out));
}

#[test]
fn test_check_rejects_larger_runners() {
    for label in ["macos-15-xlarge", "ubuntu-24.04-16core", "big-runner"] {
        let repo = fixture_repo(&[(
            ".github/workflows/ci.yml",
            &format!("jobs:\n  a:\n    runs-on: {label}\n"),
        )]);
        let out = run_check(&repo);
        assert!(!out.status.success(), "{label} should be rejected");
        assert!(stdout(&out).contains("FT-001"), "{}", stdout(&out));
    }
}

#[test]
fn test_check_rejects_rapidapi_in_workflows() {
    let repo = fixture_repo(&[(
        ".github/workflows/homer.yml",
        "jobs:\n  a:\n    runs-on: ubuntu-latest\n    env:\n      RAPIDAPI_KEY: ${{ secrets.RAPIDAPI_KEY }}\n",
    )]);
    let out = run_check(&repo);
    assert!(!out.status.success());
    assert!(stdout(&out).contains("FT-002"), "{}", stdout(&out));
}

#[test]
fn test_check_rejects_too_many_projects_and_crons() {
    let mut files: Vec<(String, String)> = (0..4)
        .map(|i| {
            (
                format!("w{i}/wrangler.toml"),
                format!("name = \"w{i}\"\n[triggers]\ncrons = [\"0 {i} * * *\"]\n"),
            )
        })
        .collect();
    files.push((".github/workflows/ci.yml".into(), OK_WORKFLOW.into()));
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let repo = fixture_repo(&refs);

    let out = run_check(&repo);
    assert!(!out.status.success());
    assert!(stdout(&out).contains("FT-004"), "{}", stdout(&out));

    let out = Command::new("bash")
        .arg(script())
        .arg("check")
        .env("FREE_TIERS_ROOT", &repo)
        .env("FREE_TIERS_WORKERS_BUDGET", "2")
        .env("FREE_TIERS_CRON_TRIGGERS_BUDGET", "5")
        .output()
        .expect("run free-tiers.sh");
    assert!(!out.status.success());
    assert!(stdout(&out).contains("FT-003"), "{}", stdout(&out));
}

#[test]
fn test_budgets_cannot_be_raised_past_the_free_limit() {
    // Six cron triggers is over the free limit of five, whatever the budget says.
    let files: Vec<(String, String)> = (0..6)
        .map(|i| {
            (
                format!("w{i}/wrangler.toml"),
                format!("name = \"w{i}\"\ncrons = [\"0 {i} * * *\"]\n"),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let repo = fixture_repo(&refs);
    let out = Command::new("bash")
        .arg(script())
        .arg("check")
        .env("FREE_TIERS_ROOT", &repo)
        .env("FREE_TIERS_CRON_TRIGGERS_BUDGET", "99")
        .output()
        .expect("run free-tiers.sh");
    assert!(!out.status.success());
    assert!(stdout(&out).contains("FT-004"), "{}", stdout(&out));
}

#[test]
fn test_check_rejects_pages_projects_without_the_prefix() {
    for name in [
        "site",
        "pane",
        "jimhillr_site",
        "Jimhillr-site",
        "jimhillr-",
    ] {
        let repo = fixture_repo(&[(
            "site/wrangler.toml",
            &format!("name = \"{name}\"\npages_build_output_dir = \"dist\"\n"),
        )]);
        let out = run_check(&repo);
        assert!(!out.status.success(), "{name} should be rejected");
        assert!(stdout(&out).contains("FT-006"), "{}", stdout(&out));
    }
}

#[test]
fn test_pages_project_refuses_a_name_without_the_prefix() {
    let dir = temp_dir("prefix");
    let path = stub_bin(&dir, &[]);
    let project = dir.join("project");
    write(
        &project.join("wrangler.toml"),
        "name = \"new-site\"\npages_build_output_dir = \"dist\"\n",
    );
    let out = Command::new("bash")
        .arg(script())
        .arg("pages-project")
        .current_dir(&project)
        .env("PATH", &path)
        .env("CLOUDFLARE_API_TOKEN", "test-token")
        .env("CLOUDFLARE_ACCOUNT_ID", "test-account")
        .output()
        .expect("run free-tiers.sh");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("jimhillr-"));
    assert_eq!(wrangler_log(&dir), "", "wrangler should not be called");
}

#[test]
fn test_budgets_come_from_limits_toml() {
    // Two Workers pass the real budget of 25 but fail a limits file whose
    // budget is 1, so the script reads its budgets from limits.toml.
    let files: Vec<(String, String)> = (0..2)
        .map(|i| (format!("w{i}/wrangler.toml"), format!("name = \"w{i}\"\n")))
        .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let repo = fixture_repo(&refs);
    assert!(run_check(&repo).status.success());

    let real = std::fs::read_to_string(get_repo_root().join("standards/radar/limits.toml"))
        .expect("read limits.toml");
    let tight = real.replacen(
        "budget = 25\nscope = \"account\"\nunit = \"workers\"",
        "budget = 1\nscope = \"account\"\nunit = \"workers\"",
        1,
    );
    assert_ne!(
        real, tight,
        "the workers budget line moved; update this test"
    );
    let limits = repo.join("limits.toml");
    write(&limits, &tight);
    let out = Command::new("bash")
        .arg(script())
        .arg("check")
        .env("FREE_TIERS_ROOT", &repo)
        .env("FREE_TIERS_LIMITS_FILE", &limits)
        .output()
        .expect("run free-tiers.sh");
    assert!(!out.status.success());
    assert!(
        stdout(&out).contains("the budget is 1 (free limit 100)"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn test_check_rejects_paid_wrangler_settings() {
    for setting in [
        "usage_model = \"standard\"",
        "[limits]\ncpu_ms = 50000",
        "logpush = true",
    ] {
        let repo = fixture_repo(&[("w/wrangler.toml", &format!("name = \"w\"\n{setting}\n"))]);
        let out = run_check(&repo);
        assert!(!out.status.success(), "{setting} should be rejected");
        assert!(stdout(&out).contains("FT-005"), "{}", stdout(&out));
    }
}

/// A PATH with stub `curl` and `wrangler`. The stub curl answers Cloudflare
/// requests with `pages.json` / `workers.json` from `dir`, and wrangler logs
/// its arguments to `wrangler.log`.
fn stub_bin(dir: &Path, pages: &[String]) -> String {
    let names: Vec<String> = pages
        .iter()
        .map(|n| format!("{{\"name\":\"{n}\"}}"))
        .collect();
    write(
        &dir.join("all-pages.json"),
        &format!("[{}]", names.join(",")),
    );
    write(
        &dir.join("workers.json"),
        "{\"success\":true,\"result\":[{\"id\":\"telemetry\"}]}",
    );
    write_exe(
        &dir.join("bin/curl"),
        &format!(
            r#"#!/usr/bin/env bash
url="${{@: -1}}"
case "$url" in
  *pages/projects*)
    page=$(sed -E 's/.*page=([0-9]+)$/\1/' <<<"$url")
    jq -c --argjson p "$page" '{{success: true, result: .[(($p - 1) * 10):($p * 10)]}}' "{dir}/all-pages.json" ;;
  *workers/scripts*) cat "{dir}/workers.json" ;;
  *d1/database*) jq -c '{{success: true, result: .}}' "{dir}/d1.json" ;;
  *) echo "unexpected url $url" >&2; exit 22 ;;
esac
"#,
            dir = dir.display()
        ),
    );
    if !dir.join("d1.json").exists() {
        write(&dir.join("d1.json"), "[]");
    }
    // `wrangler d1 create NAME` adds NAME to d1.json, as Cloudflare would.
    write_exe(
        &dir.join("bin/wrangler"),
        &format!(
            r#"#!/usr/bin/env bash
echo "$@" >> "{dir}/wrangler.log"
if [ "$1 $2" = "d1 create" ]; then
  jq --arg n "$3" '. + [{{uuid: "new-uuid", name: $n}}]' "{dir}/d1.json" > "{dir}/d1.tmp"
  mv "{dir}/d1.tmp" "{dir}/d1.json"
fi
"#,
            dir = dir.display()
        ),
    );
    format!(
        "{}:{}",
        dir.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

fn run_pages_project(dir: &Path, path: &str, budget: Option<&str>) -> Output {
    let project = dir.join("project");
    write(
        &project.join("wrangler.toml"),
        "name = \"jimhillr-new-site\"\npages_build_output_dir = \"dist\"\n",
    );
    let mut cmd = Command::new("bash");
    cmd.arg(script())
        .arg("pages-project")
        .current_dir(&project)
        .env("PATH", path)
        .env("CLOUDFLARE_API_TOKEN", "test-token")
        .env("CLOUDFLARE_ACCOUNT_ID", "test-account");
    if let Some(b) = budget {
        cmd.env("FREE_TIERS_PAGES_PROJECTS_BUDGET", b);
    }
    cmd.output().expect("run free-tiers.sh")
}

fn wrangler_log(dir: &Path) -> String {
    fs::read_to_string(dir.join("wrangler.log")).unwrap_or_default()
}

#[test]
fn test_pages_project_creates_a_missing_project_within_budget() {
    let dir = temp_dir("create");
    let existing: Vec<String> = (0..12).map(|i| format!("site-{i}")).collect();
    let path = stub_bin(&dir, &existing);
    let out = run_pages_project(&dir, &path, None);
    assert!(out.status.success(), "{}", stdout(&out));
    assert_eq!(
        wrangler_log(&dir).trim(),
        "pages project create jimhillr-new-site --production-branch main"
    );
}

#[test]
fn test_pages_project_leaves_an_existing_project_alone() {
    let dir = temp_dir("exists");
    // Put it on the second page of results to exercise paging.
    let mut existing: Vec<String> = (0..14).map(|i| format!("site-{i}")).collect();
    existing.push("jimhillr-new-site".into());
    let path = stub_bin(&dir, &existing);
    let out = run_pages_project(&dir, &path, Some("1"));
    assert!(out.status.success(), "{}", stdout(&out));
    assert_eq!(wrangler_log(&dir), "", "wrangler should not be called");
}

#[test]
fn test_pages_project_stops_at_the_budget() {
    let dir = temp_dir("budget");
    let existing: Vec<String> = (0..3).map(|i| format!("site-{i}")).collect();
    let path = stub_bin(&dir, &existing);
    let out = run_pages_project(&dir, &path, Some("3"));
    assert!(!out.status.success());
    assert!(stdout(&out).contains("not creating"), "{}", stdout(&out));
    assert_eq!(wrangler_log(&dir), "", "nothing should be created");
}

#[test]
fn test_pages_project_needs_credentials() {
    let dir = temp_dir("creds");
    let path = stub_bin(&dir, &[]);
    let project = dir.join("project");
    write(&project.join("wrangler.toml"), "name = \"x\"\n");
    let out = Command::new("bash")
        .arg(script())
        .arg("pages-project")
        .current_dir(&project)
        .env("PATH", path)
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .output()
        .expect("run free-tiers.sh");
    assert!(!out.status.success());
    assert_eq!(wrangler_log(&dir), "");
}

fn run_usage(dir: &Path, pages: usize) -> (Output, String) {
    let existing: Vec<String> = (0..pages).map(|i| format!("site-{i}")).collect();
    let path = stub_bin(dir, &existing);
    let summary = dir.join("summary.md");
    let out = Command::new("bash")
        .arg(script())
        .arg("usage")
        .env("PATH", path)
        .env("CLOUDFLARE_API_TOKEN", "test-token")
        .env("CLOUDFLARE_ACCOUNT_ID", "test-account")
        .env_remove("GH_TOKEN")
        .env("GITHUB_STEP_SUMMARY", &summary)
        .output()
        .expect("run free-tiers.sh");
    (out, fs::read_to_string(summary).unwrap_or_default())
}

#[test]
fn test_usage_passes_under_budget_and_writes_a_summary() {
    let dir = temp_dir("usage-ok");
    let (out, summary) = run_usage(&dir, 3);
    assert!(out.status.success(), "{}", stdout(&out));
    assert!(summary.contains("| Cloudflare Pages projects | 3 | 25 | 100 | ok |"));
    assert!(summary.contains("| Cloudflare Workers | 1 | 25 | 100 | ok |"));
}

#[test]
fn test_usage_fails_at_the_budget() {
    let dir = temp_dir("usage-over");
    let (out, summary) = run_usage(&dir, 25);
    assert!(!out.status.success());
    assert!(summary.contains("| Cloudflare Pages projects | 25 | 25 | 100 | OVER |"));
}

#[test]
fn test_usage_workflow_only_reads() {
    let path = get_workflow_path("all-free-tier-usage.yml");
    let content = fs::read_to_string(&path).expect("read all-free-tier-usage.yml");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("parse workflow");
    let perms = yaml["permissions"].as_mapping().expect("permissions set");
    for (_, v) in perms {
        assert_eq!(
            v, "read",
            "the usage workflow must only have read permissions"
        );
    }
    assert!(yaml["on"]["schedule"].is_sequence(), "runs on a schedule");
    assert!(
        yaml["on"]["workflow_call"].is_mapping(),
        "the nightly run can call it"
    );
    assert!(content.contains("free-tiers.sh usage"));
}

#[test]
fn test_weekly_run_scans_the_whole_history_for_secrets() {
    let path = get_workflow_path("all-free-tier-usage.yml");
    let content = fs::read_to_string(&path).expect("read all-free-tier-usage.yml");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("parse workflow");
    let steps =
        serde_yaml::to_string(&yaml["jobs"]["secrets-history"]["steps"]).unwrap_or_default();
    assert!(
        steps.contains("fetch-depth: 0"),
        "a full clone, every commit"
    );
    assert!(
        steps.contains("secrets-check.sh history"),
        "SEC-001 full-history scan"
    );
}

fn run_guard(dir: &Path, path: &str, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let project = dir.join("project");
    write(
        &project.join("wrangler.toml"),
        "name = \"jimhillr-tofu-state\"\nmain = \"build/worker/shim.mjs\"\n",
    );
    let mut cmd = Command::new("bash");
    cmd.arg(script())
        .args(args)
        .current_dir(&project)
        .env("PATH", path)
        .env("CLOUDFLARE_API_TOKEN", "test-token")
        .env("CLOUDFLARE_ACCOUNT_ID", "test-account");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run free-tiers.sh")
}

#[test]
fn test_worker_passes_within_budget_and_for_an_existing_worker() {
    let dir = temp_dir("worker-ok");
    let path = stub_bin(&dir, &[]);
    let out = run_guard(&dir, &path, &["worker"], &[]);
    assert!(out.status.success(), "{}", stdout(&out));
    write(
        &dir.join("workers.json"),
        "{\"success\":true,\"result\":[{\"id\":\"jimhillr-tofu-state\"}]}",
    );
    let out = run_guard(
        &dir,
        &path,
        &["worker"],
        &[("FREE_TIERS_WORKERS_BUDGET", "1")],
    );
    assert!(stdout(&out).contains("exists"), "{}", stdout(&out));
    assert!(out.status.success());
}

#[test]
fn test_worker_stops_at_the_budget() {
    let dir = temp_dir("worker-budget");
    let path = stub_bin(&dir, &[]);
    let out = run_guard(
        &dir,
        &path,
        &["worker"],
        &[("FREE_TIERS_WORKERS_BUDGET", "1")],
    );
    assert!(!out.status.success());
    assert!(stdout(&out).contains("not deploying"), "{}", stdout(&out));
}

#[test]
fn test_d1_database_prints_an_existing_id_without_creating() {
    let dir = temp_dir("d1-exists");
    write(
        &dir.join("d1.json"),
        r#"[{"uuid":"abc-123","name":"jimhillr-tofu-state"}]"#,
    );
    let path = stub_bin(&dir, &[]);
    let out = run_guard(&dir, &path, &["d1-database", "jimhillr-tofu-state"], &[]);
    assert!(out.status.success(), "{}", stdout(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "abc-123");
    assert_eq!(wrangler_log(&dir), "");
}

#[test]
fn test_d1_database_creates_a_missing_database() {
    let dir = temp_dir("d1-create");
    let path = stub_bin(&dir, &[]);
    let out = run_guard(&dir, &path, &["d1-database", "jimhillr-tofu-state"], &[]);
    assert!(out.status.success(), "{}", stdout(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "new-uuid");
    assert_eq!(wrangler_log(&dir).trim(), "d1 create jimhillr-tofu-state");
}

#[test]
fn test_d1_database_stops_at_the_free_limit_and_checks_the_name() {
    let dir = temp_dir("d1-limit");
    let dbs: Vec<String> = (0..10)
        .map(|i| format!(r#"{{"uuid":"u{i}","name":"jimhillr-db-{i}"}}"#))
        .collect();
    write(&dir.join("d1.json"), &format!("[{}]", dbs.join(",")));
    let path = stub_bin(&dir, &[]);
    let out = run_guard(&dir, &path, &["d1-database", "jimhillr-tofu-state"], &[]);
    assert!(!out.status.success());
    assert!(stdout(&out).contains("not creating"), "{}", stdout(&out));
    let out = run_guard(&dir, &path, &["d1-database", "state"], &[]);
    assert!(!out.status.success());
    assert_eq!(wrangler_log(&dir), "", "nothing should be created");
}

#[test]
fn test_tofu_state_deploy_goes_through_the_guards() {
    let tasks = fs::read_to_string(get_repo_root().join("tofu_state/mise.toml"))
        .expect("read tofu_state/mise.toml");
    let worker = tasks
        .find("free-tiers.sh\\\" worker")
        .expect("deploy runs the worker guard");
    let d1 = tasks
        .find("free-tiers.sh\\\" d1-database")
        .expect("deploy gets the D1 database through the guard");
    let deploy = tasks.find("wrangler deploy").expect("deploy runs wrangler");
    assert!(
        worker < deploy && d1 < deploy,
        "the guards must run before wrangler deploys"
    );
    assert!(
        !tasks.contains("d1 create"),
        "only free-tiers.sh may create D1 databases"
    );
}

#[test]
fn test_pages_deploy_goes_through_the_guard() {
    let tasks = fs::read_to_string(get_repo_root().join("standards/tasks/cloudflare-pages.toml"))
        .expect("read cloudflare-pages.toml");
    let guard = tasks
        .find("free-tiers.sh\\\" pages-project")
        .or_else(|| tasks.find("free-tiers.sh pages-project"))
        .expect("deploy must run free-tiers.sh pages-project");
    let deploy = tasks
        .find("wrangler pages deploy")
        .expect("deploy runs wrangler");
    assert!(guard < deploy, "the guard must run before wrangler deploys");
    assert!(
        !tasks.contains("pages project create"),
        "only free-tiers.sh may create Pages projects"
    );
}
