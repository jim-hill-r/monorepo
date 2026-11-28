//! `evals run`: turns the test reports a project's tasks already wrote into
//! one result per eval (the `cargo`, `trycmd` and `playwright` kinds), and makes the
//! requests of the `http` kind (see `http.rs`). `cost check` writes the
//! `ci` results of `cost` evals. Other kinds are left for later phases and
//! produce no result.

use chrono::{DateTime, Utc};
use evals_model::{
    By, Eval, EvalResult, EvalsFile, Kind, Priority, Requirement, ResultStatus, Stage,
};

use crate::check::glob_matches_tail;
use crate::http::{self, Fetch, Retry};
use crate::reports::{Outcome, TestCase, cargo_test_matches};

/// The parsed reports. `None` means no report of that kind was given.
#[derive(Debug, Default)]
pub struct Reports {
    pub cargo: Option<Vec<TestCase>>,
    pub playwright: Option<Vec<TestCase>>,
    /// trycmd cases, read from the same `cargo test` output.
    pub trycmd: Option<Vec<TestCase>>,
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
    /// The reports cover only part of the project (one of its sub-projects,
    /// in its own CI job). An eval with no report of its kind, or no matching
    /// test, then gets no result rather than `skip` or `error`: another
    /// part's job reports it, and `evals check` (EVL-003) catches an eval
    /// that matches no test anywhere.
    pub partial: bool,
    /// The deployed site that `http` evals with a `path` are joined to
    /// (`DEPLOY_URL`).
    pub deploy_url: Option<String>,
}

/// How `http` evals make their requests.
pub struct HttpRunner<'a> {
    pub fetch: &'a dyn Fetch,
    pub retry: Retry,
}

/// Results for the evals in `file` that run in `ctx.stage`, plus a note for
/// each eval whose kind isn't collected yet.
pub fn collect(
    file: &EvalsFile,
    reports: &Reports,
    ctx: &RunContext,
    runner: &HttpRunner,
) -> (Vec<EvalResult>, Vec<String>) {
    let mut results = Vec::new();
    let mut notes = Vec::new();
    for eval in file.evals.iter().filter(|e| e.stages.contains(&ctx.stage)) {
        let (cases, kind) = match &eval.kind {
            // Not a report, so the job that has the deploy URL runs it even
            // when its reports are partial.
            Kind::Http { .. } => {
                let outcome = http::run(
                    &eval.kind,
                    ctx.deploy_url.as_deref(),
                    runner.fetch,
                    runner.retry,
                );
                let mut result = result(eval, ctx, "http", outcome.status, outcome.note);
                result.value = outcome.value;
                result.threshold = outcome.threshold;
                results.push(result);
                continue;
            }
            Kind::Cargo { .. } => (&reports.cargo, "cargo"),
            Kind::Playwright { .. } => (&reports.playwright, "playwright"),
            Kind::Trycmd { .. } => (&reports.trycmd, "trycmd"),
            // `cost check --evals-results` writes these results itself, so
            // the cost rules run once (standards/docs/evals.md).
            Kind::Cost {} if ctx.stage == Stage::Ci => continue,
            other => {
                notes.push(format!(
                    "eval `{}`: {} evals aren't collected yet",
                    eval.id,
                    other.name()
                ));
                continue;
            }
        };
        if ctx.partial && !has_match(eval, cases.as_deref()) {
            continue;
        }
        let (status, note) = judge(eval, cases.as_deref(), kind);
        results.push(result(eval, ctx, kind, status, note));
    }
    (results, notes)
}

/// The failed results whose requirement is a `must`, for `--fail-on-must`.
/// An `error` doesn't count: the engine couldn't check, which isn't a
/// broken product.
pub fn failed_musts<'a>(
    results: &'a [EvalResult],
    requirements: &[Requirement],
) -> Vec<&'a EvalResult> {
    results
        .iter()
        .filter(|r| r.status == ResultStatus::Fail)
        .filter(|r| {
            requirements
                .iter()
                .any(|req| req.id == r.requirement && req.priority == Priority::Must)
        })
        .collect()
}

fn result(
    eval: &Eval,
    ctx: &RunContext,
    kind: &str,
    status: ResultStatus,
    note: String,
) -> EvalResult {
    EvalResult {
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
    }
}

/// The tests in `cases` that belong to `eval`.
fn matching<'a>(eval: &Eval, cases: &'a [TestCase]) -> Vec<&'a TestCase> {
    let selector = eval.test_selector().unwrap_or_default();
    cases
        .iter()
        .filter(|case| match eval.kind {
            Kind::Cargo { .. } => cargo_test_matches(&case.name, &selector),
            Kind::Trycmd { .. } => glob_matches_tail(&selector, &case.name),
            _ => case.name.contains(&selector),
        })
        .collect()
}

/// True if a report was given and at least one of its tests is the eval's.
fn has_match(eval: &Eval, cases: Option<&[TestCase]>) -> bool {
    cases.is_some_and(|cases| !matching(eval, cases).is_empty())
}

fn judge(eval: &Eval, cases: Option<&[TestCase]>, kind: &str) -> (ResultStatus, String) {
    let selector = eval.test_selector().unwrap_or_default();
    let Some(cases) = cases else {
        return (ResultStatus::Skip, format!("no {kind} report"));
    };
    let matching = matching(eval, cases);
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
stages = ["cd"]
kind = "http"
path = "/"
body_contains = "Cookbook"

[[eval]]
id = "free"
requirement = "COOK-003"
stages = ["ci", "observe"]
kind = "cost"
"#;

    /// Answers every request with the same page.
    struct Page(&'static str);

    impl Fetch for Page {
        fn fetch(&self, _: &str, _: &str, _: bool) -> Result<http::Response, String> {
            Ok(http::Response {
                status: 200,
                headers: Vec::new(),
                body: self.0.as_bytes().to_vec(),
                elapsed_ms: 80,
            })
        }
    }

    const ONCE: Retry = Retry {
        attempts: 1,
        delay: std::time::Duration::ZERO,
    };

    fn runner(page: &Page) -> HttpRunner<'_> {
        HttpRunner {
            fetch: page,
            retry: ONCE,
        }
    }

    fn collect(
        file: &EvalsFile,
        reports: &Reports,
        ctx: &RunContext,
    ) -> (Vec<EvalResult>, Vec<String>) {
        super::collect(file, reports, ctx, &runner(&Page("<h1>Cookbook</h1>")))
    }

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
            partial: false,
            deploy_url: None,
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
            trycmd: None,
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
        assert!(notes.is_empty(), "{notes:?}");

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
    fn cost_evals_are_left_to_cost_check_in_ci() {
        let (results, notes) = collect(&file(), &Reports::default(), &ctx());
        assert!(results.iter().all(|r| r.eval != "free"), "{results:?}");
        assert!(notes.is_empty(), "{notes:?}");

        let mut ctx = ctx();
        ctx.stage = Stage::Observe;
        let (results, notes) = collect(&file(), &Reports::default(), &ctx);
        assert!(results.is_empty(), "{results:?}");
        assert_eq!(
            notes,
            vec![
                "eval `lcp`: metric evals aren't collected yet",
                "eval `free`: cost evals aren't collected yet",
            ]
        );
    }

    #[test]
    fn only_evals_in_the_stage_run() {
        let mut ctx = ctx();
        ctx.stage = Stage::Cd;
        let (results, notes) = collect(&file(), &Reports::default(), &ctx);
        assert_eq!(
            statuses(&results),
            vec![
                ("layout", ResultStatus::Skip, "no playwright report"),
                ("home", ResultStatus::Error, "no DEPLOY_URL to join `/` to"),
            ]
        );
        assert!(notes.is_empty());
    }

    #[test]
    fn http_evals_request_the_deployed_site() {
        let mut ctx = ctx();
        ctx.stage = Stage::Cd;
        ctx.partial = true;
        ctx.deploy_url = Some("https://jimhillr-cookbook.pages.dev".into());
        let (results, _) = collect(&file(), &Reports::default(), &ctx);
        assert_eq!(
            statuses(&results),
            vec![(
                "home",
                ResultStatus::Pass,
                "GET https://jimhillr-cookbook.pages.dev/: 200 in 80 ms"
            )]
        );
        assert_eq!(results[0].kind, "http");
        assert_eq!(results[0].stage, Stage::Cd);
        assert_eq!(results[0].value, Some(80.0));

        let (results, _) = super::collect(
            &file(),
            &Reports::default(),
            &ctx,
            &runner(&Page("Not found")),
        );
        assert_eq!(results[0].status, ResultStatus::Fail);
        assert_eq!(
            results[0].note.as_deref(),
            Some(
                "GET https://jimhillr-cookbook.pages.dev/: body (9 bytes) doesn't contain `Cookbook`"
            )
        );
    }

    #[test]
    fn trycmd_evals_match_case_files_by_glob() {
        let file = match parse_evals(
            r#"
prefix = "HOME"

[[eval]]
id = "score"
requirement = "HOME-001"
stages = ["ci"]
kind = "trycmd"

[[eval]]
id = "help"
requirement = "HOME-002"
stages = ["ci"]
kind = "trycmd"
glob = "cmd/help*.toml"
"#,
        ) {
            Ok(file) => file,
            Err(errors) => panic!("{errors:?}"),
        };
        let reports = Reports {
            cargo: Some(Vec::new()),
            playwright: None,
            trycmd: Some(crate::reports::parse_trycmd_output(
                "Testing tests/cmd/HOME-001-score.toml ... ok 3ms\n\
                 Testing tests/cmd/HOME-001-score-json.toml ... failed 3ms\n\
                 Testing tests/cmd/help.toml ... ok\n\
                 Testing tests/cmd/help-hides-api-key.toml ... ok\n",
            )),
        };
        let (results, notes) = collect(&file, &reports, &ctx());
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(
            statuses(&results),
            vec![
                (
                    "score",
                    ResultStatus::Fail,
                    "1 of 2 tests failed: tests/cmd/HOME-001-score-json.toml"
                ),
                ("help", ResultStatus::Pass, "2 tests passed"),
            ]
        );
        assert_eq!(results[0].kind, "trycmd");

        let (results, _) = collect(&file, &Reports::default(), &ctx());
        assert_eq!(results[0].note.as_deref(), Some("no trycmd report"));
    }

    #[test]
    fn only_a_failed_must_fails_the_run() {
        let (requirements, errors) = evals_model::parse_requirements(
            "# Requirements\n\n## COOK-001: Must\n\n- Priority: must\n- Why: w\n- Accept: a\n\n## COOK-002: Should\n\n- Priority: should\n- Why: w\n- Accept: a\n",
        );
        assert!(errors.is_empty(), "{errors:?}");
        let mut ctx = ctx();
        ctx.stage = Stage::Cd;
        ctx.deploy_url = Some("https://a".into());
        let runner = runner(&Page("Not found"));
        let (mut results, _) = super::collect(&file(), &Reports::default(), &ctx, &runner);
        let failed: Vec<&str> = failed_musts(&results, &requirements)
            .iter()
            .map(|r| r.eval.as_str())
            .collect();
        assert_eq!(failed, vec!["home"]);

        results
            .iter_mut()
            .for_each(|r| r.requirement = "COOK-002".into());
        assert!(failed_musts(&results, &requirements).is_empty());
        results.iter_mut().for_each(|r| {
            r.requirement = "COOK-001".into();
            r.status = ResultStatus::Error;
        });
        assert!(failed_musts(&results, &requirements).is_empty());
    }

    #[test]
    fn failures_errors_and_skips() {
        let reports = Reports {
            trycmd: None,
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
            trycmd: None,
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
            trycmd: None,
            cargo: Some(parse_cargo_output(
                "test cook_002_a ... ok\ntest cook_002_b ... ignored\n",
            )),
            playwright: None,
        };
        let (results, _) = collect(&file(), &reports, &ctx());
        assert_eq!(results[1].status, ResultStatus::Pass);
        assert_eq!(results[1].note.as_deref(), Some("1 test passed, 1 skipped"));
    }

    #[test]
    fn a_partial_run_reports_only_evals_it_has_tests_for() {
        let mut ctx = ctx();
        ctx.partial = true;
        let reports = Reports {
            trycmd: None,
            cargo: Some(parse_cargo_output("test web::other_test ... ok\n")),
            playwright: Some(Vec::new()),
        };
        let (results, _) = collect(&file(), &reports, &ctx);
        assert!(results.is_empty(), "{results:?}");

        let reports = Reports {
            trycmd: None,
            cargo: Some(parse_cargo_output("test cook_002_a ... FAILED\n")),
            playwright: None,
        };
        let (results, _) = collect(&file(), &reports, &ctx);
        assert_eq!(
            statuses(&results),
            vec![(
                "units",
                ResultStatus::Fail,
                "1 of 1 tests failed: cook_002_a"
            )]
        );
    }
}
