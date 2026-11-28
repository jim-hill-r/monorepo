//! The financial ledger and runway (decision 0012, revenue phase 1).
//!
//! Every month from the start month in `standards/money/runway.toml` gets a
//! statement: its opening balance, one line for each money movement, and
//! its closing balance. Each line moves whole cents from one account to
//! another, so every statement is double-entry and balances to zero
//! (decision 0016, "Accounting rules" in `standards/docs/cost.md`):
//!
//! - revenue (`[[revenue]]` in a project's `docs/cost.toml`) moves from
//!   `income:<stream>` to [`BANK`], where the stream is the start of its
//!   `source` (`sponsors:`, `x402:`, `royalty:`, `gift:`);
//! - an owner contribution (a `[[revenue]]` whose source starts `owner:`)
//!   moves from [`OWNER`] to [`BANK`], so Jim's own money is never shown as
//!   revenue;
//! - a project's own cost moves from [`BANK`] to `expense:<vendor>` in the
//!   months it serves ([`monthly_cents`]);
//! - a shared cost (a radar vendor's monthly budget, today Claude Code's)
//!   moves from [`BANK`] to `expense:<vendor>` once for each project, split
//!   by the month's merged PRs ([`split_shared_costs`]), so every cent
//!   belongs to exactly one project.
//!
//! The runway comes from the statements: the balance, the burn (average net
//! cost of the last three months) and a status of `funded`, `low` or `out`.
//! No I/O, and money is whole cents throughout.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Month, Project, monthly_cents};

/// Where the ledger's settings live, relative to the repo root.
pub const RUNWAY_FILE: &str = "standards/money/runway.toml";

/// The cash account every line moves money into or out of.
pub const BANK: &str = "asset:bank";

/// Jim's own money put into the project (decision 0012, question 2).
pub const OWNER: &str = "equity:owner-contribution";

/// The project that carries repo-wide work and any shared cost no merged PR
/// can carry, so every cent lands on some project (decision 0016: full
/// attribution, no unattributed remainder).
pub const REPO_WIDE: &str = "standards";

/// `standards/money/runway.toml`: when the ledger starts and what counts as
/// low. Jim changes these in a PR.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunwayFile {
    /// The first month on the ledger.
    pub start: Month,
    /// Money in the bank at the start of `start`, in whole cents.
    pub opening_cents: i64,
    /// Status is `low` when the runway is under this many months of burn.
    #[serde(default = "default_low_months")]
    pub low_months: u32,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_low_months() -> u32 {
    3
}

/// Parses `standards/money/runway.toml`.
///
/// # Errors
///
/// A one-line message when the TOML doesn't parse or has an unknown field.
pub fn parse_runway_file(text: &str) -> Result<RunwayFile, String> {
    toml::from_str(text).map_err(|e| e.message().to_string())
}

/// One money movement: `cents` leave `from` and enter `to`, booked to one
/// project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Line {
    pub project: String,
    pub from: String,
    pub to: String,
    pub cents: u64,
    /// Where the number comes from, such as `cookbook/docs/cost.toml`.
    pub source: String,
}

/// The month's totals by kind of line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub income_cents: u64,
    pub owner_cents: u64,
    pub expense_cents: u64,
}

impl Totals {
    /// Income and owner contributions less expenses.
    #[must_use]
    pub fn net_cents(&self) -> i64 {
        cents_i64(self.income_cents) + cents_i64(self.owner_cents) - cents_i64(self.expense_cents)
    }
}

/// One month of the ledger, written to
/// `standards/money/statements/<yyyy-mm>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Statement {
    pub month: Month,
    pub opening_cents: i64,
    pub lines: Vec<Line>,
    pub totals: Totals,
    pub closing_cents: i64,
}

/// What the ledger is built from.
#[derive(Debug, Clone, Default)]
pub struct Books {
    /// Every project whose `docs/cost.toml` parsed.
    pub projects: Vec<Project>,
    /// Each shared vendor and its monthly budget in cents. The radar keeps
    /// no history, so every month uses today's budgets.
    pub shared: Vec<(String, u64)>,
    /// Per month, the projects each merged PR touched (see
    /// [`split_shared_costs`]).
    pub merged: BTreeMap<Month, Vec<Vec<String>>>,
}

/// `expense:<vendor>`, with the vendor lower-cased and spaces turned to
/// hyphens (`expense:claude-code`).
#[must_use]
pub fn expense_account(vendor: &str) -> String {
    format!("expense:{}", slug(vendor))
}

/// The account a `[[revenue]]` entry comes from: [`OWNER`] for an `owner:`
/// source, else `income:<stream>` (`income:other` when the source names no
/// stream).
#[must_use]
pub fn revenue_account(source: &str) -> String {
    match source.split_once(':') {
        Some(("owner", _)) => OWNER.to_string(),
        Some((stream, _)) if !stream.trim().is_empty() => format!("income:{}", slug(stream)),
        _ => "income:other".to_string(),
    }
}

fn slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn cents_i64(cents: u64) -> i64 {
    i64::try_from(cents).unwrap_or(i64::MAX)
}

/// Every line for `month`: each project's revenue and own costs, then each
/// shared cost split across projects.
#[must_use]
pub fn month_lines(books: &Books, month: Month) -> Vec<Line> {
    let mut lines = Vec::new();
    for project in &books.projects {
        let file = format!("{}/docs/cost.toml", project.name);
        for revenue in project.file.revenue.iter().filter(|r| r.month == month) {
            if revenue.cents > 0 {
                lines.push(Line {
                    project: project.name.clone(),
                    from: revenue_account(&revenue.source),
                    to: BANK.to_string(),
                    cents: revenue.cents,
                    source: format!("{file}: {}", revenue.source),
                });
            }
        }
        for cost in &project.file.cost {
            let cents = monthly_cents(cost, month);
            if cents > 0 {
                lines.push(Line {
                    project: project.name.clone(),
                    from: BANK.to_string(),
                    to: expense_account(&cost.vendor),
                    cents,
                    source: format!("{file}: {}", cost.name),
                });
            }
        }
    }
    let no_prs = Vec::new();
    let prs = books.merged.get(&month).unwrap_or(&no_prs);
    for (vendor, cents) in &books.shared {
        for (project, share) in split_shared_costs(*cents, prs) {
            if share > 0 {
                lines.push(Line {
                    project,
                    from: BANK.to_string(),
                    to: expense_account(vendor),
                    cents: share,
                    source: format!(
                        "standards/radar/radar.json: {vendor} budget, split by {} merged PR(s)",
                        prs.len()
                    ),
                });
            }
        }
    }
    lines
}

/// The month's totals, from its lines.
#[must_use]
pub fn totals(lines: &[Line]) -> Totals {
    let mut totals = Totals::default();
    for line in lines {
        let slot = if line.from == BANK {
            &mut totals.expense_cents
        } else if line.from == OWNER {
            &mut totals.owner_cents
        } else {
            &mut totals.income_cents
        };
        *slot = slot.saturating_add(line.cents);
    }
    totals
}

/// Every month's statement from `runway.start` through `through`, oldest
/// first, each opening where the one before closed. Empty when `through`
/// is before the start.
#[must_use]
pub fn ledger(runway: &RunwayFile, books: &Books, through: Month) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut opening = runway.opening_cents;
    let mut month = runway.start;
    while month <= through {
        let lines = month_lines(books, month);
        let totals = totals(&lines);
        let closing = opening.saturating_add(totals.net_cents());
        statements.push(Statement {
            month,
            opening_cents: opening,
            lines,
            totals,
            closing_cents: closing,
        });
        opening = closing;
        month = month.next();
    }
    statements
}

/// How long the money lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// At least `low_months` of runway, or nothing is being burned.
    Funded,
    /// Under `low_months` of runway.
    Low,
    /// The balance is below zero.
    Out,
}

/// `runway.json`: where the money stands after the last statement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Runway {
    /// The last month counted; the start month when there are none yet.
    pub month: Month,
    pub balance_cents: i64,
    /// The average net cost a month over the last three months (fewer when
    /// the ledger is younger), rounded toward zero. Zero or less means the
    /// money isn't running down.
    pub burn_cents: i64,
    /// `balance / burn`, to one decimal. `None` when the burn is zero or
    /// less (the runway never ends); `0` when the balance is below zero.
    pub runway_months: Option<f64>,
    pub low_months: u32,
    pub status: Status,
}

/// The runway after `statements` (from [`ledger`]).
#[must_use]
pub fn runway(file: &RunwayFile, statements: &[Statement]) -> Runway {
    let (month, balance) = statements
        .last()
        .map_or((file.start, file.opening_cents), |s| {
            (s.month, s.closing_cents)
        });
    let recent: Vec<i64> = statements
        .iter()
        .rev()
        .take(3)
        .map(|s| -s.totals.net_cents())
        .collect();
    let burn = if recent.is_empty() {
        0
    } else {
        recent.iter().sum::<i64>() / i64::try_from(recent.len()).unwrap_or(1)
    };
    let (status, runway_months) = if balance < 0 {
        (Status::Out, Some(0.0))
    } else if burn <= 0 {
        (Status::Funded, None)
    } else if balance >= burn.saturating_mul(i64::from(file.low_months)) {
        (Status::Funded, Some(tenths(balance, burn)))
    } else {
        (Status::Low, Some(tenths(balance, burn)))
    };
    Runway {
        month,
        balance_cents: balance,
        burn_cents: burn,
        runway_months,
        low_months: file.low_months,
        status,
    }
}

/// `balance / burn` rounded down to one decimal, for display only; the
/// status is worked out in whole cents.
#[allow(clippy::cast_precision_loss, reason = "a display value, not money")]
fn tenths(balance: i64, burn: i64) -> f64 {
    (balance.saturating_mul(10) / burn) as f64 / 10.0
}

/// Split `shared_cents` across projects by their share of the PRs merged in
/// the month.
///
/// `merged_prs` has one entry per merged PR: the projects it touched. A PR
/// touching `n` projects counts `1/n` to each; a project listed twice in one
/// PR counts once. A PR touching no project (a repo-wide change) counts to
/// [`REPO_WIDE`], and so does the whole cost in a month with no merged PRs.
/// Shares are whole cents that always add up to `shared_cents` (largest
/// remainder, ties to the project name that sorts first). The same rule as
/// pane's Money page.
pub fn split_shared_costs<S: AsRef<str>>(
    shared_cents: u64,
    merged_prs: &[Vec<S>],
) -> BTreeMap<String, u64> {
    if shared_cents == 0 {
        return BTreeMap::new();
    }
    let prs: Vec<Vec<&str>> = merged_prs
        .iter()
        .map(|pr| {
            let mut projects: Vec<&str> = pr.iter().map(AsRef::as_ref).collect();
            projects.sort_unstable();
            projects.dedup();
            if projects.is_empty() {
                projects.push(REPO_WIDE);
            }
            projects
        })
        .collect();
    if prs.is_empty() {
        return BTreeMap::from([(REPO_WIDE.to_string(), shared_cents)]);
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
    largest_remainder(u128::from(shared_cents), &weights)
}

/// Shares of `total` in proportion to `weights`, in whole units that add up
/// to `total`; the leftover units go to the largest remainders, ties to the
/// name that sorts first.
fn largest_remainder(total: u128, weights: &BTreeMap<&str, u128>) -> BTreeMap<String, u64> {
    let sum: u128 = weights.values().sum();
    let mut shares: Vec<(&str, u128, u128)> = weights
        .iter()
        .map(|(&name, &weight)| {
            let exact = total * weight;
            (name, exact / sum, exact % sum)
        })
        .collect();
    let given: u128 = shares.iter().map(|(_, units, _)| units).sum();
    let mut left = total - given;
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
        .map(|(name, units, _)| (name.to_string(), u64::try_from(units).unwrap_or(u64::MAX)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_cost_file;

    type TestResult = Result<(), String>;

    fn month(year: i32, month: u32) -> Month {
        Month { year, month }
    }

    fn project(name: &str, toml: &str) -> Result<Project, String> {
        let text = format!("reviewed = \"2026-11-01\"\nsummary = \"x\"\n{toml}");
        Ok(Project {
            name: name.to_string(),
            file: parse_cost_file(&text)?,
        })
    }

    fn file(start: Month, opening_cents: i64) -> RunwayFile {
        RunwayFile {
            start,
            opening_cents,
            low_months: 3,
            notes: None,
        }
    }

    /// Claude Code's $25 a month, charged to `standards` (no merged PRs).
    fn claude_only() -> Books {
        Books {
            shared: vec![("Claude Code".to_string(), 2500)],
            ..Books::default()
        }
    }

    fn balances(statement: &Statement) -> bool {
        let moved: i64 = statement
            .lines
            .iter()
            .map(|l| {
                let into_bank = i64::from(l.to == BANK) - i64::from(l.from == BANK);
                into_bank * cents_i64(l.cents)
            })
            .sum();
        statement.opening_cents + moved == statement.closing_cents
    }

    #[test]
    fn test_parses_the_runway_file_with_default_threshold() -> TestResult {
        let file = parse_runway_file("start = \"2026-11\"\nopening_cents = 0\n")?;
        assert_eq!(file.start, month(2026, 11));
        assert_eq!(file.low_months, 3);
        assert!(parse_runway_file("start = \"2026-11\"\nopening = 0\n").is_err());
        Ok(())
    }

    #[test]
    fn test_accounts_name_the_stream_and_vendor() {
        assert_eq!(expense_account("Claude Code"), "expense:claude-code");
        assert_eq!(
            revenue_account("sponsors:2026-11 payout"),
            "income:sponsors"
        );
        assert_eq!(revenue_account("owner:jim"), OWNER);
        assert_eq!(revenue_account("None yet"), "income:other");
    }

    #[test]
    fn test_no_revenue_runs_out_on_day_one() {
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &claude_only(), month(2026, 11));
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].closing_cents, -2500);
        let r = runway(&file, &statements);
        assert_eq!(r.status, Status::Out);
        assert_eq!(r.burn_cents, 2500);
        assert_eq!(r.runway_months, Some(0.0));
    }

    #[test]
    fn test_owner_contributions_keep_it_funded_with_zero_burn() -> TestResult {
        let mut books = claude_only();
        books.projects.push(project(
            "standards",
            "[[revenue]]\nmonth = \"2026-11\"\ncents = 2500\nsource = \"owner:jim\"\n",
        )?);
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &books, month(2026, 11));
        let s = &statements[0];
        assert_eq!(s.totals.owner_cents, 2500);
        assert_eq!(s.totals.income_cents, 0, "owner money is not revenue");
        assert_eq!(s.closing_cents, 0);
        let r = runway(&file, &statements);
        assert_eq!(r.burn_cents, 0);
        assert_eq!(r.status, Status::Funded);
        assert_eq!(r.runway_months, None);
        Ok(())
    }

    #[test]
    fn test_negative_balance_is_out_even_with_income() -> TestResult {
        let mut books = claude_only();
        books.projects.push(project(
            "cookbook",
            "[[revenue]]\nmonth = \"2026-11\"\ncents = 3000\nsource = \"sponsors:payout\"\n",
        )?);
        let file = file(month(2026, 11), -1000);
        let r = runway(&file, &ledger(&file, &books, month(2026, 11)));
        assert_eq!(r.balance_cents, -500);
        assert_eq!(r.burn_cents, -500);
        assert_eq!(r.status, Status::Out);
        Ok(())
    }

    #[test]
    fn test_runway_under_the_threshold_is_low() {
        let file = file(month(2026, 11), 7500);
        let r = runway(&file, &ledger(&file, &claude_only(), month(2026, 11)));
        assert_eq!(r.balance_cents, 5000);
        assert_eq!(r.runway_months, Some(2.0));
        assert_eq!(r.status, Status::Low);
    }

    #[test]
    fn test_runway_at_the_threshold_is_funded() {
        let file = file(month(2026, 11), 10_000);
        let r = runway(&file, &ledger(&file, &claude_only(), month(2026, 11)));
        assert_eq!(r.balance_cents, 7500);
        assert_eq!(r.runway_months, Some(3.0));
        assert_eq!(r.status, Status::Funded);
    }

    #[test]
    fn test_no_months_yet_uses_the_opening_balance() {
        let file = file(month(2026, 12), 4000);
        let statements = ledger(&file, &claude_only(), month(2026, 11));
        assert!(statements.is_empty());
        let r = runway(&file, &statements);
        assert_eq!(
            (r.month, r.balance_cents, r.burn_cents),
            (month(2026, 12), 4000, 0)
        );
        assert_eq!(r.status, Status::Funded);
    }

    #[test]
    fn test_burn_averages_the_last_three_months() -> TestResult {
        let mut books = claude_only();
        // A $30 one-off in September, then only Claude Code.
        books.projects.push(project(
            "homer",
            "[[cost]]\nname = \"x\"\nvendor = \"Squarespace\"\ncents = 3000\nevery = \"once\"\nsince = \"2026-09\"\ndecision = \"d.md\"\n",
        )?);
        let file = file(month(2026, 8), 100_000);
        let statements = ledger(&file, &books, month(2026, 11));
        assert_eq!(statements.len(), 4);
        assert_eq!(statements[1].opening_cents, statements[0].closing_cents);
        let r = runway(&file, &statements);
        // October, November and September: (2500 + 2500 + 5500) / 3.
        assert_eq!(r.burn_cents, 3500);
        assert_eq!(r.balance_cents, 100_000 - 4 * 2500 - 3000);
        Ok(())
    }

    #[test]
    fn test_every_statement_balances_and_books_yearly_costs_monthly() -> TestResult {
        let mut books = claude_only();
        books.projects.push(project(
            "cookbook",
            "[[cost]]\nname = \"domain\"\nvendor = \"Squarespace\"\ncents = 1200\nevery = \"year\"\nsince = \"2026-11\"\nrenews = \"2027-11-01\"\ndecision = \"d.md\"\n\n[[revenue]]\nmonth = \"2026-12\"\ncents = 700\nsource = \"royalty:acme\"\n",
        )?);
        books.merged.insert(
            month(2026, 12),
            vec![vec!["cookbook".to_string()], vec!["pane".to_string()]],
        );
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &books, month(2026, 12));
        for s in &statements {
            assert!(balances(s), "{} doesn't balance", s.month);
        }
        let dec = &statements[1];
        assert_eq!(dec.totals.income_cents, 700);
        assert_eq!(dec.totals.expense_cents, 100 + 2500);
        let lines: Vec<(&str, &str, u64)> = dec
            .lines
            .iter()
            .map(|l| (l.project.as_str(), l.to.as_str(), l.cents))
            .collect();
        assert_eq!(
            lines,
            vec![
                ("cookbook", BANK, 700),
                ("cookbook", "expense:squarespace", 100),
                ("cookbook", "expense:claude-code", 1250),
                ("pane", "expense:claude-code", 1250),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_split_adds_up_to_the_cent() {
        let prs = vec![
            vec!["cookbook", "pane"],
            vec!["pane"],
            vec![],
            vec!["homer", "homer"],
        ];
        let shares = split_shared_costs(2500, &prs);
        assert_eq!(shares.values().sum::<u64>(), 2500);
        // cookbook 1/8, pane 3/8, standards 2/8, homer 2/8.
        // pane and cookbook tie on the leftover cent; cookbook sorts first.
        assert_eq!(shares["pane"], 937);
        assert_eq!(shares["cookbook"], 313);
        assert_eq!(shares["homer"], 625);
        assert_eq!(shares["standards"], 625);
    }

    #[test]
    fn test_split_with_no_prs_goes_to_standards() {
        let none: Vec<Vec<&str>> = Vec::new();
        assert_eq!(
            split_shared_costs(2500, &none),
            BTreeMap::from([(REPO_WIDE.to_string(), 2500)])
        );
        assert!(split_shared_costs(0, &none).is_empty());
    }
}
