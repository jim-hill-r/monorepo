//! `cost check`: reads the files and runs the rules in `cost_model`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use cost_model::{
    COST_FILE, Decision, Detection, Detectors, Finding, Month, SourceFile, check_account,
    check_drift, check_project, parse_detectors, parse_limits, parse_radar,
};

use crate::git;

pub const LIMITS: &str = "standards/radar/limits.toml";
pub const RADAR: &str = "standards/radar/radar.json";
pub const DETECTORS: &str = "standards/radar/detectors.toml";
pub const WORKFLOWS: &str = ".github/workflows";

/// The detectors, or `None` when the repo has no detectors.toml.
pub fn load_detectors(root: &Path) -> Result<Option<Detectors>> {
    let Ok(text) = fs::read_to_string(root.join(DETECTORS)) else {
        return Ok(None);
    };
    parse_detectors(&text)
        .map(Some)
        .map_err(|e| anyhow::anyhow!("{DETECTORS}: {e}"))
}

/// Builds `SourceFile`s from paths and their text.
pub fn sources(files: impl IntoIterator<Item = (String, String)>) -> Vec<SourceFile> {
    files
        .into_iter()
        .map(|(path, text)| SourceFile { path, text })
        .collect()
}

/// The repo's workflows in the working tree.
pub fn workflows_here(root: &Path) -> Vec<SourceFile> {
    sources(git::read_here(root, &git::tracked(root, WORKFLOWS)))
}

/// Runs the detectors over a project's tracked files in the working tree.
pub fn detect_here(
    root: &Path,
    name: &str,
    detectors: &Detectors,
    workflows: &[SourceFile],
) -> Detection {
    let paths: Vec<String> = git::tracked(root, name)
        .into_iter()
        .filter(|p| detectors.scans(p))
        .collect();
    detectors.detect(name, &sources(git::read_here(root, &paths)), workflows)
}

pub struct Report {
    pub findings: Vec<Finding>,
    /// How many projects the per-project rules ran on.
    pub checked: usize,
}

/// Every top-level directory with a `mise.toml`, sorted.
pub fn projects(root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || !entry.path().join("mise.toml").is_file() {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

/// The top-level projects `selected` names, or every project.
fn selection(all: &[String], selected: Option<&[String]>) -> BTreeSet<String> {
    let Some(selected) = selected else {
        return all.iter().cloned().collect();
    };
    selected
        .iter()
        .filter_map(|s| {
            let top = s.trim().trim_start_matches("./").split('/').next()?;
            all.iter().find(|p| p.as_str() == top).cloned()
        })
        .collect()
}

fn decision(root: &Path, link: &str) -> Decision {
    let Ok(text) = fs::read_to_string(root.join(link)) else {
        return Decision::Missing;
    };
    let accepted = text.lines().any(|line| {
        let line = line
            .trim_start_matches(['-', '*', ' '])
            .to_ascii_lowercase();
        line.strip_prefix("status:")
            .is_some_and(|rest| rest.trim().starts_with("accepted"))
    });
    if accepted {
        Decision::Accepted
    } else {
        Decision::NotAccepted
    }
}

pub fn check(root: &Path, selected: Option<&[String]>, today: NaiveDate) -> Result<Report> {
    let read =
        |rel: &str| fs::read_to_string(root.join(rel)).with_context(|| format!("reading {rel}"));
    let limits = parse_limits(&read(LIMITS)?).map_err(|e| anyhow::anyhow!("{LIMITS}: {e}"))?;
    let radar = parse_radar(&read(RADAR)?).map_err(|e| anyhow::anyhow!("{RADAR}: {e}"))?;

    let detectors = load_detectors(root)?;
    let workflows = if detectors.is_some() {
        workflows_here(root)
    } else {
        Vec::new()
    };

    let all = projects(root)?;
    let chosen = selection(&all, selected);
    let mut findings = Vec::new();
    let mut parsed = Vec::new();
    for name in &all {
        let text = fs::read_to_string(root.join(name).join(COST_FILE)).ok();
        let (project_findings, project) = check_project(
            name,
            text.as_deref(),
            &radar,
            &limits,
            &|link| decision(root, link),
            today,
        );
        if chosen.contains(name) {
            findings.extend(project_findings);
            if let (Some(detectors), Some(project)) = (&detectors, &project) {
                let found = detect_here(root, name, detectors, &workflows);
                findings.extend(check_drift(name, &project.file, &found));
            }
        }
        parsed.extend(project);
    }
    let month = Month::new(today.year(), today.month()).context("today's month is out of range")?;
    findings.extend(check_account(&parsed, &radar, &limits, month));
    Ok(Report {
        findings,
        checked: chosen.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(
            root,
            RADAR,
            r#"[{"name": "Squarespace", "ring": "Adopt", "kind": "vendor", "budget": 0},
                {"name": "Cloudflare Pages", "ring": "Adopt", "kind": "vendor", "budget": 0}]"#,
        );
        write(
            root,
            LIMITS,
            "[[limit]]\nkey = \"pages_projects\"\nblip = \"Cloudflare Pages\"\nitem = \"x\"\nfree = 100\nbudget = 1\nscope = \"account\"\nunit = \"projects\"\npast = \"x\"\nsource = \"x\"\nchecked = \"2026-11-26\"\n",
        );
        for p in ["cookbook", "pane", "homer"] {
            write(root, &format!("{p}/mise.toml"), "");
        }
        write(root, ".github/mise.toml", "");
        let site = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\npages_projects = 1\n";
        write(root, "cookbook/docs/cost.toml", site);
        write(root, "pane/docs/cost.toml", site);
        dir
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 26).expect("date")
    }

    fn codes(report: &Report) -> Vec<(&'static str, String)> {
        report
            .findings
            .iter()
            .map(|f| (f.code.as_str(), f.path.clone()))
            .collect()
    }

    #[test]
    fn test_projects_are_top_level_dirs_with_mise_toml() {
        let dir = fixture();
        assert_eq!(
            projects(dir.path()).expect("projects"),
            ["cookbook", "homer", "pane"]
        );
    }

    #[test]
    fn test_check_reports_missing_files_and_account_totals() {
        let dir = fixture();
        let report = check(dir.path(), None, today()).expect("check");
        assert_eq!(report.checked, 3);
        assert_eq!(
            codes(&report),
            [
                ("COST-001", "homer/docs/cost.toml".to_string()),
                ("COST-005", LIMITS.to_string()),
            ]
        );
    }

    #[test]
    fn test_selected_projects_limit_per_project_rules_only() {
        let dir = fixture();
        let selected = ["cookbook/web".to_string(), "nope".to_string()];
        let report = check(dir.path(), Some(&selected), today()).expect("check");
        assert_eq!(report.checked, 1);
        assert_eq!(codes(&report), [("COST-005", LIMITS.to_string())]);
    }

    #[test]
    fn test_drift_runs_for_selected_projects_only() {
        let dir = fixture();
        let root = dir.path();
        write(
            root,
            DETECTORS,
            "[scan]\nextensions = [\"rs\"]\nskip_dirs = []\nskip_files = []\n[[detector]]\nid = \"sq\"\nblip = \"Squarespace\"\nwhy = \"x\"\nhosts = [\"squarespace.com\"]\n",
        );
        write(
            root,
            "pane/src/main.rs",
            "let a = \"https://api.squarespace.com/x\";\nlet b = \"https://api.unknown.dev/y\";\n",
        );
        let selected = ["pane".to_string()];
        let report = check(root, Some(&selected), today()).expect("check");
        assert_eq!(
            codes(&report),
            [
                ("COST-007", "pane/docs/cost.toml".to_string()),
                ("COST-009", "pane/src/main.rs".to_string()),
                ("COST-005", LIMITS.to_string()),
            ]
        );
        let selected = ["cookbook".to_string()];
        let report = check(root, Some(&selected), today()).expect("check");
        assert_eq!(codes(&report), [("COST-005", LIMITS.to_string())]);
    }

    #[test]
    fn test_decision_status() {
        let dir = fixture();
        let root = dir.path();
        write(
            root,
            "d/yes.md",
            "# 0009\n\n- Status: Accepted\n- Date: x\n",
        );
        write(root, "d/no.md", "# 0010\n\n- Status: Proposed\n");
        assert_eq!(decision(root, "d/yes.md"), Decision::Accepted);
        assert_eq!(decision(root, "d/no.md"), Decision::NotAccepted);
        assert_eq!(decision(root, "d/none.md"), Decision::Missing);
    }
}
