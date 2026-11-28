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
            fs::create_dir_all(dir.join(skipped).join("docs/issues"))?;
            fs::write(
                dir.join(skipped).join("docs/issues/skip.md"),
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
    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
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
                "app/docs/issues/a-flat-item.md",
                Some("app"),
                WorkKind::Todo,
                "Flat item."
            ),
            (
                "app/docs/issues/b-epic-item.md",
                Some("app"),
                WorkKind::Todo,
                "Epic item."
            ),
            (
                "app/docs/issues/c-phase-item.md",
                Some("app"),
                WorkKind::Fix,
                "Phase item. ```markdown - TODO (agent-ignore): Inside a code block, part of the text. ```"
            ),
            (
                "app/docs/issues/d-later.md",
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
                "standards/docs/issues/repo-wide-item.md",
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
    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
    // Human tasks come from every project's docs/issues/human/.
    let human: Vec<_> = s
        .human_issues
        .iter()
        .map(|h| (h.file.as_str(), h.project.as_deref()))
        .collect();
    assert_eq!(
        human,
        [
            (
                "app/docs/issues/human/do-the-dashboard-thing.md",
                Some("app")
            ),
            ("standards/docs/issues/human/no-note.md", None),
        ]
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
    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
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
    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
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

/// Run git in `dir` with a fixed identity and dates, failing on error.
fn git(dir: &Path, args: &[&str]) -> TestResult {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-11-01T10:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-11-01T10:00:00Z")
        .status()?;
    if !status.success() {
        return Err(format!("git {args:?} failed").into());
    }
    Ok(())
}

#[test]
fn dates_fix_comments_by_git_blame() -> TestResult {
    let repo_dir = FixtureRepo::new("blame")?;
    // The fixture's fake .git folder is only there to be skipped.
    fs::remove_dir_all(repo_dir.0.join(".git"))?;
    git(&repo_dir.0, &["init", "-q"])?;
    git(&repo_dir.0, &["add", "-A"])?;
    git(&repo_dir.0, &["commit", "-q", "-m", "fixture"])?;

    let s = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")?;
    let fix = s
        .work_items
        .iter()
        .find(|w| w.kind == WorkKind::Fix && w.file == "app/src/main.rs")
        .ok_or("no FIX comment")?;
    assert_eq!(fix.added.as_deref(), Some("2026-11-01"));
    // TODO comments aren't blamed.
    assert!(
        s.work_items
            .iter()
            .filter(|w| w.kind == WorkKind::Todo && w.file == "app/src/main.rs")
            .all(|w| w.added.is_none())
    );
    Ok(())
}

#[test]
fn fix_comments_outside_git_have_no_date() -> TestResult {
    let repo_dir = FixtureRepo::new("noblame")?;
    let s = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")?;
    let fix = s
        .work_items
        .iter()
        .find(|w| w.kind == WorkKind::Fix && w.file == "app/src/main.rs")
        .ok_or("no FIX comment")?;
    assert_eq!(fix.added, None);
    Ok(())
}

#[test]
fn reads_health_thresholds_into_the_snapshot() -> TestResult {
    let repo_dir = FixtureRepo::new("health")?;
    let s = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")?;
    assert_eq!(s.thresholds, pane_model::health::Thresholds::default());

    fs::create_dir_all(repo_dir.0.join("pane"))?;
    fs::write(
        repo_dir.0.join(repo::HEALTH_TOML),
        "[open_issues]\nred_fix_open_days = 3\n",
    )?;
    let s = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")?;
    assert_eq!(s.thresholds.open_issues.red_fix_open_days, 3);
    assert_eq!(s.thresholds.build.stale_after_days, 2);

    fs::write(
        repo_dir.0.join(repo::HEALTH_TOML),
        "[open_issues]\ntypo = 3\n",
    )?;
    let err = repo::build(&repo_dir.0, "abc1234", "2026-11-26T12:00:00Z")
        .err()
        .ok_or("a bad health.toml must fail the snapshot")?;
    assert!(err.contains("pane/health.toml"), "{err}");
    Ok(())
}

#[test]
fn the_real_health_toml_lights_real_projects() -> TestResult {
    // The real repo: every project gets its four snapshot signals.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let s = repo::build(&root, "abc1234", "2026-11-27T12:00:00Z")?;
    for project in &s.projects {
        let signals = pane_model::signals::project_signals(&s, &project.path);
        assert_eq!(signals.len(), 4, "{}", project.path);
    }
    Ok(())
}

#[test]
fn reads_cost_files_budgets_and_limits() -> TestResult {
    let repo_dir = FixtureRepo::new("costs")?;
    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
    let costs = &s.costs;
    assert_eq!(costs.month, "2026-11");

    // app/web has no cost file, so it is left out.
    let projects: Vec<_> = costs.projects.iter().map(|p| p.project.as_str()).collect();
    assert_eq!(projects, ["app", "tool"]);
    let app = &costs.projects[0];
    assert_eq!(app.file, "app/docs/cost.toml");
    assert_eq!(app.reviewed, "2026-11-25");
    assert_eq!(app.services[0].blip, "Cloudflare Pages");
    assert_eq!(app.services[0].plan, "free");
    assert_eq!(app.services[0].usage[0].expected, 2);
    // $12 a year is $1 a month; $5 of revenue in November.
    assert_eq!((app.direct_cost_cents, app.revenue_cents), (100, 500));
    // Each entry knows the line of its header, for links.
    assert_eq!(app.services[0].line, 4);
    assert_eq!(app.costs[0].line, 11);
    assert_eq!(app.revenue[0].line, 20);

    // Twelve months of money for the top-level projects, first in project
    // order; outside git there are no merged PRs to split by.
    assert!(!costs.history_from_git);
    assert_eq!(costs.history.len(), 12);
    let nov = costs.history.last().ok_or("no history")?;
    assert_eq!(nov.month, "2026-11");
    let rows: Vec<_> = nov.projects.iter().map(|m| m.project.as_str()).collect();
    assert_eq!(rows.get(..2), Some(&["app", "tool"][..]));
    assert_eq!(nov.shared_cents, 3000);
    assert_eq!(nov.merged_prs, 0);
    assert_eq!(nov.project("app").map(|m| m.revenue_cents), Some(500));

    let pages = &costs.account[0];
    assert_eq!(pages.key, "pages_projects");
    assert_eq!((pages.expected, pages.ceiling), (3, Some(25)));

    let budgets: Vec<_> = costs
        .budgets
        .iter()
        .map(|b| (b.vendor.as_str(), b.budget_cents, b.spend_cents))
        .collect();
    assert_eq!(
        budgets,
        [
            ("Cloudflare Pages", 0, 0),
            ("Squarespace", 500, 100),
            ("Claude Code", 2500, 0),
        ]
    );
    Ok(())
}

#[test]
fn fails_on_an_invalid_cost_file() -> TestResult {
    let repo_dir = FixtureRepo::new("bad_cost")?;
    fs::write(repo_dir.0.join("tool/docs/cost.toml"), "summary = 1\n")?;
    let err = repo::build(&repo_dir.0, "a", "2026-11-26T12:00:00Z")
        .err()
        .ok_or("an invalid cost file must fail the snapshot")?;
    assert!(err.starts_with("tool/docs/cost.toml: "), "{err}");
    Ok(())
}

#[test]
fn fails_on_an_invalid_radar_or_limits_file() -> TestResult {
    for (file, bad) in [
        ("standards/radar/radar.json", "{"),
        ("standards/radar/limits.toml", "[[limit]]\nkey = 1\n"),
    ] {
        let repo_dir = FixtureRepo::new(&format!("bad_{}", file.replace('/', "_")))?;
        fs::write(repo_dir.0.join(file), bad)?;
        let err = repo::build(&repo_dir.0, "a", "2026-11-26T12:00:00Z")
            .err()
            .ok_or("an invalid radar or limits file must fail the snapshot")?;
        assert!(err.starts_with(file), "{err}");
    }
    Ok(())
}

#[test]
fn no_radar_means_no_limits_and_only_unlisted_vendors() -> TestResult {
    let repo_dir = FixtureRepo::new("no_radar")?;
    fs::remove_dir_all(repo_dir.0.join("standards"))?;
    let s = repo::build(&repo_dir.0, "a", "2026-11-26T12:00:00Z")?;
    assert_eq!(s.costs.projects.len(), 2);
    assert!(s.costs.account.is_empty());
    // Only app's domain, from a vendor no radar lists.
    let budgets: Vec<_> = s
        .costs
        .budgets
        .iter()
        .map(|b| {
            (
                b.vendor.as_str(),
                b.ring.as_str(),
                b.budget_cents,
                b.spend_cents,
            )
        })
        .collect();
    assert_eq!(budgets, [("Squarespace", "", 0, 100)]);
    Ok(())
}

#[test]
fn fails_on_a_taken_at_without_a_month() -> TestResult {
    let repo_dir = FixtureRepo::new("bad_taken_at")?;
    assert!(repo::build(&repo_dir.0, "a", "yesterday").is_err());
    Ok(())
}

#[test]
fn the_real_repo_cost_files_build() -> TestResult {
    // Every docs/cost.toml, the radar and the limits in this checkout must
    // parse, or pane's release fails.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let roots = pane_snapshot::parse::config_roots(&fs::read_to_string(repo.join("mise.toml"))?)?;
    let costs = repo::costs(&repo, &roots, "2026-11-26T12:00:00Z")?;
    assert!(costs.projects.iter().any(|p| p.project == "pane"));
    assert!(costs.budgets.iter().any(|b| b.vendor == "Claude Code"));
    assert!(costs.account.iter().any(|a| a.key == "pages_projects"));
    Ok(())
}

#[test]
fn splits_shared_costs_by_merged_prs_from_git() -> TestResult {
    let repo_dir = FixtureRepo::new("money_git")?;
    fs::remove_dir_all(repo_dir.0.join(".git"))?;
    git(&repo_dir.0, &["init", "-q"])?;
    git(&repo_dir.0, &["add", "-A"])?;
    git(&repo_dir.0, &["commit", "-q", "-m", "fixture (#1)"])?;
    fs::write(repo_dir.0.join("app/README.md"), "# App\n\nChanged.\n")?;
    git(&repo_dir.0, &["commit", "-q", "-am", "app: change (#2)"])?;
    fs::write(repo_dir.0.join("tool/README.md"), "# Tool\n\nChanged.\n")?;
    git(
        &repo_dir.0,
        &["commit", "-q", "-am", "a direct push, not a PR"],
    )?;

    let s = repo::build(&repo_dir.0, "abc", "2026-11-26T12:00:00Z")?;
    assert!(s.costs.history_from_git);
    let nov = s.costs.history.last().ok_or("no history")?;
    // PR 1 touched app and tool (half each), PR 2 only app: app 1.5 of 2.
    assert_eq!(nov.merged_prs, 2);
    assert_eq!(nov.project("app").map(|m| m.shared_cost_cents), Some(2250));
    assert_eq!(nov.project("tool").map(|m| m.shared_cost_cents), Some(750));
    assert_eq!(nov.totals().shared_cost_cents, 3000);
    // October had no PRs.
    let oct = &s.costs.history[10];
    assert_eq!((oct.month.as_str(), oct.merged_prs), ("2026-10", 0));
    Ok(())
}
