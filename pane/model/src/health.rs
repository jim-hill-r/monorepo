//! The health rules: one pure function per traffic-light signal, the
//! worst-of roll-up, and the thresholds they read from `pane/health.toml`.
//!
//! The rules come from "Health: the traffic lights" in the
//! [review dashboard plan](../../docs/plans/2026-11-review-dashboard.md).
//! Each function takes plain numbers (ages in whole days, counts, pass or
//! fail) that the caller has already worked out, so the rules need no clock,
//! no date library and no I/O, and every threshold edge is unit tested here.
//!
//! Every age rule reads "more than N days": a value equal to the threshold
//! is still on the healthier side. `None` input means "no data or does not
//! apply" and gives a grey light, which the roll-up leaves out of the colour
//! but shows in the coverage count.

use serde::{Deserialize, Serialize};

use crate::{Health, Signal};

/// Every threshold the health rules use, as written in `pane/health.toml`.
///
/// A missing table or key takes its default, which is the starting value
/// from the plan; an unknown key is an error, so a typo can't silently fall
/// back to the default.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    pub build: BuildThresholds,
    pub observability: ObservabilityThresholds,
    pub review: ReviewThresholds,
    pub retros: RetroThresholds,
    pub open_issues: OpenIssueThresholds,
    pub standards: StandardsThresholds,
    pub pull_requests: PullRequestThresholds,
    pub nightly: NightlyThresholds,
    pub pr_queue: PrQueueThresholds,
    pub human_queue: HumanQueueThresholds,
    pub agent_tasks: AgentTaskThresholds,
    pub review_rotation: ReviewRotationThresholds,
}

/// Project signal **Build**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BuildThresholds {
    /// Amber when the latest CI run on `main` is more than this many days old.
    pub stale_after_days: u32,
}

impl Default for BuildThresholds {
    fn default() -> Self {
        Self {
            stale_after_days: 2,
        }
    }
}

/// Project signal **Observability**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ObservabilityThresholds {
    /// Red when errors per day since the last deploy are more than this.
    pub red_errors_per_day: u32,
    /// Amber when errors per day rose by at least this percent after the last
    /// deploy.
    pub amber_rise_percent: u32,
}

impl Default for ObservabilityThresholds {
    fn default() -> Self {
        Self {
            red_errors_per_day: 50,
            amber_rise_percent: 50,
        }
    }
}

/// Project signal **Review**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReviewThresholds {
    /// Amber at this many areas rated Poor or more.
    pub amber_poor_areas: u32,
    /// Red at this many areas rated Poor or more.
    pub red_poor_areas: u32,
}

impl Default for ReviewThresholds {
    fn default() -> Self {
        Self {
            amber_poor_areas: 1,
            red_poor_areas: 3,
        }
    }
}

/// Project signal **Retros**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RetroThresholds {
    /// Amber when an action has been open more than this many days.
    pub amber_action_open_days: u32,
}

impl Default for RetroThresholds {
    fn default() -> Self {
        Self {
            amber_action_open_days: 30,
        }
    }
}

/// Project signal **Open issues**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpenIssueThresholds {
    /// Red when a `FIX` has been open more than this many days.
    pub red_fix_open_days: u32,
}

impl Default for OpenIssueThresholds {
    fn default() -> Self {
        Self {
            red_fix_open_days: 14,
        }
    }
}

/// Project signal **Standards**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StandardsThresholds {
    /// Amber at this many open `code-review` findings or more.
    pub amber_review_findings: u32,
}

impl Default for StandardsThresholds {
    fn default() -> Self {
        Self {
            amber_review_findings: 5,
        }
    }
}

/// Project signal **Pull requests**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PullRequestThresholds {
    /// Amber when a PR has been open more than this many days.
    pub amber_open_days: u32,
    /// Red when a PR with failing checks has been open more than this many
    /// days.
    pub red_failing_days: u32,
}

impl Default for PullRequestThresholds {
    fn default() -> Self {
        Self {
            amber_open_days: 7,
            red_failing_days: 3,
        }
    }
}

/// System signal **Nightly CI and CD**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NightlyThresholds {
    /// Amber when the latest run is more than this many days old ("didn't run
    /// last night").
    pub stale_after_days: u32,
}

impl Default for NightlyThresholds {
    fn default() -> Self {
        Self {
            stale_after_days: 1,
        }
    }
}

/// System signal **PR queue**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PrQueueThresholds {
    /// Amber when the oldest open PR is more than this many days old.
    pub amber_oldest_days: u32,
    /// Amber when more than this many PRs are open.
    pub amber_open_count: u32,
    /// Red when any PR is more than this many days old.
    pub red_oldest_days: u32,
}

impl Default for PrQueueThresholds {
    fn default() -> Self {
        Self {
            amber_oldest_days: 7,
            amber_open_count: 15,
            red_oldest_days: 30,
        }
    }
}

/// System signal **Human queue**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HumanQueueThresholds {
    /// Amber when any task is more than this many days old.
    pub amber_oldest_days: u32,
    /// Red when any task is more than this many days old.
    pub red_oldest_days: u32,
}

impl Default for HumanQueueThresholds {
    fn default() -> Self {
        Self {
            amber_oldest_days: 14,
            red_oldest_days: 30,
        }
    }
}

/// System signal **Agent tasks**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentTaskThresholds {
    /// Amber when PR triage has been quiet for more than this many days.
    pub amber_triage_quiet_days: u32,
    /// Red when every agent task has been quiet for more than this many days.
    pub red_quiet_days: u32,
}

impl Default for AgentTaskThresholds {
    fn default() -> Self {
        Self {
            amber_triage_quiet_days: 1,
            red_quiet_days: 3,
        }
    }
}

/// System signal **Review rotation**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReviewRotationThresholds {
    /// Amber when the newest review is more than this many days old.
    pub amber_after_days: u32,
    /// Red when the newest review is more than this many days old.
    pub red_after_days: u32,
}

impl Default for ReviewRotationThresholds {
    fn default() -> Self {
        Self {
            amber_after_days: 8,
            red_after_days: 15,
        }
    }
}

/// The light for a row of signals: the worst non-grey one, and how many of
/// them had data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rollup {
    pub health: Health,
    /// Signals that are not grey.
    pub covered: usize,
    /// All signals in the row.
    pub total: usize,
}

impl Rollup {
    /// Coverage as pane shows it, such as `5 of 7 signals`.
    pub fn coverage(&self) -> String {
        format!("{} of {} signals", self.covered, self.total)
    }
}

/// The worst of the non-grey signals, and the coverage count. A row with
/// no signals, or only grey ones, is grey.
pub fn roll_up(signals: &[Signal]) -> Rollup {
    let covered = signals.iter().filter(|s| s.health != Health::Grey).count();
    let health = signals
        .iter()
        .map(|s| s.health)
        .max()
        .unwrap_or(Health::Grey);
    Rollup {
        health,
        covered,
        total: signals.len(),
    }
}

fn signal(name: &str, health: Health, reason: impl Into<String>) -> Signal {
    Signal {
        name: name.into(),
        health,
        reason: reason.into(),
        evidence: Vec::new(),
    }
}

fn days(n: u32) -> String {
    if n == 1 {
        "1 day".into()
    } else {
        format!("{n} days")
    }
}

/// The outcome and age of one workflow run or job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub passed: bool,
    /// Whole days since it finished.
    pub age_days: u32,
}

/// **Build**: the latest `mise run ci` job for the project on `main`.
pub fn build(latest: Option<Run>, t: &BuildThresholds) -> Signal {
    const NAME: &str = "Build";
    match latest {
        None => signal(NAME, Health::Grey, "No CI run on main found"),
        Some(run) if !run.passed => signal(NAME, Health::Red, "Latest CI run on main failed"),
        Some(run) if run.age_days > t.stale_after_days => signal(
            NAME,
            Health::Amber,
            format!("No CI run on main in {}", days(t.stale_after_days)),
        ),
        Some(run) => signal(
            NAME,
            Health::Green,
            format!("Passed {} ago", days(run.age_days)),
        ),
    }
}

/// What is known about a project's latest production deploy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deploy {
    pub succeeded: bool,
    /// The smoke run after it: `None` when the project has no smoke test yet.
    pub smoke_passed: Option<bool>,
    /// The project changed on `main` after this deploy.
    pub behind_main: bool,
}

/// **Deploy and UAT**: `None` for a project that doesn't deploy.
pub fn deploy_and_uat(latest: Option<Deploy>) -> Signal {
    const NAME: &str = "Deploy and UAT";
    let Some(d) = latest else {
        return signal(NAME, Health::Grey, "Not deployed");
    };
    if !d.succeeded {
        return signal(NAME, Health::Red, "Latest deploy failed");
    }
    match d.smoke_passed {
        Some(false) => signal(
            NAME,
            Health::Red,
            "Smoke run failed after the latest deploy",
        ),
        None => signal(NAME, Health::Amber, "No smoke test yet"),
        Some(true) if d.behind_main => signal(
            NAME,
            Health::Amber,
            "Latest deploy is older than the last change",
        ),
        Some(true) => signal(NAME, Health::Green, "Deployed and smoke passed"),
    }
}

/// Error counts for a production app around its last deploy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Errors {
    /// Average errors per day before the last deploy.
    pub per_day_before: u32,
    /// Average errors per day since the last deploy.
    pub per_day_since: u32,
    pub site_up: bool,
}

/// **Observability**: `None` for a project that is not a production app, or
/// has no telemetry yet.
pub fn observability(errors: Option<Errors>, t: &ObservabilityThresholds) -> Signal {
    const NAME: &str = "Observability";
    let Some(e) = errors else {
        return signal(NAME, Health::Grey, "No telemetry");
    };
    if !e.site_up {
        return signal(NAME, Health::Red, "Site is down");
    }
    if e.per_day_since > t.red_errors_per_day {
        return signal(
            NAME,
            Health::Red,
            format!(
                "{} errors a day, over {}",
                e.per_day_since, t.red_errors_per_day
            ),
        );
    }
    // Compare in u64 so large counts can't overflow.
    let since = u64::from(e.per_day_since) * 100;
    let limit = u64::from(e.per_day_before) * (100 + u64::from(t.amber_rise_percent));
    if e.per_day_since > e.per_day_before && since >= limit {
        return signal(
            NAME,
            Health::Amber,
            format!(
                "Errors up from {} to {} a day after the last deploy",
                e.per_day_before, e.per_day_since
            ),
        );
    }
    signal(
        NAME,
        Health::Green,
        format!("{} errors a day", e.per_day_since),
    )
}

/// The latest executive review deck for a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewStatus {
    pub age_days: u32,
    /// Scorecard areas rated Poor.
    pub poor_areas: u32,
    /// Length of one full review rotation in days (weekly reviews times the
    /// number of projects in the rotation).
    pub rotation_days: u32,
}

/// **Review**: `None` for a project that has never been reviewed.
pub fn review(latest: Option<ReviewStatus>, t: &ReviewThresholds) -> Signal {
    const NAME: &str = "Review";
    let Some(r) = latest else {
        return signal(NAME, Health::Grey, "Never reviewed");
    };
    if r.poor_areas >= t.red_poor_areas {
        return signal(
            NAME,
            Health::Red,
            format!("{} areas rated Poor", r.poor_areas),
        );
    }
    if r.poor_areas >= t.amber_poor_areas {
        let areas = if r.poor_areas == 1 {
            "1 area".into()
        } else {
            format!("{} areas", r.poor_areas)
        };
        return signal(NAME, Health::Amber, format!("{areas} rated Poor"));
    }
    if r.age_days > r.rotation_days {
        return signal(
            NAME,
            Health::Amber,
            format!(
                "Reviewed {} ago, longer than one rotation",
                days(r.age_days)
            ),
        );
    }
    signal(
        NAME,
        Health::Green,
        format!("Reviewed {} ago, no area Poor", days(r.age_days)),
    )
}

/// **Retros**: the ages in days of the latest retro's open actions, or
/// `None` when the project has no retro yet. Retros are never red.
pub fn retros(open_action_ages: Option<&[u32]>, t: &RetroThresholds) -> Signal {
    const NAME: &str = "Retros";
    let Some(ages) = open_action_ages else {
        return signal(NAME, Health::Grey, "No retro yet");
    };
    let old = ages
        .iter()
        .filter(|&&a| a > t.amber_action_open_days)
        .count();
    if old > 0 {
        return signal(
            NAME,
            Health::Amber,
            format!(
                "{old} action(s) open more than {}",
                days(t.amber_action_open_days)
            ),
        );
    }
    if ages.is_empty() {
        signal(NAME, Health::Green, "All actions done")
    } else {
        signal(
            NAME,
            Health::Green,
            format!("{} action(s) open, none overdue", ages.len()),
        )
    }
}

/// One open `FIX` for a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fix {
    /// Days since the line was added (`git blame`), when known.
    pub age_days: Option<u32>,
    /// It was filed by a failed production smoke run.
    pub from_smoke: bool,
}

/// **Open issues**: the project's open `FIX` items and how many human
/// issues block it. Always has data, so never grey.
pub fn open_issues(fixes: &[Fix], blocking_human_issues: usize, t: &OpenIssueThresholds) -> Signal {
    const NAME: &str = "Open issues";
    if fixes.iter().any(|f| f.from_smoke) {
        return signal(
            NAME,
            Health::Red,
            "A FIX from a failed production smoke run",
        );
    }
    let old = fixes
        .iter()
        .filter(|f| f.age_days.is_some_and(|a| a > t.red_fix_open_days))
        .count();
    if old > 0 {
        return signal(
            NAME,
            Health::Red,
            format!("{old} FIX open more than {}", days(t.red_fix_open_days)),
        );
    }
    if !fixes.is_empty() {
        return signal(NAME, Health::Amber, format!("{} open FIX", fixes.len()));
    }
    if blocking_human_issues > 0 {
        return signal(
            NAME,
            Health::Amber,
            format!("Blocked by {blocking_human_issues} human issue(s)"),
        );
    }
    signal(NAME, Health::Green, "No FIX")
}

/// **Standards**: `audit_errors` is `None` when the audit has not been run
/// for the project.
pub fn standards(
    audit_errors: Option<u32>,
    open_review_findings: u32,
    t: &StandardsThresholds,
) -> Signal {
    const NAME: &str = "Standards";
    match audit_errors {
        None => signal(NAME, Health::Grey, "Audit not run"),
        Some(n) if n > 0 => signal(NAME, Health::Red, format!("{n} audit error(s)")),
        Some(_) if open_review_findings >= t.amber_review_findings => signal(
            NAME,
            Health::Amber,
            format!("{open_review_findings} open code-review findings"),
        ),
        Some(_) => signal(NAME, Health::Green, "Audit clean"),
    }
}

/// One open pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PullRequest {
    pub age_days: u32,
    pub needs_human: bool,
    pub checks_failing: bool,
}

/// **Pull requests**: the open PRs touching the project.
pub fn pull_requests(prs: &[PullRequest], t: &PullRequestThresholds) -> Signal {
    const NAME: &str = "Pull requests";
    if prs
        .iter()
        .any(|p| p.checks_failing && p.age_days > t.red_failing_days)
    {
        return signal(
            NAME,
            Health::Red,
            format!(
                "A PR with failing checks open more than {}",
                days(t.red_failing_days)
            ),
        );
    }
    if prs.iter().any(|p| p.age_days > t.amber_open_days) {
        return signal(
            NAME,
            Health::Amber,
            format!("A PR open more than {}", days(t.amber_open_days)),
        );
    }
    if prs.iter().any(|p| p.needs_human) {
        return signal(NAME, Health::Amber, "A PR labelled needs-human");
    }
    if prs.is_empty() {
        signal(NAME, Health::Green, "No open PRs")
    } else {
        signal(
            NAME,
            Health::Green,
            format!(
                "{} open, none older than {}",
                prs.len(),
                days(t.amber_open_days)
            ),
        )
    }
}

/// **Nightly CI and CD**: the latest `nightly-ci.yml` and `nightly-cd.yml`
/// runs. The worse of the two wins.
pub fn nightly(ci: Option<Run>, cd: Option<Run>, t: &NightlyThresholds) -> Signal {
    const NAME: &str = "Nightly CI and CD";
    let runs = [("nightly CI", ci), ("nightly CD", cd)];
    if runs.iter().all(|(_, r)| r.is_none()) {
        return signal(NAME, Health::Grey, "No nightly runs found");
    }
    if let Some((which, _)) = runs.iter().find(|(_, r)| r.is_some_and(|r| !r.passed)) {
        return signal(NAME, Health::Red, format!("Latest {which} failed"));
    }
    if let Some((which, _)) = runs
        .iter()
        .find(|(_, r)| r.is_none_or(|r| r.age_days > t.stale_after_days))
    {
        return signal(
            NAME,
            Health::Amber,
            format!("{which} didn't run last night"),
        );
    }
    signal(NAME, Health::Green, "Both ran and passed")
}

/// **Push to main**: whether the latest `push-main.yml` run passed.
pub fn push_main(latest_passed: Option<bool>) -> Signal {
    const NAME: &str = "Push to main";
    match latest_passed {
        None => signal(NAME, Health::Grey, "No push-main run found"),
        Some(false) => signal(NAME, Health::Red, "Latest run failed"),
        Some(true) => signal(NAME, Health::Green, "Latest run passed"),
    }
}

/// The latest free-tier usage report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeTierUsage {
    /// Technologies over the budget we set in the tech radar.
    pub over_budget: u32,
    /// Technologies over the vendor's free-tier limit.
    pub over_limit: u32,
}

/// **Free tiers**: `None` when there is no usage report yet.
pub fn free_tiers(usage: Option<FreeTierUsage>) -> Signal {
    const NAME: &str = "Free tiers";
    match usage {
        None => signal(NAME, Health::Grey, "No usage report"),
        Some(u) if u.over_limit > 0 => signal(
            NAME,
            Health::Red,
            format!("{} over a vendor limit", u.over_limit),
        ),
        Some(u) if u.over_budget > 0 => signal(
            NAME,
            Health::Amber,
            format!("{} over our budget", u.over_budget),
        ),
        Some(_) => signal(NAME, Health::Green, "All within budget"),
    }
}

/// **PR queue**: the ages in days of every open PR in the repo.
pub fn pr_queue(ages: &[u32], t: &PrQueueThresholds) -> Signal {
    const NAME: &str = "PR queue";
    let oldest = ages.iter().copied().max().unwrap_or(0);
    let count = ages.len();
    if oldest > t.red_oldest_days {
        return signal(
            NAME,
            Health::Red,
            format!("Oldest PR open {}", days(oldest)),
        );
    }
    if oldest > t.amber_oldest_days {
        return signal(
            NAME,
            Health::Amber,
            format!("Oldest PR open {}", days(oldest)),
        );
    }
    if count > t.amber_open_count as usize {
        return signal(NAME, Health::Amber, format!("{count} PRs open"));
    }
    signal(NAME, Health::Green, format!("{count} PRs open"))
}

/// **Human queue**: the ages in days of every task in a `docs/issues/human/`.
pub fn human_queue(ages: &[u32], t: &HumanQueueThresholds) -> Signal {
    const NAME: &str = "Human queue";
    let oldest = ages.iter().copied().max().unwrap_or(0);
    if oldest > t.red_oldest_days {
        return signal(
            NAME,
            Health::Red,
            format!("Oldest task waiting {}", days(oldest)),
        );
    }
    if oldest > t.amber_oldest_days {
        return signal(
            NAME,
            Health::Amber,
            format!("Oldest task waiting {}", days(oldest)),
        );
    }
    signal(
        NAME,
        Health::Green,
        format!("{} task(s) waiting", ages.len()),
    )
}

/// **Agent tasks**: days since the latest PR triage activity and since the
/// latest activity of any agent task, `None` when there has been none.
pub fn agent_tasks(
    triage_quiet_days: Option<u32>,
    any_quiet_days: Option<u32>,
    t: &AgentTaskThresholds,
) -> Signal {
    const NAME: &str = "Agent tasks";
    match any_quiet_days {
        None => return signal(NAME, Health::Grey, "No agent activity found"),
        Some(d) if d > t.red_quiet_days => {
            return signal(
                NAME,
                Health::Red,
                format!("No agent activity in {}", days(d)),
            );
        }
        Some(_) => {}
    }
    match triage_quiet_days {
        Some(d) if d <= t.amber_triage_quiet_days => {
            signal(NAME, Health::Green, "PR triage and agent tasks are running")
        }
        _ => signal(
            NAME,
            Health::Amber,
            format!(
                "No PR triage activity in more than {}",
                days(t.amber_triage_quiet_days)
            ),
        ),
    }
}

/// **Review rotation**: days since the newest deck in `docs/reviews`, or
/// `None` when there is none.
pub fn review_rotation(newest_age_days: Option<u32>, t: &ReviewRotationThresholds) -> Signal {
    const NAME: &str = "Review rotation";
    match newest_age_days {
        None => signal(NAME, Health::Grey, "No reviews yet"),
        Some(d) if d > t.red_after_days => {
            signal(NAME, Health::Red, format!("No new review in {}", days(d)))
        }
        Some(d) if d > t.amber_after_days => {
            signal(NAME, Health::Amber, format!("No new review in {}", days(d)))
        }
        Some(d) => signal(
            NAME,
            Health::Green,
            format!("Newest review {} old", days(d)),
        ),
    }
}

/// Open security alerts and the latest CodeQL run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityAlerts {
    pub moderate: u32,
    /// High or critical.
    pub high: u32,
}

/// **Security scans**: `None` when no scan results are available.
pub fn security_scans(alerts: Option<SecurityAlerts>) -> Signal {
    const NAME: &str = "Security scans";
    match alerts {
        None => signal(NAME, Health::Grey, "No scan results"),
        Some(a) if a.high > 0 => signal(
            NAME,
            Health::Red,
            format!("{} high or critical alert(s)", a.high),
        ),
        Some(a) if a.moderate > 0 => signal(
            NAME,
            Health::Amber,
            format!("{} moderate alert(s)", a.moderate),
        ),
        Some(_) => signal(NAME, Health::Green, "No open alerts"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Thresholds {
        Thresholds::default()
    }

    fn passed(age_days: u32) -> Option<Run> {
        Some(Run {
            passed: true,
            age_days,
        })
    }

    fn failed(age_days: u32) -> Option<Run> {
        Some(Run {
            passed: false,
            age_days,
        })
    }

    #[test]
    fn defaults_are_the_plan_starting_values() {
        let t = t();
        assert_eq!(t.build.stale_after_days, 2);
        assert_eq!(t.observability.amber_rise_percent, 50);
        assert_eq!((t.review.amber_poor_areas, t.review.red_poor_areas), (1, 3));
        assert_eq!(t.retros.amber_action_open_days, 30);
        assert_eq!(t.open_issues.red_fix_open_days, 14);
        assert_eq!(t.standards.amber_review_findings, 5);
        assert_eq!(
            (
                t.pull_requests.amber_open_days,
                t.pull_requests.red_failing_days
            ),
            (7, 3)
        );
        assert_eq!(t.nightly.stale_after_days, 1);
        assert_eq!(
            (
                t.pr_queue.amber_oldest_days,
                t.pr_queue.amber_open_count,
                t.pr_queue.red_oldest_days
            ),
            (7, 15, 30)
        );
        assert_eq!(
            (
                t.human_queue.amber_oldest_days,
                t.human_queue.red_oldest_days
            ),
            (14, 30)
        );
        assert_eq!(
            (
                t.agent_tasks.amber_triage_quiet_days,
                t.agent_tasks.red_quiet_days
            ),
            (1, 3)
        );
        assert_eq!(
            (
                t.review_rotation.amber_after_days,
                t.review_rotation.red_after_days
            ),
            (8, 15)
        );
    }

    #[test]
    fn roll_up_is_worst_of_non_grey_with_coverage() {
        let row = [
            signal("a", Health::Green, ""),
            signal("b", Health::Grey, ""),
            signal("c", Health::Amber, ""),
        ];
        let r = roll_up(&row);
        assert_eq!(r.health, Health::Amber);
        assert_eq!((r.covered, r.total), (2, 3));
        assert_eq!(r.coverage(), "2 of 3 signals");
    }

    #[test]
    fn roll_up_of_nothing_or_all_grey_is_grey() {
        assert_eq!(roll_up(&[]).health, Health::Grey);
        let r = roll_up(&[signal("a", Health::Grey, "")]);
        assert_eq!((r.health, r.covered, r.total), (Health::Grey, 0, 1));
    }

    #[test]
    fn roll_up_red_wins() {
        let row = [signal("a", Health::Red, ""), signal("b", Health::Green, "")];
        assert_eq!(roll_up(&row).health, Health::Red);
    }

    #[test]
    fn build_edges() {
        let t = &t().build;
        assert_eq!(build(None, t).health, Health::Grey);
        assert_eq!(build(passed(0), t).health, Health::Green);
        assert_eq!(build(passed(2), t).health, Health::Green);
        assert_eq!(build(passed(3), t).health, Health::Amber);
        assert_eq!(build(failed(0), t).health, Health::Red);
        assert_eq!(build(failed(10), t).health, Health::Red);
        assert_eq!(build(passed(1), t).reason, "Passed 1 day ago");
    }

    #[test]
    fn deploy_and_uat_edges() {
        let ok = Deploy {
            succeeded: true,
            smoke_passed: Some(true),
            behind_main: false,
        };
        assert_eq!(deploy_and_uat(None).health, Health::Grey);
        assert_eq!(deploy_and_uat(Some(ok)).health, Health::Green);
        assert_eq!(
            deploy_and_uat(Some(Deploy {
                behind_main: true,
                ..ok
            }))
            .health,
            Health::Amber
        );
        assert_eq!(
            deploy_and_uat(Some(Deploy {
                smoke_passed: None,
                ..ok
            }))
            .health,
            Health::Amber
        );
        assert_eq!(
            deploy_and_uat(Some(Deploy {
                smoke_passed: Some(false),
                ..ok
            }))
            .health,
            Health::Red
        );
        assert_eq!(
            deploy_and_uat(Some(Deploy {
                succeeded: false,
                ..ok
            }))
            .health,
            Health::Red
        );
    }

    #[test]
    fn observability_edges() {
        let t = &t().observability;
        let e = |before, since| {
            Some(Errors {
                per_day_before: before,
                per_day_since: since,
                site_up: true,
            })
        };
        assert_eq!(observability(None, t).health, Health::Grey);
        assert_eq!(observability(e(10, 10), t).health, Health::Green);
        assert_eq!(observability(e(10, 5), t).health, Health::Green);
        assert_eq!(observability(e(10, 14), t).health, Health::Green);
        assert_eq!(observability(e(10, 15), t).health, Health::Amber);
        assert_eq!(observability(e(0, 0), t).health, Health::Green);
        assert_eq!(observability(e(0, 1), t).health, Health::Amber);
        assert_eq!(observability(e(50, 50), t).health, Health::Green);
        assert_eq!(observability(e(50, 51), t).health, Health::Red);
        let down = Some(Errors {
            per_day_before: 0,
            per_day_since: 0,
            site_up: false,
        });
        assert_eq!(observability(down, t).health, Health::Red);
        let huge = Some(Errors {
            per_day_before: u32::MAX,
            per_day_since: u32::MAX,
            site_up: true,
        });
        let lenient = ObservabilityThresholds {
            red_errors_per_day: u32::MAX,
            amber_rise_percent: 50,
        };
        assert_eq!(observability(huge, &lenient).health, Health::Green);
    }

    #[test]
    fn review_edges() {
        let t = &t().review;
        let r = |age_days, poor_areas| {
            Some(ReviewStatus {
                age_days,
                poor_areas,
                rotation_days: 70,
            })
        };
        assert_eq!(review(None, t).health, Health::Grey);
        assert_eq!(review(r(10, 0), t).health, Health::Green);
        assert_eq!(review(r(70, 0), t).health, Health::Green);
        assert_eq!(review(r(71, 0), t).health, Health::Amber);
        assert_eq!(review(r(10, 1), t).health, Health::Amber);
        assert_eq!(review(r(10, 1), t).reason, "1 area rated Poor");
        assert_eq!(review(r(10, 2), t).health, Health::Amber);
        assert_eq!(review(r(10, 3), t).health, Health::Red);
        assert_eq!(review(r(200, 3), t).health, Health::Red);
    }

    #[test]
    fn retros_edges() {
        let t = &t().retros;
        assert_eq!(retros(None, t).health, Health::Grey);
        assert_eq!(retros(Some(&[]), t).health, Health::Green);
        assert_eq!(retros(Some(&[]), t).reason, "All actions done");
        assert_eq!(retros(Some(&[30]), t).health, Health::Green);
        assert_eq!(retros(Some(&[5, 31]), t).health, Health::Amber);
    }

    #[test]
    fn open_issues_edges() {
        let t = &t().open_issues;
        let fix = |age| Fix {
            age_days: Some(age),
            from_smoke: false,
        };
        assert_eq!(open_issues(&[], 0, t).health, Health::Green);
        assert_eq!(open_issues(&[], 1, t).health, Health::Amber);
        assert_eq!(open_issues(&[fix(14)], 0, t).health, Health::Amber);
        assert_eq!(open_issues(&[fix(15)], 0, t).health, Health::Red);
        let unknown = Fix {
            age_days: None,
            from_smoke: false,
        };
        assert_eq!(open_issues(&[unknown], 0, t).health, Health::Amber);
        let smoke = Fix {
            age_days: Some(0),
            from_smoke: true,
        };
        assert_eq!(open_issues(&[smoke], 0, t).health, Health::Red);
    }

    #[test]
    fn standards_edges() {
        let t = &t().standards;
        assert_eq!(standards(None, 9, t).health, Health::Grey);
        assert_eq!(standards(Some(0), 0, t).health, Health::Green);
        assert_eq!(standards(Some(0), 4, t).health, Health::Green);
        assert_eq!(standards(Some(0), 5, t).health, Health::Amber);
        assert_eq!(standards(Some(1), 0, t).health, Health::Red);
    }

    #[test]
    fn pull_requests_edges() {
        let t = &t().pull_requests;
        let pr = |age_days, needs_human, checks_failing| PullRequest {
            age_days,
            needs_human,
            checks_failing,
        };
        assert_eq!(pull_requests(&[], t).health, Health::Green);
        assert_eq!(
            pull_requests(&[pr(7, false, false)], t).health,
            Health::Green
        );
        assert_eq!(
            pull_requests(&[pr(8, false, false)], t).health,
            Health::Amber
        );
        assert_eq!(
            pull_requests(&[pr(0, true, false)], t).health,
            Health::Amber
        );
        assert_eq!(
            pull_requests(&[pr(3, false, true)], t).health,
            Health::Green
        );
        assert_eq!(pull_requests(&[pr(4, false, true)], t).health, Health::Red);
    }

    #[test]
    fn nightly_edges() {
        let t = &t().nightly;
        assert_eq!(nightly(None, None, t).health, Health::Grey);
        assert_eq!(nightly(passed(1), passed(0), t).health, Health::Green);
        assert_eq!(nightly(passed(2), passed(0), t).health, Health::Amber);
        assert_eq!(nightly(passed(0), None, t).health, Health::Amber);
        assert_eq!(nightly(passed(0), failed(0), t).health, Health::Red);
        assert_eq!(
            nightly(passed(0), failed(0), t).reason,
            "Latest nightly CD failed"
        );
        assert_eq!(nightly(failed(5), passed(5), t).health, Health::Red);
    }

    #[test]
    fn push_main_edges() {
        assert_eq!(push_main(None).health, Health::Grey);
        assert_eq!(push_main(Some(true)).health, Health::Green);
        assert_eq!(push_main(Some(false)).health, Health::Red);
    }

    #[test]
    fn free_tiers_edges() {
        let u = |over_budget, over_limit| {
            Some(FreeTierUsage {
                over_budget,
                over_limit,
            })
        };
        assert_eq!(free_tiers(None).health, Health::Grey);
        assert_eq!(free_tiers(u(0, 0)).health, Health::Green);
        assert_eq!(free_tiers(u(1, 0)).health, Health::Amber);
        assert_eq!(free_tiers(u(1, 1)).health, Health::Red);
    }

    #[test]
    fn pr_queue_edges() {
        let t = &t().pr_queue;
        assert_eq!(pr_queue(&[], t).health, Health::Green);
        assert_eq!(pr_queue(&[7, 1], t).health, Health::Green);
        assert_eq!(pr_queue(&[8], t).health, Health::Amber);
        assert_eq!(pr_queue(&[1; 15], t).health, Health::Green);
        assert_eq!(pr_queue(&[1; 16], t).health, Health::Amber);
        assert_eq!(pr_queue(&[30], t).health, Health::Amber);
        assert_eq!(pr_queue(&[31], t).health, Health::Red);
    }

    #[test]
    fn human_queue_edges() {
        let t = &t().human_queue;
        assert_eq!(human_queue(&[], t).health, Health::Green);
        assert_eq!(human_queue(&[14], t).health, Health::Green);
        assert_eq!(human_queue(&[15], t).health, Health::Amber);
        assert_eq!(human_queue(&[30], t).health, Health::Amber);
        assert_eq!(human_queue(&[2, 31], t).health, Health::Red);
    }

    #[test]
    fn agent_tasks_edges() {
        let t = &t().agent_tasks;
        assert_eq!(agent_tasks(None, None, t).health, Health::Grey);
        assert_eq!(agent_tasks(Some(1), Some(0), t).health, Health::Green);
        assert_eq!(agent_tasks(Some(2), Some(0), t).health, Health::Amber);
        assert_eq!(agent_tasks(None, Some(0), t).health, Health::Amber);
        assert_eq!(agent_tasks(Some(3), Some(3), t).health, Health::Amber);
        assert_eq!(agent_tasks(Some(4), Some(4), t).health, Health::Red);
    }

    #[test]
    fn review_rotation_edges() {
        let t = &t().review_rotation;
        assert_eq!(review_rotation(None, t).health, Health::Grey);
        assert_eq!(review_rotation(Some(8), t).health, Health::Green);
        assert_eq!(review_rotation(Some(9), t).health, Health::Amber);
        assert_eq!(review_rotation(Some(15), t).health, Health::Amber);
        assert_eq!(review_rotation(Some(16), t).health, Health::Red);
    }

    #[test]
    fn security_scans_edges() {
        let a = |moderate, high| Some(SecurityAlerts { moderate, high });
        assert_eq!(security_scans(None).health, Health::Grey);
        assert_eq!(security_scans(a(0, 0)).health, Health::Green);
        assert_eq!(security_scans(a(1, 0)).health, Health::Amber);
        assert_eq!(security_scans(a(1, 1)).health, Health::Red);
    }

    #[test]
    fn thresholds_round_trip_through_json() -> Result<(), serde_json::Error> {
        let t = t();
        let back: Thresholds = serde_json::from_str(&serde_json::to_string(&t)?)?;
        assert_eq!(back, t);
        Ok(())
    }

    #[test]
    fn missing_keys_default_and_unknown_keys_fail() -> Result<(), serde_json::Error> {
        let partial: Thresholds = serde_json::from_str(r#"{"build":{"stale_after_days":4}}"#)?;
        assert_eq!(partial.build.stale_after_days, 4);
        assert_eq!(partial.pr_queue, PrQueueThresholds::default());
        assert!(serde_json::from_str::<Thresholds>(r#"{"build":{"stale_days":4}}"#).is_err());
        assert!(serde_json::from_str::<Thresholds>(r#"{"bild":{}}"#).is_err());
        Ok(())
    }
}
