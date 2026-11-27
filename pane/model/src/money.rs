//! What each project earns and costs in a month, for pane's Money page.
//!
//! The rules come from "Revenue and cost" in the
//! [review dashboard plan](../../docs/plans/2026-11-review-dashboard.md).
//! Revenue and project-only costs live in each project's `docs/cost.toml`
//! (`standards/docs/cost.md`), parsed by [`cost_model::parse_cost_file`];
//! a project without one has no revenue and no direct cost. Shared costs,
//! such as the Claude Code budget, are split across projects by their share
//! of the PRs merged that month.
//!
//! Everything here is pure and in whole US cents, so it is unit tested and
//! nothing is lost to floating point.

use std::collections::BTreeMap;

use cost_model::{Cost, CostFile, Month, Revenue, monthly_cents};
use serde::{Deserialize, Serialize};

/// One project's money for one month.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMoney {
    /// The project's path, such as `pane`.
    pub project: String,
    /// The month, written `YYYY-MM`.
    pub month: String,
    /// Revenue recorded for the month.
    pub revenue_cents: u64,
    /// The project's own costs that fall in the month, with yearly costs
    /// spread over twelve months and a once cost only in its month.
    pub direct_cost_cents: u64,
    /// The project's share of the month's shared costs.
    pub shared_cost_cents: u64,
    /// Revenue less both costs. Negative when the project doesn't pay for
    /// itself.
    pub net_cents: i64,
}

/// Revenue recorded for `month`, summing every entry for it.
pub fn monthly_revenue(revenue: &[Revenue], month: Month) -> u64 {
    revenue
        .iter()
        .filter(|r| r.month == month)
        .map(|r| r.cents)
        .fold(0, u64::saturating_add)
}

/// The project's own costs that fall in `month`.
pub fn direct_cost(costs: &[Cost], month: Month) -> u64 {
    costs
        .iter()
        .map(|c| monthly_cents(c, month))
        .fold(0, u64::saturating_add)
}

/// Revenue less direct and shared costs, saturating at the ends of `i64`.
pub fn net(revenue_cents: u64, direct_cost_cents: u64, shared_cost_cents: u64) -> i64 {
    let net =
        i128::from(revenue_cents) - i128::from(direct_cost_cents) - i128::from(shared_cost_cents);
    i64::try_from(net).unwrap_or(if net < 0 { i64::MIN } else { i64::MAX })
}

/// Split `shared_cents` across projects by their share of the PRs merged in
/// the month.
///
/// `merged_prs` has one entry per merged PR: the projects it touched. A PR
/// touching `n` projects counts `1/n` to each; a project listed twice in one
/// PR counts once. A PR touching no project (a repo-wide change) is left
/// out, and projects with no merged PRs get nothing. Shares are whole cents
/// that always add up to `shared_cents` (largest remainder, ties to the
/// project name that sorts first), unless no PR touched a project, when the
/// result is empty and the cost is not split.
pub fn split_shared_costs<S: AsRef<str>>(
    shared_cents: u64,
    merged_prs: &[Vec<S>],
) -> BTreeMap<String, u64> {
    let prs: Vec<Vec<&str>> = merged_prs
        .iter()
        .map(|pr| {
            let mut projects: Vec<&str> = pr.iter().map(AsRef::as_ref).collect();
            projects.sort_unstable();
            projects.dedup();
            projects
        })
        .filter(|projects| !projects.is_empty())
        .collect();
    if prs.is_empty() {
        return BTreeMap::new();
    }

    // Weigh every PR in units of 1/denominator so the fractions stay exact.
    let denominator = prs.iter().map(|p| p.len() as u128).fold(1, lcm);
    let mut weights: BTreeMap<&str, u128> = BTreeMap::new();
    for pr in &prs {
        let each = denominator / pr.len() as u128;
        for project in pr {
            *weights.entry(project).or_default() += each;
        }
    }
    let total = denominator * prs.len() as u128;

    let shared = u128::from(shared_cents);
    let mut shares: Vec<(&str, u128, u128)> = weights
        .iter()
        .map(|(&project, &weight)| {
            let exact = shared * weight;
            (project, exact / total, exact % total)
        })
        .collect();
    let given: u128 = shares.iter().map(|(_, cents, _)| cents).sum();
    let mut left = shared - given;

    // Hand the leftover cents to the largest remainders; the stable sort
    // keeps name order for ties.
    let mut order: Vec<usize> = (0..shares.len()).collect();
    order.sort_by(|&a, &b| shares[b].2.cmp(&shares[a].2));
    for i in order {
        if left == 0 {
            break;
        }
        shares[i].1 += 1;
        left -= 1;
    }

    shares
        .into_iter()
        .map(|(project, cents, _)| {
            (
                project.to_string(),
                u64::try_from(cents).unwrap_or(u64::MAX),
            )
        })
        .collect()
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn lcm(a: u128, b: u128) -> u128 {
    a / gcd(a, b) * b
}

/// One project's money for `month`: its revenue and direct cost from its
/// `docs/cost.toml` (`None` when it has none) and its share of shared costs.
pub fn project_money(
    project: &str,
    month: Month,
    cost_file: Option<&CostFile>,
    shared_cost_cents: u64,
) -> ProjectMoney {
    let (revenue_cents, direct_cost_cents) = cost_file.map_or((0, 0), |f| {
        (
            monthly_revenue(&f.revenue, month),
            direct_cost(&f.cost, month),
        )
    });
    ProjectMoney {
        project: project.to_string(),
        month: month.to_string(),
        revenue_cents,
        direct_cost_cents,
        shared_cost_cents,
        net_cents: net(revenue_cents, direct_cost_cents, shared_cost_cents),
    }
}

/// Every project's money for `month`, in the order given: `projects` pairs
/// each project's path with its parsed `docs/cost.toml`, `shared_cents` is
/// the month's shared costs and `merged_prs` the projects each PR merged that
/// month touched (see [`split_shared_costs`]). A PR project not in
/// `projects` takes its share but gets no row, so the rows add up to less
/// than `shared_cents`; the caller lists every project to avoid that.
pub fn month_money<S: AsRef<str>>(
    month: Month,
    projects: &[(&str, Option<&CostFile>)],
    shared_cents: u64,
    merged_prs: &[Vec<S>],
) -> Vec<ProjectMoney> {
    let shares = split_shared_costs(shared_cents, merged_prs);
    projects
        .iter()
        .map(|&(project, file)| {
            let shared = shares.get(project).copied().unwrap_or(0);
            project_money(project, month, file, shared)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use cost_model::parse_cost_file;

    use super::*;

    fn month(year: i32, month: u32) -> Month {
        Month { year, month }
    }

    fn nov() -> Month {
        month(2026, 11)
    }

    fn file(text: &str) -> CostFile {
        match parse_cost_file(&format!(
            "reviewed = \"2026-11-26\"\nsummary = \"Test.\"\n{text}"
        )) {
            Ok(f) => f,
            Err(e) => panic!("fixture doesn't parse: {e}"),
        }
    }

    const MONEY: &str = r#"
[[cost]]
name = "domain"
vendor = "Squarespace"
cents = 1200
every = "year"
since = "2026-11"
renews = "2027-11-01"
decision = "standards/docs/decisions/0000-example.md"

[[cost]]
name = "store fee"
vendor = "GitHub"
cents = 2500
every = "once"
since = "2026-11"
decision = "standards/docs/decisions/0000-example.md"

[[cost]]
name = "hosting"
vendor = "Cloudflare"
cents = 500
every = "month"
since = "2026-12"
decision = "standards/docs/decisions/0000-example.md"

[[revenue]]
month = "2026-11"
cents = 300
source = "Sponsor"

[[revenue]]
month = "2026-11"
cents = 200
source = "Store"

[[revenue]]
month = "2026-12"
cents = 9999
source = "Store"
"#;

    #[test]
    fn revenue_sums_only_the_month() {
        let f = file(MONEY);
        assert_eq!(monthly_revenue(&f.revenue, nov()), 500);
        assert_eq!(monthly_revenue(&f.revenue, month(2026, 12)), 9999);
        assert_eq!(monthly_revenue(&f.revenue, month(2026, 10)), 0);
        assert_eq!(monthly_revenue(&[], nov()), 0);
    }

    #[test]
    fn direct_cost_spreads_yearly_and_charges_once_costs_in_their_month() {
        let f = file(MONEY);
        // 1200 a year is 100 a month, plus the 2500 once cost; hosting hasn't started.
        assert_eq!(direct_cost(&f.cost, nov()), 100 + 2500);
        // December: the once cost is gone and hosting has started.
        assert_eq!(direct_cost(&f.cost, month(2026, 12)), 100 + 500);
        // Before anything started.
        assert_eq!(direct_cost(&f.cost, month(2026, 10)), 0);
    }

    #[test]
    fn net_can_be_negative_and_saturates() {
        assert_eq!(net(500, 100, 50), 350);
        assert_eq!(net(0, 100, 2500), -2600);
        assert_eq!(net(u64::MAX, 0, 0), i64::MAX);
        assert_eq!(net(0, u64::MAX, u64::MAX), i64::MIN);
    }

    #[test]
    fn no_prs_splits_nothing() {
        let none: [Vec<&str>; 0] = [];
        assert!(split_shared_costs(2500, &none).is_empty());
        // A repo-wide PR touches no project, so it doesn't count either.
        assert!(split_shared_costs(2500, &[Vec::<&str>::new()]).is_empty());
    }

    #[test]
    fn one_project_takes_all() {
        let shares = split_shared_costs(2500, &[vec!["pane"], vec!["pane"]]);
        assert_eq!(shares, BTreeMap::from([("pane".to_string(), 2500)]));
    }

    #[test]
    fn a_pr_touching_two_projects_counts_half_to_each() {
        // pane: 1 + 1/2 = 3/2; cookbook: 1/2. Total 2 PRs.
        let shares = split_shared_costs(2000, &[vec!["pane"], vec!["pane", "cookbook"]]);
        assert_eq!(shares["pane"], 1500);
        assert_eq!(shares["cookbook"], 500);
    }

    #[test]
    fn a_project_listed_twice_in_one_pr_counts_once() {
        let shares = split_shared_costs(1000, &[vec!["pane", "pane", "cookbook"]]);
        assert_eq!(shares["pane"], 500);
        assert_eq!(shares["cookbook"], 500);
    }

    #[test]
    fn shares_always_add_up_with_leftover_cents_to_the_largest_remainder() {
        // Three equal projects: 100 / 3 = 33 rem 1, so one cent is left over
        // and goes to the name that sorts first.
        let shares = split_shared_costs(100, &[vec!["c"], vec!["b"], vec!["a"]]);
        assert_eq!(shares.values().sum::<u64>(), 100);
        assert_eq!((shares["a"], shares["b"], shares["c"]), (34, 33, 33));

        // Mixed PR sizes: a = 1 + 1/3, b = 1/2 + 1/3, c = 1/2 + 1/3, total 3.
        // 2500 * 8/18 = 1111 rem 2/18, 2500 * 5/18 = 694 rem 8/18 each: one
        // cent is left over and goes to b, the first of the two largest.
        let prs = [vec!["a"], vec!["b", "c"], vec!["a", "b", "c"]];
        let shares = split_shared_costs(2500, &prs);
        assert_eq!(shares.values().sum::<u64>(), 2500);
        assert_eq!((shares["a"], shares["b"], shares["c"]), (1111, 695, 694));
    }

    #[test]
    fn zero_shared_cost_gives_zero_shares() {
        let shares = split_shared_costs(0, &[vec!["pane"], vec!["cookbook"]]);
        assert_eq!(shares.values().sum::<u64>(), 0);
        assert_eq!(shares.len(), 2);
    }

    #[test]
    fn project_without_cost_file_has_only_its_shared_share() {
        let m = project_money("homer", nov(), None, 1250);
        assert_eq!(
            m,
            ProjectMoney {
                project: "homer".into(),
                month: "2026-11".into(),
                revenue_cents: 0,
                direct_cost_cents: 0,
                shared_cost_cents: 1250,
                net_cents: -1250,
            }
        );
    }

    #[test]
    fn month_money_puts_it_together() {
        let pane = file(MONEY);
        let cookbook = file("");
        let rows = month_money(
            nov(),
            &[
                ("pane", Some(&pane)),
                ("cookbook", Some(&cookbook)),
                ("homer", None),
            ],
            2500,
            &[vec!["pane"], vec!["pane", "cookbook"]],
        );
        let by: Vec<(&str, u64, u64, u64, i64)> = rows
            .iter()
            .map(|m| {
                (
                    m.project.as_str(),
                    m.revenue_cents,
                    m.direct_cost_cents,
                    m.shared_cost_cents,
                    m.net_cents,
                )
            })
            .collect();
        assert_eq!(
            by,
            [
                ("pane", 500, 2600, 1875, 500 - 2600 - 1875),
                ("cookbook", 0, 0, 625, -625),
                ("homer", 0, 0, 0, 0),
            ]
        );
        assert!(rows.iter().all(|m| m.month == "2026-11"));
    }

    #[test]
    fn project_money_round_trips_through_json() -> Result<(), serde_json::Error> {
        let m = project_money("homer", nov(), None, 10);
        let back: ProjectMoney = serde_json::from_str(&serde_json::to_string(&m)?)?;
        assert_eq!(back, m);
        Ok(())
    }
}
