//! Walks the fixture repo, copied into a temporary directory under real file
//! names, and checks the snapshot built from it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use pane_model::{DeliverableKind, ProjectStatus, Snapshot, WorkKind};
use pane_snapshot::repo;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A temporary copy of `tests/fixtures/repo` with `.fixture` stripped from
/// every file name, plus issue files the walk must skip.
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
            fs::create_dir_all(dir.join(skipped).join("issues"))?;
            fs::write(
                dir.join(skipped).join("issues/skip.md"),
                "---\nkind: todo\npriority: priority\n---\n\nMust be skipped.\n",
            )?;
            fs::write(
                dir.join(skipped).join("gen.rs"),
                "// TODO: Must be skipped.\n",
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
            (
                "app/issues/a-flat-item.md",
                Some("app"),
                WorkKind::Todo,
                "Flat item."
            ),
            (
                "app/issues/b-epic-item.md",
                Some("app"),
                WorkKind::Todo,
                "Epic item."
            ),
            (
                "app/issues/c-phase-item.md",
                Some("app"),
                WorkKind::Fix,
                "Phase item. ```markdown - TODO (agent-ignore): Inside a code block, part of the text. ```"
            ),
            (
                "app/issues/d-later.md",
                Some("app"),
                WorkKind::Todo,
                "Later."
            ),
            (
                "app/src/main.rs",
                Some("app"),
                WorkKind::Todo,
                "Line comment."
            ),
            (
                "app/src/main.rs",
                Some("app"),
                WorkKind::Fix,
                "Trailing comment."
            ),
            (
                "app/src/main.rs",
                Some("app"),
                WorkKind::Todo,
                "Block comment."
            ),
            (
                "app/src/main.rs",
                Some("app"),
                WorkKind::Todo,
                "After commented-out code."
            ),
            (
                "issues/repo-wide-item.md",
                None,
                WorkKind::Todo,
                "Repo-wide item."
            ),
        ]
    );
    Ok(())
}

#[test]
fn collects_human_issues_docs_and_retros() -> TestResult {
    let repo_dir = FixtureRepo::new("docs")?;
    let s = repo::build(&repo_dir.0, "abc", "t")?;
    assert_eq!(s.human_issues.len(), 2);
    assert_eq!(
        s.human_issues[0].file,
        "issues/human/do-the-dashboard-thing.md"
    );

    let docs: Vec<_> = s
        .deliverables
        .iter()
        .map(|d| (d.kind, d.path.as_deref(), d.project.as_deref()))
        .collect();
    assert_eq!(
        docs,
        [
            (DeliverableKind::LiveSite, None, Some("app/web")),
            (DeliverableKind::Release, None, Some("tool")),
            (
                DeliverableKind::Decision,
                Some("app/docs/decisions/0001-pick-a-thing.md"),
                Some("app")
            ),
            (
                DeliverableKind::Plan,
                Some("app/docs/plans/2026-11-big-plan.md"),
                Some("app")
            ),
            (
                DeliverableKind::Retro,
                Some("app/docs/retros/2026-11-20-launch.md"),
                Some("app")
            ),
            (
                DeliverableKind::ReviewDeck,
                Some("docs/reviews/2026-11-25-app/deck.md"),
                Some("app")
            ),
            (
                DeliverableKind::ReviewDeck,
                Some("docs/reviews/2026-11-18-tool/deck.md"),
                Some("tool")
            ),
            (
                DeliverableKind::ReviewDeck,
                Some("docs/reviews/2026-11-11-web/deck.md"),
                Some("app/web")
            ),
        ]
    );
    assert_eq!(
        s.deliverables[0].url.as_deref(),
        Some("https://app.pages.dev")
    );
    assert_eq!(
        s.deliverables[1].downloads[0].url,
        "https://github.com/jimhillr/monorepo/releases/download/tool-nightly/the_tool-macos.zip",
        "release assets are named for the Cargo package"
    );
    assert!(
        s.deliverables[2]
            .body
            .as_deref()
            .is_some_and(|b| b.starts_with('#')),
        "docs carry their Markdown"
    );
    assert_eq!(s.retros.len(), 1);
    assert_eq!(s.retros[0].actions.len(), 2);
    Ok(())
}

#[test]
fn reads_the_review_rotation() -> TestResult {
    let repo_dir = FixtureRepo::new("reviews")?;
    let s = repo::build(&repo_dir.0, "abc", "t")?;
    let reviews: Vec<_> = s
        .reviews
        .iter()
        .map(|r| (r.date.as_str(), r.project.as_str(), r.folder()))
        .collect();
    assert_eq!(
        reviews,
        [
            ("2026-11-25", "app", "2026-11-25-app"),
            ("2026-11-18", "tool", "2026-11-18-tool"),
            ("2026-11-11", "app/web", "2026-11-11-web"),
        ]
    );
    let areas: Vec<_> = s.reviews[0]
        .scorecard
        .iter()
        .map(|r| (r.area.as_str(), r.rating.as_str()))
        .collect();
    assert_eq!(areas[0], ("Goals", "Poor"));
    assert_eq!(areas.len(), 6, "{areas:?}");
    assert!(
        s.reviews[1].scorecard.is_empty(),
        "a deck that isn't in the repo has no scorecard"
    );

    fs::remove_file(repo_dir.0.join("docs/reviews/README.md"))?;
    let s = repo::build(&repo_dir.0, "abc", "t")?;
    assert!(s.reviews.is_empty(), "no rotation table, no reviews");
    Ok(())
}

#[test]
fn the_real_rotation_table_parses() -> TestResult {
    // pane reads docs/reviews/README.md's rotation table, which the
    // deep-project-review skill writes: keep its shape.
    let readme = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reviews/README.md"),
    )?;
    let reviews = pane_snapshot::parse::reviews(&readme);
    assert!(
        !reviews.is_empty(),
        "the rotation table has no rows pane can read"
    );
    for r in &reviews {
        assert!(
            r.path.starts_with("docs/reviews/") && r.path.ends_with("/deck.md"),
            "{r:?}"
        );
    }
    Ok(())
}

#[test]
fn every_real_deck_has_a_scorecard() -> TestResult {
    // pane reads each deck's Scorecard slide (`| Area | Rating | Why |`),
    // which the deep-project-review skill writes: keep its shape.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let readme = fs::read_to_string(repo.join("docs/reviews/README.md"))?;
    for r in pane_snapshot::parse::reviews(&readme) {
        let Ok(deck) = fs::read_to_string(repo.join(&r.path)) else {
            continue; // Listed before its PR merged.
        };
        let rows = pane_snapshot::parse::scorecard(&deck);
        assert!(
            !rows.is_empty(),
            "{} has no scorecard pane can read",
            r.path
        );
        for row in &rows {
            assert!(
                ["Good", "Fair", "Poor"].contains(&row.rating.as_str()),
                "{}: {row:?} is not rated Good, Fair or Poor",
                r.path
            );
        }
    }
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
