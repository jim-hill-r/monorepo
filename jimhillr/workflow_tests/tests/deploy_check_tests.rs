//! Tests for standards/scripts/deploy-check.sh, which stops a CI/CD run from
//! deploying a project when a newer commit on main changes it, so only the
//! latest version of main that passed CI is deployed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use workflow_tests::get_repo_root;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write(path: PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir: {e}"));
    }
    fs::write(&path, content).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

/// A repo with projects `app`, `site` (affected by `app`), `other` and
/// `snap` (deploys on every push), pushed
/// to a bare `origin`. Returns the clone and the first commit.
fn setup() -> (PathBuf, String) {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let base = std::env::temp_dir().join(format!("deploy-check-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let origin = base.join("origin.git");
    let work = base.join("work");
    fs::create_dir_all(&work).unwrap_or_else(|e| panic!("mkdir: {e}"));
    git(
        &base,
        &["init", "--quiet", "--bare", "-b", "main", "origin.git"],
    );
    git(&work, &["init", "--quiet", "-b", "main"]);

    let scripts = get_repo_root().join("standards/scripts");
    for script in ["affected.sh", "deploy-check.sh"] {
        let content = fs::read_to_string(scripts.join(script))
            .unwrap_or_else(|e| panic!("read {script}: {e}"));
        write(work.join("standards/scripts").join(script), &content);
    }
    write(work.join("mise.toml"), "monorepo_root = true\n");
    write(work.join("app/mise.toml"), "# app\n");
    write(work.join("site/mise.toml"), "# affected-by: ../app\n");
    write(work.join("other/mise.toml"), "# other\n");
    write(
        work.join("snap/mise.toml"),
        "# deploy-on-every-push: reads the whole repo\n",
    );
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "--quiet", "-m", "first"]);
    git(
        &work,
        &[
            "remote",
            "add",
            "origin",
            origin.to_str().unwrap_or_default(),
        ],
    );
    git(&work, &["push", "--quiet", "origin", "main"]);
    let first = git(&work, &["rev-parse", "HEAD"]);
    (work, first)
}

/// Push a commit to origin's main that changes `file`, then check out `at`.
fn push_change(work: &Path, file: &str, at: &str) {
    write(work.join(file), "changed\n");
    git(work, &["add", "-A"]);
    git(work, &["commit", "--quiet", "-m", "change"]);
    git(work, &["push", "--quiet", "origin", "main"]);
    git(work, &["checkout", "--quiet", "--detach", at]);
}

fn check(work: &Path, project: &str) -> (i32, String) {
    let out = Command::new("bash")
        .arg("standards/scripts/deploy-check.sh")
        .arg(project)
        .current_dir(work)
        .output()
        .unwrap_or_else(|e| panic!("run deploy-check.sh: {e}"));
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr),
    )
}

#[test]
fn test_deploys_when_head_is_the_tip_of_main() {
    let (work, _) = setup();
    let (code, out) = check(&work, "site");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("tip of main"), "{out}");
}

#[test]
fn test_deploys_when_newer_commits_do_not_touch_the_project() {
    let (work, first) = setup();
    push_change(&work, "other/src.txt", &first);
    let (code, out) = check(&work, "site");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("nothing newer changes site"), "{out}");
}

#[test]
fn test_skips_when_a_newer_commit_changes_the_project() {
    let (work, first) = setup();
    push_change(&work, "app/src.txt", &first);
    let (code, out) = check(&work, "app");
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("skip:"), "{out}");
}

#[test]
fn test_skips_when_a_newer_commit_changes_what_the_project_deploys() {
    let (work, first) = setup();
    push_change(&work, "app/src.txt", &first);
    let (code, out) = check(&work, "site");
    assert_eq!(code, 3, "site is affected by ../app: {out}");
}

#[test]
fn test_skips_a_commit_that_is_not_on_main() {
    let (work, first) = setup();
    git(&work, &["checkout", "--quiet", "--detach", &first]);
    write(work.join("other/branch.txt"), "branch\n");
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "--quiet", "-m", "branch only"]);
    let (code, out) = check(&work, "other");
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("not in the last"), "{out}");
}

#[test]
fn test_deploys_an_every_push_project_at_the_tip_of_main() {
    let (work, first) = setup();
    push_change(&work, "other/src.txt", &first);
    git(&work, &["checkout", "--quiet", "--detach", "origin/main"]);
    let (code, out) = check(&work, "snap");
    assert_eq!(code, 0, "{out}");
}

#[test]
fn test_skips_an_every_push_project_once_any_newer_commit_lands() {
    let (work, first) = setup();
    push_change(&work, "other/src.txt", &first);
    let (code, out) = check(&work, "snap");
    assert_eq!(code, 3, "every commit changes what snap shows: {out}");
    assert!(out.contains("deploys on every push"), "{out}");
}

fn affected(work: &Path, args: &[&str], base: &str) -> String {
    let out = Command::new("bash")
        .arg("standards/scripts/affected.sh")
        .args(args)
        .arg(base)
        .current_dir(work)
        .env_remove("AFFECTED_DEPLOY")
        .env_remove("AFFECTED_CHANGED_ONLY")
        .output()
        .unwrap_or_else(|e| panic!("run affected.sh: {e}"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_deploy_runs_always_list_every_push_projects() {
    let (work, first) = setup();
    push_change(&work, "other/src.txt", "main");
    assert_eq!(affected(&work, &[], &first), r#"["other"]"#);
    assert_eq!(
        affected(&work, &["--deploy"], &first),
        r#"["other","snap"]"#
    );
}
