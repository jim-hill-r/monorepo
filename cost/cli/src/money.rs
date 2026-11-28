//! `cost money`: the financial ledger's statement for a month and the
//! runway, from `standards/money/runway.toml`, the actual spend and
//! payments in `standards/money/actuals.toml`, every project's
//! `[[revenue]]` and the PRs merged on the checkout's first-parent history.
//! See `standards/docs/money.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike};
use cost_model::{
    ACTUALS_FILE, Books, Month, RUNWAY_FILE, Runway, Statement, ledger, parse_actuals,
    parse_runway_file, runway,
};

use crate::check;
use crate::git::git;

/// What `cost money` wrote.
pub struct Written {
    pub statement: Option<PathBuf>,
    pub runway: PathBuf,
    pub status: Runway,
}

/// Builds the ledger through `month` and writes `<out>/runway.json` and
/// `<out>/statements/<month>.json` (no statement when `month` is before the
/// ledger starts).
pub fn money(root: &Path, month: Month, out: &Path) -> Result<Written> {
    let runway_file = parse_runway_file(
        &fs::read_to_string(root.join(RUNWAY_FILE))
            .with_context(|| format!("reading {RUNWAY_FILE}"))?,
    )
    .map_err(|e| anyhow::anyhow!("{RUNWAY_FILE}: {e}"))?;
    let books = books(root, runway_file.start)?;
    let statements = ledger(&runway_file, &books, month);
    let status = runway(&runway_file, &statements);

    fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let runway_path = out.join("runway.json");
    write_json(&runway_path, &status)?;
    let statement = match statements.last() {
        Some(last) if last.month == month => Some(write_statement(out, last)?),
        _ => None,
    };
    Ok(Written {
        statement,
        runway: runway_path,
        status,
    })
}

fn write_statement(out: &Path, statement: &Statement) -> Result<PathBuf> {
    let dir = out.join("statements");
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let path = dir.join(format!("{}.json", statement.month));
    write_json(&path, statement)?;
    Ok(path)
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("writing {}", path.display()))
}

/// Every parsed project file, the actual spend and payments, and the merged
/// PRs since `start`.
fn books(root: &Path, start: Month) -> Result<Books> {
    let actuals = parse_actuals(
        &fs::read_to_string(root.join(ACTUALS_FILE))
            .with_context(|| format!("reading {ACTUALS_FILE}"))?,
    )
    .map_err(|e| anyhow::anyhow!("{ACTUALS_FILE}: {e}"))?;
    let names = check::projects(root)?;
    let known: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    Ok(Books {
        projects: check::parsed_projects(root)?,
        actuals,
        merged: merged_prs(root, start, &known)?,
    })
}

/// The PRs merged since the start of `since`, by month, with the projects
/// each touched. Fails on a shallow clone, whose cut-off history would
/// split the shared costs wrongly.
fn merged_prs(
    root: &Path,
    since: Month,
    projects: &BTreeSet<&str>,
) -> Result<BTreeMap<Month, Vec<Vec<String>>>> {
    if git(root, &["rev-parse", "--is-shallow-repository"])?.trim() == "true" {
        bail!(
            "a shallow clone can't say which PRs merged each month; fetch the full history (actions/checkout with fetch-depth: 0)"
        );
    }
    let since_arg = format!("--since={since}-01T00:00:00Z");
    let log = git(
        root,
        &[
            "log",
            "--first-parent",
            "--diff-merges=first-parent",
            "--name-only",
            "--format=%x00%ct %s",
            &since_arg,
            "HEAD",
        ],
    )?;
    Ok(parse_log(&log, since, projects))
}

/// Parse `git log --name-only --format=%x00%ct %s`: each NUL starts a commit
/// with its committer time and subject, then the files it changed. Only
/// merged PRs count: a subject ending in `(#N)` (a squash merge) or starting
/// `Merge pull request #`. A file counts for the top-level project its first
/// path segment names, if that is one of `projects`.
fn parse_log(
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
        let Some(when) = secs
            .parse()
            .ok()
            .and_then(|s| DateTime::from_timestamp(s, 0))
        else {
            continue;
        };
        let Some(month) = Month::new(when.year(), when.month()) else {
            continue;
        };
        if month < since || !is_pr(subject) {
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
    use std::process::Command;

    use cost_model::Status;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    // 1795000000 is 2026-11-18T08:53:20Z; 1790000000 is 2026-09-21.
    const LOG: &str = "\0\
1795000000 pane: read cost files (#868)

pane/model/src/costs.rs
\0\
1795000001 cookbook and pane together (#900)

cookbook/README.md
pane/README.md
issues/x.md
\0\
1795000002 Merge pull request #12 from x/y

AGENTS.md
\0\
1795000003 a direct push, not a PR

homer/src/main.rs
\0\
1790000000 too early (#1)

homer/src/main.rs
";

    #[test]
    fn test_parse_log_keeps_merged_prs_and_their_projects() {
        let projects = BTreeSet::from(["cookbook", "pane", "homer"]);
        let nov = Month::new(2026, 11).unwrap_or(Month { year: 0, month: 1 });
        let months = parse_log(LOG, nov, &projects);
        assert_eq!(months.len(), 1);
        assert_eq!(
            months[&nov],
            vec![
                vec!["pane".to_string()],
                vec!["cookbook".to_string(), "pane".to_string()],
                vec![],
            ]
        );
    }

    fn write(root: &Path, rel: &str, text: &str) -> std::io::Result<()> {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)
    }

    fn run(root: &Path, args: &[&str]) -> std::io::Result<()> {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .env("GIT_COMMITTER_DATE", "2026-11-18T08:00:00Z")
            .env("GIT_AUTHOR_DATE", "2026-11-18T08:00:00Z")
            .status()?;
        assert!(status.success(), "git {args:?}");
        Ok(())
    }

    #[test]
    fn test_money_writes_the_statement_and_runway() -> TestResult {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        write(
            root,
            ACTUALS_FILE,
            "[[spend]]\nmonth = \"2026-11\"\nvendor = \"Claude Code\"\ncents = 2500\npaid_by = \"bank\"\nsource = \"invoice 1\"\n",
        )?;
        write(
            root,
            RUNWAY_FILE,
            "start = \"2026-11\"\nopening_cents = 10000\n",
        )?;
        for p in ["cookbook", "pane"] {
            write(root, &format!("{p}/mise.toml"), "")?;
            write(
                root,
                &format!("{p}/docs/cost.toml"),
                "reviewed = \"2026-11-26\"\nsummary = \"x\"\n",
            )?;
        }
        run(root, &["init", "-q", "-b", "main"])?;
        run(root, &["add", "."])?;
        run(root, &["commit", "-q", "-m", "start (#1)"])?;
        write(root, "pane/README.md", "x")?;
        run(root, &["add", "."])?;
        run(root, &["commit", "-q", "-m", "pane: readme (#2)"])?;

        let out = root.join("out");
        let nov = Month {
            year: 2026,
            month: 11,
        };
        let written = money(root, nov, &out)?;
        assert_eq!(written.status.position.net_cents, 7500);
        assert_eq!(written.status.status, Status::Funded);
        let path = written.statement.ok_or("no statement")?;
        assert!(path.ends_with("statements/2026-11.json"));
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
        assert_eq!(json["closing"]["bank_cents"], 7500);
        // PR 1 touched cookbook and pane, PR 2 only pane: 1/4 and 3/4.
        let shares: Vec<(String, u64)> = json["lines"]
            .as_array()
            .ok_or("lines")?
            .iter()
            .map(|l| {
                (
                    l["project"].as_str().unwrap_or_default().to_string(),
                    l["cents"].as_u64().unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(
            shares,
            vec![("cookbook".to_string(), 625), ("pane".to_string(), 1875)]
        );
        let runway: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&written.runway)?)?;
        assert_eq!(runway["status"], "funded");
        assert_eq!(runway["runway_months"], 3.0);
        Ok(())
    }

    #[test]
    fn test_money_before_the_start_writes_only_the_runway() -> TestResult {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        write(root, ACTUALS_FILE, "")?;
        write(
            root,
            RUNWAY_FILE,
            "start = \"2027-01\"\nopening_cents = 0\n",
        )?;
        run(root, &["init", "-q", "-b", "main"])?;
        run(root, &["add", "."])?;
        run(root, &["commit", "-q", "-m", "start"])?;
        let written = money(
            root,
            Month {
                year: 2026,
                month: 12,
            },
            &root.join("out"),
        )?;
        assert!(written.statement.is_none());
        assert_eq!(written.status.status, Status::Funded);
        Ok(())
    }
}
