//! Plans live in the project they are about: `<project>/docs/plans/`, where
//! `<project>` is a top-level project folder (one with a `mise.toml`).
//! Repo-wide plans are standards' own, in `standards/docs/plans/`
//! ("Plans" in standards/docs/documentation.md).
//!
//!   DOC-009  A plan is outside a top-level project's `docs/plans/` folder.
//!   DOC-010  A plan in `standards/docs/plans/` is not listed as repo-wide
//!            in `standards/plans.toml`, or that file lists one that does
//!            not exist. A plan about one project belongs in that project.
//!
//! A plan is a Markdown file in a folder named `plans`, or one whose first
//! `# ` heading starts with `Plan:`. Build output, dependencies, test
//! fixtures and athenaeum's third-party copies are not searched.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::user_docs;

/// Where a project's plans live, from the project folder.
pub const PLANS: &str = "docs/plans";

/// The list of repo-wide plans, from the repo root.
pub const REPO_WIDE: &str = "standards/plans.toml";

#[derive(Debug, Default, Deserialize)]
struct RepoWide {
    /// File name in `standards/docs/plans/` to why it is repo-wide.
    #[serde(default, rename = "repo-wide")]
    repo_wide: BTreeMap<String, String>,
}

/// Folder names never searched, at any depth.
const SKIP_DIRS: [&str; 5] = [".git", "target", "node_modules", "fixtures", "dist"];

/// Top-level folders never searched: third-party content (athenaeum).
const SKIP_TOP: [&str; 1] = ["athenaeum"];

/// One plan in the wrong place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub path: String,
    pub message: String,
}

/// Whether `rel` (a `/`-separated path from the repo root) is a plan.
fn is_plan(rel: &str, text: &str) -> bool {
    let dirs: Vec<&str> = rel.split('/').collect();
    let in_plans_dir = dirs[..dirs.len() - 1].contains(&"plans");
    in_plans_dir || first_heading(text).is_some_and(|h| h.to_lowercase().starts_with("plan:"))
}

/// The text of the first `# ` heading outside a code block.
fn first_heading(text: &str) -> Option<&str> {
    let mut fenced = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fenced = !fenced;
        } else if let Some(h) = t.strip_prefix("# ").filter(|_| !fenced) {
            return Some(h.trim());
        }
    }
    None
}

/// Whether `rel` is directly in a top-level project's `docs/plans/`.
fn well_placed(rel: &str, projects: &[String]) -> bool {
    let Some((project, rest)) = rel.split_once('/') else {
        return false;
    };
    let Some(file) = rest.strip_prefix(PLANS).and_then(|r| r.strip_prefix('/')) else {
        return false;
    };
    !file.contains('/') && projects.iter().any(|p| p == project)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.file_type()?.is_dir() {
            let top = dir == root;
            if SKIP_DIRS.contains(&name.as_str()) || (top && SKIP_TOP.contains(&name.as_str())) {
                continue;
            }
            walk(root, &path, out)?;
        } else if Path::new(&name).extension().is_some_and(|e| e == "md") {
            let rel = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
    Ok(())
}

/// DOC-010: standards' plans against the repo-wide list.
fn check_repo_wide(root: &Path, files: &[String]) -> Result<Vec<Finding>> {
    let path = root.join(REPO_WIDE);
    let list: RepoWide = match fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).with_context(|| format!("parsing {REPO_WIDE}"))?,
        Err(_) => RepoWide::default(),
    };
    let dir = format!("standards/{PLANS}/");
    let mut findings = Vec::new();
    for rel in files {
        let Some(name) = rel.strip_prefix(&dir) else {
            continue;
        };
        if name.contains('/') || name == "README.md" || list.repo_wide.contains_key(name) {
            continue;
        }
        findings.push(Finding {
            rule: "DOC-010",
            path: rel.clone(),
            message: format!(
                "{rel} is not listed in {REPO_WIDE}: move it to the {PLANS}/ of the project \
                 it is about, or list it there with why it is repo-wide"
            ),
        });
    }
    for name in list.repo_wide.keys() {
        let rel = format!("{dir}{name}");
        if !files.contains(&rel) {
            findings.push(Finding {
                rule: "DOC-010",
                message: format!("{REPO_WIDE} lists {rel}, which does not exist"),
                path: rel,
            });
        }
    }
    Ok(findings)
}

/// DOC-009 and DOC-010 over the repo at `root`.
pub fn check(root: &Path) -> Result<Vec<Finding>> {
    let projects = user_docs::projects(root)?;
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort();
    let mut findings = Vec::new();
    for rel in &files {
        let text = fs::read_to_string(root.join(rel)).unwrap_or_default();
        if is_plan(rel, &text) && !well_placed(rel, &projects) {
            findings.push(Finding {
                rule: "DOC-009",
                message: format!(
                    "{rel} is a plan outside a project's {PLANS}/: move it to the {PLANS}/ \
                     of the project it is about (standards/{PLANS}/ for repo-wide plans)"
                ),
                path: rel.clone(),
            });
        }
    }
    findings.extend(check_repo_wide(root, &files)?);
    Ok(findings)
}

/// Text output for people.
pub fn render(findings: &[Finding]) -> String {
    let mut out = String::new();
    for f in findings {
        let _ = writeln!(out, "error: {}: {}", f.rule, f.message);
    }
    if findings.is_empty() {
        out.push_str("plans: every plan is in a project's docs/plans/\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(files: &[(&str, &str)]) -> tempfile::TempDir {
        let mut files = files.to_vec();
        if !files.iter().any(|(p, _)| *p == REPO_WIDE) {
            files.push((REPO_WIDE, "[repo-wide]\n"));
        }
        let dir = tempfile::tempdir().unwrap();
        for p in ["app", "standards"] {
            fs::create_dir_all(dir.path().join(p)).unwrap();
            fs::write(dir.path().join(p).join("mise.toml"), "").unwrap();
        }
        for (path, text) in &files {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        dir
    }

    fn paths(root: &Path) -> Vec<String> {
        check(root).unwrap().into_iter().map(|f| f.path).collect()
    }

    #[test]
    fn plans_in_a_projects_docs_plans_pass() {
        let r = repo(&[
            ("app/docs/plans/2026-11-x.md", "# Plan: x\n"),
            ("app/docs/plans/README.md", "# Plans\n"),
            ("standards/docs/plans/2026-11-y.md", "# Plan: y\n"),
            ("standards/docs/plans/README.md", "# Plans\n"),
            (
                REPO_WIDE,
                "[repo-wide]\n\"2026-11-y.md\" = \"every project\"\n",
            ),
            (
                REPO_WIDE,
                "[repo-wide]\n\"2026-11-y.md\" = \"every project\"\n",
            ),
            ("app/docs/decisions/0001-x.md", "# 0001: x\n"),
        ]);
        assert!(paths(r.path()).is_empty());
    }

    #[test]
    fn plans_elsewhere_fail() {
        let r = repo(&[
            ("app/plans/2026-11-a.md", "# A\n"),
            ("app/web/docs/plans/2026-11-b.md", "# B\n"),
            ("standards/docs/2026-11-c.md", "# Plan: c\n"),
            (
                REPO_WIDE,
                "[repo-wide]\n\"2026-11-y.md\" = \"every project\"\n",
            ),
            ("standards/docs/plans/2026-11-y.md", "# Plan: y\n"),
            ("docs/plans/2026-11-d.md", "# D\n"),
            ("app/docs/plans/old/2026-11-e.md", "# E\n"),
        ]);
        assert_eq!(
            paths(r.path()),
            [
                "app/docs/plans/old/2026-11-e.md",
                "app/plans/2026-11-a.md",
                "app/web/docs/plans/2026-11-b.md",
                "docs/plans/2026-11-d.md",
                "standards/docs/2026-11-c.md",
            ]
        );
    }

    fn rules(root: &Path) -> Vec<(&'static str, String)> {
        check(root)
            .unwrap()
            .into_iter()
            .map(|f| (f.rule, f.path))
            .collect()
    }

    #[test]
    fn standards_plans_must_be_listed_as_repo_wide() {
        let r = repo(&[
            ("standards/docs/plans/2026-11-y.md", "# Plan: y\n"),
            ("standards/docs/plans/2026-11-app.md", "# Plan: app\n"),
            (
                REPO_WIDE,
                "[repo-wide]\n\"2026-11-y.md\" = \"a\"\n\"2026-11-gone.md\" = \"b\"\n",
            ),
        ]);
        assert_eq!(
            rules(r.path()),
            [
                ("DOC-010", "standards/docs/plans/2026-11-app.md".to_string()),
                (
                    "DOC-010",
                    "standards/docs/plans/2026-11-gone.md".to_string()
                ),
            ]
        );
    }

    #[test]
    fn fixtures_third_party_and_code_blocks_are_ignored() {
        let r = repo(&[
            ("app/tests/fixtures/repo/docs/plans/x.md", "# X\n"),
            ("athenaeum/resources/plans/x.md", "# Plan: theirs\n"),
            ("app/README.md", "```\n# Plan: example\n```\n# App\n"),
            ("app/notes.md", "# Planning notes\n"),
        ]);
        assert!(paths(r.path()).is_empty());
    }
}
