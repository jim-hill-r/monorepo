//! The verdict rules from the Verdicts section of the eval engine plan.
//!
//! | Verdict | Rule |
//! | --- | --- |
//! | Not met | The latest result of any production-stage eval failed, or Jim's latest rating is "not met" and no `cd` result has come in since. Amber (at risk) for `should` and `could`. |
//! | At risk | An eval failed `flaky_failures` of its last `flaky_window` runs, a metric is within `near_threshold` of its limit, the judge says not met (unless Jim rated it met since), a `ci` eval's latest result failed, an eval errored or never ran, or the newest production result is older than `stale_days` (or not younger than `fresh_days`). |
//! | Met | Every eval's latest result passes, and a production-stage result is younger than `fresh_days`. |
//! | Unknown | Retired, no evals, or no production-stage results yet. |

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::evals::{Eval, Kind, Stage};
use crate::requirements::{Priority, Requirement};
use crate::results::{EvalResult, ResultStatus};
use crate::{ParseError, ids::parse_requirement_id};

/// The tunable numbers in `evals/verdicts.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub near_threshold: f64,
    pub flaky_failures: usize,
    pub flaky_window: usize,
    pub fresh_days: i64,
    pub stale_days: i64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            near_threshold: 0.10,
            flaky_failures: 2,
            flaky_window: 7,
            fresh_days: 2,
            stale_days: 3,
        }
    }
}

impl Thresholds {
    /// Parses `verdicts.toml`.
    pub fn from_toml(text: &str) -> Result<Self, ParseError> {
        let thresholds: Thresholds =
            toml::from_str(text).map_err(|e| ParseError::whole_file(e.message().trim()))?;
        if !(0.0..1.0).contains(&thresholds.near_threshold) {
            return Err(ParseError::whole_file(
                "near_threshold must be at least 0 and below 1",
            ));
        }
        if thresholds.flaky_failures == 0 || thresholds.flaky_failures > thresholds.flaky_window {
            return Err(ParseError::whole_file(
                "flaky_failures must be between 1 and flaky_window",
            ));
        }
        if thresholds.fresh_days <= 0 || thresholds.fresh_days > thresholds.stale_days {
            return Err(ParseError::whole_file(
                "fresh_days must be positive and no more than stale_days",
            ));
        }
        Ok(thresholds)
    }
}

/// One requirement's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    /// Green.
    Met,
    /// Amber.
    AtRisk,
    /// Red.
    NotMet,
    /// Grey.
    Unknown,
}

/// A verdict and the reasons for it, for people and for `FIX` lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerdictReport {
    pub requirement: String,
    pub verdict: Verdict,
    pub reasons: Vec<String>,
}

/// Computes a requirement's verdict.
///
/// `evals` and `results` may hold other requirements' entries; only those
/// naming `requirement` are used. Results are matched to evals by eval id,
/// and Jim's ratings are the results with stage `human`.
pub fn verdict(
    requirement: &Requirement,
    evals: &[Eval],
    results: &[EvalResult],
    thresholds: &Thresholds,
    now: DateTime<Utc>,
) -> VerdictReport {
    let report = |verdict, reasons| VerdictReport {
        requirement: requirement.id.clone(),
        verdict,
        reasons,
    };
    if !requirement.is_active() {
        return report(Verdict::Unknown, vec!["retired".into()]);
    }

    let evals: Vec<&Eval> = evals
        .iter()
        .filter(|e| e.requirement == requirement.id)
        .collect();
    let mut results: Vec<&EvalResult> = results
        .iter()
        .filter(|r| r.requirement == requirement.id && r.status != ResultStatus::Skip)
        .collect();
    results.sort_by_key(|r| r.at);

    // Results per (eval, stage), oldest first.
    let mut runs: BTreeMap<(&str, Stage), Vec<&EvalResult>> = BTreeMap::new();
    for eval in &evals {
        for stage in &eval.stages {
            runs.insert((eval.id.as_str(), *stage), Vec::new());
        }
    }
    for result in &results {
        if let Some(list) = runs.get_mut(&(result.eval.as_str(), result.stage)) {
            list.push(result);
        }
    }

    let latest_human = results.iter().rev().find(|r| r.stage == Stage::Human);
    let latest_cd = results.iter().rev().find(|r| r.stage == Stage::Cd);
    let newest_production = results
        .iter()
        .rev()
        .find(|r| r.stage.is_production() && runs.contains_key(&(r.eval.as_str(), r.stage)));

    // Not met.
    let mut failures = Vec::new();
    for ((eval, stage), list) in &runs {
        if let Some(last) = list.last()
            && stage.is_production()
            && last.status == ResultStatus::Fail
        {
            failures.push(format!(
                "{eval} failed in {} at {}",
                stage.as_str(),
                last.at
            ));
        }
    }
    if let Some(human) = latest_human {
        let redeployed_since = latest_cd.is_some_and(|cd| cd.at > human.at);
        if human.status == ResultStatus::Fail && !redeployed_since {
            failures.push(format!("Jim rated it not met at {}", human.at));
        }
    }
    if !failures.is_empty() {
        let verdict = if requirement.priority == Priority::Must {
            Verdict::NotMet
        } else {
            Verdict::AtRisk
        };
        return report(verdict, failures);
    }

    // Unknown.
    if evals.is_empty() {
        return report(Verdict::Unknown, vec!["no evals".into()]);
    }
    let Some(newest_production) = newest_production else {
        return report(
            Verdict::Unknown,
            vec!["no production-stage results yet".into()],
        );
    };

    // At risk.
    let mut risks = Vec::new();
    for ((eval_id, stage), list) in &runs {
        let Some(last) = list.last() else {
            risks.push(format!("{eval_id} has no {} results yet", stage.as_str()));
            continue;
        };
        let recent = list.iter().rev().take(thresholds.flaky_window);
        let recent_failures = recent.filter(|r| r.status == ResultStatus::Fail).count();
        if recent_failures >= thresholds.flaky_failures {
            risks.push(format!(
                "{eval_id} failed {recent_failures} of its last {} {} runs",
                list.len().min(thresholds.flaky_window),
                stage.as_str()
            ));
        }
        match (last.status, stage) {
            (ResultStatus::Error, _) => {
                risks.push(format!(
                    "{eval_id} errored in {}; fix the eval",
                    stage.as_str()
                ));
            }
            (ResultStatus::Fail, Stage::Ci) => {
                risks.push(format!("{eval_id} is failing in ci"));
            }
            (ResultStatus::Fail, Stage::Judge) => {
                let jim_disagreed =
                    latest_human.is_some_and(|h| h.status == ResultStatus::Pass && h.at > last.at);
                if !jim_disagreed {
                    risks.push(format!("the judge says {eval_id} is not met"));
                }
            }
            _ => {}
        }
        if last.status == ResultStatus::Pass {
            let eval = evals.iter().find(|e| e.id == *eval_id);
            if let (Some(eval), Some(value)) = (eval, last.value)
                && near_threshold(&eval.kind, value, thresholds.near_threshold)
            {
                risks.push(format!(
                    "{eval_id} is within {:.0}% of its threshold ({value})",
                    thresholds.near_threshold * 100.0
                ));
            }
        }
    }
    let age = now - newest_production.at;
    if age > Duration::days(thresholds.stale_days) {
        risks.push(format!(
            "newest production result is {} days old",
            age.num_days()
        ));
    } else if age >= Duration::days(thresholds.fresh_days) {
        risks.push(format!(
            "no production result in the last {} days",
            thresholds.fresh_days
        ));
    }

    if risks.is_empty() {
        report(Verdict::Met, Vec::new())
    } else {
        report(Verdict::AtRisk, risks)
    }
}

/// True if a passing metric value is within `fraction` of its limit.
fn near_threshold(kind: &Kind, value: f64, fraction: f64) -> bool {
    let Kind::Metric { min, max, .. } = kind else {
        return false;
    };
    let near_max = max.is_some_and(|max| value > max - max.abs() * fraction);
    let near_min = min.is_some_and(|min| value < min + min.abs() * fraction);
    near_max || near_min
}

/// Requirements per project prefix are compared by number; kept here so
/// callers sort verdicts the way the requirements file lists them.
pub fn requirement_order(id: &str) -> (String, u32) {
    parse_requirement_id(id)
        .map(|(prefix, number)| (prefix.to_string(), number))
        .unwrap_or_else(|| (id.to_string(), 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirements::Status;
    use crate::results::By;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 10, 12, 0, 0)
            .single()
            .unwrap_or_default()
    }

    fn hours_ago(hours: i64) -> DateTime<Utc> {
        now() - Duration::hours(hours)
    }

    fn requirement(priority: Priority) -> Requirement {
        Requirement {
            id: "COOK-001".into(),
            title: "Readable on a phone".into(),
            priority,
            why: "w".into(),
            accept: "a".into(),
            status: Status::Active,
            line: 1,
        }
    }

    fn eval(id: &str, stages: &[Stage], kind: Kind) -> Eval {
        Eval {
            id: id.into(),
            requirement: "COOK-001".into(),
            stages: stages.to_vec(),
            kind,
            description: None,
        }
    }

    fn playwright() -> Kind {
        Kind::Playwright { grep: None }
    }

    fn lcp() -> Kind {
        Kind::Metric {
            source: "web-analytics".into(),
            metric: "lcp_p75_ms".into(),
            min: None,
            max: Some(2500.0),
            window: "7d".into(),
        }
    }

    fn result(eval: &str, stage: Stage, status: ResultStatus, hours: i64) -> EvalResult {
        EvalResult {
            project: "cookbook".into(),
            requirement: "COOK-001".into(),
            eval: eval.into(),
            stage,
            kind: "playwright".into(),
            commit: "abc1234".into(),
            deploy: None,
            status,
            value: None,
            threshold: None,
            evidence: None,
            run: None,
            at: hours_ago(hours),
            by: if stage == Stage::Human {
                By::Jim
            } else {
                By::Ci
            },
        }
    }

    fn layout_evals() -> Vec<Eval> {
        vec![eval("layout", &[Stage::Ci, Stage::Cd], playwright())]
    }

    fn green_results() -> Vec<EvalResult> {
        vec![
            result("layout", Stage::Ci, ResultStatus::Pass, 5),
            result("layout", Stage::Cd, ResultStatus::Pass, 4),
        ]
    }

    fn run(priority: Priority, evals: &[Eval], results: &[EvalResult]) -> VerdictReport {
        verdict(
            &requirement(priority),
            evals,
            results,
            &Thresholds::default(),
            now(),
        )
    }

    #[test]
    fn met_when_everything_passes_recently() {
        let report = run(Priority::Must, &layout_evals(), &green_results());
        assert_eq!(report.verdict, Verdict::Met, "{report:?}");
        assert!(report.reasons.is_empty());
    }

    #[test]
    fn unknown_without_evals_or_production_results() {
        let report = run(Priority::Must, &[], &[]);
        assert_eq!(report.verdict, Verdict::Unknown);
        assert_eq!(report.reasons, vec!["no evals"]);

        let ci_only = [result("layout", Stage::Ci, ResultStatus::Pass, 1)];
        let report = run(Priority::Must, &layout_evals(), &ci_only);
        assert_eq!(report.verdict, Verdict::Unknown);
    }

    #[test]
    fn unknown_when_retired() {
        let mut retired = requirement(Priority::Must);
        retired.status = Status::Retired {
            date: chrono::NaiveDate::default(),
            why: "gone".into(),
        };
        let report = verdict(
            &retired,
            &layout_evals(),
            &green_results(),
            &Thresholds::default(),
            now(),
        );
        assert_eq!(report.verdict, Verdict::Unknown);
    }

    #[test]
    fn production_failure_is_not_met_for_must_and_at_risk_otherwise() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 1));
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::NotMet
        );
        assert_eq!(
            run(Priority::Should, &layout_evals(), &results).verdict,
            Verdict::AtRisk
        );
        assert_eq!(
            run(Priority::Could, &layout_evals(), &results).verdict,
            Verdict::AtRisk
        );
    }

    #[test]
    fn a_later_pass_clears_a_production_failure() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 10));
        results.push(result("layout", Stage::Cd, ResultStatus::Pass, 1));
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::Met
        );
    }

    #[test]
    fn jims_not_met_rating_holds_until_the_next_deploy() {
        let mut results = green_results();
        results.push(result("human", Stage::Human, ResultStatus::Fail, 2));
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(report.verdict, Verdict::NotMet);
        assert!(report.reasons[0].contains("Jim rated it not met"));

        results.push(result("layout", Stage::Cd, ResultStatus::Pass, 1));
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::Met
        );
    }

    #[test]
    fn ci_failure_is_at_risk() {
        let mut results = green_results();
        results.push(result("layout", Stage::Ci, ResultStatus::Fail, 1));
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert_eq!(report.reasons, vec!["layout is failing in ci"]);
    }

    #[test]
    fn errors_are_at_risk_not_failures() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Error, 1));
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert!(report.reasons[0].contains("errored"), "{report:?}");
    }

    #[test]
    fn skips_are_ignored() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Skip, 1));
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::Met
        );
    }

    #[test]
    fn flaky_evals_are_at_risk() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 3));
        results.push(result("layout", Stage::Cd, ResultStatus::Pass, 2));
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 4));
        results.push(result("layout", Stage::Cd, ResultStatus::Pass, 1));
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert!(
            report.reasons[0].contains("failed 2 of its last"),
            "{report:?}"
        );
    }

    #[test]
    fn failures_older_than_the_flaky_window_are_forgotten() {
        let mut results = green_results();
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 100));
        results.push(result("layout", Stage::Cd, ResultStatus::Fail, 99));
        for hours in 1..=7 {
            results.push(result("layout", Stage::Cd, ResultStatus::Pass, hours));
        }
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::Met
        );
    }

    #[test]
    fn metrics_near_their_threshold_are_at_risk() {
        let evals = vec![eval("lcp", &[Stage::Observe], lcp())];
        let mut near = result("lcp", Stage::Observe, ResultStatus::Pass, 1);
        near.value = Some(2400.0);
        let report = run(Priority::Must, &evals, &[near.clone()]);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert!(report.reasons[0].contains("within 10%"), "{report:?}");

        near.value = Some(2000.0);
        assert_eq!(run(Priority::Must, &evals, &[near]).verdict, Verdict::Met);
    }

    #[test]
    fn near_threshold_checks_min_and_max() {
        let kind = Kind::Metric {
            source: "github".into(),
            metric: "m".into(),
            min: Some(100.0),
            max: None,
            window: "1d".into(),
        };
        assert!(near_threshold(&kind, 105.0, 0.1));
        assert!(!near_threshold(&kind, 115.0, 0.1));
        assert!(!near_threshold(&playwright(), 1.0, 0.1));
    }

    #[test]
    fn judge_failure_is_at_risk_unless_jim_rated_it_met_since() {
        let mut evals = layout_evals();
        evals.push(eval(
            "voice",
            &[Stage::Judge],
            Kind::Judge { rubric: "r".into() },
        ));
        let mut results = green_results();
        results.push(result("voice", Stage::Judge, ResultStatus::Fail, 3));
        let report = run(Priority::Must, &evals, &results);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert_eq!(report.reasons, vec!["the judge says voice is not met"]);

        results.push(result("human", Stage::Human, ResultStatus::Pass, 2));
        assert_eq!(run(Priority::Must, &evals, &results).verdict, Verdict::Met);
    }

    #[test]
    fn stale_production_results_are_at_risk() {
        let results = vec![
            result("layout", Stage::Ci, ResultStatus::Pass, 1),
            result("layout", Stage::Cd, ResultStatus::Pass, 24 * 4),
        ];
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert_eq!(
            report.reasons,
            vec!["newest production result is 4 days old"]
        );

        let results = vec![
            result("layout", Stage::Ci, ResultStatus::Pass, 1),
            result("layout", Stage::Cd, ResultStatus::Pass, 50),
        ];
        let report = run(Priority::Must, &layout_evals(), &results);
        assert_eq!(
            report.reasons,
            vec!["no production result in the last 2 days"]
        );
    }

    #[test]
    fn an_eval_that_never_ran_is_at_risk() {
        let mut evals = layout_evals();
        evals.push(eval("lcp", &[Stage::Observe], lcp()));
        let report = run(Priority::Must, &evals, &green_results());
        assert_eq!(report.verdict, Verdict::AtRisk);
        assert_eq!(report.reasons, vec!["lcp has no observe results yet"]);
    }

    #[test]
    fn ignores_other_requirements_and_unknown_evals() {
        let mut other = result("layout", Stage::Cd, ResultStatus::Fail, 1);
        other.requirement = "COOK-002".into();
        let stray = result("removed-eval", Stage::Cd, ResultStatus::Fail, 1);
        let mut results = green_results();
        results.push(other);
        results.push(stray);
        assert_eq!(
            run(Priority::Must, &layout_evals(), &results).verdict,
            Verdict::Met
        );
    }

    #[test]
    fn reads_the_thresholds_file() {
        let text = include_str!("../../verdicts.toml");
        assert_eq!(Thresholds::from_toml(text), Ok(Thresholds::default()));
    }

    #[test]
    fn rejects_bad_thresholds() {
        let bad = [
            "near_threshold = 1.5\nflaky_failures = 2\nflaky_window = 7\nfresh_days = 2\nstale_days = 3",
            "near_threshold = 0.1\nflaky_failures = 8\nflaky_window = 7\nfresh_days = 2\nstale_days = 3",
            "near_threshold = 0.1\nflaky_failures = 2\nflaky_window = 7\nfresh_days = 4\nstale_days = 3",
            "near_threshold = 0.1\nflaky_failures = 2\nflaky_window = 7\nfresh_days = 2",
            "near_threshold = 0.1\nflaky_failures = 2\nflaky_window = 7\nfresh_days = 2\nstale_days = 3\nextra = 1",
        ];
        for text in bad {
            assert!(Thresholds::from_toml(text).is_err(), "{text}");
        }
    }

    #[test]
    fn verdicts_serialize_in_kebab_case() {
        let json = serde_json::to_string(&Verdict::NotMet).unwrap_or_default();
        assert_eq!(json, "\"not-met\"");
    }

    #[test]
    fn orders_requirements_by_number() {
        let mut ids = vec!["COOK-010", "COOK-002", "COOK-1000"];
        ids.sort_by_key(|id| requirement_order(id));
        assert_eq!(ids, vec!["COOK-002", "COOK-010", "COOK-1000"]);
    }
}
