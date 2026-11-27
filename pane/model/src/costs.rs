//! What every project's `docs/cost.toml` says, put together for the snapshot.
//!
//! `pane_snapshot` reads each project's `docs/cost.toml`, the vendor budgets
//! in `standards/radar/radar.json` and the free limits in
//! `standards/radar/limits.toml`, and [`build_costs`] turns them into
//! [`Costs`]: per project its services, plans, expected usage, cost and
//! revenue lines and this month's money; account-wide expected usage against
//! each limit's budget; and each vendor's spend this month against its
//! monthly radar budget. The format is in `standards/docs/cost.md`.
//!
//! Pure and in whole US cents, like [`crate::money`], so it is unit tested
//! and compiles to wasm.

use std::collections::BTreeMap;

use cost_model::{CostFile, Every, LimitsFile, Month, Plan, Radar, Scope, monthly_cents};
use serde::{Deserialize, Serialize};

use crate::money::{ProjectMoney, direct_cost, month_money, monthly_revenue};

/// Every project's costs, and the account's, for one month.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Costs {
    /// The month the money figures are for, `YYYY-MM`: the month the
    /// snapshot was taken in. Empty in a snapshot written before costs were
    /// read.
    pub month: String,
    /// One entry per project with a `docs/cost.toml`, in project order. A
    /// project without one has no services, no cost and no revenue.
    #[serde(default)]
    pub projects: Vec<ProjectCosts>,
    /// Every account-wide limit in `limits.toml`, in its order, with what
    /// all projects together expect to use.
    #[serde(default)]
    pub account: Vec<AccountUsage>,
    /// Every vendor in the radar, in its order, with its monthly budget and
    /// what projects spend with it this month; then any vendor a cost names
    /// that the radar doesn't list (which `cost check` reports as COST-003).
    #[serde(default)]
    pub budgets: Vec<VendorBudget>,
    /// Every project's money for the twelve months ending with
    /// [`Costs::month`], oldest first, for the Money page's chart.
    #[serde(default)]
    pub history: Vec<MonthMoney>,
    /// Whether the merged PRs behind each month's shared-cost split came
    /// from git. `false` (a checkout without history) means no month split
    /// its shared costs, so every project's share is 0.
    #[serde(default)]
    pub history_from_git: bool,
}

/// One month of every project's money.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonthMoney {
    /// `YYYY-MM`.
    pub month: String,
    /// The month's shared costs: every radar vendor's monthly budget, which
    /// is what the account is allowed to spend on it (today the Claude Code
    /// budget). The radar keeps no history, so every month uses today's.
    pub shared_cents: u64,
    /// How many merged PRs touched a project that month; the shared cost is
    /// split by them ([`crate::money::split_shared_costs`]).
    pub merged_prs: u32,
    /// One row per project, in project order.
    #[serde(default)]
    pub projects: Vec<ProjectMoney>,
}

impl MonthMoney {
    /// The month's row for `project`, if it is listed.
    pub fn project(&self, project: &str) -> Option<&ProjectMoney> {
        self.projects.iter().find(|m| m.project == project)
    }

    /// Revenue, direct cost, shared cost and net summed over every project.
    pub fn totals(&self) -> ProjectMoney {
        let sum =
            |f: fn(&ProjectMoney) -> u64| self.projects.iter().map(f).fold(0, u64::saturating_add);
        let revenue = sum(|m| m.revenue_cents);
        let direct = sum(|m| m.direct_cost_cents);
        let shared = sum(|m| m.shared_cost_cents);
        ProjectMoney {
            project: String::new(),
            month: self.month.clone(),
            revenue_cents: revenue,
            direct_cost_cents: direct,
            shared_cost_cents: shared,
            net_cents: crate::money::net(revenue, direct, shared),
        }
    }
}

/// One project's `docs/cost.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCosts {
    /// The project's path, such as `pane`.
    pub project: String,
    /// The file it came from, repo-relative, such as `pane/docs/cost.toml`.
    pub file: String,
    /// The file's summary sentence.
    pub summary: String,
    /// When the file was last checked against the code, `YYYY-MM-DD`.
    pub reviewed: String,
    #[serde(default)]
    pub services: Vec<ServiceUse>,
    #[serde(default)]
    pub costs: Vec<CostLine>,
    #[serde(default)]
    pub revenue: Vec<RevenueLine>,
    /// Revenue recorded for [`Costs::month`].
    pub revenue_cents: u64,
    /// The project's own costs that fall in [`Costs::month`].
    pub direct_cost_cents: u64,
}

/// One outside service a project uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceUse {
    /// The vendor blip in the radar, such as `Cloudflare Pages`.
    pub blip: String,
    /// `free` or `paid`.
    pub plan: String,
    pub why: String,
    /// The line of its `[[service]]` header in [`ProjectCosts::file`]; 0
    /// when unknown.
    #[serde(default)]
    pub line: u32,
    /// Expected use of each limit, sorted by key.
    #[serde(default)]
    pub usage: Vec<Usage>,
}

/// A project's expected use of one limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// The limit's key in `limits.toml`, such as `pages_projects`.
    pub key: String,
    /// What is counted, such as `Pages projects`; the key when `limits.toml`
    /// doesn't list it.
    pub item: String,
    /// The unit, such as `projects`; empty when `limits.toml` doesn't list
    /// the key.
    pub unit: String,
    pub expected: u64,
    /// For a per-project limit, the budget (or the free limit when there is
    /// no budget) this project's use is held to. `None` for an account-wide
    /// limit, which [`AccountUsage`] holds against the sum, and for an
    /// unknown key.
    pub ceiling: Option<u64>,
}

/// A `[[cost]]`: money that is the project's own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostLine {
    pub name: String,
    pub vendor: String,
    pub cents: u64,
    /// `month`, `year` or `once`.
    pub every: String,
    /// The first month it is paid, `YYYY-MM`.
    pub since: String,
    /// The next charge, `YYYY-MM-DD`.
    pub renews: Option<String>,
    /// What it adds to [`Costs::month`]: a yearly cost spread over twelve
    /// months, a once cost only in its month.
    pub month_cents: u64,
    /// The line of its `[[cost]]` header in [`ProjectCosts::file`]; 0 when
    /// unknown.
    #[serde(default)]
    pub line: u32,
}

/// A `[[revenue]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueLine {
    /// `YYYY-MM`.
    pub month: String,
    pub cents: u64,
    pub source: String,
    /// The line of its `[[revenue]]` header in [`ProjectCosts::file`]; 0
    /// when unknown.
    #[serde(default)]
    pub line: u32,
}

/// An account-wide limit and what every project together expects to use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUsage {
    pub key: String,
    /// The radar blip the limit belongs to.
    pub blip: String,
    pub item: String,
    pub unit: String,
    /// The sum of every project's expected use.
    pub expected: u64,
    /// The vendor's free limit, when it is a number.
    pub free: Option<u64>,
    /// Our budget inside the free limit, when there is one.
    pub budget: Option<u64>,
    /// The budget, or the free limit when there is no budget: what
    /// `expected` is held to (COST-005).
    pub ceiling: Option<u64>,
    /// The projects that expect to use some, in project order.
    #[serde(default)]
    pub by_project: Vec<ProjectAmount>,
}

impl AccountUsage {
    /// Whether the projects together expect more than the ceiling.
    pub fn is_over(&self) -> bool {
        self.ceiling.is_some_and(|c| self.expected > c)
    }
}

/// A vendor's monthly budget from the radar and this month's spend with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorBudget {
    pub vendor: String,
    /// The radar ring (`Adopt`, `Trial`, `Assess` or `Hold`); empty for a
    /// vendor the radar doesn't list.
    pub ring: String,
    /// The monthly budget in the radar, in cents; 0 for an unlisted vendor.
    pub budget_cents: u64,
    /// What projects' own costs with this vendor add to the month. Shared
    /// costs such as Claude Code are the budget itself, not a project's
    /// cost, so they don't count here.
    pub spend_cents: u64,
    /// The projects with a cost this month, in project order.
    #[serde(default)]
    pub by_project: Vec<ProjectAmount>,
}

impl VendorBudget {
    /// Whether projects spend more with the vendor than its budget (COST-006).
    pub fn is_over(&self) -> bool {
        self.spend_cents > self.budget_cents
    }
}

/// One project's part of a total: units of a limit, or cents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAmount {
    pub project: String,
    pub amount: u64,
}

/// Put every project's parsed `docs/cost.toml` together for `month`.
///
/// `projects` pairs each project's path with its parsed file, in project
/// order; leave out a project without one.
pub fn build_costs(
    month: Month,
    projects: &[(&str, &CostFile)],
    radar: &Radar,
    limits: &LimitsFile,
) -> Costs {
    let limit = |key: &str| limits.limit.iter().find(|l| l.key == key);

    let project_costs = projects
        .iter()
        .map(|&(project, file)| ProjectCosts {
            project: project.to_string(),
            file: format!("{project}/{}", cost_model::COST_FILE),
            summary: file.summary.clone(),
            reviewed: file.reviewed.format("%Y-%m-%d").to_string(),
            services: file
                .service
                .iter()
                .map(|s| ServiceUse {
                    blip: s.blip.clone(),
                    plan: match s.plan {
                        Plan::Free => "free",
                        Plan::Paid => "paid",
                    }
                    .to_string(),
                    why: s.why.clone(),
                    line: 0,
                    usage: s
                        .usage
                        .iter()
                        .map(|(key, &expected)| {
                            let found = limit(key);
                            Usage {
                                key: key.clone(),
                                item: found.map_or_else(|| key.clone(), |l| l.item.clone()),
                                unit: found.map(|l| l.unit.clone()).unwrap_or_default(),
                                expected,
                                ceiling: found
                                    .filter(|l| l.scope == Scope::Project)
                                    .and_then(|l| l.ceiling()),
                            }
                        })
                        .collect(),
                })
                .collect(),
            costs: file
                .cost
                .iter()
                .map(|c| CostLine {
                    name: c.name.clone(),
                    vendor: c.vendor.clone(),
                    cents: c.cents,
                    every: match c.every {
                        Every::Month => "month",
                        Every::Year => "year",
                        Every::Once => "once",
                    }
                    .to_string(),
                    since: c.since.to_string(),
                    renews: c.renews.map(|d| d.format("%Y-%m-%d").to_string()),
                    month_cents: monthly_cents(c, month),
                    line: 0,
                })
                .collect(),
            revenue: file
                .revenue
                .iter()
                .map(|r| RevenueLine {
                    month: r.month.to_string(),
                    cents: r.cents,
                    source: r.source.clone(),
                    line: 0,
                })
                .collect(),
            revenue_cents: monthly_revenue(&file.revenue, month),
            direct_cost_cents: direct_cost(&file.cost, month),
        })
        .collect();

    let account = limits
        .limit
        .iter()
        .filter(|l| l.scope == Scope::Account)
        .map(|l| {
            let by_project: Vec<ProjectAmount> = projects
                .iter()
                .filter_map(|&(project, file)| {
                    let used = file
                        .service
                        .iter()
                        .filter_map(|s| s.usage.get(&l.key))
                        .fold(0, |a: u64, &b| a.saturating_add(b));
                    (used > 0).then(|| ProjectAmount {
                        project: project.to_string(),
                        amount: used,
                    })
                })
                .collect();
            AccountUsage {
                key: l.key.clone(),
                blip: l.blip.clone(),
                item: l.item.clone(),
                unit: l.unit.clone(),
                expected: by_project
                    .iter()
                    .fold(0, |a: u64, p| a.saturating_add(p.amount)),
                free: l.free,
                budget: l.budget,
                ceiling: l.ceiling(),
                by_project,
            }
        })
        .collect();

    // Each vendor's spend this month, by project in project order.
    let mut spend: BTreeMap<&str, Vec<ProjectAmount>> = BTreeMap::new();
    let mut unlisted: Vec<&str> = Vec::new();
    for &(project, file) in projects {
        let mut by_vendor: BTreeMap<&str, u64> = BTreeMap::new();
        for cost in &file.cost {
            let cents = monthly_cents(cost, month);
            if cents > 0 {
                let total = by_vendor.entry(cost.vendor.as_str()).or_default();
                *total = total.saturating_add(cents);
            }
        }
        for (vendor, cents) in by_vendor {
            if radar.vendor(vendor).is_none() && !unlisted.contains(&vendor) {
                unlisted.push(vendor);
            }
            spend.entry(vendor).or_default().push(ProjectAmount {
                project: project.to_string(),
                amount: cents,
            });
        }
    }
    let budget = |vendor: &str, ring: &str, budget_cents: u64| {
        let by_project = spend.get(vendor).cloned().unwrap_or_default();
        VendorBudget {
            vendor: vendor.to_string(),
            ring: ring.to_string(),
            budget_cents,
            spend_cents: by_project
                .iter()
                .fold(0, |a: u64, p| a.saturating_add(p.amount)),
            by_project,
        }
    };
    let mut budgets: Vec<VendorBudget> = radar
        .blips
        .iter()
        .filter(|b| b.is_vendor())
        .map(|b| budget(&b.name, &b.ring, b.budget_cents()))
        .collect();
    unlisted.sort_unstable();
    budgets.extend(unlisted.into_iter().map(|v| budget(v, "", 0)));

    Costs {
        month: month.to_string(),
        projects: project_costs,
        account,
        budgets,
        history: Vec::new(),
        history_from_git: false,
    }
}

/// The `n` months ending with `last`, oldest first.
pub fn months_ending(last: Month, n: u32) -> Vec<Month> {
    let index = |m: Month| i64::from(m.year) * 12 + i64::from(m.month) - 1;
    let end = index(last);
    (0..i64::from(n))
        .rev()
        .filter_map(|back| {
            let i = end - back;
            let year = i32::try_from(i.div_euclid(12)).ok()?;
            let month = u32::try_from(i.rem_euclid(12) + 1).ok()?;
            Month::new(year, month)
        })
        .collect()
}

/// A month's shared costs: every radar vendor's monthly budget, in cents.
pub fn shared_cents(radar: &Radar) -> u64 {
    radar
        .blips
        .iter()
        .filter(|b| b.is_vendor())
        .map(cost_model::Blip::budget_cents)
        .fold(0, u64::saturating_add)
}

/// Every project's money for each of `months`.
///
/// `projects` lists every project, in order, with its parsed
/// `docs/cost.toml` (`None` without one); `shared_cents` is each month's
/// shared cost; `merged_prs` holds, per month, the projects each PR merged
/// that month touched. A month with no entry splits nothing.
pub fn build_history(
    months: &[Month],
    projects: &[(&str, Option<&CostFile>)],
    shared_cents: u64,
    merged_prs: &BTreeMap<Month, Vec<Vec<String>>>,
) -> Vec<MonthMoney> {
    months
        .iter()
        .map(|&month| {
            let prs = merged_prs.get(&month).map_or(&[][..], Vec::as_slice);
            MonthMoney {
                month: month.to_string(),
                shared_cents,
                merged_prs: u32::try_from(prs.iter().filter(|p| !p.is_empty()).count())
                    .unwrap_or(u32::MAX),
                projects: month_money(month, projects, shared_cents, prs),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use cost_model::{parse_cost_file, parse_limits, parse_radar};

    use super::*;

    type TestResult = Result<(), String>;

    const LIMITS: &str = r#"
[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
item = "Pages projects"
free = 100
budget = 25
scope = "account"
unit = "projects"
past = "Creation fails"
source = "https://example.com"
checked = "2026-11-26"

[[limit]]
key = "workers_requests_per_day"
blip = "Cloudflare Workers"
item = "Worker requests"
free = 100000
scope = "project"
unit = "requests a day"
past = "Requests fail"
source = "https://example.com"
checked = "2026-11-26"

[[limit]]
key = "d1_databases"
blip = "Cloudflare D1"
item = "D1 databases"
free = 10
budget = 1
scope = "account"
unit = "databases"
past = "Creation fails"
source = "https://example.com"
checked = "2026-11-26"
"#;

    const RADAR: &str = r#"[
  {"name": "Rust", "ring": "Adopt", "kind": "technology", "budget": 0},
  {"name": "Cloudflare Pages", "ring": "Adopt", "kind": "vendor", "budget": 0},
  {"name": "Squarespace", "ring": "Adopt", "kind": "vendor", "budget": 2},
  {"name": "Claude Code", "ring": "Adopt", "kind": "vendor", "budget": 25}
]"#;

    const SITE: &str = r#"
reviewed = "2026-11-26"
summary = "A site. $1 a year."

[[service]]
blip = "Cloudflare Pages"
plan = "free"
why = "Hosts it."
[service.usage]
pages_projects = 2
workers_requests_per_day = 500
unknown_key = 3

[[cost]]
name = "domain"
vendor = "Squarespace"
cents = 1200
every = "year"
since = "2026-11"
renews = "2027-11-01"
decision = "standards/docs/decisions/0000-example.md"

[[cost]]
name = "sticker"
vendor = "Sticker Mule"
cents = 300
every = "once"
since = "2026-11"
decision = "standards/docs/decisions/0000-example.md"

[[revenue]]
month = "2026-11"
cents = 450
source = "Sponsor"

[[revenue]]
month = "2026-10"
cents = 50
source = "Sponsor"
"#;

    const TOOL: &str = r#"
reviewed = "2026-11-20"
summary = "A CLI. $0 a month."

[[service]]
blip = "Cloudflare Pages"
plan = "paid"
why = "Docs."
[service.usage]
pages_projects = 24

[[cost]]
name = "domain"
vendor = "Squarespace"
cents = 2400
every = "year"
since = "2026-01"
renews = "2027-01-01"
decision = "standards/docs/decisions/0000-example.md"
"#;

    fn build(month: Month) -> Result<Costs, String> {
        let site = parse_cost_file(SITE)?;
        let tool = parse_cost_file(TOOL)?;
        Ok(build_costs(
            month,
            &[("site", &site), ("tool", &tool)],
            &parse_radar(RADAR)?,
            &parse_limits(LIMITS)?,
        ))
    }

    fn nov() -> Month {
        Month {
            year: 2026,
            month: 11,
        }
    }

    #[test]
    fn projects_keep_their_services_plans_and_usage() -> TestResult {
        let costs = build(nov())?;
        assert_eq!(costs.month, "2026-11");
        let site = &costs.projects[0];
        assert_eq!(site.project, "site");
        assert_eq!(site.file, "site/docs/cost.toml");
        assert_eq!(site.reviewed, "2026-11-26");
        assert_eq!(site.summary, "A site. $1 a year.");
        let service = &site.services[0];
        assert_eq!(
            (service.blip.as_str(), service.plan.as_str()),
            ("Cloudflare Pages", "free")
        );
        assert_eq!(costs.projects[1].services[0].plan, "paid");

        let usage: Vec<(&str, &str, &str, u64, Option<u64>)> = service
            .usage
            .iter()
            .map(|u| {
                (
                    u.key.as_str(),
                    u.item.as_str(),
                    u.unit.as_str(),
                    u.expected,
                    u.ceiling,
                )
            })
            .collect();
        assert_eq!(
            usage,
            [
                // Account-wide: held against the sum, not per project.
                ("pages_projects", "Pages projects", "projects", 2, None),
                // An unknown key keeps its name and has no ceiling.
                ("unknown_key", "unknown_key", "", 3, None),
                // Per project: no budget, so the free limit.
                (
                    "workers_requests_per_day",
                    "Worker requests",
                    "requests a day",
                    500,
                    Some(100_000)
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn money_this_month_spreads_yearly_costs_and_sums_revenue() -> TestResult {
        let costs = build(nov())?;
        let site = &costs.projects[0];
        // 1200 a year is 100 a month, plus the 300 once cost.
        assert_eq!(site.direct_cost_cents, 400);
        assert_eq!(site.revenue_cents, 450);
        assert_eq!(site.costs[0].every, "year");
        assert_eq!(site.costs[0].since, "2026-11");
        assert_eq!(site.costs[0].renews.as_deref(), Some("2027-11-01"));
        assert_eq!(site.costs[0].month_cents, 100);
        assert_eq!(site.costs[1].renews, None);
        assert_eq!(site.revenue.len(), 2);
        assert_eq!(site.revenue[1].month, "2026-10");

        // December: the once cost and November's revenue are gone.
        let dec = build(Month {
            year: 2026,
            month: 12,
        })?;
        assert_eq!(dec.projects[0].direct_cost_cents, 100);
        assert_eq!(dec.projects[0].revenue_cents, 0);
        assert_eq!(dec.projects[0].costs[1].month_cents, 0);
        Ok(())
    }

    #[test]
    fn account_usage_sums_every_project_against_its_ceiling() -> TestResult {
        let costs = build(nov())?;
        let keys: Vec<&str> = costs.account.iter().map(|a| a.key.as_str()).collect();
        assert_eq!(
            keys,
            ["pages_projects", "d1_databases"],
            "account limits only, in order"
        );

        let pages = &costs.account[0];
        assert_eq!(pages.expected, 26);
        assert_eq!(
            (pages.free, pages.budget, pages.ceiling),
            (Some(100), Some(25), Some(25))
        );
        assert!(pages.is_over());
        let by: Vec<(&str, u64)> = pages
            .by_project
            .iter()
            .map(|p| (p.project.as_str(), p.amount))
            .collect();
        assert_eq!(by, [("site", 2), ("tool", 24)]);

        let d1 = &costs.account[1];
        assert_eq!(d1.expected, 0);
        assert!(d1.by_project.is_empty());
        assert!(!d1.is_over());
        Ok(())
    }

    #[test]
    fn budgets_list_every_radar_vendor_then_unlisted_ones() -> TestResult {
        let costs = build(nov())?;
        let rows: Vec<(&str, &str, u64, u64)> = costs
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
        assert_eq!(
            rows,
            [
                ("Cloudflare Pages", "Adopt", 0, 0),
                // site 1200/12 = 100, tool 2400/12 = 200.
                ("Squarespace", "Adopt", 200, 300),
                ("Claude Code", "Adopt", 2500, 0),
                ("Sticker Mule", "", 0, 300),
            ]
        );
        assert!(costs.budgets[1].is_over());
        assert!(!costs.budgets[2].is_over());
        let by: Vec<(&str, u64)> = costs.budgets[1]
            .by_project
            .iter()
            .map(|p| (p.project.as_str(), p.amount))
            .collect();
        assert_eq!(by, [("site", 100), ("tool", 200)]);
        Ok(())
    }

    #[test]
    fn no_projects_still_lists_limits_and_budgets() -> TestResult {
        let costs = build_costs(nov(), &[], &parse_radar(RADAR)?, &parse_limits(LIMITS)?);
        assert!(costs.projects.is_empty());
        assert_eq!(costs.account.len(), 2);
        assert!(costs.account.iter().all(|a| a.expected == 0));
        assert_eq!(costs.budgets.len(), 3);
        Ok(())
    }

    #[test]
    fn months_ending_counts_back_across_years() {
        let months: Vec<String> = months_ending(
            Month {
                year: 2026,
                month: 2,
            },
            4,
        )
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(months, ["2025-11", "2025-12", "2026-01", "2026-02"]);
        assert_eq!(months_ending(nov(), 12).len(), 12);
        assert!(months_ending(nov(), 0).is_empty());
    }

    #[test]
    fn shared_cents_sums_vendor_budgets_only() -> TestResult {
        // Squarespace $2 and Claude Code $25; Rust is a technology.
        assert_eq!(shared_cents(&parse_radar(RADAR)?), 2700);
        Ok(())
    }

    #[test]
    fn history_splits_each_month_by_its_merged_prs() -> TestResult {
        let site = parse_cost_file(SITE)?;
        let oct = Month {
            year: 2026,
            month: 10,
        };
        let prs = BTreeMap::from([(
            nov(),
            vec![
                vec!["site".to_string()],
                vec!["site".to_string(), "tool".to_string()],
                Vec::new(),
            ],
        )]);
        let history = build_history(
            &[oct, nov()],
            &[("site", Some(&site)), ("tool", None)],
            2000,
            &prs,
        );
        assert_eq!(history.len(), 2);

        // October: no merged PRs, so nothing is split.
        assert_eq!(history[0].month, "2026-10");
        assert_eq!(history[0].merged_prs, 0);
        assert_eq!(history[0].totals().shared_cost_cents, 0);
        assert_eq!(
            history[0].project("site").map(|m| m.revenue_cents),
            Some(50)
        );

        // November: site 1.5 of 2 PRs, tool 0.5; the repo-wide PR doesn't count.
        let n = &history[1];
        assert_eq!(n.merged_prs, 2);
        assert_eq!(n.shared_cents, 2000);
        let site_nov = n.project("site").ok_or("no site row")?;
        assert_eq!(site_nov.shared_cost_cents, 1500);
        assert_eq!(site_nov.direct_cost_cents, 400);
        assert_eq!(n.project("tool").map(|m| m.shared_cost_cents), Some(500));
        let totals = n.totals();
        assert_eq!(totals.shared_cost_cents, 2000);
        assert_eq!(totals.revenue_cents, 450);
        assert_eq!(totals.net_cents, 450 - 400 - 2000);
        assert!(n.project("nope").is_none());
        Ok(())
    }

    #[test]
    fn costs_round_trip_through_json() -> Result<(), Box<dyn std::error::Error>> {
        let costs = build(nov())?;
        let back: Costs = serde_json::from_str(&serde_json::to_string(&costs)?)?;
        assert_eq!(back, costs);
        Ok(())
    }
}
