//! User docs: every top-level project has a user guide for the people who
//! use what it ships, at `<project>/docs/user/README.md`, with the sections
//! in "User docs" in standards/docs/documentation.md (decision 0017).
//!
//!   DOC-005  The project has no `docs/user/README.md`.
//!   DOC-006  The guide is missing one of the required `##` sections.
//!   DOC-007  A required section has no text under it.
//!   DOC-008  The baseline, `standards/user-docs.toml`, is out of date: it
//!            lists a project that passes, a folder that is not a project,
//!            or an issue file that does not exist.
//!
//! A project listed in the baseline may fail DOC-005 to DOC-007; every other
//! project must pass. The baseline only shrinks.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Where each project's user guide lives, from the project folder.
pub const GUIDE: &str = "docs/user/README.md";

/// The baseline file, from the repo root.
pub const BASELINE: &str = "standards/user-docs.toml";

/// A baseline value for a project under `# On Hold Projects`, which gets no
/// issue file (write-issue skill).
pub const ON_HOLD: &str = "on hold";

/// The `##` sections every guide has, in this order, matched by the start
/// of the heading, ignoring case (`## How to use cookbook` matches `How to`).
pub const SECTIONS: [&str; 4] = ["Who it's for", "Getting started", "How to", "Getting help"];

#[derive(Debug, Default, Deserialize)]
struct Baseline {
    /// Project folder to the issue file tracking its guide, or `on hold`.
    #[serde(default)]
    missing: BTreeMap<String, String>,
}

/// One problem found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub project: String,
    pub message: String,
}

/// The result of a check over the repo.
#[derive(Debug, Default, Serialize)]
pub struct Report {
    /// Findings that fail the check.
    pub errors: Vec<Finding>,
    /// Findings allowed by the baseline, reported for information.
    pub baselined: Vec<Finding>,
    /// Projects whose guide passes.
    pub passing: Vec<String>,
}

/// Top-level project folders: directories with a `mise.toml`, not hidden.
pub fn projects(root: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !entry.path().join("mise.toml").is_file() {
            continue;
        }
        out.push(name);
    }
    out.sort();
    Ok(out)
}

fn normalize(s: &str) -> String {
    s.trim().replace('\u{2019}', "'").to_lowercase()
}

/// How one line of a guide counts.
enum Line {
    /// A `##` heading: a new section.
    Section(String),
    /// An `#` heading: ends the sections, so text after it is in none.
    Title,
    /// Text, a deeper heading or a code fence: content of the section above.
    Text,
    /// A blank line or an HTML comment.
    Blank,
}

/// Each `##` section's heading and whether any text follows it before the
/// next `#` or `##` heading. Lines in fenced code blocks are text, never
/// headings; HTML comments are not text.
fn sections(text: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let (mut in_fence, mut in_comment) = (false, false);
    for line in text.lines() {
        let t = line.trim();
        let kind = if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            Line::Text
        } else if in_fence {
            Line::Text
        } else if in_comment || t.starts_with("<!--") {
            in_comment = !t.contains("-->");
            Line::Blank
        } else if let Some(h) = t.strip_prefix("## ") {
            Line::Section(normalize(h))
        } else if t.starts_with("# ") {
            Line::Title
        } else if t.is_empty() {
            Line::Blank
        } else {
            Line::Text
        };
        match kind {
            Line::Section(h) => out.push((h, false)),
            Line::Title => out.push((String::new(), false)),
            Line::Text => {
                if let Some(last) = out.last_mut() {
                    last.1 = true;
                }
            }
            Line::Blank => {}
        }
    }
    out
}

/// DOC-006 and DOC-007 for one guide's text.
pub fn check_guide(project: &str, text: &str) -> Vec<Finding> {
    let sections = sections(text);
    let mut findings = Vec::new();
    for want in SECTIONS {
        let key = normalize(want);
        let found = sections
            .iter()
            .find(|(h, _)| !h.is_empty() && h.starts_with(&key));
        let (rule, message) = match found {
            None => (
                "DOC-006",
                format!("{project}/{GUIDE} has no '## {want}' section"),
            ),
            Some((_, false)) => (
                "DOC-007",
                format!("{project}/{GUIDE}: '## {want}' has no text under it"),
            ),
            Some(_) => continue,
        };
        findings.push(Finding {
            rule,
            project: project.to_string(),
            message,
        });
    }
    findings
}

/// DOC-005 to DOC-007 for one project.
pub fn check_project(root: &Path, project: &str) -> Result<Vec<Finding>> {
    let path = root.join(project).join(GUIDE);
    if !path.is_file() {
        return Ok(vec![Finding {
            rule: "DOC-005",
            project: project.to_string(),
            message: format!("{project} has no user guide at {project}/{GUIDE}"),
        }]);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(check_guide(project, &text))
}

/// Check every project against the baseline.
pub fn check(root: &Path) -> Result<Report> {
    let baseline_path = root.join(BASELINE);
    let baseline: Baseline = if baseline_path.is_file() {
        let text = fs::read_to_string(&baseline_path)?;
        toml::from_str(&text).with_context(|| format!("parsing {BASELINE}"))?
    } else {
        Baseline::default()
    };
    let projects = projects(root)?;
    let mut report = Report::default();
    let stale = |project: &str, message: String| Finding {
        rule: "DOC-008",
        project: project.to_string(),
        message,
    };
    for (project, issue) in &baseline.missing {
        if !projects.contains(project) {
            report.errors.push(stale(
                project,
                format!("{BASELINE} lists {project}, which is not a top-level project"),
            ));
        } else if issue != ON_HOLD && !root.join(issue).is_file() {
            report.errors.push(stale(
                project,
                format!(
                    "{BASELINE}: {project}'s issue file {issue} does not exist; \
                     name the issue that tracks its user guide"
                ),
            ));
        }
    }
    for project in &projects {
        let findings = check_project(root, project)?;
        let listed = baseline.missing.contains_key(project);
        if findings.is_empty() {
            if listed {
                report.errors.push(stale(
                    project,
                    format!("{project}'s user guide passes; remove it from {BASELINE}"),
                ));
            }
            report.passing.push(project.clone());
        } else if listed {
            report.baselined.extend(findings);
        } else {
            report.errors.extend(findings);
        }
    }
    Ok(report)
}

/// The check's output: one line per error, then a summary.
pub fn render(report: &Report) -> String {
    let mut s = String::new();
    for f in &report.errors {
        let _ = writeln!(s, "error[{}] {}", f.rule, f.message);
    }
    let mut waiting: Vec<&str> = report
        .baselined
        .iter()
        .map(|f| f.project.as_str())
        .collect();
    waiting.dedup();
    let _ = writeln!(
        s,
        "user docs: {} passing, {} in the baseline, {} error(s)",
        report.passing.len(),
        waiting.len(),
        report.errors.len()
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "# Cookbook user guide\n\n## Who it's for\n\nCooks.\n\n\
        ## Getting started\n\nOpen the site.\n\n## How to plan a week\n\n### Steps\n\n\
        ## Getting help\n\nUse the feedback button.\n";

    fn repo(projects: &[(&str, Option<&str>)], baseline: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (p, guide) in projects {
            fs::create_dir_all(dir.path().join(p).join("docs/user")).unwrap();
            fs::create_dir_all(dir.path().join(p).join("docs/issues")).unwrap();
            fs::write(dir.path().join(p).join("mise.toml"), "").unwrap();
            if let Some(g) = guide {
                fs::write(dir.path().join(p).join(GUIDE), g).unwrap();
            }
        }
        fs::create_dir_all(dir.path().join("standards")).unwrap();
        fs::create_dir_all(dir.path().join(".github")).unwrap();
        fs::write(dir.path().join(".github/mise.toml"), "").unwrap();
        fs::write(dir.path().join(BASELINE), baseline).unwrap();
        dir
    }

    fn rules(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.rule).collect()
    }

    #[test]
    fn a_full_guide_passes() {
        assert!(check_guide("cookbook", GOOD).is_empty());
    }

    #[test]
    fn missing_and_empty_sections() {
        let text = "# Guide\n\n## Who it\u{2019}s for\n\n<!-- who -->\n\n## Getting started\n\n```\nmise run dev\n```\n";
        let f = check_guide("x", text);
        assert_eq!(rules(&f), ["DOC-007", "DOC-006", "DOC-006"]);
        assert!(f[0].message.contains("Who it's for"));
        assert!(f[1].message.contains("How to"));
    }

    #[test]
    fn headings_inside_code_blocks_do_not_count() {
        let text = GOOD.replace("## Getting help", "```\n## Getting help\n```");
        assert_eq!(rules(&check_guide("x", &text)), ["DOC-006"]);
    }

    #[test]
    fn projects_are_top_level_folders_with_mise_toml() {
        let dir = repo(&[("b", None), ("a", None)], "");
        fs::create_dir_all(dir.path().join("a/web")).unwrap();
        fs::write(dir.path().join("a/web/mise.toml"), "").unwrap();
        assert_eq!(projects(dir.path()).unwrap(), ["a", "b"]);
    }

    #[test]
    fn unlisted_projects_must_pass() {
        let dir = repo(&[("a", Some(GOOD)), ("b", None)], "");
        let r = check(dir.path()).unwrap();
        assert_eq!(rules(&r.errors), ["DOC-005"]);
        assert_eq!(r.passing, ["a"]);
    }

    #[test]
    fn the_baseline_allows_gaps_with_an_issue() {
        let dir = repo(
            &[("a", None), ("b", Some("# B\n"))],
            "[missing]\na = \"a/docs/issues/user-docs.md\"\nb = \"on hold\"\n",
        );
        fs::write(dir.path().join("a/docs/issues/user-docs.md"), "x").unwrap();
        let r = check(dir.path()).unwrap();
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(rules(&r.baselined)[0], "DOC-005");
        assert_eq!(
            render(&r),
            "user docs: 0 passing, 2 in the baseline, 0 error(s)\n"
        );
    }

    #[test]
    fn a_stale_baseline_fails() {
        let dir = repo(
            &[("a", Some(GOOD)), ("b", None)],
            "[missing]\na = \"on hold\"\nb = \"b/docs/issues/gone.md\"\nzzz = \"on hold\"\n",
        );
        let r = check(dir.path()).unwrap();
        assert_eq!(rules(&r.errors), ["DOC-008", "DOC-008", "DOC-008"]);
        let text = render(&r);
        assert!(text.contains("b's issue file b/docs/issues/gone.md does not exist"));
        assert!(text.contains("zzz, which is not a top-level project"));
        assert!(text.contains("a's user guide passes; remove it"));
    }
}
