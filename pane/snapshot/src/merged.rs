//! The PRs merged on the checkout's first-parent history, by month, with
//! the top-level projects each touched: what splits the shared costs on the
//! Money page (`pane_model::money::split_shared_costs`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use cost_model::Month;

use crate::{blame, time};

/// Merged PRs per month since the first day of `since`, or `None` when git
/// can't say: not a git checkout, or a shallow clone whose history is cut
/// off.
pub fn by_month(
    root: &Path,
    since: Month,
    projects: &BTreeSet<&str>,
) -> Option<BTreeMap<Month, Vec<Vec<String>>>> {
    if blame::is_shallow(root) {
        return None;
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "log",
            "--first-parent",
            "--diff-merges=first-parent",
            "--name-only",
            "--format=%x00%ct %s",
        ])
        .arg(format!("--since={since}-01T00:00:00Z"))
        .arg("HEAD")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_log(
        &String::from_utf8_lossy(&out.stdout),
        since,
        projects,
    ))
}

/// Parse `git log --name-only --format=%x00%ct %s`: each NUL starts a commit
/// with its committer time and subject, then the files it changed. Only
/// merged PRs count: a subject ending in `(#N)` (a squash merge) or starting
/// `Merge pull request #`. A file counts for the top-level project its first
/// path segment names, if that is one of `projects`. Commits before `since`
/// are left out.
pub fn parse_log(
    log: &str,
    since: Month,
    projects: &BTreeSet<&str>,
) -> BTreeMap<Month, Vec<Vec<String>>> {
    let mut months: BTreeMap<Month, Vec<Vec<String>>> = BTreeMap::new();
    for commit in log.split('\0').filter(|c| !c.trim().is_empty()) {
        let mut lines = commit.lines();
        let Some((secs, subject)) = lines.next().and_then(|l| l.split_once(' ')) else {
            continue;
        };
        let Ok(secs) = secs.parse::<u64>() else {
            continue;
        };
        if !is_pr(subject) {
            continue;
        }
        let Some(month) = time::rfc3339(secs)
            .get(..7)
            .and_then(|m| m.parse::<Month>().ok())
        else {
            continue;
        };
        if month < since {
            continue;
        }
        let touched: BTreeSet<String> = lines
            .filter_map(|file| file.trim().split('/').next())
            .filter(|top| projects.contains(top))
            .map(str::to_owned)
            .collect();
        months
            .entry(month)
            .or_default()
            .push(touched.into_iter().collect());
    }
    months
}

/// Whether a first-parent commit subject is a merged PR.
fn is_pr(subject: &str) -> bool {
    let subject = subject.trim();
    if subject.starts_with("Merge pull request #") {
        return true;
    }
    subject
        .strip_suffix(')')
        .and_then(|s| s.rsplit_once("(#"))
        .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(year: i32, month: u32) -> Month {
        Month { year, month }
    }

    // 1795000000 is 2026-11-18; 1790000000 is 2026-09-21.
    const LOG: &str = "\0\
1795000000 pane: read cost files (#868)

pane/model/src/costs.rs
pane/snapshot/src/repo.rs
\0\
1795000001 cookbook and pane together (#900)

cookbook/README.md
pane/README.md
issues/x.md
\0\
1795000002 Merge pull request #12 from x/y

homer/src/main.rs
\0\
1795000003 a direct push, not a PR

pane/README.md
\0\
1795000004 repo-wide docs (#901)

AGENTS.md
\0\
1790000000 too old (#1)

pane/README.md
";

    #[test]
    fn groups_merged_prs_by_month_and_top_level_project() {
        let projects = BTreeSet::from(["pane", "cookbook", "homer"]);
        let got = parse_log(LOG, month(2026, 10), &projects);
        let nov = &got[&month(2026, 11)];
        assert_eq!(
            nov,
            &vec![
                vec!["pane".to_string()],
                vec!["cookbook".to_string(), "pane".to_string()],
                vec!["homer".to_string()],
                // A repo-wide PR touches no project but is still listed.
                Vec::<String>::new(),
            ]
        );
        assert!(!got.contains_key(&month(2026, 9)), "before since");
    }

    #[test]
    fn recognises_merged_pr_subjects() {
        assert!(is_pr("pane: thing (#868)"));
        assert!(is_pr("Merge pull request #5 from a/b"));
        assert!(!is_pr("pane: thing"));
        assert!(!is_pr("pane: thing (#)"));
        assert!(!is_pr("pane: (#86x)"));
    }

    #[test]
    fn empty_or_garbled_logs_give_nothing() {
        let projects = BTreeSet::from(["pane"]);
        assert!(parse_log("", month(2026, 1), &projects).is_empty());
        assert!(parse_log("\0not a time (#1)\npane/x\n", month(2026, 1), &projects).is_empty());
    }
}
