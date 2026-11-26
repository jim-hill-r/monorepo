//! Reads the test reports a project's own tasks wrote, so `evals run`
//! never runs a test itself.
//!
//! - Cargo: the text `cargo test` prints (`test a::b ... ok`). Stable Cargo
//!   has no JSON test output.
//! - Playwright: the JSON reporter's file.

use anyhow::{Context, Result};
use serde::Deserialize;

/// What happened to one test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed,
    Skipped,
}

/// One test from a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    /// Cargo: the test path (`tests::cook_001_layout`). Playwright: the full
    /// title (file, describe blocks and test title joined with ` > `),
    /// followed by the test's tags.
    pub name: String,
    pub outcome: Outcome,
}

/// Parses `cargo test` output. Lines that aren't test results are ignored.
pub fn parse_cargo_output(text: &str) -> Vec<TestCase> {
    text.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("test ")?;
            let (name, result) = rest.rsplit_once(" ... ")?;
            let outcome = match result.trim() {
                "ok" => Outcome::Passed,
                "FAILED" => Outcome::Failed,
                r if r.starts_with("ignored") => Outcome::Skipped,
                _ => return None,
            };
            Some(TestCase {
                name: name.trim().to_string(),
                outcome,
            })
        })
        .collect()
}

/// True if a Cargo test's own name (the last `::` segment) starts with
/// `filter`, the way `evals.toml` ties `cook_001_*` tests to COOK-001.
pub fn cargo_test_matches(name: &str, filter: &str) -> bool {
    name.rsplit("::").next().unwrap_or(name).starts_with(filter)
}

#[derive(Deserialize)]
struct PlaywrightReport {
    #[serde(default)]
    suites: Vec<PlaywrightSuite>,
}

#[derive(Deserialize)]
struct PlaywrightSuite {
    #[serde(default)]
    title: String,
    #[serde(default)]
    specs: Vec<PlaywrightSpec>,
    #[serde(default)]
    suites: Vec<PlaywrightSuite>,
}

#[derive(Deserialize)]
struct PlaywrightSpec {
    title: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    tests: Vec<PlaywrightTest>,
}

#[derive(Deserialize)]
struct PlaywrightTest {
    status: String,
}

/// Parses a Playwright JSON report. A spec fails if any of its runs (one
/// per browser project) was unexpected, is skipped if every run was, and
/// passes otherwise (flaky runs that passed on retry count as passes).
pub fn parse_playwright_report(text: &str) -> Result<Vec<TestCase>> {
    let report: PlaywrightReport =
        serde_json::from_str(text).context("reading the Playwright JSON report")?;
    let mut cases = Vec::new();
    for suite in &report.suites {
        collect_suite(suite, &mut Vec::new(), &mut cases);
    }
    Ok(cases)
}

fn collect_suite<'a>(
    suite: &'a PlaywrightSuite,
    path: &mut Vec<&'a str>,
    cases: &mut Vec<TestCase>,
) {
    if !suite.title.is_empty() {
        path.push(&suite.title);
    }
    for spec in &suite.specs {
        let mut name = path.clone();
        name.push(&spec.title);
        let mut name = name.join(" > ");
        for tag in &spec.tags {
            let tag = if tag.starts_with('@') {
                tag.clone()
            } else {
                format!("@{tag}")
            };
            if !name.contains(&tag) {
                name.push(' ');
                name.push_str(&tag);
            }
        }
        let outcome = if spec.tests.iter().any(|t| t.status == "unexpected") {
            Outcome::Failed
        } else if spec.tests.iter().all(|t| t.status == "skipped") {
            Outcome::Skipped
        } else {
            Outcome::Passed
        };
        cases.push(TestCase { name, outcome });
    }
    for child in &suite.suites {
        collect_suite(child, path, cases);
    }
    if !suite.title.is_empty() {
        path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARGO: &str = "\
   Compiling cookbook v0.1.0
running 4 tests
test units::tests::cook_002_converts_cups ... ok
test units::tests::cook_002_rounds ... FAILED
test search::tests::finds_leeks ... ignored, needs network
test search::tests::cook_003_prints ... ignored
test result: FAILED. 1 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out

   Doc-tests cookbook
test src/lib.rs - units::convert (line 12) ... ok
";

    #[test]
    fn parses_cargo_test_output() {
        let cases = parse_cargo_output(CARGO);
        let outcomes: Vec<_> = cases.iter().map(|c| (c.name.as_str(), c.outcome)).collect();
        assert_eq!(
            outcomes,
            vec![
                ("units::tests::cook_002_converts_cups", Outcome::Passed),
                ("units::tests::cook_002_rounds", Outcome::Failed),
                ("search::tests::finds_leeks", Outcome::Skipped),
                ("search::tests::cook_003_prints", Outcome::Skipped),
                ("src/lib.rs - units::convert (line 12)", Outcome::Passed),
            ]
        );
    }

    #[test]
    fn cargo_filters_match_the_test_name_not_the_module() {
        assert!(cargo_test_matches(
            "units::tests::cook_002_rounds",
            "cook_002"
        ));
        assert!(cargo_test_matches("cook_002_rounds", "cook_002"));
        assert!(!cargo_test_matches("cook_002::tests::rounds", "cook_002"));
        assert!(!cargo_test_matches(
            "units::tests::cook_0021_x",
            "cook_002_"
        ));
    }

    const PLAYWRIGHT: &str = r#"{
      "config": {},
      "suites": [
        {
          "title": "recipe.spec.ts",
          "specs": [
            {"title": "fits a phone @smoke @req:COOK-001", "tags": ["smoke", "req:COOK-001"],
             "tests": [{"status": "expected"}, {"status": "flaky"}]},
            {"title": "big text", "tags": ["@req:COOK-001"],
             "tests": [{"status": "expected"}, {"status": "unexpected"}]}
          ],
          "suites": [
            {"title": "offline", "specs": [
              {"title": "still readable @req:COOK-004", "tests": [{"status": "skipped"}]}
            ]}
          ]
        }
      ],
      "errors": []
    }"#;

    #[test]
    fn parses_a_playwright_report() -> Result<()> {
        let cases = parse_playwright_report(PLAYWRIGHT)?;
        let outcomes: Vec<_> = cases.iter().map(|c| (c.name.as_str(), c.outcome)).collect();
        assert_eq!(
            outcomes,
            vec![
                (
                    "recipe.spec.ts > fits a phone @smoke @req:COOK-001",
                    Outcome::Passed
                ),
                ("recipe.spec.ts > big text @req:COOK-001", Outcome::Failed),
                (
                    "recipe.spec.ts > offline > still readable @req:COOK-004",
                    Outcome::Skipped
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn rejects_a_report_that_is_not_json() {
        assert!(parse_playwright_report("<html>").is_err());
    }
}
