//! Reads the test reports a project's own tasks wrote, so `evals run`
//! never runs a test itself.
//!
//! - Cargo: the text `cargo test` prints (`test a::b ... ok`). Stable Cargo
//!   has no JSON test output.
//! - Playwright: the JSON reporter's file.
//! - trycmd: the `Testing tests/cmd/<case>.toml ... ok` lines trycmd prints
//!   while `cargo test` runs, so they are in the same saved output.

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

/// The first error line of a saved `cargo test` output whose build failed
/// before any test ran (`error: could not compile` or `failed to run custom
/// build command`, and no `test result:` line), so its evals can fail with
/// the reason instead of matching no test. `None` when tests ran.
pub fn cargo_build_failure(text: &str) -> Option<String> {
    let lines: Vec<String> = text
        .lines()
        .map(|l| strip_ansi(l).trim().to_string())
        .collect();
    if lines.iter().any(|l| l.starts_with("test result:")) {
        return None;
    }
    let broke = lines.iter().any(|l| {
        l.starts_with("error: could not compile")
            || l.starts_with("error: failed to run custom build command")
    });
    if !broke {
        return None;
    }
    lines
        .into_iter()
        .find(|l| l.starts_with("error:") || l.starts_with("error["))
}

/// Parses the case lines trycmd prints inside `cargo test` output, one
/// [`TestCase`] per case file, named by its path (`tests/cmd/help.toml`).
/// Colour codes are stripped, and the `:setup`/`:teardown` lines of a
/// case's sandbox are left out.
pub fn parse_trycmd_output(text: &str) -> Vec<TestCase> {
    text.lines()
        .filter_map(|line| {
            let line = strip_ansi(line);
            let rest = line.trim().strip_prefix("Testing ")?;
            let (name, result) = rest.split_once(" ... ")?;
            let name = name.trim();
            if !name.ends_with(".toml") && !name.ends_with(".trycmd") && !name.ends_with(".md") {
                return None;
            }
            let outcome = match result.split_whitespace().next()? {
                "ok" => Outcome::Passed,
                "failed" => Outcome::Failed,
                "ignored" => Outcome::Skipped,
                _ => return None,
            };
            Some(TestCase {
                name: name.to_string(),
                outcome,
            })
        })
        .collect()
}

/// Removes ANSI escape sequences (`ESC [ ... letter`).
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
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

    const BUILD_SCRIPT_FAILED: &str = "\
   Compiling cookbook_data_md v0.1.0 (/repo/cookbook/data_md)
error: failed to run custom build command for `cookbook_data_md v0.1.0 (/repo/cookbook/data_md)`

Caused by:
  process didn't exit successfully: `/repo/target/debug/build/x/build-script-build` (exit status: 101)
  --- stderr
  thread 'main' panicked at build.rs:40:9:
  day-200.md has no day tag
";

    const COMPILE_FAILED: &str = "\
   Compiling cookbook v0.1.0 (/repo/cookbook)
\u{1b}[1m\u{1b}[31merror[E0425]\u{1b}[0m: cannot find value `days` in this scope
  --> src/lib.rs:12:5
error: could not compile `cookbook` (lib test) due to 1 previous error
";

    #[test]
    fn finds_a_build_that_failed_before_its_tests() {
        assert_eq!(
            cargo_build_failure(BUILD_SCRIPT_FAILED).as_deref(),
            Some(
                "error: failed to run custom build command for `cookbook_data_md v0.1.0 (/repo/cookbook/data_md)`"
            )
        );
        assert!(parse_cargo_output(BUILD_SCRIPT_FAILED).is_empty());
        assert_eq!(
            cargo_build_failure(COMPILE_FAILED).as_deref(),
            Some("error[E0425]: cannot find value `days` in this scope")
        );
    }

    #[test]
    fn a_build_that_ran_its_tests_did_not_fail() {
        assert_eq!(cargo_build_failure(CARGO), None);
        assert_eq!(cargo_build_failure(""), None);
        // A test that prints a compiler error is still a test run.
        let text = "error: could not compile `x`\ntest result: ok. 1 passed; 0 failed\n";
        assert_eq!(cargo_build_failure(text), None);
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

    #[test]
    fn reads_trycmd_cases_from_cargo_output() {
        let text = "\
running 1 test
Testing tests/cmd/HOME-001-score.toml ... ok 16ms 614us 3ns
Testing tests/cmd/HOME-001-score.toml:teardown ... ok
\u{1b}[2mTesting\u{1b}[0m tests/cmd/HOME-002-help.toml ... \u{1b}[31mfailed\u{1b}[0m 2ms
Testing tests/cmd/skip.toml ... ignored
Testing is a word in a sentence ... ok
test cli_transcripts ... FAILED
";
        assert_eq!(
            parse_trycmd_output(text),
            vec![
                TestCase {
                    name: "tests/cmd/HOME-001-score.toml".into(),
                    outcome: Outcome::Passed,
                },
                TestCase {
                    name: "tests/cmd/HOME-002-help.toml".into(),
                    outcome: Outcome::Failed,
                },
                TestCase {
                    name: "tests/cmd/skip.toml".into(),
                    outcome: Outcome::Skipped,
                },
            ]
        );
        assert!(
            parse_cargo_output(text)
                .iter()
                .all(|c| c.name == "cli_transcripts")
        );
    }
}
