//! The financial ledger and runway (decision 0012, revenue phase 1).
//!
//! Every month from the start month in `standards/money/runway.toml` gets a
//! statement: where the money stood at the start, one line for each money
//! movement, and where it stood at the end. Each line moves whole cents
//! from one account to another, so every statement is double-entry
//! (decision 0016, "Accounting rules" in `standards/docs/cost.md`).
//!
//! Only **actual** money is booked (Jim, 2026-11-28: "A budget is not actual
//! spend. Our accounting should only count actual spend."):
//!
//! - revenue (`[[revenue]]` in a project's `docs/cost.toml`) moves from
//!   `income:<stream>` to [`BANK`], where the stream is the start of its
//!   `source` (`sponsors:`, `x402:`, `royalty:`, `gift:`);
//! - spend (`[[spend]]` in `standards/money/actuals.toml`, each with the
//!   receipt or invoice it comes from) moves to `expense:<vendor>`, from
//!   [`BANK`] when the project's account paid it, or from [`OWED_JIM`] when
//!   Jim paid it ("everything will be paid by me ... so I can get paid back
//!   eventually"); spend for one project is booked to it, shared spend is
//!   split by the month's merged PRs ([`split_shared_costs`]);
//! - Jim's salary, 1% of the month's revenue with a ceiling of $1,000,000 a
//!   calendar year, moves from [`OWED_SALARY`] to [`SALARY`];
//! - 1% for the Planet, 1% of the month's revenue, moves from
//!   [`OWED_PLANET`] to [`PLANET`];
//! - a payment (`[[payment]]` in `actuals.toml`) pays one of those
//!   liabilities back from [`BANK`].
//!
//! Budgets in the radar and expected `[[cost]]` entries are never booked.
//! The runway comes from the statements: the net position (bank less what
//! is owed), the burn (average net cost of the last three months) and a
//! status of `funded`, `low` or `out`. No I/O, and money is whole cents.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Month, Project};

/// Where the ledger's settings live, relative to the repo root.
pub const RUNWAY_FILE: &str = "standards/money/runway.toml";

/// Where actual spend and payments are entered, relative to the repo root.
pub const ACTUALS_FILE: &str = "standards/money/actuals.toml";

/// The project's own bank account.
pub const BANK: &str = "asset:bank";

/// Spend Jim paid himself, owed back to him.
pub const OWED_JIM: &str = "liability:owed-to-jim:expenses";

/// Cash Jim lent the project (a `[[revenue]]` whose source starts
/// `owner:`), owed back to him. Never counted as revenue.
pub const OWED_LOANS: &str = "liability:owed-to-jim:loans";

/// Jim's salary, accrued and not yet paid.
pub const OWED_SALARY: &str = "liability:owed-to-jim:salary";

/// 1% for the Planet, accrued and not yet paid.
pub const OWED_PLANET: &str = "liability:one-percent-for-the-planet";

/// The salary expense.
pub const SALARY: &str = "expense:salary";

/// The 1% for the Planet expense.
pub const PLANET: &str = "expense:one-percent-for-the-planet";

/// Jim's salary and 1% for the Planet are each this many hundredths of
/// the month's revenue (1%).
pub const SHARE_PERCENT: u64 = 1;

/// The most salary booked in one calendar year: $1,000,000.
pub const SALARY_CAP_CENTS: u64 = 100_000_000;

/// The project that carries repo-wide work and any shared spend no merged
/// PR can carry, so every cent lands on some project (decision 0016: full
/// attribution, no unattributed remainder).
pub const REPO_WIDE: &str = "standards";

/// `standards/money/runway.toml`: when the ledger starts and what counts as
/// low. Jim changes these in a PR.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunwayFile {
    /// The first month on the ledger.
    pub start: Month,
    /// Money in the project's bank account at the start of `start`, in
    /// whole cents.
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

/// Who paid a `[[spend]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaidBy {
    /// Jim, from his own money; owed back to him.
    Jim,
    /// The project's own bank account.
    Bank,
}

/// Money actually spent, from a receipt, invoice or billing page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spend {
    /// The month the spend serves (accrual: a yearly charge is entered as
    /// twelve monthly entries).
    pub month: Month,
    /// A vendor blip in `radar.json`, such as `Claude Code`.
    pub vendor: String,
    pub cents: u64,
    pub paid_by: PaidBy,
    /// The receipt, invoice or billing page the number comes from.
    pub source: String,
    /// The project it was for. Without it, the spend is shared and split by
    /// the month's merged PRs.
    #[serde(default)]
    pub project: Option<String>,
}

/// Who a `[[payment]]` paid back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Payee {
    /// Spend Jim paid.
    JimExpenses,
    /// Cash Jim lent.
    JimLoans,
    /// Jim's salary.
    JimSalary,
    /// 1% for the Planet.
    Planet,
}

impl Payee {
    /// The liability the payment reduces.
    #[must_use]
    pub fn account(self) -> &'static str {
        match self {
            Payee::JimExpenses => OWED_JIM,
            Payee::JimLoans => OWED_LOANS,
            Payee::JimSalary => OWED_SALARY,
            Payee::Planet => OWED_PLANET,
        }
    }
}

/// Money actually paid out of the bank against a liability.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payment {
    pub month: Month,
    pub payee: Payee,
    pub cents: u64,
    /// The bank record the number comes from.
    pub source: String,
}

/// `standards/money/actuals.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actuals {
    #[serde(default)]
    pub spend: Vec<Spend>,
    #[serde(default)]
    pub payment: Vec<Payment>,
}

/// Parses `standards/money/actuals.toml`.
///
/// # Errors
///
/// A one-line message when the TOML doesn't parse or has an unknown field
/// or value.
pub fn parse_actuals(text: &str) -> Result<Actuals, String> {
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

/// The month's revenue and expenses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub income_cents: u64,
    pub expense_cents: u64,
}

impl Totals {
    /// Income less expenses.
    #[must_use]
    pub fn net_cents(&self) -> i64 {
        cents_i64(self.income_cents) - cents_i64(self.expense_cents)
    }
}

/// Where the money stands: the bank and what is owed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Position {
    pub bank_cents: i64,
    /// Spend Jim paid, cash he lent and salary not yet paid him.
    pub owed_to_jim_cents: i64,
    pub owed_to_planet_cents: i64,
    /// The bank less everything owed.
    pub net_cents: i64,
}

impl Position {
    fn opening(bank_cents: i64) -> Self {
        Self {
            bank_cents,
            net_cents: bank_cents,
            ..Self::default()
        }
    }

    /// The position after `lines`.
    #[must_use]
    pub fn after(mut self, lines: &[Line]) -> Self {
        for line in lines {
            let cents = cents_i64(line.cents);
            self.apply(&line.from, -cents);
            self.apply(&line.to, cents);
        }
        self.net_cents = self.bank_cents - self.owed_to_jim_cents - self.owed_to_planet_cents;
        self
    }

    /// Money entering (`cents > 0`) or leaving an account. A liability grows
    /// when money leaves it, as a credit does.
    fn apply(&mut self, account: &str, cents: i64) {
        if account == BANK {
            self.bank_cents += cents;
        } else if account == OWED_PLANET {
            self.owed_to_planet_cents -= cents;
        } else if account.starts_with("liability:owed-to-jim:") {
            self.owed_to_jim_cents -= cents;
        }
    }
}

/// One month of the ledger, written to
/// `standards/money/statements/<yyyy-mm>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Statement {
    pub month: Month,
    pub opening: Position,
    pub lines: Vec<Line>,
    pub totals: Totals,
    pub closing: Position,
}

/// What the ledger is built from.
#[derive(Debug, Clone, Default)]
pub struct Books {
    /// Every project whose `docs/cost.toml` parsed, for its `[[revenue]]`.
    pub projects: Vec<Project>,
    /// `standards/money/actuals.toml`.
    pub actuals: Actuals,
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

/// The account a `[[revenue]]` entry comes from: [`OWED_LOANS`] for an
/// `owner:` source (Jim's cash, owed back), else `income:<stream>`
/// (`income:other` when the source names no stream).
#[must_use]
pub fn revenue_account(source: &str) -> String {
    match source.split_once(':') {
        Some(("owner", _)) => OWED_LOANS.to_string(),
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

/// Each project's revenue lines for `month`.
fn revenue_lines(books: &Books, month: Month) -> Vec<Line> {
    let mut lines = Vec::new();
    for project in &books.projects {
        for revenue in project.file.revenue.iter().filter(|r| r.month == month) {
            if revenue.cents > 0 {
                lines.push(Line {
                    project: project.name.clone(),
                    from: revenue_account(&revenue.source),
                    to: BANK.to_string(),
                    cents: revenue.cents,
                    source: format!("{}/docs/cost.toml: {}", project.name, revenue.source),
                });
            }
        }
    }
    lines
}

/// The month's `[[spend]]`, booked to its project or split across projects.
fn spend_lines(books: &Books, month: Month) -> Vec<Line> {
    let no_prs = Vec::new();
    let prs = books.merged.get(&month).unwrap_or(&no_prs);
    let mut lines = Vec::new();
    for spend in books.actuals.spend.iter().filter(|s| s.month == month) {
        let from = match spend.paid_by {
            PaidBy::Jim => OWED_JIM,
            PaidBy::Bank => BANK,
        };
        let line = |project: String, cents: u64, source: String| Line {
            project,
            from: from.to_string(),
            to: expense_account(&spend.vendor),
            cents,
            source,
        };
        let source = format!("{ACTUALS_FILE}: {}", spend.source);
        match &spend.project {
            Some(project) if spend.cents > 0 => {
                lines.push(line(project.clone(), spend.cents, source));
            }
            Some(_) => {}
            None => {
                let source = format!("{source}, split by {} merged PR(s)", prs.len());
                for (project, share) in split_shared_costs(spend.cents, prs) {
                    if share > 0 {
                        lines.push(line(project, share, source.clone()));
                    }
                }
            }
        }
    }
    lines
}

/// Jim's salary and 1% for the Planet on the month's revenue, split across
/// projects by their revenue. `salary_left` is what the year's ceiling
/// still allows, and goes down by the salary booked.
fn share_lines(revenue: &[Line], salary_left: &mut u64) -> Vec<Line> {
    let mut by_project: BTreeMap<&str, u128> = BTreeMap::new();
    for line in revenue.iter().filter(|l| l.from.starts_with("income:")) {
        *by_project.entry(&line.project).or_default() += u128::from(line.cents);
    }
    let income: u64 = by_project
        .values()
        .map(|&c| u64::try_from(c).unwrap_or(u64::MAX))
        .fold(0, u64::saturating_add);
    if income == 0 {
        return Vec::new();
    }
    // The salary rounds down and the Planet's share rounds up, so neither
    // takes less than its rule gives the other side.
    let salary = (income * SHARE_PERCENT / 100).min(*salary_left);
    *salary_left -= salary;
    let planet = (income * SHARE_PERCENT).div_ceil(100);
    let mut lines = Vec::new();
    let rules = [
        (
            salary,
            OWED_SALARY,
            SALARY,
            "1% of revenue as Jim's salary, up to $1,000,000 a year",
        ),
        (
            planet,
            OWED_PLANET,
            PLANET,
            "1% of revenue for 1% for the Planet",
        ),
    ];
    for (cents, from, to, source) in rules {
        for (project, share) in largest_remainder(u128::from(cents), &by_project) {
            if share > 0 {
                lines.push(Line {
                    project,
                    from: from.to_string(),
                    to: to.to_string(),
                    cents: share,
                    source: source.to_string(),
                });
            }
        }
    }
    lines
}

/// The month's `[[payment]]`s.
fn payment_lines(books: &Books, month: Month) -> Vec<Line> {
    books
        .actuals
        .payment
        .iter()
        .filter(|p| p.month == month && p.cents > 0)
        .map(|p| Line {
            project: REPO_WIDE.to_string(),
            from: BANK.to_string(),
            to: p.payee.account().to_string(),
            cents: p.cents,
            source: format!("{ACTUALS_FILE}: {}", p.source),
        })
        .collect()
}

/// Every line for `month`: revenue, spend, salary and 1% for the Planet,
/// then payments. `salary_left` is what the year's salary ceiling still
/// allows.
#[must_use]
pub fn month_lines(books: &Books, month: Month, salary_left: &mut u64) -> Vec<Line> {
    let mut lines = revenue_lines(books, month);
    let shares = share_lines(&lines, salary_left);
    lines.extend(spend_lines(books, month));
    lines.extend(shares);
    lines.extend(payment_lines(books, month));
    lines
}

/// The month's totals, from its lines.
#[must_use]
pub fn totals(lines: &[Line]) -> Totals {
    let mut totals = Totals::default();
    for line in lines {
        if line.from.starts_with("income:") {
            totals.income_cents = totals.income_cents.saturating_add(line.cents);
        }
        if line.to.starts_with("expense:") {
            totals.expense_cents = totals.expense_cents.saturating_add(line.cents);
        }
    }
    totals
}

/// Every month's statement from `runway.start` through `through`, oldest
/// first, each opening where the one before closed. Empty when `through`
/// is before the start.
#[must_use]
pub fn ledger(runway: &RunwayFile, books: &Books, through: Month) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut opening = Position::opening(runway.opening_cents);
    let mut month = runway.start;
    let mut salary_left = SALARY_CAP_CENTS;
    while month <= through {
        if month.month == 1 {
            salary_left = SALARY_CAP_CENTS;
        }
        let lines = month_lines(books, month, &mut salary_left);
        let closing = opening.after(&lines);
        statements.push(Statement {
            month,
            opening,
            totals: totals(&lines),
            lines,
            closing,
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
    /// The net position is below zero: the project owes more than it has.
    Out,
}

/// `runway.json`: where the money stands after the last statement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Runway {
    /// The last month counted; the start month when there are none yet.
    pub month: Month,
    pub position: Position,
    /// The average net cost a month over the last three months (fewer when
    /// the ledger is younger), rounded toward zero. Zero or less means the
    /// money isn't running down.
    pub burn_cents: i64,
    /// `net / burn`, to one decimal. `None` when the burn is zero or less
    /// (the runway never ends); `0` when the net position is below zero.
    pub runway_months: Option<f64>,
    pub low_months: u32,
    pub status: Status,
}

/// The runway after `statements` (from [`ledger`]).
#[must_use]
pub fn runway(file: &RunwayFile, statements: &[Statement]) -> Runway {
    let (month, position) = statements
        .last()
        .map_or((file.start, Position::opening(file.opening_cents)), |s| {
            (s.month, s.closing)
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
    let net = position.net_cents;
    let (status, runway_months) = if net < 0 {
        (Status::Out, Some(0.0))
    } else if burn <= 0 {
        (Status::Funded, None)
    } else if net >= burn.saturating_mul(i64::from(file.low_months)) {
        (Status::Funded, Some(tenths(net, burn)))
    } else {
        (Status::Low, Some(tenths(net, burn)))
    };
    Runway {
        month,
        position,
        burn_cents: burn,
        runway_months,
        low_months: file.low_months,
        status,
    }
}

/// `net / burn` rounded down to one decimal, for display only; the status
/// is worked out in whole cents.
#[allow(clippy::cast_precision_loss, reason = "a display value, not money")]
fn tenths(net: i64, burn: i64) -> f64 {
    (net.saturating_mul(10) / burn) as f64 / 10.0
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
    if sum == 0 {
        return BTreeMap::new();
    }
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

    fn revenue(name: &str, month: &str, cents: u64, source: &str) -> Result<Project, String> {
        project(
            name,
            &format!("[[revenue]]\nmonth = \"{month}\"\ncents = {cents}\nsource = \"{source}\"\n"),
        )
    }

    fn file(start: Month, opening_cents: i64) -> RunwayFile {
        RunwayFile {
            start,
            opening_cents,
            low_months: 3,
            notes: None,
        }
    }

    fn spend(month: Month, cents: u64, paid_by: PaidBy, project: Option<&str>) -> Spend {
        Spend {
            month,
            vendor: "Claude Code".to_string(),
            cents,
            paid_by,
            source: "invoice 1".to_string(),
            project: project.map(str::to_string),
        }
    }

    /// Jim paid $25 of Claude Code in November, shared (no merged PRs).
    fn jim_paid_claude() -> Books {
        Books {
            actuals: Actuals {
                spend: vec![spend(month(2026, 11), 2500, PaidBy::Jim, None)],
                payment: Vec::new(),
            },
            ..Books::default()
        }
    }

    /// Every line moves cents between two accounts, so the closing position
    /// is the opening one moved by income less expenses.
    fn balances(s: &Statement) -> bool {
        s.closing.net_cents == s.opening.net_cents + s.totals.net_cents()
            && s.closing.net_cents
                == s.closing.bank_cents
                    - s.closing.owed_to_jim_cents
                    - s.closing.owed_to_planet_cents
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
    fn test_parses_actuals_and_rejects_unknown_payers() -> TestResult {
        let actuals = parse_actuals(
            "[[spend]]\nmonth = \"2026-11\"\nvendor = \"Claude Code\"\ncents = 2500\npaid_by = \"jim\"\nsource = \"invoice\"\n\n[[payment]]\nmonth = \"2027-01\"\npayee = \"jim-expenses\"\ncents = 100\nsource = \"bank\"\n",
        )?;
        assert_eq!(actuals.spend[0].paid_by, PaidBy::Jim);
        assert_eq!(actuals.spend[0].project, None);
        assert_eq!(actuals.payment[0].payee, Payee::JimExpenses);
        assert!(parse_actuals("[[spend]]\nmonth = \"2026-11\"\nvendor = \"x\"\ncents = 1\npaid_by = \"budget\"\nsource = \"x\"\n").is_err());
        assert_eq!(parse_actuals("")?, Actuals::default());
        Ok(())
    }

    #[test]
    fn test_accounts_name_the_stream_and_vendor() {
        assert_eq!(expense_account("Claude Code"), "expense:claude-code");
        assert_eq!(
            revenue_account("sponsors:2026-11 payout"),
            "income:sponsors"
        );
        assert_eq!(revenue_account("owner:jim"), OWED_LOANS);
        assert_eq!(revenue_account("None yet"), "income:other");
    }

    #[test]
    fn test_no_spend_entered_books_nothing() {
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &Books::default(), month(2026, 11));
        assert!(
            statements[0].lines.is_empty(),
            "no budget or estimate is booked"
        );
        let r = runway(&file, &statements);
        assert_eq!((r.burn_cents, r.status), (0, Status::Funded));
    }

    #[test]
    fn test_spend_jim_paid_is_owed_back_to_him() {
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &jim_paid_claude(), month(2026, 11));
        let s = &statements[0];
        assert!(balances(s));
        assert_eq!(s.lines[0].from, OWED_JIM);
        assert_eq!(s.lines[0].to, "expense:claude-code");
        assert_eq!(s.lines[0].project, REPO_WIDE);
        assert_eq!(
            s.closing.bank_cents, 0,
            "Jim's money never touched the bank"
        );
        assert_eq!(s.closing.owed_to_jim_cents, 2500);
        assert_eq!(s.closing.net_cents, -2500);
        let r = runway(&file, &statements);
        assert_eq!(r.status, Status::Out);
        assert_eq!(r.burn_cents, 2500);
        assert_eq!(r.runway_months, Some(0.0));
    }

    #[test]
    fn test_paying_jim_back_clears_what_is_owed() {
        let mut books = jim_paid_claude();
        books.actuals.payment.push(Payment {
            month: month(2026, 12),
            payee: Payee::JimExpenses,
            cents: 2500,
            source: "transfer".to_string(),
        });
        let file = file(month(2026, 11), 5000);
        let statements = ledger(&file, &books, month(2026, 12));
        let dec = &statements[1];
        assert!(balances(dec));
        assert_eq!(
            dec.totals,
            Totals::default(),
            "a repayment is neither income nor expense"
        );
        assert_eq!(dec.closing.bank_cents, 2500);
        assert_eq!(dec.closing.owed_to_jim_cents, 0);
        assert_eq!(dec.closing.net_cents, 2500);
    }

    #[test]
    fn test_revenue_accrues_salary_and_planet_shares() -> TestResult {
        let mut books = Books::default();
        books
            .projects
            .push(revenue("cookbook", "2026-11", 1050, "sponsors:payout")?);
        books
            .projects
            .push(revenue("pane", "2026-11", 2000, "royalty:acme")?);
        let file = file(month(2026, 11), 0);
        let s = &ledger(&file, &books, month(2026, 11))[0];
        assert!(balances(s));
        assert_eq!(s.totals.income_cents, 3050);
        let sum =
            |to: &str| -> u64 { s.lines.iter().filter(|l| l.to == to).map(|l| l.cents).sum() };
        // 1% of $30.50: the salary rounds down, the Planet's share up.
        assert_eq!(sum(SALARY), 30);
        assert_eq!(sum(PLANET), 31);
        assert_eq!(s.closing.owed_to_jim_cents, 30);
        assert_eq!(s.closing.owed_to_planet_cents, 31);
        assert_eq!(s.closing.bank_cents, 3050);
        let pane_salary: u64 = s
            .lines
            .iter()
            .filter(|l| l.to == SALARY && l.project == "pane")
            .map(|l| l.cents)
            .sum();
        assert_eq!(pane_salary, 20, "split by each project's revenue");
        Ok(())
    }

    #[test]
    fn test_owner_cash_is_a_loan_not_revenue() -> TestResult {
        let books = Books {
            projects: vec![revenue("standards", "2026-11", 5000, "owner:jim")?],
            ..Books::default()
        };
        let file = file(month(2026, 11), 0);
        let s = &ledger(&file, &books, month(2026, 11))[0];
        assert_eq!(s.totals.income_cents, 0);
        assert_eq!(s.lines.len(), 1, "no salary or Planet share on a loan");
        assert_eq!(s.closing.bank_cents, 5000);
        assert_eq!(s.closing.owed_to_jim_cents, 5000);
        assert_eq!(s.closing.net_cents, 0);
        Ok(())
    }

    #[test]
    fn test_salary_stops_at_the_yearly_ceiling_and_resets_in_january() -> TestResult {
        // $60M of revenue in each of November and December, and in January.
        let books = Books {
            projects: vec![
                revenue("a", "2026-11", 6_000_000_000, "x402:api")?,
                revenue("b", "2026-12", 6_000_000_000, "x402:api")?,
                revenue("c", "2027-01", 6_000_000_000, "x402:api")?,
            ],
            ..Books::default()
        };
        let file = file(month(2026, 11), 0);
        let statements = ledger(&file, &books, month(2027, 1));
        let salary = |s: &Statement| -> u64 {
            s.lines
                .iter()
                .filter(|l| l.to == SALARY)
                .map(|l| l.cents)
                .sum()
        };
        assert_eq!(salary(&statements[0]), 60_000_000);
        assert_eq!(salary(&statements[1]), 40_000_000, "the $1M ceiling");
        assert_eq!(salary(&statements[2]), 60_000_000);
        let planet: u64 = statements[1]
            .lines
            .iter()
            .filter(|l| l.to == PLANET)
            .map(|l| l.cents)
            .sum();
        assert_eq!(planet, 60_000_000, "the Planet's share has no ceiling");
        Ok(())
    }

    #[test]
    fn test_bank_paid_spend_for_one_project() {
        let books = Books {
            actuals: Actuals {
                spend: vec![spend(month(2026, 11), 1200, PaidBy::Bank, Some("cookbook"))],
                payment: Vec::new(),
            },
            ..Books::default()
        };
        let file = file(month(2026, 11), 10_000);
        let statements = ledger(&file, &books, month(2026, 11));
        let line = &statements[0].lines[0];
        assert_eq!(
            (line.project.as_str(), line.from.as_str()),
            ("cookbook", BANK)
        );
        assert_eq!(statements[0].closing.bank_cents, 8800);
    }

    #[test]
    fn test_shared_spend_splits_by_merged_prs() {
        let mut books = jim_paid_claude();
        books.merged.insert(
            month(2026, 11),
            vec![vec!["cookbook".to_string()], vec!["pane".to_string()]],
        );
        let file = file(month(2026, 11), 0);
        let s = &ledger(&file, &books, month(2026, 11))[0];
        let lines: Vec<(&str, u64)> = s
            .lines
            .iter()
            .map(|l| (l.project.as_str(), l.cents))
            .collect();
        assert_eq!(lines, vec![("cookbook", 1250), ("pane", 1250)]);
    }

    #[test]
    fn test_runway_under_the_threshold_is_low() {
        let file = file(month(2026, 11), 7500);
        let r = runway(&file, &ledger(&file, &jim_paid_claude(), month(2026, 11)));
        assert_eq!(r.position.net_cents, 5000);
        assert_eq!(r.runway_months, Some(2.0));
        assert_eq!(r.status, Status::Low);
    }

    #[test]
    fn test_runway_at_the_threshold_is_funded() {
        let file = file(month(2026, 11), 10_000);
        let r = runway(&file, &ledger(&file, &jim_paid_claude(), month(2026, 11)));
        assert_eq!(r.position.net_cents, 7500);
        assert_eq!(r.runway_months, Some(3.0));
        assert_eq!(r.status, Status::Funded);
    }

    #[test]
    fn test_negative_net_is_out_even_with_cash_in_the_bank() -> TestResult {
        let mut books = jim_paid_claude();
        books
            .projects
            .push(revenue("cookbook", "2026-11", 1000, "gift:x")?);
        let file = file(month(2026, 11), 0);
        let r = runway(&file, &ledger(&file, &books, month(2026, 11)));
        assert!(r.position.bank_cents > 0);
        assert!(r.position.net_cents < 0);
        assert_eq!(r.status, Status::Out);
        Ok(())
    }

    #[test]
    fn test_no_months_yet_uses_the_opening_balance() {
        let file = file(month(2026, 12), 4000);
        let statements = ledger(&file, &jim_paid_claude(), month(2026, 11));
        assert!(statements.is_empty());
        let r = runway(&file, &statements);
        assert_eq!(
            (r.month, r.position.net_cents, r.burn_cents),
            (month(2026, 12), 4000, 0)
        );
        assert_eq!(r.status, Status::Funded);
    }

    #[test]
    fn test_burn_averages_the_last_three_months() {
        let mut books = Books::default();
        for (m, cents) in [(8, 100), (9, 3000), (10, 2500), (11, 2500)] {
            books
                .actuals
                .spend
                .push(spend(month(2026, m), cents, PaidBy::Bank, Some("homer")));
        }
        let file = file(month(2026, 8), 100_000);
        let statements = ledger(&file, &books, month(2026, 11));
        assert_eq!(statements.len(), 4);
        assert_eq!(statements[1].opening, statements[0].closing);
        let r = runway(&file, &statements);
        assert_eq!(r.burn_cents, (2500 + 2500 + 3000) / 3);
        assert_eq!(r.position.bank_cents, 100_000 - 8100);
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
        // cookbook 1/8, pane 3/8, standards 2/8, homer 2/8; pane and cookbook
        // tie on the leftover cent and cookbook sorts first.
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
