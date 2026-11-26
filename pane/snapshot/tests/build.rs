//! Walks the fixture repo, copied into a temporary directory under real file
//! names, and checks the snapshot built from it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use pane_model::{ProjectStatus, Snapshot, WorkKind};
use pane_snapshot::repo;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A temporary copy of `tests/fixtures/repo` with `.fixture` stripped from
/// every file name, plus `ISSUES.md` files the walk must skip.
struct FixtureRepo(PathBuf);

impl FixtureRepo {
    fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("pane_snapshot_{name}_{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/repo");
        copy_fixtures(&src, &dir)?;
        for skipped in [
            "app/target/debug",
            "app/web/node_modules/x",
            ".git",
            "app/dist",
        ] {
            fs::create_dir_all(dir.join(skipped))?;
            fs::write(
                dir.join(skipped).join("ISSUES.md"),
                "# Priority Issues\n\n- TODO: Must be skipped.\n",
            )?;
        }
        Ok(Self(dir))
    }
}

impl Drop for FixtureRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_fixtures(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir() {
            copy_fixtures(&entry.path(), &dst.join(&name))?;
        } else if let Some(real) = name.strip_suffix(".fixture") {
            fs::copy(entry.path(), dst.join(real))?;
        }
    }
    Ok(())
}

#[test]
fn builds_projects_from_config_roots() -> TestResult {
    let repo_dir = FixtureRepo::new("projects")?;
    let s = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")?;
    assert_eq!(s.commit, "abc1234");
    assert_eq!(s.taken_at, "2026-11-26T12:00:00Z");

    let paths: Vec<_> = s.projects.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, ["app", "app/web", "tool"]);

    let app = &s.projects[0];
    assert_eq!(app.kind, "rust");
    assert_eq!(app.status, ProjectStatus::Priority);
    assert_eq!(
        app.summary.as_deref(),
        Some("App does one thing well. It is named for what it is.")
    );
    assert_eq!(app.deploy_url, None);

    let web = &s.projects[1];
    assert_eq!(web.kind, "dioxus-web");
    assert_eq!(web.deploy_url.as_deref(), Some("https://app.pages.dev"));
    assert_eq!(web.status, ProjectStatus::Priority);
    assert_eq!(web.summary, None);

    let tool = &s.projects[2];
    assert_eq!(tool.status, ProjectStatus::OnHold);
    assert_eq!(
        tool.summary, None,
        "a README with only a heading has no summary"
    );
    Ok(())
}

#[test]
fn collects_every_issues_file_and_skips_build_dirs() -> TestResult {
    let repo_dir = FixtureRepo::new("items")?;
    let s = repo::build(&repo_dir.0, "abc", "t")?;
    let items: Vec<_> = s
        .work_items
        .iter()
        .map(|i| {
            (
                i.file.as_str(),
                i.project.as_deref(),
                i.kind,
                i.text.as_str(),
            )
        })
        .collect();
    assert_eq!(
        items,
        [
            ("ISSUES.md", None, WorkKind::Todo, "Repo-wide item."),
            ("app/ISSUES.md", Some("app"), WorkKind::Todo, "Flat item."),
            ("app/ISSUES.md", Some("app"), WorkKind::Todo, "Epic item."),
            ("app/ISSUES.md", Some("app"), WorkKind::Fix, "Phase item."),
            ("app/ISSUES.md", Some("app"), WorkKind::Todo, "Later."),
        ]
    );
    Ok(())
}

#[test]
fn fails_without_config_roots() -> TestResult {
    let repo_dir = FixtureRepo::new("no_roots")?;
    fs::write(repo_dir.0.join("mise.toml"), "[tools]\n")?;
    assert!(repo::build(&repo_dir.0, "a", "t").is_err());
    fs::remove_file(repo_dir.0.join("mise.toml"))?;
    assert!(repo::build(&repo_dir.0, "a", "t").is_err());
    Ok(())
}

#[test]
fn binary_writes_a_readable_snapshot() -> TestResult {
    let repo_dir = FixtureRepo::new("binary")?;
    let out = repo_dir.0.join("out/data/snapshot.json");
    let status = Command::new(env!("CARGO_BIN_EXE_pane_snapshot"))
        .arg("--repo")
        .arg(&repo_dir.0)
        .arg("--out")
        .arg(&out)
        .args(["--commit", "deadbeef"])
        .status()?;
    assert!(status.success());
    let s = Snapshot::from_json(&fs::read_to_string(&out)?)?;
    assert_eq!(s.commit, "deadbeef");
    assert_eq!(s.projects.len(), 3);
    assert_eq!(s.taken_at.len(), 20, "defaults to now as RFC 3339");
    Ok(())
}

#[test]
fn binary_rejects_unknown_arguments() -> TestResult {
    let status = Command::new(env!("CARGO_BIN_EXE_pane_snapshot"))
        .arg("--nope")
        .status()?;
    assert!(!status.success());
    Ok(())
}
