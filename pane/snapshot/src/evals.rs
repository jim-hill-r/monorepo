//! Reads a checkout of the `evals-results` branch into
//! [`pane_model::requirements::Requirements`].
//!
//! The branch holds `verdicts.json` (each project's requirements and their
//! verdicts) and `results/<project>/<yyyy-mm>.jsonl` (one line per eval
//! run); see "Results" in `standards/docs/evals.md`. [`parse`] is pure over
//! file contents; [`read`] finds the files.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use pane_model::requirements::{EvalRun, EvalRuns, HISTORY, Requirements};
use serde::Deserialize;

/// The verdicts file at the branch root.
pub const VERDICTS: &str = "verdicts.json";
/// The folder of results files at the branch root.
pub const RESULTS: &str = "results";

/// Runs by `(eval, stage)`.
type EvalMap = BTreeMap<(String, String), Vec<EvalRun>>;

/// The fields of a results line pane needs beyond the run itself.
#[derive(Deserialize)]
struct ResultLine {
    project: String,
    requirement: String,
    eval: String,
    stage: String,
    #[serde(flatten)]
    run: EvalRun,
}

/// Builds the requirements from `verdicts.json` and every results file's
/// `(name, contents)`, attaching to each requirement its evals' newest
/// [`HISTORY`] runs per stage, newest first. Results for a requirement the
/// verdicts don't list are dropped (it was removed or renamed). A file that
/// doesn't parse is an error naming it, so bad data never shows as missing.
pub fn parse(verdicts: &str, results: &[(String, String)]) -> Result<Requirements, String> {
    let mut requirements: Requirements =
        serde_json::from_str(verdicts).map_err(|e| format!("{VERDICTS}: {e}"))?;

    // (project, requirement) -> (eval, stage) -> runs
    let mut runs: BTreeMap<(String, String), EvalMap> = BTreeMap::new();
    for (name, text) in results {
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let r: ResultLine =
                serde_json::from_str(line).map_err(|e| format!("{name}:{}: {e}", i + 1))?;
            runs.entry((r.project, r.requirement))
                .or_default()
                .entry((r.eval, r.stage))
                .or_default()
                .push(r.run);
        }
    }

    for project in &mut requirements.projects {
        for req in &mut project.requirements {
            let Some(evals) = runs.remove(&(project.project.clone(), req.id.clone())) else {
                continue;
            };
            req.evals = evals
                .into_iter()
                .map(|((eval, stage), mut runs)| {
                    // RFC 3339 times in UTC sort as text; the newest first.
                    runs.sort_by(|a, b| b.at.cmp(&a.at));
                    runs.truncate(HISTORY);
                    EvalRuns { eval, stage, runs }
                })
                .collect();
        }
    }
    Ok(requirements)
}

/// Reads the checkout at `dir`. `Ok(None)` when it has no `verdicts.json`
/// (the branch wasn't fetched), so the Requirements light is grey.
pub fn read(dir: &Path) -> Result<Option<Requirements>, String> {
    let verdicts = dir.join(VERDICTS);
    if !verdicts.exists() {
        return Ok(None);
    }
    let verdicts =
        fs::read_to_string(&verdicts).map_err(|e| format!("{}: {e}", verdicts.display()))?;
    let mut files = Vec::new();
    let results = dir.join(RESULTS);
    if results.is_dir() {
        collect_jsonl(&results, &results, &mut files)?;
    }
    files.sort();
    parse(&verdicts, &files).map(Some)
}

fn collect_jsonl(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_jsonl(base, &path, out)?;
        } else if path.extension().is_some_and(|x| x == "jsonl") {
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let name = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .display()
                .to_string();
            out.push((format!("{RESULTS}/{name}"), text));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pane_model::requirements::{RunStatus, Verdict};

    const VERDICTS_JSON: &str = r#"{"generated_at":"2026-11-28T04:00:00Z","projects":[
      {"project":"pane","musts":1,"musts_in_production":1,
       "coverage":{"requirements":2,"met":1,"percent":50.0,"musts_met":1,"musts_percent":100.0,"warning":false},
       "requirements":[
         {"requirement":"PANE-001","title":"Today","priority":"must","verdict":"met"},
         {"requirement":"PANE-002","title":"Main","priority":"should","verdict":"at-risk","reasons":["flaky"]}]}]}"#;

    fn line(req: &str, eval: &str, stage: &str, status: &str, at: &str) -> String {
        format!(
            r#"{{"project":"pane","requirement":"{req}","eval":"{eval}","stage":"{stage}","kind":"cargo","commit":"abc","status":"{status}","evidence":"https://e/{at}","at":"{at}","by":"ci"}}"#
        )
    }

    #[test]
    fn attaches_runs_per_eval_and_stage_newest_first() -> Result<(), String> {
        let nov = [
            line("PANE-001", "today", "ci", "pass", "2026-11-26T01:00:00Z"),
            line("PANE-001", "today", "ci", "fail", "2026-11-27T01:00:00Z"),
            line("PANE-001", "today", "cd", "pass", "2026-11-27T02:00:00Z"),
            line("PANE-009", "gone", "ci", "pass", "2026-11-27T02:00:00Z"),
            String::new(),
        ]
        .join("\n");
        let oct = line("PANE-001", "today", "ci", "pass", "2026-10-30T01:00:00Z");
        let r = parse(
            VERDICTS_JSON,
            &[
                ("results/pane/2026-10.jsonl".into(), oct),
                ("results/pane/2026-11.jsonl".into(), nov),
            ],
        )?;
        let reqs = &r.projects[0].requirements;
        let evals = &reqs[0].evals;
        let keys: Vec<(&str, &str, usize)> = evals
            .iter()
            .map(|e| (e.eval.as_str(), e.stage.as_str(), e.runs.len()))
            .collect();
        assert_eq!(keys, [("today", "cd", 1), ("today", "ci", 3)]);
        let ci = &evals[1];
        assert_eq!(ci.latest().map(|r| r.status), Some(RunStatus::Fail));
        assert_eq!(ci.runs[2].at, "2026-10-30T01:00:00Z");
        assert_eq!(reqs[1].verdict, Verdict::AtRisk);
        assert!(reqs[1].evals.is_empty());
        Ok(())
    }

    #[test]
    fn keeps_only_the_newest_runs() -> Result<(), String> {
        let lines: Vec<String> = (0..HISTORY + 5)
            .map(|i| {
                line(
                    "PANE-001",
                    "today",
                    "ci",
                    "pass",
                    &format!("2026-11-{:02}T00:00:00Z", i + 1),
                )
            })
            .collect();
        let r = parse(
            VERDICTS_JSON,
            &[("results/pane/2026-11.jsonl".into(), lines.join("\n"))],
        )?;
        let runs = &r.projects[0].requirements[0].evals[0].runs;
        assert_eq!(runs.len(), HISTORY);
        assert_eq!(runs[0].at, "2026-11-15T00:00:00Z");
        Ok(())
    }

    #[test]
    fn a_bad_file_names_itself() {
        let bad = parse(
            VERDICTS_JSON,
            &[("results/pane/2026-11.jsonl".into(), "{oops".into())],
        );
        assert!(bad.is_err_and(|e| e.starts_with("results/pane/2026-11.jsonl:1:")));
        assert!(parse("[]", &[]).is_err_and(|e| e.starts_with("verdicts.json:")));
    }
}
