//! `chores retire-approved`: which `[[retired]]` entries in
//! `standards/resources.toml` the weekly `all-retire-resources.yml` run
//! deletes ("Part 4: deleting safely" and "Phase 3: approved deletion" in
//! standards/docs/plans/2026-11-resource-cleanup.md, "Safe deletion order"
//! in standards/docs/resources.md).
//!
//! Jim approving the PR that added an entry is the approval to delete it.
//! This applies the guards and writes the plan; the workflow deletes. It
//! reads only files, so the guards are tested without an account.
//!
//! | Outcome | When |
//! | --- | --- |
//! | `delete` | A Pages entry past `delete_after` that is live and passes every guard |
//! | `held` | A guard stops it: the name is claimed on `main`, its `replaced_by` isn't, a production deploy in the last 7 days, an open human issue still names it, its deployments can't be read, or Pages couldn't be listed |
//! | `pending` | `delete_after` is today or later |
//! | `gone` | Past `delete_after` and no longer live: the sweep sets `deleted` |
//! | `manual` | A kind this workflow can't delete yet (only Pages today: the deploy token has Pages Write and nothing else that deletes) |
//!
//! Entries with `deleted` set are done and skipped. Nothing is ever deleted
//! because it is unclaimed: only entries Jim approved are looked at.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::new_issue;
use crate::retire::{self, PagesProject};
use crate::sweep;

/// Days after a production deployment during which a retired Pages project
/// is held: something still deploys to it.
pub const RECENT_DEPLOY_DAYS: i64 = 7;

/// The kinds this workflow deletes.
pub const DELETABLE: &[&str] = &["pages"];

/// One `[[retired]]` entry, as `standards/resources.toml` has it.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Retired {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    /// What replaces it; when set, something on `main` must claim it.
    #[serde(default)]
    pub replaced_by: String,
    #[serde(default = "yes")]
    pub match_name: bool,
    #[serde(default)]
    pub delete_after: String,
    #[serde(default)]
    pub deleted: String,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
struct Entries {
    #[serde(default)]
    retired: Vec<Retired>,
}

/// What the workflow listed: the same file as `retire-temporary` reads,
/// plus whether the Pages listing worked at all.
#[derive(Debug, Default, Deserialize)]
pub struct Live {
    /// False when the Cloudflare secrets were missing or the listing
    /// failed, so an empty `pages` means "unknown", not "none".
    #[serde(default)]
    pub pages_listed: bool,
    #[serde(default)]
    pub pages: Vec<PagesProject>,
}

/// An open human task file and its text.
#[derive(Debug, Clone)]
pub struct HumanIssue {
    pub path: String,
    pub text: String,
}

/// One entry and what happens to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub kind: String,
    pub name: String,
    pub url: String,
    pub reason: String,
}

/// The plan for one run.
#[derive(Debug, Default, Serialize)]
pub struct Plan {
    pub today: String,
    pub delete: Vec<Item>,
    pub held: Vec<Item>,
    pub pending: Vec<Item>,
    pub gone: Vec<Item>,
    pub manual: Vec<Item>,
    /// Entries with `deleted` set.
    pub done: usize,
}

/// Everything the guards look at.
pub struct Inputs<'a> {
    pub entries: &'a [Retired],
    /// `(kind, name)` claimed on `main`, from `resources.sh claims`.
    pub claimed: &'a BTreeSet<(String, String)>,
    pub live: &'a Live,
    pub human: &'a [HumanIssue],
}

/// Apply the guards as of `today` (YYYY-MM-DD).
pub fn plan(inputs: &Inputs<'_>, today: &str) -> Result<Plan> {
    let now =
        retire::day_number(today).with_context(|| format!("'{today}' is not a YYYY-MM-DD date"))?;
    let mut out = Plan {
        today: today.to_string(),
        ..Plan::default()
    };
    for e in inputs.entries {
        if !e.deleted.is_empty() {
            out.done += 1;
            continue;
        }
        let (bucket, reason) = judge(e, inputs, now);
        let item = Item {
            kind: e.kind.clone(),
            name: e.name.clone(),
            url: e.url.clone(),
            reason,
        };
        match bucket {
            Bucket::Delete => out.delete.push(item),
            Bucket::Held => out.held.push(item),
            Bucket::Pending => out.pending.push(item),
            Bucket::Gone => out.gone.push(item),
            Bucket::Manual => out.manual.push(item),
        }
    }
    Ok(out)
}

enum Bucket {
    Delete,
    Held,
    Pending,
    Gone,
    Manual,
}

fn judge(e: &Retired, inputs: &Inputs<'_>, now: i64) -> (Bucket, String) {
    let Some(after) = retire::day_number(&e.delete_after) else {
        return (
            Bucket::Held,
            format!("delete_after '{}' is not a date", e.delete_after),
        );
    };
    if now <= after {
        return (Bucket::Pending, format!("deleted after {}", e.delete_after));
    }
    if !DELETABLE.contains(&e.kind.as_str()) {
        return (
            Bucket::Manual,
            format!(
                "this workflow deletes only {}; follow the runbook",
                DELETABLE.join(", ")
            ),
        );
    }
    if inputs.claimed.contains(&(e.kind.clone(), e.name.clone())) {
        return (
            Bucket::Held,
            "still claimed by a file on main: remove the claim or the entry".into(),
        );
    }
    if !e.replaced_by.is_empty() && !inputs.claimed.iter().any(|(_, n)| *n == e.replaced_by) {
        return (
            Bucket::Held,
            format!(
                "its replacement `{}` isn't claimed on main yet",
                e.replaced_by
            ),
        );
    }
    if let Some(issue) = inputs.human.iter().find(|h| mentions(&h.text, e)) {
        return (
            Bucket::Held,
            format!(
                "the open human task {} still names it: an inbound reference may remain",
                issue.path
            ),
        );
    }
    judge_pages(e, inputs.live, now)
}

fn judge_pages(e: &Retired, live: &Live, now: i64) -> (Bucket, String) {
    if !live.pages_listed {
        return (
            Bucket::Held,
            "Pages projects couldn't be listed, so nothing is deleted".into(),
        );
    }
    let Some(project) = live.pages.iter().find(|p| p.name == e.name) else {
        return (
            Bucket::Gone,
            "no longer live: the sweep sets `deleted`".into(),
        );
    };
    let mut newest: Option<i64> = None;
    for d in project
        .deployments
        .iter()
        .filter(|d| d.environment == "production")
    {
        let Some(day) = retire::day_number(&d.created_on) else {
            return (
                Bucket::Held,
                format!("production deployment {} has no readable date", d.id),
            );
        };
        newest = Some(newest.map_or(day, |n| n.max(day)));
    }
    match newest {
        Some(day) if now - day <= RECENT_DEPLOY_DAYS => (
            Bucket::Held,
            format!(
                "a production deployment {} days ago: something still deploys to it",
                now - day
            ),
        ),
        Some(day) => (
            Bucket::Delete,
            format!("last production deployment {} days ago", now - day),
        ),
        None => (Bucket::Delete, "no production deployments".into()),
    }
}

/// Whether a text names the entry: its url (with or without the scheme),
/// or its name as a whole word unless `match_name` is false.
pub fn mentions(text: &str, e: &Retired) -> bool {
    let url = e.url.trim().trim_end_matches('/');
    if !url.is_empty() {
        let bare = url.split_once("://").map_or(url, |(_, r)| r);
        if text.contains(bare) {
            return true;
        }
    }
    e.match_name && !e.name.is_empty() && has_word(text, &e.name)
}

fn has_word(text: &str, word: &str) -> bool {
    let part = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(part) && !after.is_some_and(part)
    })
}

/// The run summary, in Markdown.
pub fn render(plan: &Plan) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "### Retired resources ({})\n", plan.today);
    let _ = writeln!(
        s,
        "- To delete: {}\n- Held by a guard: {}\n- Not due yet: {}\n- Already gone: {}\n- Delete by hand: {}\n- Done earlier: {}",
        plan.delete.len(),
        plan.held.len(),
        plan.pending.len(),
        plan.gone.len(),
        plan.manual.len(),
        plan.done
    );
    let groups = [
        ("Delete", &plan.delete),
        ("Held", &plan.held),
        ("Not due", &plan.pending),
        ("Gone", &plan.gone),
        ("By hand", &plan.manual),
    ];
    if groups.iter().all(|(_, items)| items.is_empty()) {
        return s;
    }
    s.push_str("\n| Outcome | Kind | Name | Why |\n| --- | --- | --- | --- |\n");
    for (label, items) in groups {
        for i in items {
            let _ = writeln!(s, "| {label} | {} | `{}` | {} |", i.kind, i.name, i.reason);
        }
    }
    s
}

fn load_entries(root: &Path) -> Result<Vec<Retired>> {
    let path = root.join(sweep::RESOURCES);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let entries: Entries = toml::from_str(&std::fs::read_to_string(&path)?)
        .with_context(|| format!("reading {}", sweep::RESOURCES))?;
    Ok(entries.retired)
}

/// Every tracked human task file (`issues/human/` in any folder), except
/// the folder READMEs.
fn load_human(root: &Path) -> Result<Vec<HumanIssue>> {
    let out = Command::new("git")
        .args(["ls-files", "--", "*issues/human/*.md"])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files failed in {}", root.display());
    }
    let mut issues = Vec::new();
    for path in String::from_utf8_lossy(&out.stdout).lines() {
        if path.ends_with("/README.md") || !root.join(path).is_file() {
            continue;
        }
        let text =
            std::fs::read_to_string(root.join(path)).with_context(|| format!("reading {path}"))?;
        issues.push(HumanIssue {
            path: path.to_string(),
            text,
        });
    }
    Ok(issues)
}

/// Read the files, write the plan as JSON to `json` when given, and return
/// it.
pub fn run(
    root: &Path,
    live: &Path,
    claims: &Path,
    today: Option<&str>,
    json: Option<&Path>,
) -> Result<Plan> {
    let text =
        std::fs::read_to_string(live).with_context(|| format!("reading {}", live.display()))?;
    let live: Live =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", live.display()))?;
    let tsv =
        std::fs::read_to_string(claims).with_context(|| format!("reading {}", claims.display()))?;
    let claimed = sweep::load_claims(root, &tsv)?.claimed;
    let entries = load_entries(root)?;
    let human = load_human(root)?;
    let today = today.map_or_else(new_issue::today, str::to_string);
    let plan = plan(
        &Inputs {
            entries: &entries,
            claimed: &claimed,
            live: &live,
            human: &human,
        },
        &today,
    )?;
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_string_pretty(&plan)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retire::Deployment;

    fn entry(kind: &str, name: &str, url: &str, after: &str) -> Retired {
        Retired {
            kind: kind.into(),
            name: name.into(),
            url: url.into(),
            replaced_by: String::new(),
            match_name: true,
            delete_after: after.into(),
            deleted: String::new(),
        }
    }

    fn project(name: &str, prod: &[&str]) -> PagesProject {
        PagesProject {
            name: name.into(),
            canonical: String::new(),
            deployments: prod
                .iter()
                .enumerate()
                .map(|(i, c)| Deployment {
                    id: format!("d{i}"),
                    environment: "production".into(),
                    branch: "main".into(),
                    created_on: (*c).into(),
                })
                .collect(),
        }
    }

    fn listed(pages: Vec<PagesProject>) -> Live {
        Live {
            pages_listed: true,
            pages,
        }
    }

    fn run_plan(entries: &[Retired], live: &Live, human: &[HumanIssue]) -> Plan {
        let claimed = BTreeSet::new();
        plan(
            &Inputs {
                entries,
                claimed: &claimed,
                live,
                human,
            },
            "2026-12-07",
        )
        .unwrap()
    }

    fn names(items: &[Item]) -> Vec<&str> {
        items.iter().map(|i| i.name.as_str()).collect()
    }

    const OLD: &str = "https://old.pages.dev";

    #[test]
    fn a_due_unclaimed_quiet_pages_project_is_deleted() {
        let live = listed(vec![project("old", &["2026-11-20T10:00:00Z"])]);
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-05")], &live, &[]);
        assert_eq!(names(&p.delete), ["old"]);
        assert!(p.delete[0].reason.contains("17 days ago"));
    }

    #[test]
    fn nothing_is_deleted_on_or_before_delete_after() {
        let live = listed(vec![project("old", &[])]);
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-07")], &live, &[]);
        assert!(p.delete.is_empty());
        assert_eq!(names(&p.pending), ["old"]);
    }

    #[test]
    fn a_claim_on_main_holds_it() {
        let live = listed(vec![project("old", &[])]);
        let claimed = BTreeSet::from([("pages".to_string(), "old".to_string())]);
        let entries = [entry("pages", "old", OLD, "2026-12-01")];
        let p = plan(
            &Inputs {
                entries: &entries,
                claimed: &claimed,
                live: &live,
                human: &[],
            },
            "2026-12-07",
        )
        .unwrap();
        assert_eq!(names(&p.held), ["old"]);
        assert!(p.held[0].reason.contains("claimed"));
    }

    #[test]
    fn a_worker_of_the_same_name_is_not_a_claim_on_the_pages_project() {
        let live = listed(vec![project("shared-name", &[])]);
        let claimed = BTreeSet::from([("worker".to_string(), "shared-name".to_string())]);
        let mut e = entry(
            "pages",
            "shared-name",
            "https://shared-name.pages.dev",
            "2026-12-05",
        );
        e.replaced_by = "shared-name".into();
        let entries = [e];
        let p = plan(
            &Inputs {
                entries: &entries,
                claimed: &claimed,
                live: &live,
                human: &[],
            },
            "2026-12-07",
        )
        .unwrap();
        assert_eq!(names(&p.delete), ["shared-name"]);
    }

    #[test]
    fn an_unclaimed_replacement_holds_it() {
        let live = listed(vec![project("old", &[])]);
        let mut e = entry("pages", "old", OLD, "2026-12-01");
        e.replaced_by = "jimhillr-new".into();
        let p = run_plan(&[e], &live, &[]);
        assert_eq!(names(&p.held), ["old"]);
        assert!(p.held[0].reason.contains("jimhillr-new"));
    }

    #[test]
    fn a_production_deploy_in_the_last_7_days_holds_it() {
        let live = listed(vec![project(
            "old",
            &["2026-10-01T00:00:00Z", "2026-11-30T23:00:00Z"],
        )]);
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-01")], &live, &[]);
        assert_eq!(names(&p.held), ["old"]);
        assert!(p.held[0].reason.contains("7 days ago"));
        // Eight days is quiet enough.
        let live = listed(vec![project("old", &["2026-11-29T23:00:00Z"])]);
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-01")], &live, &[]);
        assert_eq!(names(&p.delete), ["old"]);
    }

    #[test]
    fn previews_do_not_count_as_deploys() {
        let mut proj = project("old", &[]);
        proj.deployments.push(Deployment {
            id: "p".into(),
            environment: "preview".into(),
            branch: "x".into(),
            created_on: "2026-12-06T00:00:00Z".into(),
        });
        let p = run_plan(
            &[entry("pages", "old", OLD, "2026-12-01")],
            &listed(vec![proj]),
            &[],
        );
        assert_eq!(names(&p.delete), ["old"]);
    }

    #[test]
    fn an_unreadable_deploy_date_holds_it() {
        let live = listed(vec![project("old", &["soon"])]);
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-01")], &live, &[]);
        assert_eq!(names(&p.held), ["old"]);
    }

    #[test]
    fn an_open_human_task_naming_it_holds_it() {
        let live = listed(vec![project("old", &[])]);
        let human = [HumanIssue {
            path: "issues/human/remove-auth0-urls.md".into(),
            text: "Remove https://old.pages.dev/callback from Auth0.".into(),
        }];
        let p = run_plan(&[entry("pages", "old", OLD, "2026-12-01")], &live, &human);
        assert_eq!(names(&p.held), ["old"]);
        assert!(
            p.held[0]
                .reason
                .contains("issues/human/remove-auth0-urls.md")
        );
    }

    #[test]
    fn mentions_matches_the_url_or_the_whole_name() {
        let mut e = entry(
            "pages",
            "cahokia-web",
            "https://cahokia-web.pages.dev/",
            "x",
        );
        assert!(mentions("delete cahokia-web.pages.dev now", &e));
        assert!(mentions("the `cahokia-web` project", &e));
        assert!(!mentions("the jimhillr-cahokia-web project", &e));
        assert!(!mentions("cahokia-webapp", &e));
        e.match_name = false;
        assert!(!mentions("the `cahokia-web` project", &e));
        assert!(mentions("https://cahokia-web.pages.dev", &e));
    }

    #[test]
    fn an_unlisted_account_holds_everything() {
        let p = run_plan(
            &[entry("pages", "old", OLD, "2026-12-01")],
            &Live::default(),
            &[],
        );
        assert_eq!(names(&p.held), ["old"]);
        assert!(p.gone.is_empty());
    }

    #[test]
    fn a_project_no_longer_live_is_gone() {
        let p = run_plan(
            &[entry("pages", "old", OLD, "2026-12-01")],
            &listed(vec![]),
            &[],
        );
        assert_eq!(names(&p.gone), ["old"]);
    }

    #[test]
    fn other_kinds_are_left_for_a_person() {
        let p = run_plan(
            &[entry("d1", "old-db", "", "2026-12-01")],
            &listed(vec![]),
            &[],
        );
        assert_eq!(names(&p.manual), ["old-db"]);
    }

    #[test]
    fn done_entries_and_bad_dates() {
        let mut done = entry("pages", "a", "", "2026-12-01");
        done.deleted = "2026-12-02".into();
        let bad = entry("pages", "b", "", "next week");
        let p = run_plan(&[done, bad], &listed(vec![project("a", &[])]), &[]);
        assert_eq!(p.done, 1);
        assert_eq!(names(&p.held), ["b"]);
        assert!(p.delete.is_empty());
    }

    #[test]
    fn a_bad_today_is_an_error() {
        let claimed = BTreeSet::new();
        let live = Live::default();
        let inputs = Inputs {
            entries: &[],
            claimed: &claimed,
            live: &live,
            human: &[],
        };
        assert!(plan(&inputs, "soon").is_err());
    }

    #[test]
    fn render_lists_every_outcome() {
        let live = listed(vec![project("old", &[])]);
        let entries = [
            entry("pages", "old", OLD, "2026-12-01"),
            entry("pages", "later", "", "2026-12-30"),
        ];
        let text = render(&run_plan(&entries, &live, &[]));
        assert!(text.contains("- To delete: 1"));
        assert!(text.contains("| Delete | pages | `old` | no production deployments |"));
        assert!(text.contains("| Not due | pages | `later` | deleted after 2026-12-30 |"));
        let empty = render(&run_plan(&[], &live, &[]));
        assert!(!empty.contains("| Outcome |"));
    }

    #[test]
    fn run_reads_the_repo_and_writes_the_plan() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        };
        git(&["init", "-q"]);
        std::fs::create_dir_all(root.join("standards")).unwrap();
        std::fs::create_dir_all(root.join("issues/human")).unwrap();
        std::fs::write(
            root.join(sweep::RESOURCES),
            "[[retired]]\nkind = \"pages\"\nname = \"old\"\nurl = \"https://old.pages.dev\"\ndelete_after = \"2026-12-01\"\n\n[[retired]]\nkind = \"pages\"\nname = \"other\"\nurl = \"https://other.pages.dev\"\ndelete_after = \"2026-12-01\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("issues/human/remove-urls.md"),
            "Remove other.pages.dev from Auth0.",
        )
        .unwrap();
        std::fs::write(root.join("issues/human/README.md"), "old other").unwrap();
        git(&["add", "-A"]);
        let live = root.join("live.json");
        std::fs::write(
            &live,
            r#"{"pages_listed":true,"pages":[{"name":"old"},{"name":"other"}]}"#,
        )
        .unwrap();
        let claims = root.join("claims.tsv");
        std::fs::write(&claims, "worker\told\n").unwrap();
        let out = root.join("plan.json");
        let p = run(root, &live, &claims, Some("2026-12-07"), Some(&out)).unwrap();
        assert_eq!(names(&p.delete), ["old"]);
        assert_eq!(names(&p.held), ["other"]);
        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out).unwrap()).unwrap();
        assert_eq!(written["delete"][0]["name"], "old");
    }
}
