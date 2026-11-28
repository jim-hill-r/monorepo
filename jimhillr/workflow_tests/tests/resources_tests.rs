//! Tests for standards/scripts/resources.sh (standards/docs/resources.md):
//! RES-001 to RES-004 and the entries PR CI labels `needs-human`. Each test
//! builds a throwaway git repo, so nothing reads the real Cloudflare account.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use workflow_tests::*;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn script() -> PathBuf {
    get_repo_root().join("standards/scripts/resources.sh")
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "resources-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write(dir: &Path, path: &str, content: &str) {
    let path = dir.join(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write file");
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("run git");
    assert!(out.status.success(), "git {:?} failed: {:?}", args, out);
}

/// A repo whose first commit (tagged `base`) holds `files`.
fn repo(files: &[(&str, &str)]) -> PathBuf {
    let dir = temp_dir();
    git(&dir, &["init", "-q"]);
    for (path, content) in files {
        write(&dir, path, content);
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--allow-empty", "-m", "base"]);
    git(&dir, &["tag", "base"]);
    dir
}

fn run(dir: &Path, args: &[&str], base: Option<&str>) -> Output {
    let mut cmd = Command::new("bash");
    cmd.arg(script())
        .args(args)
        .env("RESOURCES_ROOT", dir)
        .env_remove("AFFECTED_BASE")
        .env_remove("RESOURCES_BASE")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .current_dir(dir);
    if let Some(b) = base {
        cmd.env("RESOURCES_BASE", b);
    }
    cmd.output().expect("run resources.sh")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn check(dir: &Path, base: Option<&str>) -> Output {
    run(dir, &["check"], base)
}

const SITE: &str = "name = \"jimhillr-old\"\npages_build_output_dir = \"dist\"\n";

fn retired(name: &str, url: &str, extra: &str) -> String {
    format!(
        "[[retired]]\nkind = \"pages\"\nname = \"{name}\"\nurl = \"{url}\"\nreplaced_by = \"jimhillr-new\"\nwhy = \"Renamed.\"\nretired = \"2026-11-27\"\ndelete_after = \"2026-12-04\"\ndeleted = \"\"\n{extra}"
    )
}

#[test]
fn test_check_passes_on_the_repo() {
    let out = Command::new("bash")
        .arg(script())
        .arg("check")
        .env_remove("AFFECTED_BASE")
        .env_remove("RESOURCES_BASE")
        .current_dir(get_repo_root())
        .output()
        .expect("run resources.sh");
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn test_claims_come_from_wrangler_and_release_tags() {
    let dir = repo(&[
        ("site/wrangler.toml", SITE),
        (
            "api/wrangler.toml",
            "name = \"jimhillr-api\"\n[[d1_databases]]\nbinding = \"DB\"\ndatabase_name = \"jimhillr-db\"\n[[kv_namespaces]]\nbinding = \"CACHE\"\nid = \"x\"\n[triggers]\ncrons = [\"0 6 * * 2\"]\n",
        ),
        ("game/mise.toml", "[env]\nRELEASE_TAG = \"game-nightly\"\n"),
    ]);
    let out = run(&dir, &["claims"], None);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "cron\tjimhillr-api/0 6 * * 2\nd1\tjimhillr-db\nkv\tjimhillr-api/CACHE\npages\tjimhillr-old\nrelease\tgame-nightly\nworker\tjimhillr-api\n"
    );
    // The same claims are read from a git ref.
    let at_base = run(&dir, &["claims", "base"], None);
    assert_eq!(at_base.stdout, out.stdout);
}

#[test]
fn test_res001_a_rename_needs_a_retirement() {
    let dir = repo(&[("site/wrangler.toml", SITE)]);
    write(
        &dir,
        "site/wrangler.toml",
        "name = \"jimhillr-new\"\npages_build_output_dir = \"dist\"\n",
    );
    let out = check(&dir, Some("base"));
    assert!(!out.status.success());
    assert!(
        text(&out).contains("error[RES-001] pages `jimhillr-old`"),
        "{}",
        text(&out)
    );

    // Without a base, RES-001 doesn't run (the plain audit).
    assert!(check(&dir, None).status.success());

    write(
        &dir,
        "standards/resources.toml",
        &retired("jimhillr-old", "https://jimhillr-old.pages.dev", ""),
    );
    let out = check(&dir, Some("base"));
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn test_res001_a_keep_entry_also_covers_it() {
    let dir = repo(&[("game/mise.toml", "[env]\nRELEASE_TAG = \"game-nightly\"\n")]);
    write(&dir, "game/mise.toml", "[env]\n");
    let out = check(&dir, Some("base"));
    assert!(
        text(&out).contains("release `game-nightly`"),
        "{}",
        text(&out)
    );

    write(
        &dir,
        "standards/resources.toml",
        "[[keep]]\nkind = \"release\"\nname = \"game-nightly\"\nwhy = \"Old links still point at it.\"\nreview = \"2027-05-26\"\n",
    );
    let out = check(&dir, Some("base"));
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn test_res001_a_removed_binding_needs_a_retirement() {
    let dir = repo(&[(
        "api/wrangler.toml",
        "name = \"jimhillr-api\"\n[[d1_databases]]\nbinding = \"DB\"\ndatabase_name = \"jimhillr-db\"\n",
    )]);
    write(&dir, "api/wrangler.toml", "name = \"jimhillr-api\"\n");
    let out = check(&dir, Some("base"));
    assert!(text(&out).contains("d1 `jimhillr-db`"), "{}", text(&out));
}

#[test]
fn test_res001_a_missing_base_is_an_error() {
    let dir = repo(&[]);
    let out = check(&dir, Some("no-such-ref"));
    assert!(!out.status.success());
    assert!(text(&out).contains("is not a commit"), "{}", text(&out));
}

#[test]
fn test_res002_no_references_to_retired_addresses() {
    let dir = repo(&[(
        "standards/resources.toml",
        &retired("jimhillr-old", "https://jimhillr-old.pages.dev", ""),
    )]);
    write(
        &dir,
        "README.md",
        "Play at https://jimhillr-old.pages.dev/\n",
    );
    write(
        &dir,
        "site/test.rs",
        "const BASE: &str = \"jimhillr-old\";\n",
    );
    git(&dir, &["add", "-A"]);
    let out = check(&dir, None);
    assert!(!out.status.success());
    let t = text(&out);
    assert!(t.contains("error[RES-002] README.md:1"), "{t}");
    assert!(t.contains("error[RES-002] site/test.rs:1"), "{t}");
}

#[test]
fn test_res002_skips_dated_history_and_longer_names() {
    let dir = repo(&[(
        "standards/resources.toml",
        &retired("pane", "https://pane-dbc.pages.dev", ""),
    )]);
    for path in [
        "standards/docs/plans/2026-11-rename.md",
        "standards/docs/decisions/0001-x.md",
        "pane/docs/reviews/2026-11-27.md",
        "infra/docs/issues/human/delete-old.md",
        "CHANGELOG.md",
    ] {
        write(&dir, path, "Old site: https://pane-dbc.pages.dev\n");
    }
    // A longer name that contains the retired one is a different resource.
    write(
        &dir,
        "pane/cloudflare/wrangler.toml",
        "name = \"jimhillr-pane\"\n",
    );
    git(&dir, &["add", "-A"]);
    let out = check(&dir, None);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn test_res002_match_name_false_checks_only_the_url() {
    let dir = repo(&[(
        "standards/resources.toml",
        &retired("pane", "https://pane-dbc.pages.dev", "match_name = false\n"),
    )]);
    write(&dir, "pane/README.md", "The pane project.\n");
    git(&dir, &["add", "-A"]);
    assert!(check(&dir, None).status.success());

    // With the default, the bare word is a reference.
    write(
        &dir,
        "standards/resources.toml",
        &retired("pane", "https://pane-dbc.pages.dev", ""),
    );
    let out = check(&dir, None);
    assert!(text(&out).contains("pane/README.md:1"), "{}", text(&out));
}

fn upload(retention: &str) -> String {
    format!(
        "jobs:\n  a:\n    runs-on: ubuntu-24.04\n    steps:\n      - name: Upload\n        uses: actions/upload-artifact@v7\n        with:\n          name: x\n          path: out\n{retention}      - run: echo next\n"
    )
}

#[test]
fn test_res003_artifact_retention() {
    let cases = [
        ("          retention-days: 7\n", true),
        ("          retention-days: 1  # short\n", true),
        ("          retention-days: 14\n", false),
        ("", false),
        (
            "          # RES-003: decks are reviewed over weeks.\n          retention-days: 90\n",
            true,
        ),
    ];
    for (retention, ok) in cases {
        let dir = repo(&[(".github/workflows/ci.yml", &upload(retention))]);
        let out = check(&dir, None);
        assert_eq!(out.status.success(), ok, "{retention:?}: {}", text(&out));
        if !ok {
            assert!(text(&out).contains("error[RES-003]"), "{}", text(&out));
        }
    }
}

#[test]
fn test_res003_a_reason_in_another_step_does_not_count() {
    let wf = "jobs:\n  a:\n    steps:\n      - run: echo hi  # RES-003: not this step\n      - uses: actions/upload-artifact@v7\n        with:\n          retention-days: 30\n";
    let dir = repo(&[(".github/workflows/ci.yml", wf)]);
    let out = check(&dir, None);
    assert!(text(&out).contains("error[RES-003]"), "{}", text(&out));
}

#[test]
fn test_res004_entries_are_well_formed() {
    let dir = repo(&[(
        "standards/resources.toml",
        "[[retired]]\nkind = \"pages\"\nname = \"x\"\nwhy = \"y\"\nretired = \"2026-11-27\"\ndelete_after = \"2026-11-30\"\n\n[[keep]]\nkind = \"website\"\nname = \"z\"\nwhy = \"y\"\nreview = \"soon\"\n",
    )]);
    let out = check(&dir, None);
    assert!(!out.status.success());
    let t = text(&out);
    assert!(t.contains("at least 7 days after"), "{t}");
    assert!(t.contains("`kind` must be one of"), "{t}");
    assert!(t.contains("`review` must be a YYYY-MM-DD date"), "{t}");
}

#[test]
fn test_new_entries_lists_only_what_the_change_adds() {
    let dir = repo(&[(
        "standards/resources.toml",
        &retired("jimhillr-old", "https://jimhillr-old.pages.dev", ""),
    )]);
    let out = run(&dir, &["new-entries", "base"], None);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");

    let mut both = retired("jimhillr-old", "https://jimhillr-old.pages.dev", "");
    both.push_str("\n[[keep]]\nkind = \"pages\"\nname = \"landing\"\nwhy = \"Jim's.\"\nreview = \"2027-05-26\"\n");
    write(&dir, "standards/resources.toml", &both);
    let out = run(&dir, &["new-entries", "base"], None);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "keep\tpages\tlanding\n"
    );
}

#[test]
fn test_wired_into_the_audit_hooks_and_pr_ci() {
    let root = get_repo_root();
    let read = |p: &str| fs::read_to_string(root.join(p)).expect(p);
    assert!(read("standards/scripts/audit.sh").contains("resources.sh check"));
    assert!(read("standards/scripts/hooks.sh").contains("RESOURCES_BASE="));
    let ci = read(".github/workflows/pull-request-ci.yml");
    assert!(ci.contains("resources.sh new-entries"));
    assert!(ci.contains("needs-human"));
}
