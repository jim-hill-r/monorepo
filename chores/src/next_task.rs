//! `chores next-task`: pick the next issue the way the `start-a-new-task`
//! skill orders them.
//!
//! 1. `priority` issues of each project under `# Priority Projects` in
//!    `standards/docs/issues/README.md`, in list order.
//! 2. Any other `priority` issue, then backlog issues (the priority
//!    projects in list order, then the rest by name).
//!
//! Every issue lives in its project's `docs/issues/`; repo-wide work is in
//! `standards/docs/issues/`.
//!
//! Within a group, issues sort by project, epic, phase, `order` and path, so
//! an epic's earliest phase and lowest order come first. Human tasks,
//! `agent-ignore` issues and on-hold projects are never picked.
//!
//! Tier balance (Jim, 2026-11-28): when the product share of recent work
//! (threads working now and PRs merged in the last week, see
//! [`crate::tiers`]) is under half, each pick takes the first qualifying
//! product issue in that order instead, if there is one. A soft heuristic:
//! with no product issue left it falls back to the plain order.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::issues::{self, Issue, ProjectLists};
use crate::tiers::{self, Balance, Side};

/// What to leave out and how many to pick.
#[derive(Debug, Default)]
pub struct Options {
    /// Only issues of these projects (a top-level folder, or a path prefix
    /// such as `cahokia/web`). `repo` means `standards`, home of repo-wide
    /// work.
    pub projects: Vec<String>,
    /// Skip these projects or epics: work another thread already has.
    pub exclude: Vec<String>,
    /// Skip these issue files (paths from the repo root), such as ones an
    /// open PR already deletes.
    pub skip: Vec<String>,
    /// How many to pick. With more than one, each pick is from a different
    /// project (or, in `standards`, a different epic).
    pub count: usize,
    /// Projects (or `epic:` names, `repo`) of the threads working now, one
    /// entry per thread, for the tier balance.
    pub working: Vec<String>,
    /// Days of `main` history to count merged PRs over (0: none).
    pub days: u32,
    /// Turn the tier balance off and use the plain order.
    pub no_balance: bool,
}

#[derive(Debug, Serialize)]
pub struct Pick {
    pub issue: Issue,
    /// Which project or epic this pick takes; pass it to `--exclude` to
    /// keep another thread off it.
    pub key: String,
    /// Product or platform (decision 0019's tiers).
    pub side: Side,
    /// Why it was chosen, in words.
    pub reason: String,
    /// The folder's README.md, when it has one.
    pub readme: Option<String>,
    /// The README section named after the issue's epic.
    pub epic_notes: Option<String>,
}

/// A `TODO` or `FIX` comment, the last place to look.
#[derive(Debug, Serialize)]
pub struct Todo {
    pub path: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub picks: Vec<Pick>,
    /// The product/platform split before these picks, when balancing.
    pub balance: Option<Balance>,
    /// Only filled when no issue file qualified.
    pub todos: Vec<Todo>,
}

pub fn run(root: &Path, options: &Options) -> Result<Outcome> {
    let lists = issues::project_lists(root)?;
    let all = issues::load(root)?;
    let balance = (options.projects.is_empty() && !options.no_balance).then(|| {
        let mut b = Balance::default();
        tiers::count_working(root, &options.working, &mut b);
        if options.days > 0 {
            tiers::count_merged(root, options.days, &mut b);
        }
        b
    });
    let side = |i: &Issue| tiers::side(root, i.project());
    let mut picks = pick_balanced(&all, &lists, options, balance.clone(), &side);
    for p in &mut picks {
        let readme = format!("{}/README.md", p.issue.folder());
        if let Ok(text) = fs::read_to_string(root.join(&readme)) {
            p.epic_notes = p
                .issue
                .epic_name()
                .and_then(|e| issues::epic_notes(&text, e));
            p.readme = Some(readme);
        }
    }
    let todos = if picks.is_empty() {
        find_todos(root, &lists, options)?
    } else {
        Vec::new()
    };
    Ok(Outcome {
        picks,
        balance,
        todos,
    })
}

/// The ordering key: (group, rank within the group's project list).
fn group(issue: &Issue, lists: &ProjectLists) -> (u8, usize, String) {
    let project = issue.project();
    let listed = project.and_then(|p| lists.priority.iter().position(|l| l == p));
    let name = project.unwrap_or("").to_string();
    let priority = issue.priority == "priority";
    match (priority, listed) {
        (true, Some(i)) => (1, i, name),
        (true, None) => (2, 0, name),
        (false, Some(i)) => (4, i, name),
        (false, None) => (5, 0, name),
    }
}

fn reason(issue: &Issue, lists: &ProjectLists) -> String {
    let (g, i, name) = group(issue, lists);
    match g {
        1 => format!("priority issue of priority project #{} ({name})", i + 1),
        2 => format!("priority issue of {name}"),
        4 => format!("backlog issue of priority project #{} ({name})", i + 1),
        _ => format!("backlog issue of {name}"),
    }
}

/// The project holding repo-wide work, whose picks are keyed by epic so
/// several threads can share it.
const REPO_WIDE: &str = "standards";

/// The project or epic a pick takes up.
pub fn key(issue: &Issue) -> String {
    match (issue.project(), issue.epic_name()) {
        (Some(REPO_WIDE), Some(e)) => format!("epic:{e}"),
        (Some(REPO_WIDE), None) => format!("repo:{}", issue.path),
        (Some(p), _) => p.to_string(),
        (None, _) => format!("repo:{}", issue.path),
    }
}

fn matches_project(issue: &Issue, wanted: &str) -> bool {
    let wanted = wanted.trim_end_matches('/');
    let wanted = if wanted == "repo" { REPO_WIDE } else { wanted };
    issue.path.starts_with(&format!("{wanted}/"))
}

fn excluded(issue: &Issue, exclude: &[String]) -> bool {
    let key = key(issue);
    exclude.iter().any(|e| {
        let e = e.trim();
        e == key
            || matches_project(issue, e)
            || issue
                .epic_name()
                .is_some_and(|epic| epic.eq_ignore_ascii_case(e.trim_start_matches("epic:")))
    })
}

/// The plain order, with no tier balance.
#[cfg(test)]
pub fn pick(all: &[Issue], lists: &ProjectLists, options: &Options) -> Vec<Pick> {
    pick_balanced(all, lists, options, None, &|_| Side::Platform)
}

fn candidates<'a>(all: &'a [Issue], lists: &ProjectLists, options: &Options) -> Vec<&'a Issue> {
    let mut candidates: Vec<&Issue> = all
        .iter()
        .filter(|i| !i.is_human() && !i.has_tag("agent-ignore"))
        .filter(|i| matches!(i.kind.as_str(), "todo" | "fix"))
        .filter(|i| {
            i.project()
                .is_none_or(|p| !lists.on_hold.iter().any(|h| h == p))
        })
        .filter(|i| {
            options.projects.is_empty() || options.projects.iter().any(|p| matches_project(i, p))
        })
        .filter(|i| !excluded(i, &options.exclude))
        .filter(|i| {
            !options
                .skip
                .iter()
                .any(|s| s.trim_start_matches("./") == i.path)
        })
        .collect();
    candidates.sort_by_key(|i| {
        (
            group(i, lists),
            i.epic_name().unwrap_or("").to_string(),
            i.phase().unwrap_or(0),
            i.order.unwrap_or(u64::MAX),
            i.path.clone(),
        )
    });
    candidates
}

/// Picks in the plain order, except that while `balance` says products
/// are under half, each pick is the first product issue left (if any).
/// Each pick then counts as a working thread on its side.
pub fn pick_balanced(
    all: &[Issue],
    lists: &ProjectLists,
    options: &Options,
    mut balance: Option<Balance>,
    side: &dyn Fn(&Issue) -> Side,
) -> Vec<Pick> {
    let candidates = candidates(all, lists, options);
    let count = options.count.max(1);
    let mut picks: Vec<Pick> = Vec::new();
    while picks.len() < count {
        let free = |i: &&&Issue| !picks.iter().any(|p| p.key == key(i));
        let wants_product = balance.as_ref().is_some_and(Balance::wants_product);
        let preferred = if wants_product {
            candidates
                .iter()
                .filter(free)
                .find(|i| side(i) == Side::Product)
        } else {
            None
        };
        let first = candidates.iter().find(free);
        let Some(issue) = preferred.or(first) else {
            break;
        };
        let s = side(issue);
        let mut why = reason(issue, lists);
        if let (Some(b), Some(f)) = (&balance, first)
            && wants_product
        {
            let share = b.product_share().unwrap_or(0.0) * 100.0;
            if s == Side::Platform {
                why.push_str(" (no product issue left to prefer)");
            } else if f.path == issue.path {
                let _ = write!(
                    why,
                    "; first in order and a product issue (product at {share:.0}%)"
                );
            } else {
                let _ = write!(
                    why,
                    "; product preferred over {} (product at {share:.0}%, under the 50% floor)",
                    f.path
                );
            }
        }
        if let Some(b) = &mut balance {
            b.add_working(s);
        }
        picks.push(Pick {
            reason: why,
            key: key(issue),
            side: s,
            issue: (*issue).clone(),
            readme: None,
            epic_notes: None,
        });
    }
    picks
}

/// `TODO` and `FIX` comments in code, skipping `agent-ignore` ones,
/// Markdown, on-hold projects and build output. At most `count` (min 5).
fn find_todos(root: &Path, lists: &ProjectLists, options: &Options) -> Result<Vec<Todo>> {
    let mut out = Vec::new();
    let limit = options.count.max(5);
    scan(root, root, lists, options, limit, &mut out)?;
    Ok(out)
}

const CODE: &[&str] = &[
    "rs", "ts", "tsx", "js", "mjs", "sh", "toml", "yml", "yaml", "py", "css", "html",
];
const MARKERS: &[&str] = &["// ", "# ", "/* ", "<!-- ", "-- ", "* "];

fn scan(
    root: &Path,
    dir: &Path,
    lists: &ProjectLists,
    options: &Options,
    limit: usize,
    out: &mut Vec<Todo>,
) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        if out.len() >= limit {
            return Ok(());
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = issues::relative(root, &path);
        let top = rel.split('/').next().unwrap_or("");
        if dir == root
            && (lists.on_hold.iter().any(|h| h == top)
                || options.exclude.iter().any(|e| e == top)
                || (!options.projects.is_empty() && !options.projects.iter().any(|p| p == top)))
        {
            continue;
        }
        if entry.file_type()?.is_dir() {
            if !(name.starts_with('.')
                || name == "target"
                || name == "node_modules"
                || name == "dist")
            {
                scan(root, &path, lists, options, limit, out)?;
            }
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !CODE.contains(&ext) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            if out.len() >= limit {
                break;
            }
            let t = line.trim_start();
            let hit = MARKERS.iter().any(|m| {
                t.strip_prefix(m.trim_end()).is_some_and(|rest| {
                    let rest = rest.trim_start();
                    rest.starts_with("TODO") || rest.starts_with("FIX")
                })
            });
            if hit && !t.contains("agent-ignore") {
                out.push(Todo {
                    path: rel.clone(),
                    line: n + 1,
                    text: t.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// The plain-text report agents read.
pub fn render(outcome: &Outcome) -> String {
    let mut s = String::new();
    if outcome.picks.is_empty() && outcome.todos.is_empty() {
        s.push_str(
            "No issue qualifies. Investigate the codebase and record a few real issues \
             (start-a-new-task step 1.5).\n",
        );
        return s;
    }
    if outcome.picks.is_empty() {
        s.push_str("No issue file qualifies. TODO/FIX comments to work instead:\n");
        for t in &outcome.todos {
            let _ = writeln!(s, "- {}:{}: {}", t.path, t.line, t.text);
        }
        return s;
    }
    if let Some(b) = &outcome.balance {
        let _ = writeln!(s, "Tier balance: {}\n", b.describe());
    }
    for (n, p) in outcome.picks.iter().enumerate() {
        let i = &p.issue;
        let _ = writeln!(s, "{}. {}", n + 1, i.path);
        let _ = writeln!(
            s,
            "   {} {} | {} | {} | key: {}",
            i.priority,
            i.kind.to_uppercase(),
            match p.side {
                Side::Product => "product",
                Side::Platform => "platform",
            },
            p.reason,
            p.key
        );
        if let Some(epic) = &i.epic {
            let order = i.order.map(|o| format!(", order {o}")).unwrap_or_default();
            let _ = writeln!(s, "   epic: {epic}{order}");
        }
        if !i.tags.is_empty() {
            let _ = writeln!(s, "   tags: {}", i.tags.join(", "));
        }
        if let Some(readme) = &p.readme {
            let _ = writeln!(s, "   folder notes: {readme}");
        }
        s.push('\n');
        for line in i.body.lines() {
            let _ = writeln!(s, "   {line}");
        }
        if let Some(notes) = &p.epic_notes {
            s.push_str("\n   Epic notes from the folder README:\n");
            for line in notes.lines() {
                let _ = writeln!(s, "   > {line}");
            }
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(path: &str, priority: &str, extra: &str) -> Issue {
        let text = format!(
            "---\nkind: todo\npriority: {priority}\n{extra}added: 2026-11-27\n---\n\nDo {path}.\n"
        );
        issues::parse(path, &text).expect("front matter")
    }

    fn lists() -> ProjectLists {
        ProjectLists {
            priority: vec!["sparkhill".into(), "cookbook".into()],
            on_hold: vec!["luggage".into()],
        }
    }

    fn paths(picks: &[Pick]) -> Vec<&str> {
        picks.iter().map(|p| p.issue.path.as_str()).collect()
    }

    fn all() -> Vec<Issue> {
        vec![
            issue("pane/docs/issues/p.md", "priority", ""),
            issue("cookbook/docs/issues/c.md", "priority", ""),
            issue("sparkhill/webapp/docs/issues/s.md", "priority", ""),
            issue("standards/docs/issues/r.md", "priority", ""),
            issue("standards/docs/issues/b.md", "backlog", ""),
            issue("luggage/docs/issues/l.md", "priority", ""),
            issue("homer/docs/issues/h.md", "backlog", ""),
            issue("cost/docs/issues/human/x.md", "priority", ""),
            issue(
                "bogan/docs/issues/i.md",
                "priority",
                "tags: [agent-ignore]\n",
            ),
        ]
    }

    #[test]
    fn follows_the_skill_order() {
        let opts = Options {
            count: 10,
            ..Options::default()
        };
        assert_eq!(
            paths(&pick(&all(), &lists(), &opts)),
            vec![
                "sparkhill/webapp/docs/issues/s.md",
                "cookbook/docs/issues/c.md",
                "pane/docs/issues/p.md",
                "standards/docs/issues/r.md",
                "homer/docs/issues/h.md",
                "standards/docs/issues/b.md",
            ]
        );
    }

    #[test]
    fn earliest_phase_and_lowest_order_first() {
        let all = vec![
            issue(
                "standards/docs/issues/b.md",
                "priority",
                "epic: \"Cost / Phase 2: x\"\norder: 10\n",
            ),
            issue(
                "standards/docs/issues/a.md",
                "priority",
                "epic: \"Cost / Phase 1: y\"\norder: 20\n",
            ),
            issue(
                "standards/docs/issues/c.md",
                "priority",
                "epic: \"Cost / Phase 1: y\"\norder: 10\n",
            ),
        ];
        let picks = pick(&all, &lists(), &Options::default());
        assert_eq!(paths(&picks), vec!["standards/docs/issues/c.md"]);
        assert_eq!(picks[0].key, "epic:Cost");
    }

    #[test]
    fn one_pick_per_project_and_exclusions() {
        let mut all = all();
        all.push(issue("cookbook/web/docs/issues/w.md", "priority", ""));
        let opts = Options {
            count: 3,
            exclude: vec!["sparkhill".into()],
            ..Options::default()
        };
        assert_eq!(
            paths(&pick(&all, &lists(), &opts)),
            vec![
                "cookbook/docs/issues/c.md",
                "pane/docs/issues/p.md",
                "standards/docs/issues/r.md"
            ]
        );
    }

    #[test]
    fn scoped_to_a_project_and_skips() {
        let opts = Options {
            projects: vec!["cookbook".into()],
            ..Options::default()
        };
        assert_eq!(
            paths(&pick(&all(), &lists(), &opts)),
            vec!["cookbook/docs/issues/c.md"]
        );
        let opts = Options {
            projects: vec!["cookbook".into()],
            skip: vec!["cookbook/docs/issues/c.md".into()],
            ..Options::default()
        };
        assert!(pick(&all(), &lists(), &opts).is_empty());
        let opts = Options {
            projects: vec!["repo".into()],
            exclude: vec!["epic:Nope".into()],
            ..Options::default()
        };
        assert_eq!(
            paths(&pick(&all(), &lists(), &opts)),
            vec!["standards/docs/issues/r.md"]
        );
    }

    fn product_side(i: &Issue) -> Side {
        match i.project() {
            Some("cookbook" | "sparkhill" | "homer") => Side::Product,
            _ => Side::Platform,
        }
    }

    #[test]
    fn prefers_product_while_products_are_under_half() {
        let opts = Options {
            count: 4,
            ..Options::default()
        };
        // 1 product thread and 3 platform ones: 25%.
        let balance = Balance {
            working_product: 1,
            working_platform: 3,
            ..Balance::default()
        };
        let picks = pick_balanced(&all(), &lists(), &opts, Some(balance), &product_side);
        // Products first until the share reaches half (3 of 6), then the
        // plain order resumes with pane, and the next product (a backlog
        // one) is preferred over the standards priority issue.
        assert_eq!(
            paths(&picks),
            vec![
                "sparkhill/webapp/docs/issues/s.md",
                "cookbook/docs/issues/c.md",
                "pane/docs/issues/p.md",
                "homer/docs/issues/h.md",
            ]
        );
        assert!(
            picks[0]
                .reason
                .contains("first in order and a product issue")
        );
        assert_eq!(picks[2].side, Side::Platform);
        assert!(
            picks[3]
                .reason
                .contains("product preferred over standards/docs/issues/r.md")
        );
    }

    #[test]
    fn plain_order_when_products_are_at_half_or_unknown() {
        let opts = Options {
            count: 2,
            ..Options::default()
        };
        let even = Balance {
            working_product: 2,
            working_platform: 2,
            ..Balance::default()
        };
        let plain = vec![
            "sparkhill/webapp/docs/issues/s.md",
            "cookbook/docs/issues/c.md",
        ];
        assert_eq!(
            paths(&pick_balanced(
                &all(),
                &lists(),
                &opts,
                Some(even),
                &product_side
            )),
            plain
        );
        assert_eq!(
            paths(&pick_balanced(
                &all(),
                &lists(),
                &opts,
                Some(Balance::default()),
                &product_side
            )),
            plain
        );
    }

    #[test]
    fn falls_back_to_platform_when_no_product_issue_is_left() {
        let only_platform = vec![
            issue("standards/docs/issues/r.md", "priority", ""),
            issue("pane/docs/issues/p.md", "backlog", ""),
        ];
        let behind = Balance {
            working_platform: 5,
            ..Balance::default()
        };
        let opts = Options {
            count: 2,
            ..Options::default()
        };
        let picks = pick_balanced(&only_platform, &lists(), &opts, Some(behind), &product_side);
        assert_eq!(
            paths(&picks),
            vec!["standards/docs/issues/r.md", "pane/docs/issues/p.md"]
        );
        assert!(picks[0].reason.contains("no product issue left"));
    }

    #[test]
    fn falls_back_to_todo_comments() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("standards/docs/issues"))?;
        fs::write(
            root.join("standards/docs/issues/README.md"),
            "# Priority Projects\n\n# On Hold Projects\n- luggage\n",
        )?;
        fs::create_dir_all(root.join("homer/src"))?;
        fs::create_dir_all(root.join("luggage/src"))?;
        fs::write(
            root.join("homer/src/lib.rs"),
            "fn a() {}\n// TODO (agent-generated): validate input\n// TODO (agent-ignore): leave\nlet todo = 1;\n",
        )?;
        fs::write(root.join("luggage/src/lib.rs"), "// FIX: on hold\n")?;
        let outcome = run(root, &Options::default())?;
        assert!(outcome.picks.is_empty());
        assert_eq!(outcome.todos.len(), 1);
        assert_eq!(outcome.todos[0].path, "homer/src/lib.rs");
        assert_eq!(outcome.todos[0].line, 2);
        assert!(render(&outcome).contains("homer/src/lib.rs:2"));
        Ok(())
    }

    #[test]
    fn reads_epic_notes_from_the_folder_readme() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("standards/docs/issues"))?;
        fs::write(
            root.join("standards/docs/issues/README.md"),
            "# Priority Projects\n- cookbook\n\n# Epic notes\n\n## Cost check\n\nSee the plan.\n",
        )?;
        fs::write(
            root.join("standards/docs/issues/x.md"),
            "---\nkind: fix\npriority: priority\nepic: \"Cost check / Phase 3: strict\"\norder: 10\nadded: 2026-11-27\n---\n\nMake it strict.\n",
        )?;
        let outcome = run(root, &Options::default())?;
        assert_eq!(outcome.picks.len(), 1);
        assert_eq!(
            outcome.picks[0].epic_notes.as_deref(),
            Some("See the plan.")
        );
        let text = render(&outcome);
        assert!(text.contains("1. standards/docs/issues/x.md"));
        assert!(text.contains("priority FIX | platform | priority issue of standards"));
        assert!(text.contains("   Make it strict."));
        assert!(text.contains("   > See the plan."));
        Ok(())
    }
}
