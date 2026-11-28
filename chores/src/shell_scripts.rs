//! QUAL-404: scripts stay Rust-only (Jim, 2026-11-27, decision 0015). No
//! new tracked `.sh` file may be added; the ones not yet ported into
//! `chores` are listed in `standards/shell-scripts.toml`, which only
//! shrinks.
//!
//!   QUAL-404  A tracked `.sh` file is not on the list, or the list names a
//!             file that is no longer tracked (delete its line).

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The allow list, from the repo root.
pub const ALLOW_LIST: &str = "standards/shell-scripts.toml";

#[derive(Debug, Default, Deserialize)]
struct AllowList {
    /// Tracked `.sh` files still to port, from the repo root.
    #[serde(default)]
    scripts: Vec<String>,
}

/// The result of the check.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Report {
    /// Tracked `.sh` files not on the list.
    pub new: Vec<String>,
    /// Listed files that are no longer tracked.
    pub stale: Vec<String>,
    /// How many listed scripts are left to port.
    pub left: usize,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.new.is_empty() && self.stale.is_empty()
    }
}

/// Compare the tracked `.sh` files with the allow list.
pub fn compare(tracked: &[String], listed: &[String]) -> Report {
    let tracked: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
    let listed: BTreeSet<&str> = listed.iter().map(String::as_str).collect();
    Report {
        new: tracked
            .difference(&listed)
            .map(ToString::to_string)
            .collect(),
        stale: listed
            .difference(&tracked)
            .map(ToString::to_string)
            .collect(),
        left: listed.intersection(&tracked).count(),
    }
}

fn tracked_scripts(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--", "*.sh"])
        .output()
        .context("running git ls-files")?;
    anyhow::ensure!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(ToString::to_string)
        .collect())
}

/// Check the repo at `root`.
pub fn check(root: &Path) -> Result<Report> {
    let path = root.join(ALLOW_LIST);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {ALLOW_LIST}"))?;
    let list: AllowList = toml::from_str(&text).with_context(|| format!("parsing {ALLOW_LIST}"))?;
    Ok(compare(&tracked_scripts(root)?, &list.scripts))
}

/// The report as audit lines.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    for f in &report.new {
        let _ = writeln!(
            out,
            "error[QUAL-404] {f}: new shell script; write it as a chores command instead \
             (scripts stay Rust-only, standards/docs/plans/2026-11-code-quality.md)"
        );
    }
    for f in &report.stale {
        let _ = writeln!(
            out,
            "error[QUAL-404] {ALLOW_LIST} lists {f}, which is no longer tracked; delete its line"
        );
    }
    if report.ok() {
        let _ = writeln!(out, "shell scripts: ok ({} left to port)", report.left);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn listed_scripts_pass() {
        let r = compare(&s(&["a.sh", "b/c.sh"]), &s(&["b/c.sh", "a.sh"]));
        assert!(r.ok());
        assert_eq!(r.left, 2);
        assert!(render(&r).contains("2 left to port"));
    }

    #[test]
    fn a_new_script_fails() {
        let r = compare(&s(&["a.sh", "new.sh"]), &s(&["a.sh"]));
        assert_eq!(r.new, s(&["new.sh"]));
        assert!(!r.ok());
        assert!(render(&r).contains("error[QUAL-404] new.sh: new shell script"));
    }

    #[test]
    fn a_ported_script_leaves_the_list() {
        let r = compare(&s(&["a.sh"]), &s(&["a.sh", "gone.sh"]));
        assert_eq!(r.stale, s(&["gone.sh"]));
        assert!(!r.ok());
        assert!(render(&r).contains("lists gone.sh, which is no longer tracked"));
    }

    #[test]
    fn checks_a_repo() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(root)
                    .args(args)
                    .env_remove("GIT_DIR")
                    .env_remove("GIT_INDEX_FILE")
                    .env_remove("GIT_WORK_TREE")
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-q"]);
        fs::create_dir_all(root.join("standards/scripts")).unwrap();
        fs::write(root.join("standards/scripts/old.sh"), "").unwrap();
        fs::write(root.join("standards/scripts/new.sh"), "").unwrap();
        fs::write(
            root.join(ALLOW_LIST),
            "scripts = [\"standards/scripts/old.sh\"]\n",
        )
        .unwrap();
        git(&["add", "standards/scripts/old.sh"]);
        assert!(check(root).unwrap().ok(), "untracked files don't count");
        git(&["add", "standards/scripts/new.sh"]);
        assert_eq!(check(root).unwrap().new, s(&["standards/scripts/new.sh"]));
    }
}
