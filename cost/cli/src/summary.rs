//! `cost summary --base <ref>`: compares the working tree with the merge
//! base of `<ref>` and prints the PR's cost summary.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use cost_model::{
    COST_FILE, Detectors, LimitsFile, Month, Project, Radar, Snapshot, Summary, parse_cost_file,
    parse_limits, parse_radar, summarize,
};

use crate::check::{
    DETECTORS, LIMITS, RADAR, WORKFLOWS, detect_here, load_detectors, projects, sources,
    workflows_here,
};
use crate::git;

/// Where the cost data lives at one commit: the working tree or a rev.
enum Side<'a> {
    Here,
    At { rev: &'a str, tree: Vec<String> },
}

impl Side<'_> {
    fn read(&self, root: &Path, paths: &[String]) -> Result<BTreeMap<String, String>> {
        match self {
            Side::Here => Ok(git::read_here(root, paths)),
            Side::At { rev, .. } => git::read_at(root, rev, paths),
        }
    }

    fn projects(&self, root: &Path) -> Result<Vec<String>> {
        match self {
            Side::Here => projects(root),
            Side::At { tree, .. } => Ok(tree
                .iter()
                .filter_map(|p| p.strip_suffix("/mise.toml"))
                .filter(|p| !p.contains('/') && !p.starts_with('.'))
                .map(str::to_string)
                .collect()),
        }
    }

    fn files_under(&self, root: &Path, prefix: &str) -> Vec<String> {
        match self {
            Side::Here => git::tracked(root, prefix),
            Side::At { tree, .. } => {
                let dir = format!("{prefix}/");
                tree.iter()
                    .filter(|p| p.starts_with(&dir))
                    .cloned()
                    .collect()
            }
        }
    }

    /// The snapshot, with detection for `touched` projects.
    fn snapshot(
        &self,
        root: &Path,
        detectors: Option<&Detectors>,
        touched: &BTreeSet<String>,
    ) -> Result<Snapshot> {
        let names = self.projects(root)?;
        let mut wanted = vec![LIMITS.to_string(), RADAR.to_string()];
        wanted.extend(names.iter().map(|n| format!("{n}/{COST_FILE}")));
        let text = self.read(root, &wanted)?;
        // A file that doesn't parse at the base counts as empty, so a PR
        // that fixes it doesn't fail the summary; `cost check` reports it.
        let limits: LimitsFile = text
            .get(LIMITS)
            .and_then(|t| parse_limits(t).ok())
            .unwrap_or_default();
        let radar: Radar = text
            .get(RADAR)
            .and_then(|t| parse_radar(t).ok())
            .unwrap_or_default();
        let projects = names
            .iter()
            .filter_map(|n| {
                let file = parse_cost_file(text.get(&format!("{n}/{COST_FILE}"))?).ok()?;
                Some(Project {
                    name: n.clone(),
                    file,
                })
            })
            .collect();

        let mut detected = BTreeMap::new();
        if let Some(detectors) = detectors {
            let workflows = match self {
                Side::Here => workflows_here(root),
                Side::At { .. } => sources(self.read(root, &self.files_under(root, WORKFLOWS))?),
            };
            for name in names.iter().filter(|n| touched.contains(*n)) {
                let found = match self {
                    Side::Here => detect_here(root, name, detectors, &workflows),
                    Side::At { .. } => {
                        let paths: Vec<String> = self
                            .files_under(root, name)
                            .into_iter()
                            .filter(|p| detectors.scans(p))
                            .collect();
                        detectors.detect(name, &sources(self.read(root, &paths)?), &workflows)
                    }
                };
                detected.insert(name.clone(), found.blips());
            }
        }
        Ok(Snapshot {
            projects,
            detected,
            radar,
            limits,
        })
    }
}

/// Whether a changed file is cost-sensitive. `diff` is its changed lines.
fn cost_sensitive(path: &str, diff: &str, detectors: Option<&Detectors>) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let top_level_cost = path.split('/').count() == 3 && path.ends_with(&format!("/{COST_FILE}"));
    if top_level_cost || [LIMITS, RADAR, DETECTORS].contains(&path) || name == "wrangler.toml" {
        return true;
    }
    let changed = || {
        diff.lines()
            .filter(|l| {
                (l.starts_with('+') || l.starts_with('-'))
                    && !l.starts_with("+++")
                    && !l.starts_with("---")
            })
            .map(|l| &l[1..])
    };
    if path.starts_with(&format!("{WORKFLOWS}/")) {
        return changed().any(|l| {
            let t = l.trim_start();
            t.starts_with("schedule:") || t.starts_with("- cron:") || t.starts_with("runs-on:")
        });
    }
    if matches!(name, "Cargo.toml" | "package.json")
        && let Some(d) = detectors
    {
        return changed().any(|l| d.line_matches(path, l));
    }
    false
}

pub fn summary(root: &Path, base: &str, month: Month) -> Result<Summary> {
    let merge_base = git::git(root, &["merge-base", base, "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| base.to_string());
    let changed: Vec<String> = git::git(root, &["diff", "--name-only", &merge_base])
        .context("listing changed files")?
        .lines()
        .map(str::to_string)
        .collect();
    let detectors = load_detectors(root)?;

    let candidates: Vec<&String> = changed
        .iter()
        .filter(|p| {
            let name = p.rsplit('/').next().unwrap_or(p);
            p.starts_with(&format!("{WORKFLOWS}/")) || matches!(name, "Cargo.toml" | "package.json")
        })
        .collect();
    let mut cost_files = Vec::new();
    for path in &changed {
        let diff = if candidates.contains(&path) {
            git::git(root, &["diff", "-U0", &merge_base, "--", path]).unwrap_or_default()
        } else {
            String::new()
        };
        if cost_sensitive(path, &diff, detectors.as_ref()) {
            cost_files.push(path.clone());
        }
    }

    let touched: BTreeSet<String> = changed
        .iter()
        .filter_map(|p| p.split_once('/').map(|(top, _)| top.to_string()))
        .collect();
    let base_side = Side::At {
        rev: &merge_base,
        tree: git::tree(root, &merge_base)?,
    };
    let before = base_side.snapshot(root, detectors.as_ref(), &touched)?;
    let after = Side::Here.snapshot(root, detectors.as_ref(), &touched)?;
    Ok(summarize(&before, &after, cost_files, month))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn run(root: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .output()
            .expect("git")
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    const PAGES: &str = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\npages_projects = 1\n";

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(
            root,
            RADAR,
            r#"[{"name": "Cloudflare Pages", "ring": "Adopt", "kind": "vendor", "budget": 0},
                {"name": "PostHog", "ring": "Trial", "kind": "vendor", "budget": 0}]"#,
        );
        write(
            root,
            LIMITS,
            "[[limit]]\nkey = \"pages_projects\"\nblip = \"Cloudflare Pages\"\nitem = \"x\"\nfree = 100\nbudget = 25\nscope = \"account\"\nunit = \"projects\"\npast = \"x\"\nsource = \"x\"\nchecked = \"2026-11-26\"\n",
        );
        write(
            root,
            DETECTORS,
            "[scan]\nextensions = [\"rs\", \"json\"]\nskip_dirs = []\nskip_files = []\n[[detector]]\nid = \"ph\"\nblip = \"PostHog\"\nwhy = \"x\"\nhosts = [\"posthog.com\"]\nfiles = [\"package.json\"]\npatterns = ['\"posthog-js\"']\n",
        );
        write(root, "app/mise.toml", "");
        write(root, "app/docs/cost.toml", PAGES);
        write(root, "app/package.json", "{}\n");
        write(root, "lib/mise.toml", "");
        write(root, "lib/src/lib.rs", "fn x() {}\n");
        run(root, &["init", "-q", "-b", "main"]);
        run(root, &["add", "-A"]);
        run(root, &["commit", "-qm", "base"]);
        dir
    }

    fn month() -> Month {
        Month::new(2026, 11).expect("month")
    }

    #[test]
    fn test_unrelated_change_has_no_labels() {
        let dir = repo();
        write(dir.path(), "lib/src/lib.rs", "fn y() {}\n");
        let s = summary(dir.path(), "main", month()).expect("summary");
        assert!(s.labels.is_empty(), "{s:?}");
    }

    #[test]
    fn test_usage_change_is_labelled_cost() {
        let dir = repo();
        write(
            dir.path(),
            "app/docs/cost.toml",
            &PAGES.replace("= 1", "= 2"),
        );
        let s = summary(dir.path(), "main", month()).expect("summary");
        assert_eq!(s.labels, ["cost"]);
        assert_eq!(s.changed_files, ["app/docs/cost.toml"]);
        assert_eq!(s.account[0].after, 2);
    }

    #[test]
    fn test_new_sdk_in_a_manifest_needs_jim() {
        let dir = repo();
        write(dir.path(), "app/package.json", "{\"posthog-js\": \"1\"}\n");
        let s = summary(dir.path(), "main", month()).expect("summary");
        assert_eq!(s.labels, ["cost", "needs-human"]);
        assert_eq!(s.changed_files, ["app/package.json"]);
        assert!(
            s.needs_human[0].contains("app's code starts using PostHog"),
            "{s:?}"
        );
    }

    #[test]
    fn test_scheduled_workflow_change_is_a_cost_file() {
        let dir = repo();
        write(dir.path(), ".github/workflows/app-x.yml", "on:\n  push:\n");
        run(dir.path(), &["add", "-A"]);
        run(dir.path(), &["commit", "-qm", "wf"]);
        run(dir.path(), &["branch", "base"]);
        write(
            dir.path(),
            ".github/workflows/app-x.yml",
            "on:\n  schedule:\n    - cron: '0 0 * * *'\n",
        );
        let s = summary(dir.path(), "base", month()).expect("summary");
        assert_eq!(s.changed_files, [".github/workflows/app-x.yml"]);
        assert_eq!(s.labels, ["cost"]);
    }
}
