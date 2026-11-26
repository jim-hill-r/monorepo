//! `evals run`: turns the test reports a project's tasks already wrote into
//! one result per eval. Phase 0 collects the `cargo` and `playwright` kinds;
//! other kinds are left for later phases and produce no result.

use chrono::{DateTime, Utc};
use evals_model::{By, Eval, EvalResult, EvalsFile, Kind, ResultStatus, Stage};

use crate::reports::{Outcome, TestCase, cargo_test_matches};

/// The parsed reports. `None` means no report of that kind was given.
#[derive(Debug, Default)]
pub struct Reports {
    pub cargo: Option<Vec<TestCase>>,
    pub playwright: Option<Vec<TestCase>>,
}

/// Where and when this run happened, copied into every result.
#[derive(Debug, Clone)]
pub struct RunContext {
    pub project: String,
    pub stage: Stage,
    pub commit: String,
    pub run: Option<String>,
    pub evidence: Option<String>,
    pub at: DateTime<Utc>,
}

/// Results for the evals in `file` that run in `ctx.stage`, plus a note for
/// each eval whose kind isn't collected yet.
pub fn collect(
    file: &EvalsFile,
    reports: &Reports,
    ctx: &RunContext,
) -> (Vec<EvalResult>, Vec<String>) {
    let mut results = Vec::new();
    let mut notes = Vec::new();
    for eval in file.evals.iter().filter(|e| e.stages.contains(&ctx.stage)) {
        let (cases, kind) = match &eval.kind {
            Kind::Cargo { .. } => (&reports.cargo, "cargo"),
            Kind::Playwright { .. } => (&reports.playwright, "playwright"),
            other => {
                notes.push(format!(
                    "eval `{}`: {} evals aren't collected yet",
                    eval.id,
                    other.name()
                ));
                continue;
            }
        };
        let (status, note) = judge(eval, cases.as_deref(), kind);
        results.push(EvalResult {
            project: ctx.project.clone(),
            requirement: eval.requirement.clone(),
            eval: eval.id.clone(),
            stage: ctx.stage,
            kind: kind.to_string(),
            commit: ctx.commit.clone(),
            deploy: None,
            status,
            value: None,
            threshold: None,
            evidence: ctx.evidence.clone(),
            run: ctx.run.clone(),
            note: Some(note),
            at: ctx.at,
            by: By::Ci,
        });
    }
    (results, notes)
}

fn judge(eval: &Eval, cases: Option<&[TestCase]>, kind: &str) -> (ResultStatus, String) {
    let selector = eval.test_selector().unwrap_or_default();
    let Some(cases) = cases else {
        return (ResultStatus::Skip, format!("no {kind} report"));
    };
    let matching: Vec<&TestCase> = cases
        .iter()
        .filter(|case| match eval.kind {
            Kind::Cargo { .. } => cargo_test_matches(&case.name, &selector),
            _ => case.name.contains(&selector),
        })
        .collect();
    if matching.is_empty() {
        return (
            ResultStatus::Error,
            format!("no test in the {kind} report matched `{selector}`"),
        );
    }
    let failed: Vec<&str> = matching
        .iter()
        .filter(|c| c.outcome == Outcome::Failed)
        .map(|c| c.name.as_str())
        .collect();
    if !failed.is_empty() {
        return (
            ResultStatus::Fail,
            format!(
                "{} of {} tests failed: {}",
                failed.len(),
                matching.len(),
                failed.join(", ")
            ),
        );
    }
    let passed = matching
        .iter()
        .filter(|c| c.outcome == Outcome::Passed)
        .count();
    if passed == 0 {
        return (
            ResultStatus::Skip,
            format!("all {} matching tests were skipped", matching.len()),
        );
    }
    let skipped = matching.len() - passed;
    let note = if skipped == 0 {
        format!("{passed} {} passed", tests(passed))
    } else {
        format!("{passed} {} passed, {skipped} skipped", tests(passed))
    };
    (ResultStatus::Pass, note)
}

fn tests(n: usize) -> &'static str {
    if n == 1 { "test" } else { "tests" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::{parse_cargo_output, parse_playwright_report};
    use evals_model::parse_evals;

    const EVALS: &str = r#"
prefix = "COOK"

[[eval]]
id = "layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "units"
requirement = "COOK-002"
stages = ["ci"]
kind = "cargo"

[[eval]]
id = "lcp"
requirement = "COOK-001"
stages = ["observe"]
kind = "metric"
source = "web-analytics"
metric = "lcp_p75_ms"
max = 2500
window = "7d"

[[eval]]
id = "home"
requirement = "COOK-001"
stages = ["ci"]
kind = "http"
path = "/"
"#;

    fn file() -> EvalsFile {
        match parse_evals(EVALS) {
            Ok(file) => file,
            Err(errors) => panic!("{errors:?}"),
        }
    }

    fn ctx() -> RunContext {
        RunContext {
            project: "cookbook".into(),
            stage: Stage::Ci,
            commit: "abc1234".into(),
            run: Some("42".into()),
            evidence: Some("https://github.com/o/r/actions/runs/42".into()),
            at: DateTime::<Utc>::default(),
        }
    }

    fn statuses(results: &[EvalResult]) -> Vec<(&str, ResultStatus, &str)> {
        results
            .iter()
            .map(|r| {
                (
                    r.eval.as_str(),
                    r.status,
                    r.note.as_deref().unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn collects_cargo_and_playwright_evals_for_the_stage() -> anyhow::Result<()> {
        let reports = Reports {
            cargo: Some(parse_cargo_output(
                "test units::tests::cook_002_cups ... ok\ntest units::tests::cook_002_grams ... ok\n",
            )),
            playwright: Some(parse_playwright_report(
                r#"{"suites":[{"title":"a.spec.ts","specs":[{"title":"fits @req:COOK-001","tests":[{"status":"expected"}]}]}]}"#,
            )?),
        };
        let (results, notes) = collect(&file(), &reports, &ctx());
        assert_eq!(
            statuses(&results),
            vec![
                ("layout", ResultStatus::Pass, "1 test passed"),
                ("units", ResultStatus::Pass, "2 tests passed"),
            ]
        );
        assert_eq!(notes, vec!["eval `home`: http evals aren't collected yet"]);

        let first = &results[0];
        assert_eq!(first.project, "cookbook");
        assert_eq!(first.requirement, "COOK-001");
        assert_eq!(first.kind, "playwright");
        assert_eq!(first.stage, Stage::Ci);
        assert_eq!(first.by, By::Ci);
        assert_eq!(first.run.as_deref(), Some("42"));
        Ok(())
    }

    #[test]
    fn only_evals_in_the_stage_run() {
        let mut ctx = ctx();
        ctx.stage = Stage::Cd;
        let (results, notes) = collect(&file(), &Reports::default(), &ctx);
        assert_eq!(
            statuses(&results),
            vec![("layout", ResultStatus::Skip, "no playwright report")]
        );
        assert!(notes.is_empty());
    }

    #[test]
    fn failures_errors_and_skips() {
        let reports = Reports {
            cargo: Some(parse_cargo_output(
                "test cook_002_a ... ok\ntest cook_002_b ... FAILED\n",
            )),
            playwright: Some(Vec::new()),
        };
        let (results, _) = collect(&file(), &reports, &ctx());
        assert_eq!(
            statuses(&results),
            vec![
                (
                    "layout",
                    ResultStatus::Error,
                    "no test in the playwright report matched `@req:COOK-001`"
                ),
                (
                    "units",
                    ResultStatus::Fail,
                    "1 of 2 tests failed: cook_002_b"
                ),
            ]
        );

        let reports = Reports {
            cargo: Some(parse_cargo_output(
                "test cook_002_a ... ignored\ntest cook_002_b ... ignored\n",
            )),
            playwright: None,
        };
        let (results, _) = collect(&file(), &reports, &ctx());
        assert_eq!(
            statuses(&results),
            vec![
                ("layout", ResultStatus::Skip, "no playwright report"),
                (
                    "units",
                    ResultStatus::Skip,
                    "all 2 matching tests were skipped"
                ),
            ]
        );
    }

    #[test]
    fn a_skipped_test_beside_a_pass_still_passes() {
        let reports = Reports {
            cargo: Some(parse_cargo_output(
                "test cook_002_a ... ok\ntest cook_002_b ... ignored\n",
            )),
            playwright: None,
        };
        let (results, _) = collect(&file(), &reports, &ctx());
        assert_eq!(results[1].status, ResultStatus::Pass);
        assert_eq!(results[1].note.as_deref(), Some("1 test passed, 1 skipped"));
    }
}
