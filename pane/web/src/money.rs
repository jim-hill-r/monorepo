//! The Money page and each Project page's Money section: what every project
//! earns and costs ("Revenue and cost" in the review dashboard plan),
//! from the snapshot's `costs` (`pane_model::costs`). Every number links to
//! the line on GitHub it comes from.

use dioxus::prelude::*;
use pane_model::Snapshot;
use pane_model::costs::{Costs, MonthMoney, ProjectCosts, VendorBudget};
use pane_model::money::ProjectMoney;

use crate::route::Route;
use crate::{WithSnapshot, view};

/// The radar file the shared costs come from.
const RADAR: &str = "standards/radar/radar.json";

/// Whole cents as US dollars: `$1,234.56`, or `−$25.00` below zero.
pub fn dollars(cents: i64) -> String {
    let sign = if cents < 0 { "−" } else { "" };
    let abs = cents.unsigned_abs();
    let whole = (abs / 100).to_string();
    let mut grouped = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(c);
    }
    format!("{sign}${grouped}.{:02}", abs % 100)
}

/// Unsigned cents as dollars.
pub fn dollars_u(cents: u64) -> String {
    dollars(i64::try_from(cents).unwrap_or(i64::MAX))
}

/// The snapshot month's money, the last month of the history.
pub fn this_month(costs: &Costs) -> Option<&MonthMoney> {
    costs.history.last()
}

/// The rows with any money this month, in project order, and how many
/// projects had none.
pub fn rows_with_money(month: &MonthMoney) -> (Vec<&ProjectMoney>, usize) {
    let (some, none): (Vec<&ProjectMoney>, Vec<&ProjectMoney>) = month
        .projects
        .iter()
        .partition(|m| m.revenue_cents > 0 || m.direct_cost_cents > 0 || m.shared_cost_cents > 0);
    (some, none.len())
}

/// A project's `docs/cost.toml`, if it has one.
pub fn project_costs<'a>(costs: &'a Costs, project: &str) -> Option<&'a ProjectCosts> {
    costs.projects.iter().find(|p| p.project == project)
}

/// Where a project's revenue for `month` comes from: its first
/// `[[revenue]]` line for the month, else the file.
pub fn revenue_source(costs: &Costs, project: &str, month: &str) -> Option<(String, u32)> {
    let file = project_costs(costs, project)?;
    let line = file
        .revenue
        .iter()
        .find(|r| r.month == month && r.line > 0)
        .map_or(0, |r| r.line);
    Some((file.file.clone(), line))
}

/// Where a project's direct cost for the snapshot month comes from: its
/// first `[[cost]]` line that adds to the month, else the file.
pub fn cost_source(costs: &Costs, project: &str) -> Option<(String, u32)> {
    let file = project_costs(costs, project)?;
    let line = file
        .costs
        .iter()
        .find(|c| c.month_cents > 0 && c.line > 0)
        .map_or(0, |c| c.line);
    Some((file.file.clone(), line))
}

/// A link to `file` at `line` (the whole file for line 0) at `commit`.
pub fn source_url(commit: &str, file: &str, line: u32) -> String {
    if line == 0 {
        view::github_file_url(commit, file)
    } else {
        view::github_line_url(commit, file, line)
    }
}

/// The vendors worth listing: a monthly budget, or projects spending with
/// them this month.
pub fn listed_budgets(costs: &Costs) -> Vec<&VendorBudget> {
    costs
        .budgets
        .iter()
        .filter(|b| b.budget_cents > 0 || b.spend_cents > 0)
        .collect()
}

/// One bar of the 12-month net chart, in SVG units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    pub month: String,
    pub net_cents: i64,
    pub x: i64,
    pub y: i64,
    pub height: i64,
}

/// Chart geometry: the width and height of the plot, and each bar's slot.
pub const CHART_WIDTH: i64 = 600;
pub const CHART_HEIGHT: i64 = 160;
const SLOT: i64 = CHART_WIDTH / 12;
pub const BAR_WIDTH: i64 = SLOT - 12;

/// Bars for each month's total net, scaled so the largest magnitude fills
/// the plot, and the baseline's y (where net is zero). Bars above the
/// baseline are a profit, below it a loss.
pub fn net_bars(history: &[MonthMoney]) -> (Vec<Bar>, i64) {
    let nets: Vec<(String, i64)> = history
        .iter()
        .map(|m| (m.month.clone(), m.totals().net_cents))
        .collect();
    let max = nets.iter().map(|(_, n)| *n).max().unwrap_or(0).max(0);
    let min = nets.iter().map(|(_, n)| *n).min().unwrap_or(0).min(0);
    let span = (max - min).max(1);
    // i128 so cents times pixels can't overflow.
    let scale = |cents: i64| {
        i64::try_from(i128::from(cents) * i128::from(CHART_HEIGHT) / i128::from(span)).unwrap_or(0)
    };
    let baseline = scale(max);
    let bars = nets
        .into_iter()
        .enumerate()
        .map(|(i, (month, net))| {
            let h = scale(net.abs()).max(i64::from(net != 0));
            let x = i64::try_from(i).unwrap_or(0) * SLOT + (SLOT - BAR_WIDTH) / 2;
            let y = if net >= 0 { baseline - h } else { baseline };
            Bar {
                month,
                net_cents: net,
                x,
                y,
                height: h,
            }
        })
        .collect();
    (bars, baseline)
}

/// `2026-11` as `Nov`.
pub fn month_short(month: &str) -> &'static str {
    const NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    month
        .get(5..7)
        .and_then(|m| m.parse::<usize>().ok())
        .and_then(|m| NAMES.get(m.wrapping_sub(1)))
        .copied()
        .unwrap_or("")
}

#[component]
pub fn Money() -> Element {
    rsx! {
        h1 { "Money" }
        WithSnapshot { body: move |s: Snapshot| rsx! { MoneyBody { snapshot: s } } }
    }
}

#[component]
fn MoneyBody(snapshot: Snapshot) -> Element {
    let costs = snapshot.costs.clone();
    let commit = snapshot.commit.clone();
    let Some(month) = this_month(&costs).cloned() else {
        return rsx! {
            p { class: "muted", "This snapshot has no cost data. It comes from each project's docs/cost.toml." }
        };
    };
    let (rows, quiet) = rows_with_money(&month);
    let rows: Vec<ProjectMoney> = rows.into_iter().cloned().collect();
    let totals = month.totals();
    let budgets: Vec<VendorBudget> = listed_budgets(&costs).into_iter().cloned().collect();
    let history = costs.history.clone();
    rsx! {
        p {
            "What each project earned and cost in {month.month}. Shared costs ("
            a { href: "{view::github_file_url(&commit, RADAR)}", "the radar's monthly budgets" }
            ", {dollars_u(month.shared_cents)} a month) are split by each project's share of the "
            "{month.merged_prs} PRs merged this month."
        }
        if !costs.history_from_git {
            p { class: "muted", role: "note", "data-testid": "money-no-history",
                "This snapshot was built without git history, so shared costs aren't split across projects."
            }
        }
        section { class: "group", "data-testid": "money-month",
            h2 { "This month" }
            table { class: "rotation money",
                thead {
                    tr {
                        th { scope: "col", "Project" }
                        th { scope: "col", class: "num", "Revenue" }
                        th { scope: "col", class: "num", "Direct cost" }
                        th { scope: "col", class: "num", "Shared" }
                        th { scope: "col", class: "num", "Net" }
                    }
                }
                tbody {
                    for m in rows {
                        MoneyRow { key: "{m.project}", m: m.clone(), costs: costs.clone(), commit: commit.clone() }
                    }
                }
                tfoot {
                    tr {
                        th { scope: "row", "Total" }
                        td { class: "num", "{dollars_u(totals.revenue_cents)}" }
                        td { class: "num", "{dollars_u(totals.direct_cost_cents)}" }
                        td { class: "num", "{dollars_u(totals.shared_cost_cents)}" }
                        td { class: "num", strong { "{dollars(totals.net_cents)}" } }
                    }
                }
            }
            if quiet > 0 {
                p { class: "muted", "{quiet} more projects had no revenue or cost this month." }
            }
        }
        section { class: "group", "data-testid": "money-chart",
            h2 { "Net, last 12 months" }
            NetChart { history: history.clone() }
            details {
                summary { "As a table" }
                table { class: "rotation money",
                    thead {
                        tr {
                            th { scope: "col", "Month" }
                            th { scope: "col", class: "num", "Revenue" }
                            th { scope: "col", class: "num", "Cost" }
                            th { scope: "col", class: "num", "Net" }
                        }
                    }
                    tbody {
                        for m in history {
                            MonthRow { key: "{m.month}", m }
                        }
                    }
                }
            }
        }
        section { class: "group", "data-testid": "money-shared",
            h2 { "Shared costs and vendor budgets" }
            p { class: "muted",
                "Each vendor's monthly budget in the tech radar, and what projects' own costs spend with it this month."
            }
            table { class: "rotation money",
                thead {
                    tr {
                        th { scope: "col", "Vendor" }
                        th { scope: "col", "Ring" }
                        th { scope: "col", class: "num", "Budget a month" }
                        th { scope: "col", class: "num", "Projects' spend" }
                    }
                }
                tbody {
                    for b in budgets {
                        tr { key: "{b.vendor}",
                            th { scope: "row",
                                a { href: "{view::github_file_url(&commit, RADAR)}", "{b.vendor}" }
                            }
                            td { if b.ring.is_empty() { "Not on the radar" } else { "{b.ring}" } }
                            td { class: "num", "{dollars_u(b.budget_cents)}" }
                            td { class: if b.is_over() { "num over" } else { "num" },
                                "{dollars_u(b.spend_cents)}"
                                if b.is_over() { " (over budget)" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A linked dollar figure, or plain text when there is no source file.
#[component]
fn Figure(text: String, source: Option<(String, u32)>, commit: String) -> Element {
    match source {
        Some((file, line)) => rsx! { a { href: "{source_url(&commit, &file, line)}", "{text}" } },
        None => rsx! { "{text}" },
    }
}

#[component]
fn MoneyRow(m: ProjectMoney, costs: Costs, commit: String) -> Element {
    let segments: Vec<String> = m.project.split('/').map(str::to_owned).collect();
    let revenue = revenue_source(&costs, &m.project, &m.month);
    let cost = cost_source(&costs, &m.project);
    rsx! {
        tr {
            th { scope: "row",
                Link { to: Route::ProjectPage { path: segments }, "{m.project}" }
            }
            td { class: "num", Figure { text: dollars_u(m.revenue_cents), source: revenue, commit: commit.clone() } }
            td { class: "num", Figure { text: dollars_u(m.direct_cost_cents), source: cost, commit: commit.clone() } }
            td { class: "num", Figure { text: dollars_u(m.shared_cost_cents), source: Some((RADAR.to_owned(), 0)), commit: commit.clone() } }
            td { class: if m.net_cents < 0 { "num loss" } else { "num" }, "{dollars(m.net_cents)}" }
        }
    }
}

#[component]
fn MonthRow(m: MonthMoney) -> Element {
    let t = m.totals();
    let cost = t.direct_cost_cents.saturating_add(t.shared_cost_cents);
    rsx! {
        tr {
            th { scope: "row", "{m.month}" }
            td { class: "num", "{dollars_u(t.revenue_cents)}" }
            td { class: "num", "{dollars_u(cost)}" }
            td { class: "num", "{dollars(t.net_cents)}" }
        }
    }
}

/// One series, so the title names it and there is no legend; each bar has
/// its month and value as a hover title, and the table below repeats them.
#[component]
fn NetChart(history: Vec<MonthMoney>) -> Element {
    let (bars, baseline) = net_bars(&history);
    let label_y = CHART_HEIGHT + 16;
    rsx! {
        svg {
            class: "money-chart",
            view_box: "0 0 {CHART_WIDTH} {CHART_HEIGHT + 24}",
            role: "img",
            "aria-label": "Total net by month for the last 12 months",
            line { class: "baseline", x1: "0", x2: "{CHART_WIDTH}", y1: "{baseline}", y2: "{baseline}" }
            for b in bars {
                g { key: "{b.month}",
                    title { "{b.month}: {dollars(b.net_cents)}" }
                    rect { class: "bar", x: "{b.x}", y: "{b.y}", width: "{BAR_WIDTH}", height: "{b.height}", rx: "2" }
                    text { class: "tick", x: "{b.x + BAR_WIDTH / 2}", y: "{label_y}", text_anchor: "middle", "{month_short(&b.month)}" }
                }
            }
        }
    }
}

/// The Money section of a Project page.
#[component]
pub fn ProjectMoneySection(snapshot: Snapshot, path: String) -> Element {
    let costs = snapshot.costs.clone();
    let commit = snapshot.commit.clone();
    let file = project_costs(&costs, &path).cloned();
    let month = this_month(&costs).and_then(|m| m.project(&path)).cloned();
    rsx! {
        section { class: "group", "data-testid": "project-money",
            h2 { "Money" }
            match file {
                None => rsx! {
                    p { class: "muted", "No docs/cost.toml, so no services, revenue or costs of its own." }
                },
                Some(f) => rsx! {
                    p {
                        "{f.summary} "
                        a { href: "{view::github_file_url(&commit, &f.file)}", "docs/cost.toml" }
                        span { class: "muted", " (reviewed {f.reviewed})" }
                    }
                    if let Some(m) = month {
                        p { "data-testid": "project-money-month",
                            "{m.month}: revenue "
                            Figure { text: dollars_u(m.revenue_cents), source: revenue_source(&costs, &path, &m.month), commit: commit.clone() }
                            ", direct cost "
                            Figure { text: dollars_u(m.direct_cost_cents), source: cost_source(&costs, &path), commit: commit.clone() }
                            ", shared "
                            Figure { text: dollars_u(m.shared_cost_cents), source: Some((RADAR.to_owned(), 0)), commit: commit.clone() }
                            ", net "
                            strong { "{dollars(m.net_cents)}" }
                            ". "
                            Link { to: Route::Money {}, "All projects" }
                        }
                    }
                    if !f.services.is_empty() {
                        h3 { "Services" }
                        ul { class: "rows",
                            for s in f.services.clone() {
                                li { key: "{s.blip}{s.line}",
                                    a { href: "{source_url(&commit, &f.file, s.line)}", "{s.blip}" }
                                    span { class: "muted", " {s.plan} plan. " }
                                    "{s.why}"
                                    for u in s.usage {
                                        span { class: "muted", key: "{u.key}",
                                            " {u.item}: {u.expected}"
                                            if let Some(c) = u.ceiling { " of {c}" }
                                            if !u.unit.is_empty() { " {u.unit}" }
                                            "."
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !f.costs.is_empty() {
                        h3 { "Its own costs" }
                        ul { class: "rows",
                            for c in f.costs.clone() {
                                li { key: "{c.name}{c.line}",
                                    a { href: "{source_url(&commit, &f.file, c.line)}", "{c.name}" }
                                    " {dollars_u(c.cents)} each {c.every} to {c.vendor}, since {c.since}"
                                    if let Some(r) = &c.renews { ", renews {r}" }
                                    "."
                                }
                            }
                        }
                    }
                    if !f.revenue.is_empty() {
                        h3 { "Revenue" }
                        ul { class: "rows",
                            for r in f.revenue.clone() {
                                li { key: "{r.month}{r.line}",
                                    a { href: "{source_url(&commit, &f.file, r.line)}", "{r.month}" }
                                    " {dollars_u(r.cents)} from {r.source}."
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pane_model::costs::{CostLine, RevenueLine};

    use super::*;

    fn money(project: &str, revenue: u64, direct: u64, shared: u64) -> ProjectMoney {
        ProjectMoney {
            project: project.to_owned(),
            month: "2026-11".to_owned(),
            revenue_cents: revenue,
            direct_cost_cents: direct,
            shared_cost_cents: shared,
            net_cents: pane_model::money::net(revenue, direct, shared),
        }
    }

    fn month(name: &str, projects: Vec<ProjectMoney>) -> MonthMoney {
        MonthMoney {
            month: name.to_owned(),
            shared_cents: 2500,
            merged_prs: 3,
            projects,
        }
    }

    fn costs() -> Costs {
        Costs {
            month: "2026-11".to_owned(),
            projects: vec![ProjectCosts {
                project: "app".to_owned(),
                file: "app/docs/cost.toml".to_owned(),
                summary: "An app.".to_owned(),
                reviewed: "2026-11-25".to_owned(),
                services: vec![],
                costs: vec![
                    CostLine {
                        name: "later".to_owned(),
                        vendor: "Squarespace".to_owned(),
                        cents: 100,
                        every: "month".to_owned(),
                        since: "2027-01".to_owned(),
                        renews: None,
                        month_cents: 0,
                        line: 5,
                    },
                    CostLine {
                        name: "domain".to_owned(),
                        vendor: "Squarespace".to_owned(),
                        cents: 1200,
                        every: "year".to_owned(),
                        since: "2026-01".to_owned(),
                        renews: Some("2027-01-01".to_owned()),
                        month_cents: 100,
                        line: 12,
                    },
                ],
                revenue: vec![
                    RevenueLine {
                        month: "2026-10".to_owned(),
                        cents: 1,
                        source: "x".to_owned(),
                        line: 20,
                    },
                    RevenueLine {
                        month: "2026-11".to_owned(),
                        cents: 500,
                        source: "Sponsor".to_owned(),
                        line: 24,
                    },
                ],
                revenue_cents: 500,
                direct_cost_cents: 100,
            }],
            history: vec![
                month("2026-10", vec![money("app", 0, 0, 0)]),
                month(
                    "2026-11",
                    vec![
                        money("app", 500, 100, 2000),
                        money("tool", 0, 0, 500),
                        money("quiet", 0, 0, 0),
                    ],
                ),
            ],
            history_from_git: true,
            ..Costs::default()
        }
    }

    #[test]
    fn formats_dollars_with_commas_and_a_minus() {
        assert_eq!(dollars(0), "$0.00");
        assert_eq!(dollars(5), "$0.05");
        assert_eq!(dollars(2500), "$25.00");
        assert_eq!(dollars(123_456_789), "$1,234,567.89");
        assert_eq!(dollars(-2600), "−$26.00");
        assert_eq!(dollars(i64::MIN), "−$92,233,720,368,547,758.08");
        assert_eq!(dollars_u(u64::MAX), "$92,233,720,368,547,758.07");
    }

    #[test]
    fn this_month_is_the_last_and_hides_projects_with_no_money() {
        let c = costs();
        let m = this_month(&c).map(|m| m.month.as_str());
        assert_eq!(m, Some("2026-11"));
        let (rows, quiet) = rows_with_money(&c.history[1]);
        let names: Vec<&str> = rows.iter().map(|r| r.project.as_str()).collect();
        assert_eq!(names, ["app", "tool"]);
        assert_eq!(quiet, 1);
        assert!(this_month(&Costs::default()).is_none());
    }

    #[test]
    fn numbers_link_to_their_source_lines() {
        let c = costs();
        assert_eq!(
            revenue_source(&c, "app", "2026-11"),
            Some(("app/docs/cost.toml".to_owned(), 24))
        );
        // No revenue that month: the file.
        assert_eq!(
            revenue_source(&c, "app", "2026-09"),
            Some(("app/docs/cost.toml".to_owned(), 0))
        );
        // The first cost that adds to the month, not the one that hasn't started.
        assert_eq!(
            cost_source(&c, "app"),
            Some(("app/docs/cost.toml".to_owned(), 12))
        );
        assert_eq!(cost_source(&c, "tool"), None);
        assert_eq!(
            source_url("abc", "app/docs/cost.toml", 12),
            "https://github.com/jimhillr/monorepo/blob/abc/app/docs/cost.toml#L12"
        );
        assert_eq!(
            source_url("abc", "app/docs/cost.toml", 0),
            "https://github.com/jimhillr/monorepo/blob/abc/app/docs/cost.toml"
        );
    }

    #[test]
    fn lists_vendors_with_a_budget_or_spend() {
        let budget = |vendor: &str, budget_cents, spend_cents| VendorBudget {
            vendor: vendor.to_owned(),
            ring: "Adopt".to_owned(),
            budget_cents,
            spend_cents,
            by_project: vec![],
        };
        let c = Costs {
            budgets: vec![
                budget("Free", 0, 0),
                budget("Claude Code", 2500, 0),
                budget("Squarespace", 0, 100),
            ],
            ..Costs::default()
        };
        let names: Vec<&str> = listed_budgets(&c)
            .iter()
            .map(|b| b.vendor.as_str())
            .collect();
        assert_eq!(names, ["Claude Code", "Squarespace"]);
    }

    #[test]
    fn net_bars_sit_on_a_shared_baseline() {
        let history = vec![
            month("2026-10", vec![money("a", 1000, 0, 0)]),
            month("2026-11", vec![money("a", 0, 0, 3000)]),
            month("2026-12", vec![money("a", 0, 0, 0)]),
        ];
        let (bars, baseline) = net_bars(&history);
        // Span is 1000 - (-3000) = 4000 cents over 160 units.
        assert_eq!(baseline, 40);
        assert_eq!((bars[0].y, bars[0].height), (0, 40));
        assert_eq!((bars[1].y, bars[1].height), (40, 120));
        assert_eq!(bars[2].height, 0);
        assert_eq!(bars[1].net_cents, -3000);
        assert!(bars[1].x > bars[0].x);

        // All losses: the baseline is at the top.
        let (_, baseline) = net_bars(&history[1..2]);
        assert_eq!(baseline, 0);
        // Nothing at all still draws.
        let (bars, baseline) = net_bars(&[]);
        assert!(bars.is_empty());
        assert_eq!(baseline, 0);
    }

    #[test]
    fn short_month_names() {
        assert_eq!(month_short("2026-01"), "Jan");
        assert_eq!(month_short("2026-12"), "Dec");
        assert_eq!(month_short("2026-13"), "");
        assert_eq!(month_short("bad"), "");
    }
}
