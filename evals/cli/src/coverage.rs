//! Eval coverage: how much of what a project promises is green on
//! production evidence. A project's coverage is the share of its active
//! requirements whose verdict is met (a met verdict needs a fresh
//! production-stage result), with its `must` requirements shown
//! separately. A project with no requirements has 0%. The repo total is the
//! mean over every project, so a project with no requirements pulls it down
//! instead of being left out. `evals store` writes the numbers into
//! `verdicts.json` and the badges `badge.svg` (repo) and
//! `badges/<project>.svg` on the `evals-results` branch, the way
//! `coverage.sh store` does for code coverage. See "Eval coverage" in
//! standards/docs/evals.md.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use evals_model::Verdict;
use serde::{Deserialize, Serialize};

use crate::store::{RequirementVerdict, VerdictsFile};

/// The badges folder inside a checkout of the `evals-results` branch.
pub const BADGES_DIR: &str = "badges";
/// The repo badge inside a checkout of the `evals-results` branch.
pub const REPO_BADGE: &str = "badge.svg";

/// One project's eval coverage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    /// Active requirements.
    pub requirements: usize,
    /// Active requirements whose verdict is met.
    pub met: usize,
    /// `met` as a percentage of `requirements`, 0 when there are none.
    pub percent: f64,
    /// Active `must` requirements whose verdict is met.
    pub musts_met: usize,
    /// `musts_met` as a percentage of the project's musts, absent when it
    /// has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub musts_percent: Option<f64>,
    /// Below the warning level (`coverage_warn` in evals/verdicts.toml),
    /// judged on the musts, or on every requirement when there are no musts.
    pub warning: bool,
}

impl Coverage {
    /// Coverage of one project's verdicts.
    pub fn of(requirements: &[RequirementVerdict], warn: f64) -> Self {
        let met = requirements
            .iter()
            .filter(|r| r.verdict == Verdict::Met)
            .count();
        let musts: Vec<&RequirementVerdict> = requirements
            .iter()
            .filter(|r| r.priority == evals_model::Priority::Must)
            .collect();
        let musts_met = musts.iter().filter(|r| r.verdict == Verdict::Met).count();
        let percent = share(met, requirements.len()).unwrap_or(0.0);
        let musts_percent = share(musts_met, musts.len());
        Coverage {
            requirements: requirements.len(),
            met,
            percent,
            musts_met,
            musts_percent,
            warning: musts_percent.unwrap_or(percent) < warn,
        }
    }

    /// The percentage the badge colour and the warning go by.
    fn judged(&self) -> f64 {
        self.musts_percent.unwrap_or(self.percent)
    }
}

/// The repo's eval coverage, over every project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoCoverage {
    /// Projects counted (every top-level project, with requirements or not).
    pub projects: usize,
    /// Projects with at least one active requirement.
    pub projects_with_requirements: usize,
    /// The mean of the projects' `percent`, 0 when there are no projects.
    pub percent: f64,
    /// Met musts over all musts in the repo, absent when there are none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub musts_percent: Option<f64>,
    /// The mean over projects of the percentage each project's warning goes
    /// by (its musts met, or every requirement when it has no musts, 0 with
    /// no requirements). The repo warning and badge colour go by this, so a
    /// project with no requirements keeps the badge from turning green.
    pub judged_percent: f64,
    /// The warning level from evals/verdicts.toml.
    pub warn_below: f64,
    pub warning: bool,
}

impl RepoCoverage {
    pub fn of(projects: &[Coverage], musts: usize, warn: f64) -> Self {
        let mean = |f: fn(&Coverage) -> f64| {
            if projects.is_empty() {
                0.0
            } else {
                round(projects.iter().map(f).sum::<f64>() / projects.len() as f64)
            }
        };
        let percent = mean(|c| c.percent);
        let judged_percent = mean(Coverage::judged);
        let musts_met = projects.iter().map(|c| c.musts_met).sum();
        let musts_percent = share(musts_met, musts);
        RepoCoverage {
            projects: projects.len(),
            projects_with_requirements: projects.iter().filter(|c| c.requirements > 0).count(),
            percent,
            musts_percent,
            judged_percent,
            warn_below: warn,
            warning: judged_percent < warn,
        }
    }
}

fn share(part: usize, whole: usize) -> Option<f64> {
    (whole > 0).then(|| round(part as f64 * 100.0 / whole as f64))
}

/// Rounds to one decimal place.
fn round(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// `73%`, or `73%, musts 50%` when there are musts.
pub fn summary(percent: f64, musts_percent: Option<f64>) -> String {
    match musts_percent {
        Some(musts) => format!("{percent:.0}%, musts {musts:.0}%"),
        None => format!("{percent:.0}%"),
    }
}

/// Green from the warning level, yellow from three quarters of it, red
/// below: the same bands as the code coverage badge at the default 80%.
fn color(judged: f64, warn: f64) -> &'static str {
    if judged >= warn {
        "#4c1"
    } else if judged >= warn * 0.75 {
        "#dfb317"
    } else {
        "#e05d44"
    }
}

/// A flat badge, the same shape as `coverage.sh badge`.
pub fn badge(label: &str, value: &str, fill: &str) -> String {
    // About 6.5px per character in Verdana 11px, plus padding.
    let lw = label.chars().count() * 13 / 2 + 10;
    let vw = value.chars().count() * 13 / 2 + 10;
    let w = lw + vw;
    let label = escape(label);
    let value = escape(value);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="20" role="img" aria-label="{label}: {value}"><title>{label}: {value}</title><linearGradient id="s" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="r"><rect width="{w}" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#r)"><rect width="{lw}" height="20" fill="#555"/><rect x="{lw}" width="{vw}" height="20" fill="{fill}"/><rect width="{w}" height="20" fill="url(#s)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11"><text x="{lx}" y="14">{label}</text><text x="{vx}" y="14">{value}</text></g></svg>
"##,
        lx = lw / 2,
        vx = lw + vw / 2,
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The label on every eval coverage badge.
pub const LABEL: &str = "eval coverage";

/// Writes `badge.svg` and `badges/<project>.svg` under `dir`, and returns
/// the files written, relative to `dir`.
pub fn write_badges(dir: &Path, file: &VerdictsFile) -> Result<Vec<String>> {
    let warn = file.coverage.warn_below;
    let badges_dir = dir.join(BADGES_DIR);
    fs::create_dir_all(&badges_dir)
        .with_context(|| format!("creating {}", badges_dir.display()))?;
    let mut written = Vec::new();
    let repo = &file.coverage;
    let svg = badge(
        LABEL,
        &summary(repo.percent, repo.musts_percent),
        color(repo.judged_percent, warn),
    );
    fs::write(dir.join(REPO_BADGE), svg).with_context(|| format!("writing {REPO_BADGE}"))?;
    written.push(REPO_BADGE.to_string());
    for project in &file.projects {
        let c = &project.coverage;
        let value = if c.requirements == 0 {
            "0%, no requirements".to_string()
        } else {
            summary(c.percent, c.musts_percent)
        };
        let svg = badge(LABEL, &value, color(c.judged(), warn));
        let rel = format!("{BADGES_DIR}/{}.svg", project.project);
        fs::write(dir.join(&rel), svg).with_context(|| format!("writing {rel}"))?;
        written.push(rel);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use evals_model::Priority;

    fn req(id: &str, priority: Priority, verdict: Verdict) -> RequirementVerdict {
        RequirementVerdict {
            requirement: id.into(),
            title: id.into(),
            priority,
            verdict,
            reasons: Vec::new(),
        }
    }

    #[test]
    fn counts_met_requirements_and_musts_separately() {
        let c = Coverage::of(
            &[
                req("A-001", Priority::Must, Verdict::Met),
                req("A-002", Priority::Must, Verdict::Unknown),
                req("A-003", Priority::Should, Verdict::Met),
                req("A-004", Priority::Could, Verdict::AtRisk),
            ],
            80.0,
        );
        assert_eq!((c.requirements, c.met, c.musts_met), (4, 2, 1));
        assert_eq!(c.percent, 50.0);
        assert_eq!(c.musts_percent, Some(50.0));
        assert!(c.warning);
        assert_eq!(color(c.judged(), 80.0), "#e05d44");
    }

    #[test]
    fn no_requirements_is_zero_and_a_warning() {
        let c = Coverage::of(&[], 80.0);
        assert_eq!(c.percent, 0.0);
        assert_eq!(c.musts_percent, None);
        assert!(c.warning);
    }

    #[test]
    fn without_musts_the_warning_goes_by_every_requirement() {
        let c = Coverage::of(&[req("A-001", Priority::Should, Verdict::Met)], 80.0);
        assert_eq!(c.musts_percent, None);
        assert!(!c.warning);
        assert_eq!(color(c.judged(), 80.0), "#4c1");
    }

    #[test]
    fn the_repo_total_is_the_mean_over_projects() {
        let full = Coverage::of(&[req("A-001", Priority::Must, Verdict::Met)], 80.0);
        let empty = Coverage::of(&[], 80.0);
        let repo = RepoCoverage::of(&[full, empty], 1, 80.0);
        assert_eq!(
            repo.percent, 50.0,
            "a project with no requirements counts as 0%"
        );
        assert_eq!(repo.musts_percent, Some(100.0));
        assert_eq!(repo.projects_with_requirements, 1);
        assert_eq!(repo.judged_percent, 50.0);
        assert!(
            repo.warning,
            "a project with no requirements counts against it"
        );
    }

    #[test]
    fn colours_follow_the_warning_level() {
        assert_eq!(color(80.0, 80.0), "#4c1");
        assert_eq!(color(60.0, 80.0), "#dfb317");
        assert_eq!(color(59.9, 80.0), "#e05d44");
    }

    #[test]
    fn badges_are_svg_with_the_value_in_the_title() {
        let svg = badge(LABEL, &summary(73.3, Some(50.0)), "#4c1");
        assert!(svg.starts_with("<svg "));
        assert!(
            svg.contains("<title>eval coverage: 73%, musts 50%</title>"),
            "{svg}"
        );
        assert!(badge("a<b", "1%", "#4c1").contains("a&lt;b"));
    }
}
