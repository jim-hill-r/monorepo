//! The `evals-results` data branch: `results/<project>/<yyyy-mm>.jsonl`
//! (one line per result, appended) and `verdicts.json` (every requirement's
//! current verdict). This module reads and writes a checkout of that branch;
//! `standards/scripts/evals.sh store` commits and pushes it. See "Results"
//! in the eval engine plan. Files, never a database.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use evals_model::{
    EvalResult, HumanChecks, Priority, Stage, Thresholds, Verdict, parse_evals, parse_requirements,
    requirement_order, verdict,
};
use serde::{Deserialize, Serialize};

use crate::check::{EVALS, REQUIREMENTS, read_optional, top_level_dirs};
use crate::coverage::{Coverage, RepoCoverage, summary};
use crate::human_checks::{read_checks, read_settings};
use crate::publish::{parse_results, to_jsonl};

/// The results folder inside a checkout of the branch.
pub const RESULTS_DIR: &str = "results";
/// The verdicts file inside a checkout of the branch.
pub const VERDICTS_FILE: &str = "verdicts.json";
/// The thresholds, relative to the repo root.
pub const THRESHOLDS: &str = "evals/verdicts.toml";

/// Appends each result to `results/<project>/<yyyy-mm>.jsonl` under `dir`,
/// by the month of its `at`, and returns the files it wrote, relative to
/// `dir`.
pub fn append_results(dir: &Path, results: &[EvalResult]) -> Result<Vec<String>> {
    let mut files: BTreeMap<String, Vec<EvalResult>> = BTreeMap::new();
    for result in results {
        if !is_project_name(&result.project) {
            bail!(
                "result for `{}`: not a project directory name",
                result.project
            );
        }
        let file = format!(
            "{RESULTS_DIR}/{}/{}.jsonl",
            result.project,
            result.at.format("%Y-%m")
        );
        files.entry(file).or_default().push(result.clone());
    }
    for (file, rows) in &files {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let mut out = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;
        out.write_all(to_jsonl(rows)?.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(files.into_keys().collect())
}

/// A top-level project directory name: snake_case, so it is safe as a path.
fn is_project_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Reads every `results/<project>/<yyyy-mm>.jsonl` under `dir`. The yearly
/// archives (`results/<project>/archive/`) are older than any verdict
/// looks, so they are not read. A missing folder means no results yet.
pub fn read_results(dir: &Path) -> Result<Vec<EvalResult>> {
    let results_dir = dir.join(RESULTS_DIR);
    if !results_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for project in
        fs::read_dir(&results_dir).with_context(|| format!("listing {}", results_dir.display()))?
    {
        let project = project?.path();
        if !project.is_dir() {
            continue;
        }
        for file in
            fs::read_dir(&project).with_context(|| format!("listing {}", project.display()))?
        {
            let file = file?.path();
            if file.is_file() && file.extension().is_some_and(|e| e == "jsonl") {
                files.push(file);
            }
        }
    }
    files.sort();
    let mut results = Vec::new();
    for file in files {
        let text =
            fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
        results.extend(parse_results(&text).with_context(|| file.display().to_string())?);
    }
    Ok(results)
}

/// `verdicts.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictsFile {
    pub generated_at: DateTime<Utc>,
    /// The repo's eval coverage, over every project.
    pub coverage: RepoCoverage,
    pub projects: Vec<ProjectVerdicts>,
}

/// One project's verdicts, with its coverage beside them so a project with
/// few evals never looks healthier than one with many.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectVerdicts {
    pub project: String,
    /// Active `must` requirements.
    pub musts: usize,
    /// Active `must` requirements with an eval in a production stage.
    pub musts_in_production: usize,
    /// The project's eval coverage (`coverage.rs`).
    pub coverage: Coverage,
    pub requirements: Vec<RequirementVerdict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementVerdict {
    pub requirement: String,
    pub title: String,
    pub priority: Priority,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

/// Computes every project's verdicts from the requirements, evals and
/// human-check answers on `main` (the repo at `root`) and the stored
/// `results`. Every top-level project (a folder with a `mise.toml` or a
/// `docs/requirements.md`) is listed, so one with no requirements shows
/// 0% eval coverage. Projects whose files don't parse are left out with a
/// warning on standard error (`evals check` reports them).
pub fn compute(
    root: &Path,
    results: &[EvalResult],
    only: Option<&str>,
    now: DateTime<Utc>,
) -> Result<VerdictsFile> {
    let thresholds = read_optional(&root.join(THRESHOLDS))?
        .map(|text| Thresholds::from_toml(&text))
        .transpose()
        .map_err(|e| anyhow::anyhow!("{THRESHOLDS}: {e}"))?
        .unwrap_or_default();
    let settings = read_settings(root).unwrap_or_default();
    let answers = read_checks(root)?;
    let checks = HumanChecks {
        settings: &settings,
        answers: &answers,
    };

    let mut projects = Vec::new();
    for project in top_level_dirs(root)? {
        if only.is_some_and(|p| p != project) {
            continue;
        }
        let dir = root.join(&project);
        let text = match read_optional(&dir.join(REQUIREMENTS))? {
            Some(text) => text,
            None if dir.join("mise.toml").is_file() => String::new(),
            None => continue,
        };
        let (requirements, errors) = parse_requirements(&text);
        if !errors.is_empty() {
            eprintln!("evals: skipping {project}: {project}/{REQUIREMENTS} does not parse");
            continue;
        }
        let evals = match read_optional(&dir.join(EVALS))? {
            None => Vec::new(),
            Some(text) => match parse_evals(&text) {
                Ok(file) => file.evals,
                Err(_) => {
                    eprintln!("evals: skipping {project}: {project}/{EVALS} does not parse");
                    continue;
                }
            },
        };
        let project_results: Vec<EvalResult> = results
            .iter()
            .filter(|r| r.project == project)
            .cloned()
            .collect();

        let mut verdicts: Vec<RequirementVerdict> = requirements
            .iter()
            .filter(|r| r.is_active())
            .map(|requirement| {
                let report = verdict(
                    requirement,
                    &evals,
                    &project_results,
                    checks,
                    &thresholds,
                    now,
                );
                RequirementVerdict {
                    requirement: requirement.id.clone(),
                    title: requirement.title.clone(),
                    priority: requirement.priority,
                    verdict: report.verdict,
                    reasons: report.reasons,
                }
            })
            .collect();
        verdicts.sort_by_key(|v| requirement_order(&v.requirement));

        let musts: Vec<&str> = requirements
            .iter()
            .filter(|r| r.is_active() && r.priority == Priority::Must)
            .map(|r| r.id.as_str())
            .collect();
        let musts_in_production = musts
            .iter()
            .filter(|id| {
                evals.iter().any(|e| {
                    e.requirement == **id && e.stages.iter().any(|s: &Stage| s.is_production())
                })
            })
            .count();
        projects.push(ProjectVerdicts {
            project,
            musts: musts.len(),
            musts_in_production,
            coverage: Coverage::of(&verdicts, thresholds.coverage_warn),
            requirements: verdicts,
        });
    }
    let coverage = RepoCoverage::of(
        &projects
            .iter()
            .map(|p| p.coverage.clone())
            .collect::<Vec<_>>(),
        projects.iter().map(|p| p.musts).sum(),
        thresholds.coverage_warn,
    );
    Ok(VerdictsFile {
        generated_at: now,
        coverage,
        projects,
    })
}

/// Writes `verdicts.json` under `dir`, pretty-printed so its diffs read well.
pub fn write_verdicts(dir: &Path, file: &VerdictsFile) -> Result<()> {
    let path = dir.join(VERDICTS_FILE);
    let mut text = serde_json::to_string_pretty(file)?;
    text.push('\n');
    fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}

/// Plain-text verdicts: one block per project, one line per requirement.
pub fn render(file: &VerdictsFile) -> String {
    if file.projects.is_empty() {
        return "No projects found.\n".to_string();
    }
    let mut out = String::new();
    for project in &file.projects {
        let coverage = &project.coverage;
        let warning = if coverage.warning { ", warning" } else { "" };
        if project.requirements.is_empty() {
            out.push_str(&format!(
                "{}: no requirements yet (eval coverage 0%{warning})\n",
                project.project
            ));
            continue;
        }
        let count = |v| {
            project
                .requirements
                .iter()
                .filter(|r| r.verdict == v)
                .count()
        };
        out.push_str(&format!(
            "{}: {} met, {} at risk, {} not met, {} unknown ({} of {} musts evaluated in production; eval coverage {}{warning})\n",
            project.project,
            count(Verdict::Met),
            count(Verdict::AtRisk),
            count(Verdict::NotMet),
            count(Verdict::Unknown),
            project.musts_in_production,
            project.musts,
            summary(coverage.percent, coverage.musts_percent),
        ));
        for r in &project.requirements {
            let reasons = if r.reasons.is_empty() {
                String::new()
            } else {
                format!(": {}", r.reasons.join("; "))
            };
            out.push_str(&format!(
                "  {} {:<8} {} ({}){reasons}\n",
                r.requirement,
                verdict_word(r.verdict),
                r.title,
                priority_word(r.priority),
            ));
        }
    }
    let repo = &file.coverage;
    if file.projects.len() > 1 {
        out.push_str(&format!(
            "eval coverage: {} over {} projects ({} with requirements); {:.0}% of musts met per project on average, warning below {:.0}%{}\n",
            summary(repo.percent, repo.musts_percent),
            repo.projects,
            repo.projects_with_requirements,
            repo.judged_percent,
            repo.warn_below,
            if repo.warning { " (warning)" } else { "" },
        ));
    }
    out
}

fn verdict_word(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Met => "met",
        Verdict::AtRisk => "at-risk",
        Verdict::NotMet => "not-met",
        Verdict::Unknown => "unknown",
    }
}

fn priority_word(priority: Priority) -> &'static str {
    match priority {
        Priority::Must => "must",
        Priority::Should => "should",
        Priority::Could => "could",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const REQUIREMENTS_MD: &str = "# Cookbook requirements

## COOK-001: A recipe is easy to cook from on a phone

- Priority: must
- Why: Jim cooks from his phone.
- Accept: no sideways scrolling.

## COOK-002: Sign-in works

- Priority: should
- Why: Jim saves favourites.
- Accept: signing in returns to the recipe.

## COOK-003: Recipes can be printed

- Priority: could
- Why: For cooking away from a phone.
- Accept: fits on one page.
- Status: retired (2026-12-01, nobody prints recipes)
";

    const EVALS_TOML: &str = r#"prefix = "COOK"

[[eval]]
id = "layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "auth0-callbacks"
requirement = "COOK-002"
stages = ["human"]
kind = "human-check"
setting = "auth0-app"
"#;

    const SETTINGS_TOML: &str = r#"[[setting]]
id = "auth0-app"
vendor = "Auth0"
where = "Applications"
expect = "The callback URL is listed."
why = "Sign-in fails without it."
every = "30d"
"#;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 10, 12, 0, 0)
            .single()
            .unwrap_or_default()
    }

    fn line(eval: &str, stage: &str, status: &str, at: &str) -> String {
        format!(
            r#"{{"project":"cookbook","requirement":"COOK-001","eval":"{eval}","stage":"{stage}","kind":"playwright","commit":"c","status":"{status}","at":"{at}","by":"ci"}}"#
        )
    }

    fn repo() -> Result<tempfile::TempDir> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("cookbook/docs"))?;
        fs::create_dir_all(root.path().join("standards"))?;
        fs::create_dir_all(root.path().join("evals"))?;
        fs::write(
            root.path().join("cookbook/docs/requirements.md"),
            REQUIREMENTS_MD,
        )?;
        fs::write(root.path().join("cookbook/evals.toml"), EVALS_TOML)?;
        fs::write(root.path().join("standards/settings.toml"), SETTINGS_TOML)?;
        fs::write(
            root.path().join("evals/verdicts.toml"),
            include_str!("../../verdicts.toml"),
        )?;
        Ok(root)
    }

    #[test]
    fn appends_by_project_and_month() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let text = [
            line("layout", "ci", "pass", "2026-11-30T23:00:00Z"),
            line("layout", "cd", "pass", "2026-12-01T01:00:00Z"),
        ]
        .join("\n");
        let results = parse_results(&text)?;
        let files = append_results(dir.path(), &results)?;
        assert_eq!(
            files,
            vec![
                "results/cookbook/2026-11.jsonl",
                "results/cookbook/2026-12.jsonl"
            ]
        );
        append_results(dir.path(), &results[1..])?;
        let december = fs::read_to_string(dir.path().join("results/cookbook/2026-12.jsonl"))?;
        assert_eq!(december.lines().count(), 2);
        assert_eq!(read_results(dir.path())?.len(), 3);
        Ok(())
    }

    #[test]
    fn rejects_a_project_that_is_not_a_directory_name() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut results = parse_results(&line("layout", "ci", "pass", "2026-12-01T00:00:00Z"))?;
        results[0].project = "../main".into();
        assert!(append_results(dir.path(), &results).is_err());
        Ok(())
    }

    #[test]
    fn reading_skips_archives_and_a_missing_folder() -> Result<()> {
        let dir = tempfile::tempdir()?;
        assert!(read_results(dir.path())?.is_empty());
        fs::create_dir_all(dir.path().join("results/cookbook/archive"))?;
        fs::write(
            dir.path().join("results/cookbook/archive/2024.jsonl.gz"),
            "not json",
        )?;
        assert!(read_results(dir.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn computes_verdicts_with_human_checks() -> Result<()> {
        let root = repo()?;
        let results = parse_results(
            &[
                line("layout", "ci", "pass", "2026-12-10T08:00:00Z"),
                line("layout", "cd", "pass", "2026-12-10T09:00:00Z"),
            ]
            .join("\n"),
        )?;
        // No answer yet: COOK-002 is at risk.
        let file = compute(root.path(), &results, None, now())?;
        assert_eq!(file.projects.len(), 1);
        let project = &file.projects[0];
        assert_eq!((project.musts, project.musts_in_production), (1, 1));
        let ids: Vec<_> = project
            .requirements
            .iter()
            .map(|r| r.requirement.as_str())
            .collect();
        assert_eq!(
            ids,
            vec!["COOK-001", "COOK-002"],
            "retired ones are left out"
        );
        assert_eq!(project.requirements[0].verdict, Verdict::Met);
        assert_eq!(project.requirements[1].verdict, Verdict::AtRisk);

        fs::write(
            root.path().join("standards/settings-checks.jsonl"),
            r#"{"setting":"auth0-app","status":"fail","note":"callback missing","at":"2026-12-09T00:00:00Z","by":"jim"}
"#,
        )?;
        let file = compute(root.path(), &results, None, now())?;
        let cook_002 = &file.projects[0].requirements[1];
        // A should requirement shows a failure as amber.
        assert_eq!(cook_002.verdict, Verdict::AtRisk);
        assert!(
            cook_002.reasons[0].contains("callback missing"),
            "{cook_002:?}"
        );

        let text = render(&file);
        assert!(text.starts_with(
            "cookbook: 1 met, 1 at risk, 0 not met, 0 unknown (1 of 1 musts evaluated in production; eval coverage 50%, musts 100%)\n  COOK-001 met      A recipe is easy to cook from on a phone (must)\n"
        ), "{text}");

        assert!(
            compute(root.path(), &results, Some("pane"), now())?
                .projects
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn a_project_with_no_requirements_shows_zero_coverage() -> Result<()> {
        let root = repo()?;
        fs::create_dir_all(root.path().join("homer"))?;
        fs::write(root.path().join("homer/mise.toml"), "")?;
        fs::create_dir_all(root.path().join("notes"))?;
        let results = parse_results(
            &[
                line("layout", "ci", "pass", "2026-12-10T08:00:00Z"),
                line("layout", "cd", "pass", "2026-12-10T09:00:00Z"),
            ]
            .join("\n"),
        )?;
        let file = compute(root.path(), &results, None, now())?;
        let names: Vec<_> = file.projects.iter().map(|p| p.project.as_str()).collect();
        assert_eq!(
            names,
            vec!["cookbook", "homer"],
            "a folder with neither mise.toml nor requirements is not a project"
        );
        let homer = &file.projects[1];
        assert_eq!(homer.coverage.percent, 0.0);
        assert!(homer.coverage.warning);
        // cookbook 50% (1 of 2 met), homer 0%: the mean is 25%.
        assert_eq!(file.coverage.percent, 25.0);
        assert_eq!(file.coverage.musts_percent, Some(100.0));
        let text = render(&file);
        assert!(
            text.contains("homer: no requirements yet (eval coverage 0%, warning)\n"),
            "{text}"
        );
        assert!(
            text.ends_with("eval coverage: 25%, musts 100% over 2 projects (1 with requirements); 50% of musts met per project on average, warning below 80% (warning)\n"),
            "{text}"
        );

        let dir = tempfile::tempdir()?;
        let written = crate::coverage::write_badges(dir.path(), &file)?;
        assert_eq!(
            written,
            vec!["badge.svg", "badges/cookbook.svg", "badges/homer.svg"]
        );
        let homer_badge = fs::read_to_string(dir.path().join("badges/homer.svg"))?;
        assert!(
            homer_badge.contains("eval coverage: 0%, no requirements"),
            "{homer_badge}"
        );
        Ok(())
    }

    #[test]
    fn writes_verdicts_json() -> Result<()> {
        let root = repo()?;
        let dir = tempfile::tempdir()?;
        let file = compute(root.path(), &[], None, now())?;
        write_verdicts(dir.path(), &file)?;
        let text = fs::read_to_string(dir.path().join(VERDICTS_FILE))?;
        let back: VerdictsFile = serde_json::from_str(&text)?;
        assert_eq!(back, file);
        assert!(text.contains("\"verdict\": \"unknown\""), "{text}");
        Ok(())
    }
}
