//! The per-PR cost summary: what a change does to services, usage, money
//! and budgets, and which labels (`cost`, `needs-human`) the PR gets.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Serialize;

use crate::rules::money;
use crate::{LimitsFile, Month, Plan, Project, Radar, Scope, monthly_cents};

/// The cost data at one commit.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Every project whose `docs/cost.toml` parsed.
    pub projects: Vec<Project>,
    /// The blips the detectors found, for the projects the change touches.
    pub detected: BTreeMap<String, BTreeSet<String>>,
    pub radar: Radar,
    pub limits: LimitsFile,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// `cost` when anything cost-related changed, plus `needs-human` when
    /// the change is Jim's decision.
    pub labels: Vec<String>,
    /// Why the PR needs Jim, one line each.
    pub needs_human: Vec<String>,
    /// Cost-sensitive files the change touches.
    pub changed_files: Vec<String>,
    pub projects: Vec<ProjectChange>,
    /// Account-wide limits whose summed usage moved.
    pub account: Vec<UsageChange>,
    /// Radar budgets and rings, and limits.toml budgets and free limits.
    pub budgets: Vec<BudgetChange>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ProjectChange {
    pub name: String,
    /// Blips added to or removed from its `[[service]]` list.
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub usage: Vec<UsageChange>,
    /// Its own money this month, in cents.
    pub cents_before: u64,
    pub cents_after: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsageChange {
    pub key: String,
    pub unit: String,
    pub before: u64,
    pub after: u64,
    /// Our budget, or the free limit, after the change.
    pub ceiling: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BudgetChange {
    pub what: String,
    pub before: String,
    pub after: String,
}

fn usage_by_key(project: Option<&Project>) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for service in project.iter().flat_map(|p| &p.file.service) {
        for (key, n) in &service.usage {
            *out.entry(key.clone()).or_default() += n;
        }
    }
    out
}

fn listed(project: Option<&Project>) -> BTreeSet<String> {
    project
        .iter()
        .flat_map(|p| &p.file.service)
        .map(|s| s.blip.clone())
        .collect()
}

fn cents(project: Option<&Project>, month: Month) -> u64 {
    project
        .iter()
        .flat_map(|p| &p.file.cost)
        .map(|c| monthly_cents(c, month))
        .sum()
}

fn opt(n: Option<u64>) -> String {
    n.map_or_else(|| "none".to_string(), |n| n.to_string())
}

/// Compares `base` with `head`. `changed_files` are the cost-sensitive
/// files the change touches; `month` prices recurring money.
pub fn summarize(
    base: &Snapshot,
    head: &Snapshot,
    changed_files: Vec<String>,
    month: Month,
) -> Summary {
    let mut summary = Summary {
        changed_files,
        ..Summary::default()
    };
    let find = |s: &'_ Snapshot, name: &str| -> Option<Project> {
        s.projects.iter().find(|p| p.name == name).cloned()
    };
    let names: BTreeSet<&str> = base
        .projects
        .iter()
        .chain(&head.projects)
        .map(|p| p.name.as_str())
        .chain(base.detected.keys().map(String::as_str))
        .chain(head.detected.keys().map(String::as_str))
        .collect();

    for name in names {
        let (before, after) = (find(base, name), find(head, name));
        let (was, now) = (listed(before.as_ref()), listed(after.as_ref()));
        let mut change = ProjectChange {
            name: name.to_string(),
            added: now.difference(&was).cloned().collect(),
            removed: was.difference(&now).cloned().collect(),
            cents_before: cents(before.as_ref(), month),
            cents_after: cents(after.as_ref(), month),
            ..ProjectChange::default()
        };
        for blip in &change.added {
            summary
                .needs_human
                .push(format!("{name} adds {blip} to its services"));
        }

        let empty = BTreeSet::new();
        let seen_before = base.detected.get(name).unwrap_or(&empty);
        for blip in head.detected.get(name).unwrap_or(&empty) {
            if !seen_before.contains(blip) && !was.contains(blip) && !now.contains(blip) {
                summary.needs_human.push(format!(
                    "{name}'s code starts using {blip}, which its docs/cost.toml doesn't list"
                ));
            }
        }

        let (u0, u1) = (usage_by_key(before.as_ref()), usage_by_key(after.as_ref()));
        let keys: BTreeSet<&String> = u0.keys().chain(u1.keys()).collect();
        for key in keys {
            let (b, a) = (
                u0.get(key).copied().unwrap_or(0),
                u1.get(key).copied().unwrap_or(0),
            );
            if b == a {
                continue;
            }
            let limit = head.limits.limit.iter().find(|l| &l.key == key);
            change.usage.push(UsageChange {
                key: key.clone(),
                unit: limit.map(|l| l.unit.clone()).unwrap_or_default(),
                before: b,
                after: a,
                ceiling: limit
                    .filter(|l| l.scope == Scope::Project)
                    .and_then(|l| l.ceiling()),
            });
        }

        if let Some(after) = &after {
            for service in after.file.service.iter().filter(|s| s.plan == Plan::Paid) {
                let was_paid = before.as_ref().is_some_and(|b| {
                    b.file
                        .service
                        .iter()
                        .any(|s| s.blip == service.blip && s.plan == Plan::Paid)
                });
                if !was_paid {
                    summary
                        .needs_human
                        .push(format!("{name} moves {} to a paid plan", service.blip));
                }
            }
            for cost in after.file.cost.iter().filter(|c| c.cents > 0) {
                let old = before
                    .as_ref()
                    .and_then(|b| b.file.cost.iter().find(|c| c.name == cost.name));
                if old.is_none_or(|o| o.cents < cost.cents || o.every != cost.every) {
                    summary.needs_human.push(format!(
                        "{name} adds or raises cost `{}`: {} every {:?}",
                        cost.name,
                        money(cost.cents),
                        cost.every
                    ));
                }
            }
        }

        if !change.added.is_empty()
            || !change.removed.is_empty()
            || !change.usage.is_empty()
            || change.cents_before != change.cents_after
        {
            summary.projects.push(change);
        }
    }

    for limit in head
        .limits
        .limit
        .iter()
        .filter(|l| l.scope == Scope::Account)
    {
        let total = |s: &Snapshot| -> u64 {
            s.projects
                .iter()
                .flat_map(|p| &p.file.service)
                .filter_map(|svc| svc.usage.get(&limit.key))
                .sum()
        };
        let (b, a) = (total(base), total(head));
        if b != a {
            summary.account.push(UsageChange {
                key: limit.key.clone(),
                unit: limit.unit.clone(),
                before: b,
                after: a,
                ceiling: limit.ceiling(),
            });
        }
    }

    let vendors: BTreeSet<&str> = base
        .radar
        .blips
        .iter()
        .chain(&head.radar.blips)
        .filter(|b| b.is_vendor())
        .map(|b| b.name.as_str())
        .collect();
    for name in vendors {
        let (b, a) = (base.radar.vendor(name), head.radar.vendor(name));
        let (cb, ca) = (
            b.map_or(0, |b| b.budget_cents()),
            a.map_or(0, |a| a.budget_cents()),
        );
        if cb != ca {
            summary.budgets.push(BudgetChange {
                what: format!("{name} budget (radar)"),
                before: format!("{} a month", money(cb)),
                after: format!("{} a month", money(ca)),
            });
            if ca > cb {
                summary
                    .needs_human
                    .push(format!("raises the {name} budget to {} a month", money(ca)));
            }
        }
        let (rb, ra) = (
            b.map_or("none", |b| b.ring.as_str()),
            a.map_or("none", |a| a.ring.as_str()),
        );
        if rb != ra {
            summary.budgets.push(BudgetChange {
                what: format!("{name} ring (radar)"),
                before: rb.to_string(),
                after: ra.to_string(),
            });
            if a.is_some_and(|a| a.is_approved()) && !b.is_some_and(|b| b.is_approved()) {
                summary
                    .needs_human
                    .push(format!("approves vendor {name} ({ra} on the radar)"));
            }
        }
    }

    for limit in &head.limits.limit {
        let old = base.limits.limit.iter().find(|l| l.key == limit.key);
        let Some(old) = old else { continue };
        if old.budget != limit.budget {
            summary.budgets.push(BudgetChange {
                what: format!("{} budget (limits.toml)", limit.key),
                before: opt(old.budget),
                after: opt(limit.budget),
            });
            if old.ceiling() < limit.ceiling() || (old.budget.is_some() && limit.budget.is_none()) {
                summary.needs_human.push(format!(
                    "raises the {} budget from {} to {}",
                    limit.key,
                    opt(old.budget),
                    opt(limit.budget)
                ));
            }
        }
        if old.free != limit.free {
            summary.budgets.push(BudgetChange {
                what: format!("{} free limit (limits.toml)", limit.key),
                before: opt(old.free),
                after: opt(limit.free),
            });
        }
    }

    if !summary.needs_human.is_empty() {
        summary.labels = vec!["cost".into(), "needs-human".into()];
    } else if !summary.changed_files.is_empty()
        || !summary.projects.is_empty()
        || !summary.account.is_empty()
        || !summary.budgets.is_empty()
    {
        summary.labels = vec!["cost".into()];
    }
    summary
}

fn headroom(after: u64, ceiling: Option<u64>) -> String {
    match ceiling {
        Some(c) if after <= c => (c - after).to_string(),
        Some(c) => format!("**over by {}**", after - c),
        None => "no limit".to_string(),
    }
}

/// The summary as Markdown, for the job summary.
pub fn render_summary(summary: &Summary) -> String {
    let mut out = String::from("## Cost summary\n\n");
    if summary.labels.is_empty() {
        out.push_str("No cost changes: no cost files, services, usage, money or budgets moved.\n");
        return out;
    }
    let labels: Vec<String> = summary.labels.iter().map(|l| format!("`{l}`")).collect();
    let _ = writeln!(out, "Labels: {}\n", labels.join(", "));
    if !summary.needs_human.is_empty() {
        out.push_str(
            "**Needs Jim.** Only Jim adds a vendor, money or a budget (standards/docs/cost.md):\n\n",
        );
        for reason in &summary.needs_human {
            let _ = writeln!(out, "- {reason}");
        }
        out.push('\n');
    }
    if !summary.changed_files.is_empty() {
        out.push_str("### Cost files changed\n\n");
        for file in &summary.changed_files {
            let _ = writeln!(out, "- `{file}`");
        }
        out.push('\n');
    }
    if !summary.projects.is_empty() {
        out.push_str("### Projects\n\n| Project | Change |\n| --- | --- |\n");
        for p in &summary.projects {
            let mut parts = Vec::new();
            for blip in &p.added {
                parts.push(format!("adds {blip}"));
            }
            for blip in &p.removed {
                parts.push(format!("drops {blip}"));
            }
            for u in &p.usage {
                parts.push(format!(
                    "{} {} → {}{}",
                    u.key,
                    u.before,
                    u.after,
                    u.ceiling
                        .map(|c| format!(" (headroom {})", headroom(u.after, Some(c))))
                        .unwrap_or_default()
                ));
            }
            if p.cents_before != p.cents_after {
                parts.push(format!(
                    "money this month {} → {}",
                    money(p.cents_before),
                    money(p.cents_after)
                ));
            }
            let _ = writeln!(out, "| {} | {} |", p.name, parts.join("; "));
        }
        out.push('\n');
    }
    if !summary.account.is_empty() {
        out.push_str(
            "### Account-wide usage\n\n| Limit | Before | After | Budget | Headroom |\n| --- | --- | --- | --- | --- |\n",
        );
        for u in &summary.account {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                u.key,
                u.before,
                u.after,
                opt(u.ceiling),
                headroom(u.after, u.ceiling)
            );
        }
        out.push('\n');
    }
    if !summary.budgets.is_empty() {
        out.push_str("### Budgets\n\n| What | Before | After |\n| --- | --- | --- |\n");
        for b in &summary.budgets {
            let _ = writeln!(out, "| {} | {} | {} |", b.what, b.before, b.after);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_cost_file, parse_limits, parse_radar};

    const RADAR: &str = r#"[
      {"name": "Cloudflare Pages", "ring": "Adopt", "kind": "vendor", "budget": 0},
      {"name": "PostHog", "ring": "Trial", "kind": "vendor", "budget": 0},
      {"name": "Squarespace", "ring": "Adopt", "kind": "vendor", "budget": 0},
      {"name": "Neon", "ring": "Assess", "kind": "vendor", "budget": 0}
    ]"#;

    const LIMITS: &str = r#"
[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
item = "Pages projects"
free = 100
budget = 25
scope = "account"
unit = "projects"
past = "x"
source = "x"
checked = "2026-11-26"
"#;

    fn project(name: &str, body: &str) -> Project {
        Project {
            name: name.to_string(),
            file: parse_cost_file(&format!(
                "reviewed = \"2026-11-26\"\nsummary = \"x\"\n{body}"
            ))
            .expect("cost file"),
        }
    }

    const PAGES: &str = "[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\npages_projects = 1\n";

    fn snapshot(projects: Vec<Project>) -> Snapshot {
        Snapshot {
            projects,
            detected: BTreeMap::new(),
            radar: parse_radar(RADAR).expect("radar"),
            limits: parse_limits(LIMITS).expect("limits"),
        }
    }

    fn month() -> Month {
        Month::new(2026, 11).expect("month")
    }

    #[test]
    fn test_no_change_no_labels() {
        let s = snapshot(vec![project("app", PAGES)]);
        let summary = summarize(&s, &s.clone(), vec![], month());
        assert!(summary.labels.is_empty());
        assert!(render_summary(&summary).contains("No cost changes"));
    }

    #[test]
    fn test_usage_estimate_gets_cost_only() {
        let base = snapshot(vec![project("app", PAGES)]);
        let head = snapshot(vec![project("app", &PAGES.replace("= 1", "= 2"))]);
        let summary = summarize(&base, &head, vec!["app/docs/cost.toml".into()], month());
        assert_eq!(summary.labels, ["cost"]);
        assert!(summary.needs_human.is_empty());
        assert_eq!(summary.account[0].before, 1);
        assert_eq!(summary.account[0].after, 2);
        let md = render_summary(&summary);
        assert!(md.contains("| pages_projects | 1 | 2 | 25 | 23 |"), "{md}");
    }

    #[test]
    fn test_new_vendor_money_and_paid_plan_need_jim() {
        let base = snapshot(vec![project("app", PAGES)]);
        let head = snapshot(vec![project(
            "app",
            &format!(
                "{}[[service]]\nblip = \"PostHog\"\nplan = \"paid\"\nwhy = \"x\"\n[[cost]]\nname = \"domain\"\nvendor = \"Squarespace\"\ncents = 1200\nevery = \"year\"\nsince = \"2026-11\"\n",
                PAGES
            ),
        )]);
        let summary = summarize(&base, &head, vec![], month());
        assert_eq!(summary.labels, ["cost", "needs-human"]);
        assert_eq!(
            summary.needs_human,
            [
                "app adds PostHog to its services",
                "app moves PostHog to a paid plan",
                "app adds or raises cost `domain`: $12.00 every Year",
            ]
        );
        assert_eq!(summary.projects[0].cents_after, 100);
    }

    #[test]
    fn test_code_using_an_unlisted_vendor_needs_jim() {
        let base = snapshot(vec![project("app", PAGES)]);
        let mut head = base.clone();
        head.detected
            .insert("app".into(), BTreeSet::from(["PostHog".to_string()]));
        let summary = summarize(&base, &head, vec![], month());
        assert_eq!(summary.labels, ["cost", "needs-human"]);
        assert!(summary.needs_human[0].contains("code starts using PostHog"));
    }

    #[test]
    fn test_budget_raises_and_approvals_need_jim_but_cuts_do_not() {
        let base = snapshot(vec![]);
        let mut head = base.clone();
        head.radar = parse_radar(&RADAR.replace("\"Assess\"", "\"Trial\"")).expect("radar");
        head.limits = parse_limits(&LIMITS.replace("budget = 25", "budget = 30")).expect("limits");
        let summary = summarize(&base, &head, vec![], month());
        assert_eq!(
            summary.needs_human,
            [
                "approves vendor Neon (Trial on the radar)",
                "raises the pages_projects budget from 25 to 30"
            ]
        );

        let mut cut = base.clone();
        cut.limits = parse_limits(&LIMITS.replace("budget = 25", "budget = 20")).expect("limits");
        let summary = summarize(&base, &cut, vec![], month());
        assert_eq!(summary.labels, ["cost"]);
        assert_eq!(summary.budgets.len(), 1);
    }

    #[test]
    fn test_summary_serializes_for_the_workflow() {
        let base = snapshot(vec![]);
        let head = snapshot(vec![project("app", PAGES)]);
        let summary = summarize(&base, &head, vec![], month());
        let json = serde_json::to_value(&summary).expect("json");
        assert_eq!(json["labels"][1], "needs-human");
        assert_eq!(json["projects"][0]["added"][0], "Cloudflare Pages");
    }
}
