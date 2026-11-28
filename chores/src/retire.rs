//! `chores retire-temporary`: which temporary resources the weekly
//! `all-retire-resources.yml` run deletes ("Part 5: temporary resources" in
//! standards/docs/plans/2026-11-resource-cleanup.md, "Temporary resources"
//! in standards/docs/resources.md).
//!
//! The workflow lists what is live into one JSON file; this applies the
//! rules and writes the plan. It reads only that file, so the rules are
//! tested without an account, and the workflow does the deleting.
//!
//! | Resource | Deleted when |
//! | --- | --- |
//! | Pages preview deployment | older than 14 days, unless it is the newest one of a branch with an open PR |
//! | Pages production deployment | older than 30 days and not the project's current deployment |
//! | Branch | its newest PR closed more than 7 days ago, and no PR on it is open |
//!
//! Never deleted: the default branch, `evals-results` (the eval engine's
//! data branch), a branch with no PR (the sweep lists it and the digest
//! asks Jim), and anything whose date can't be read.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::new_issue;

/// Days a preview deployment lives.
pub const PREVIEW_DAYS: i64 = 14;
/// Days a non-current production deployment lives (rollback is a redeploy
/// from git).
pub const PRODUCTION_DAYS: i64 = 30;
/// Days a branch stays after its PR closed, so it can still be reopened
/// from the branch.
pub const BRANCH_DAYS: i64 = 7;
/// Branches that are never deleted, besides the default branch.
pub const NEVER_DELETE: &[&str] = &["evals-results"];

/// What the workflow read from Cloudflare and GitHub.
#[derive(Debug, Deserialize)]
pub struct Live {
    #[serde(default = "main_branch")]
    pub default_branch: String,
    #[serde(default)]
    pub pages: Vec<PagesProject>,
    #[serde(default)]
    pub branches: Vec<String>,
    #[serde(default)]
    pub pulls: Vec<Pull>,
}

fn main_branch() -> String {
    "main".to_string()
}

impl Default for Live {
    fn default() -> Self {
        Live {
            default_branch: main_branch(),
            pages: Vec::new(),
            branches: Vec::new(),
            pulls: Vec::new(),
        }
    }
}

/// One Pages project and its deployments.
#[derive(Debug, Default, Deserialize)]
pub struct PagesProject {
    pub name: String,
    /// The id of the current production deployment.
    #[serde(default)]
    pub canonical: String,
    #[serde(default)]
    pub deployments: Vec<Deployment>,
}

/// One Pages deployment, as the Cloudflare API lists it.
#[derive(Debug, Default, Deserialize)]
pub struct Deployment {
    pub id: String,
    /// `production` or `preview`.
    #[serde(default)]
    pub environment: String,
    #[serde(default)]
    pub branch: String,
    /// RFC 3339, such as `2026-11-20T12:34:56.789Z`.
    #[serde(default)]
    pub created_on: String,
}

/// One PR of this repo whose head branch is in this repo.
#[derive(Debug, Default, Deserialize)]
pub struct Pull {
    pub number: u64,
    pub head: String,
    /// `open` or `closed`.
    pub state: String,
    /// RFC 3339, for a closed PR.
    #[serde(default)]
    pub closed_at: Option<String>,
}

/// One deployment to delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeleteDeployment {
    pub project: String,
    pub id: String,
    pub environment: String,
    pub branch: String,
    pub age_days: i64,
}

/// One branch to delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeleteBranch {
    pub name: String,
    pub pr: u64,
    pub closed_days: i64,
}

/// What to delete, and what was kept on purpose.
#[derive(Debug, Default, Serialize)]
pub struct Plan {
    pub today: String,
    pub deployments: Vec<DeleteDeployment>,
    pub branches: Vec<DeleteBranch>,
    /// Deployments looked at and kept.
    pub kept_deployments: usize,
    /// Branches with no PR: listed, never deleted here.
    pub no_pr: Vec<String>,
    /// Anything skipped because a date couldn't be read.
    pub unreadable: Vec<String>,
}

/// Days since 1970-01-01 for the date at the start of `stamp`
/// (`YYYY-MM-DD`, optionally followed by a time).
pub fn day_number(stamp: &str) -> Option<i64> {
    let date = stamp.get(..10)?;
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || date.as_bytes().get(4) != Some(&b'-') {
        return None;
    }
    // Howard Hinnant's days-from-civil algorithm.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Apply the rules to what is live, as of `today` (YYYY-MM-DD).
pub fn plan(live: &Live, today: &str) -> Result<Plan> {
    let now = day_number(today).with_context(|| format!("'{today}' is not a YYYY-MM-DD date"))?;
    let open: BTreeSet<&str> = live
        .pulls
        .iter()
        .filter(|p| p.state == "open")
        .map(|p| p.head.as_str())
        .collect();
    let mut out = Plan {
        today: today.to_string(),
        ..Plan::default()
    };
    for project in &live.pages {
        plan_pages(project, &open, now, &mut out);
    }
    plan_branches(live, &open, now, &mut out);
    Ok(out)
}

fn plan_pages(project: &PagesProject, open: &BTreeSet<&str>, now: i64, out: &mut Plan) {
    // The newest preview of each branch with an open PR stays, so the PR's
    // preview link keeps working however long it is open.
    let mut newest: BTreeMap<&str, (&str, &str)> = BTreeMap::new();
    for d in &project.deployments {
        if d.environment != "production" && open.contains(d.branch.as_str()) {
            let entry = newest
                .entry(d.branch.as_str())
                .or_insert((d.created_on.as_str(), d.id.as_str()));
            if d.created_on.as_str() > entry.0 {
                *entry = (d.created_on.as_str(), d.id.as_str());
            }
        }
    }
    let kept: BTreeSet<&str> = newest.values().map(|(_, id)| *id).collect();
    for d in &project.deployments {
        let Some(created) = day_number(&d.created_on) else {
            out.unreadable
                .push(format!("pages {} deployment {}", project.name, d.id));
            continue;
        };
        let age = now - created;
        let delete = if d.environment == "production" {
            d.id != project.canonical && !project.canonical.is_empty() && age > PRODUCTION_DAYS
        } else {
            !kept.contains(d.id.as_str()) && age > PREVIEW_DAYS
        };
        if delete {
            out.deployments.push(DeleteDeployment {
                project: project.name.clone(),
                id: d.id.clone(),
                environment: d.environment.clone(),
                branch: d.branch.clone(),
                age_days: age,
            });
        } else {
            out.kept_deployments += 1;
        }
    }
}

fn plan_branches(live: &Live, open: &BTreeSet<&str>, now: i64, out: &mut Plan) {
    let mut newest: BTreeMap<&str, &Pull> = BTreeMap::new();
    for p in &live.pulls {
        let entry = newest.entry(p.head.as_str()).or_insert(p);
        if p.number > entry.number {
            *entry = p;
        }
    }
    for branch in &live.branches {
        if *branch == live.default_branch
            || NEVER_DELETE.contains(&branch.as_str())
            || open.contains(branch.as_str())
        {
            continue;
        }
        let Some(pr) = newest.get(branch.as_str()) else {
            out.no_pr.push(branch.clone());
            continue;
        };
        let Some(closed) = pr.closed_at.as_deref().and_then(day_number) else {
            out.unreadable
                .push(format!("branch {branch} (PR #{})", pr.number));
            continue;
        };
        let days = now - closed;
        if days > BRANCH_DAYS {
            out.branches.push(DeleteBranch {
                name: branch.clone(),
                pr: pr.number,
                closed_days: days,
            });
        }
    }
}

/// The run summary, in Markdown.
pub fn render(plan: &Plan) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "### Temporary resources ({})\n", plan.today);
    let _ = writeln!(
        s,
        "- Pages deployments to delete: {} (kept {})",
        plan.deployments.len(),
        plan.kept_deployments
    );
    let _ = writeln!(
        s,
        "- Branches of closed PRs to delete: {}",
        plan.branches.len()
    );
    let _ = writeln!(
        s,
        "- Branches with no PR (listed only; the sweep and digest handle them): {}",
        plan.no_pr.len()
    );
    if !plan.unreadable.is_empty() {
        let _ = writeln!(
            s,
            "- Skipped, date unreadable: {}",
            plan.unreadable.join(", ")
        );
    }
    if !plan.deployments.is_empty() {
        s.push_str("\n| Project | Deployment | Environment | Branch | Age (days) |\n| --- | --- | --- | --- | --- |\n");
        for d in &plan.deployments {
            let _ = writeln!(
                s,
                "| {} | `{}` | {} | `{}` | {} |",
                d.project, d.id, d.environment, d.branch, d.age_days
            );
        }
    }
    if !plan.branches.is_empty() {
        s.push_str("\n| Branch | PR | Closed (days ago) |\n| --- | --- | --- |\n");
        for b in &plan.branches {
            let _ = writeln!(s, "| `{}` | #{} | {} |", b.name, b.pr, b.closed_days);
        }
    }
    s
}

/// Read the live JSON, write the plan as JSON to `json` when given, and
/// return it.
pub fn run(live: &Path, today: Option<&str>, json: Option<&Path>) -> Result<Plan> {
    let text =
        std::fs::read_to_string(live).with_context(|| format!("reading {}", live.display()))?;
    let live: Live =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", live.display()))?;
    let today = today.map_or_else(new_issue::today, str::to_string);
    let plan = plan(&live, &today)?;
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_string_pretty(&plan)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dep(id: &str, env: &str, branch: &str, created: &str) -> Deployment {
        Deployment {
            id: id.into(),
            environment: env.into(),
            branch: branch.into(),
            created_on: created.into(),
        }
    }

    fn pull(number: u64, head: &str, state: &str, closed: Option<&str>) -> Pull {
        Pull {
            number,
            head: head.into(),
            state: state.into(),
            closed_at: closed.map(Into::into),
        }
    }

    fn ids(plan: &Plan) -> Vec<&str> {
        plan.deployments.iter().map(|d| d.id.as_str()).collect()
    }

    fn branches(plan: &Plan) -> Vec<&str> {
        plan.branches.iter().map(|b| b.name.as_str()).collect()
    }

    #[test]
    fn day_number_counts_from_the_epoch() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("1970-01-02T00:00:00Z"), Some(1));
        assert_eq!(
            day_number("2026-12-01")
                .zip(day_number("2026-11-28"))
                .map(|(a, b)| a - b),
            Some(3)
        );
        // Across a leap day.
        assert_eq!(
            day_number("2028-03-01")
                .zip(day_number("2028-02-28"))
                .map(|(a, b)| a - b),
            Some(2)
        );
        assert_eq!(day_number("2026-13-01"), None);
        assert_eq!(day_number("yesterday"), None);
        assert_eq!(day_number(""), None);
    }

    #[test]
    fn previews_older_than_14_days_go() {
        let live = Live {
            pages: vec![PagesProject {
                name: "jimhillr-cookbook".into(),
                canonical: "prod".into(),
                deployments: vec![
                    dep("old", "preview", "gone-branch", "2026-11-13T10:00:00Z"),
                    dep("edge", "preview", "gone-branch", "2026-11-14T10:00:00Z"),
                    dep("new", "preview", "gone-branch", "2026-11-27T10:00:00Z"),
                ],
            }],
            ..Live::default()
        };
        let plan = plan(&live, "2026-11-28").unwrap();
        // 15 days goes; exactly 14 stays.
        assert_eq!(ids(&plan), ["old"]);
        assert_eq!(plan.kept_deployments, 2);
        assert_eq!(plan.deployments[0].age_days, 15);
    }

    #[test]
    fn the_newest_preview_of_an_open_pr_stays_however_old() {
        let live = Live {
            pages: vec![PagesProject {
                name: "jimhillr-pane".into(),
                canonical: "prod".into(),
                deployments: vec![
                    dep("first", "preview", "long-pr", "2026-10-01T10:00:00Z"),
                    dep("latest", "preview", "long-pr", "2026-10-20T10:00:00Z"),
                    dep("closed", "preview", "done-pr", "2026-10-20T10:00:00Z"),
                ],
            }],
            pulls: vec![
                pull(10, "long-pr", "open", None),
                pull(9, "done-pr", "closed", Some("2026-10-21T00:00:00Z")),
            ],
            ..Live::default()
        };
        let plan = plan(&live, "2026-11-28").unwrap();
        assert_eq!(ids(&plan), ["first", "closed"]);
    }

    #[test]
    fn production_keeps_the_current_one_and_30_days() {
        let live = Live {
            pages: vec![PagesProject {
                name: "jimhillr-bear-down".into(),
                canonical: "current".into(),
                deployments: vec![
                    dep("current", "production", "main", "2026-01-01T00:00:00Z"),
                    dep("rollback", "production", "main", "2026-10-29T00:00:00Z"),
                    dep("ancient", "production", "main", "2026-10-28T00:00:00Z"),
                ],
            }],
            ..Live::default()
        };
        let plan = plan(&live, "2026-11-28").unwrap();
        assert_eq!(ids(&plan), ["ancient"]);
    }

    #[test]
    fn production_is_left_alone_without_a_current_deployment() {
        let live = Live {
            pages: vec![PagesProject {
                name: "x".into(),
                canonical: String::new(),
                deployments: vec![dep("a", "production", "main", "2025-01-01")],
            }],
            ..Live::default()
        };
        assert!(plan(&live, "2026-11-28").unwrap().deployments.is_empty());
    }

    #[test]
    fn unreadable_dates_are_kept_and_reported() {
        let live = Live {
            pages: vec![PagesProject {
                name: "x".into(),
                canonical: "c".into(),
                deployments: vec![dep("a", "preview", "b", "")],
            }],
            branches: vec!["b".into()],
            pulls: vec![pull(3, "b", "closed", None)],
            ..Live::default()
        };
        let plan = plan(&live, "2026-11-28").unwrap();
        assert!(plan.deployments.is_empty());
        assert!(plan.branches.is_empty());
        assert_eq!(plan.unreadable.len(), 2);
    }

    #[test]
    fn branches_go_7_days_after_their_pr_closed() {
        let live = Live {
            branches: vec![
                "main".into(),
                "evals-results".into(),
                "merged-long-ago".into(),
                "merged-this-week".into(),
                "reopened".into(),
                "no-pr".into(),
            ],
            pulls: vec![
                pull(1, "merged-long-ago", "closed", Some("2026-11-20T23:59:59Z")),
                pull(
                    2,
                    "merged-this-week",
                    "closed",
                    Some("2026-11-21T00:00:00Z"),
                ),
                pull(3, "reopened", "closed", Some("2026-10-01T00:00:00Z")),
                pull(4, "reopened", "open", None),
                pull(5, "main", "closed", Some("2026-01-01T00:00:00Z")),
                pull(6, "evals-results", "closed", Some("2026-01-01T00:00:00Z")),
            ],
            ..Live::default()
        };
        let plan = plan(&live, "2026-11-28").unwrap();
        assert_eq!(branches(&plan), ["merged-long-ago"]);
        assert_eq!(plan.branches[0].pr, 1);
        assert_eq!(plan.no_pr, ["no-pr"]);
    }

    #[test]
    fn the_newest_pr_of_a_branch_decides() {
        // An old PR closed long ago, then a newer one on the same branch
        // closed yesterday: the branch stays a week from yesterday.
        let live = Live {
            branches: vec!["reused".into()],
            pulls: vec![
                pull(20, "reused", "closed", Some("2026-11-27T00:00:00Z")),
                pull(5, "reused", "closed", Some("2026-09-01T00:00:00Z")),
            ],
            ..Live::default()
        };
        assert!(plan(&live, "2026-11-28").unwrap().branches.is_empty());
    }

    #[test]
    fn a_custom_default_branch_is_never_deleted() {
        let live: Live = serde_json::from_str(
            r#"{"default_branch":"trunk","branches":["trunk"],
                "pulls":[{"number":1,"head":"trunk","state":"closed","closed_at":"2020-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        assert!(plan(&live, "2026-11-28").unwrap().branches.is_empty());
    }

    #[test]
    fn a_bad_today_is_an_error() {
        assert!(plan(&Live::default(), "soon").is_err());
    }

    #[test]
    fn render_lists_what_goes() {
        let live = Live {
            pages: vec![PagesProject {
                name: "jimhillr-cookbook".into(),
                canonical: "c".into(),
                deployments: vec![dep("abc", "preview", "feat", "2026-10-01")],
            }],
            branches: vec!["feat".into()],
            pulls: vec![pull(42, "feat", "closed", Some("2026-10-02"))],
            ..Live::default()
        };
        let text = render(&plan(&live, "2026-11-28").unwrap());
        assert!(text.contains("Pages deployments to delete: 1 (kept 0)"));
        assert!(text.contains("| jimhillr-cookbook | `abc` | preview | `feat` | 58 |"));
        assert!(text.contains("| `feat` | #42 | 57 |"));
    }

    #[test]
    fn run_reads_and_writes_files() {
        let dir = tempfile::tempdir().unwrap();
        let live = dir.path().join("live.json");
        std::fs::write(&live, r#"{"branches":["x"],"pulls":[]}"#).unwrap();
        let out = dir.path().join("plan.json");
        let plan = run(&live, Some("2026-11-28"), Some(&out)).unwrap();
        assert_eq!(plan.no_pr, ["x"]);
        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out).unwrap()).unwrap();
        assert_eq!(written["no_pr"][0], "x");
        assert_eq!(written["today"], "2026-11-28");
    }
}
