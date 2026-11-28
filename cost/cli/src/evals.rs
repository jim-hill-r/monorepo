//! `cost check --evals-results`: one eval result per `cost` eval, in the eval
//! engine's results format (standards/docs/evals.md), so the engine doesn't
//! run the cost rules a second time.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use cost_model::Finding;
use evals_model::{By, EvalResult, Kind, ResultStatus, Stage, parse_evals};

/// Where and when the check ran, copied into every result.
#[derive(Debug, Clone)]
pub struct RunContext {
    pub commit: String,
    pub run: Option<String>,
    pub evidence: Option<String>,
    pub at: DateTime<Utc>,
}

impl RunContext {
    /// The GitHub Actions run from the environment, or a local run.
    pub fn from_env(commit: Option<String>, at: DateTime<Utc>) -> Self {
        let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        let run = env("GITHUB_RUN_ID");
        let evidence = match (env("GITHUB_SERVER_URL"), env("GITHUB_REPOSITORY"), &run) {
            (Some(server), Some(repo), Some(id)) => {
                Some(format!("{server}/{repo}/actions/runs/{id}"))
            }
            _ => None,
        };
        Self {
            commit: commit
                .or_else(|| env("GITHUB_SHA"))
                .unwrap_or_else(|| "unknown".into()),
            run,
            evidence,
            at,
        }
    }
}

/// The `ci` results for every `cost` eval of `projects`. A project with no
/// `evals.toml`, or one that doesn't parse (`evals check` reports that),
/// gets none.
pub fn results(
    root: &Path,
    projects: &[String],
    findings: &[Finding],
    ctx: &RunContext,
) -> Vec<EvalResult> {
    let mut out = Vec::new();
    for project in projects {
        let Ok(text) = fs::read_to_string(root.join(project).join("evals.toml")) else {
            continue;
        };
        let Ok(file) = parse_evals(&text) else {
            continue;
        };
        let evals: Vec<_> = file
            .evals
            .iter()
            .filter(|e| matches!(e.kind, Kind::Cost {}) && e.stages.contains(&Stage::Ci))
            .collect();
        if evals.is_empty() {
            continue;
        }
        let (status, note) = verdict(project, findings);
        for eval in evals {
            out.push(EvalResult {
                project: project.clone(),
                requirement: eval.requirement.clone(),
                eval: eval.id.clone(),
                stage: Stage::Ci,
                kind: "cost".into(),
                commit: ctx.commit.clone(),
                deploy: None,
                status,
                value: None,
                threshold: None,
                evidence: ctx.evidence.clone(),
                run: ctx.run.clone(),
                note: Some(note.clone()),
                at: ctx.at,
                by: By::Ci,
            });
        }
    }
    out
}

/// Fails on any finding in the project's files that `--strict` would fail
/// on; the warning-only codes (COST-008 to COST-010) are listed in the note
/// but pass. Account-wide findings belong to no one project and don't count.
fn verdict(project: &str, findings: &[Finding]) -> (ResultStatus, String) {
    let prefix = format!("{project}/");
    let mine: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.path.starts_with(&prefix))
        .collect();
    let codes = |fail: bool| {
        let mut codes: Vec<&str> = mine
            .iter()
            .filter(|f| f.code.can_fail() == fail)
            .map(|f| f.code.as_str())
            .collect();
        codes.sort_unstable();
        codes.dedup();
        codes
    };
    let (failing, warnings) = (codes(true), codes(false));
    let status = if failing.is_empty() {
        ResultStatus::Pass
    } else {
        ResultStatus::Fail
    };
    let note = match (failing.is_empty(), warnings.is_empty()) {
        (true, true) => "no cost findings".to_string(),
        (true, false) => format!("warnings only: {}", warnings.join(", ")),
        (false, true) => format!("cost check found {}", failing.join(", ")),
        (false, false) => format!(
            "cost check found {}; warnings: {}",
            failing.join(", "),
            warnings.join(", ")
        ),
    };
    (status, note)
}

/// Appends the results to a JSON Lines file, creating it and its folder.
pub fn append(path: &Path, results: &[EvalResult]) -> Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    for result in results {
        writeln!(file, "{}", serde_json::to_string(result)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cost_model::Code;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn finding(code: Code, path: &str) -> Finding {
        Finding {
            code,
            path: path.into(),
            message: "x".into(),
        }
    }

    fn ctx() -> RunContext {
        RunContext {
            commit: "abc1234".into(),
            run: Some("42".into()),
            evidence: None,
            at: DateTime::<Utc>::default(),
        }
    }

    const EVALS: &str = r#"prefix = "COOK"

[[eval]]
id = "free"
requirement = "COOK-010"
stages = ["ci", "observe"]
kind = "cost"

[[eval]]
id = "free-actuals"
requirement = "COOK-011"
stages = ["observe"]
kind = "cost"

[[eval]]
id = "units"
requirement = "COOK-002"
stages = ["ci"]
kind = "cargo"
"#;

    #[test]
    fn test_one_ci_result_per_cost_eval() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(root, "cookbook/evals.toml", EVALS);
        write(root, "pane/evals.toml", "this is not toml [");
        let projects = ["cookbook".to_string(), "pane".into(), "homer".into()];
        let results = results(root, &projects, &[], &ctx());
        assert_eq!(results.len(), 1, "{results:?}");
        let r = &results[0];
        assert_eq!(
            (r.project.as_str(), r.requirement.as_str(), r.eval.as_str()),
            ("cookbook", "COOK-010", "free")
        );
        assert_eq!(
            (r.stage, r.kind.as_str(), r.by),
            (Stage::Ci, "cost", By::Ci)
        );
        assert_eq!(r.status, ResultStatus::Pass);
        assert_eq!(r.note.as_deref(), Some("no cost findings"));
        assert_eq!(
            (r.commit.as_str(), r.run.as_deref()),
            ("abc1234", Some("42"))
        );
    }

    #[test]
    fn test_project_findings_that_can_fail_fail_the_eval() {
        let findings = [
            finding(Code::Cost007, "cookbook/docs/cost.toml"),
            finding(Code::Cost009, "cookbook/web/src/main.rs"),
            finding(Code::Cost001, "cookbook2/docs/cost.toml"),
            finding(Code::Cost005, "standards/radar/limits.toml"),
            finding(Code::Cost007, "cookbook/docs/cost.toml"),
        ];
        assert_eq!(
            verdict("cookbook", &findings),
            (
                ResultStatus::Fail,
                "cost check found COST-007; warnings: COST-009".to_string()
            )
        );
        assert_eq!(
            verdict("cookbook", &findings[1..2]),
            (ResultStatus::Pass, "warnings only: COST-009".to_string())
        );
        assert_eq!(
            verdict("cookbook", &findings[2..4]),
            (ResultStatus::Pass, "no cost findings".to_string())
        );
        assert_eq!(
            verdict("cookbook2", &findings),
            (ResultStatus::Fail, "cost check found COST-001".to_string())
        );
    }

    #[test]
    fn test_append_adds_json_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(root, "cookbook/evals.toml", EVALS);
        let results = results(root, &["cookbook".to_string()], &[], &ctx());
        let path = root.join("out/evals-results.jsonl");
        append(&path, &results).expect("append");
        append(&path, &results).expect("append");
        let text = fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        let back: EvalResult = serde_json::from_str(lines[1]).expect("json");
        assert_eq!(back, results[0]);
    }
}
