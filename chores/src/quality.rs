//! The code quality ratchet's rules (QUAL-501, QUAL-206, QUAL-407 in
//! `standards/docs/code-quality.md`): turn clippy's JSON output and file
//! lengths into findings, count them per lint, and compare the counts with
//! a project's committed `docs/quality.toml`. Pure functions over text, so
//! the rules are tested on fixtures; running clippy is in `quality_run`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// The lint name for a file over [`MAX_FILE_LINES`] (QUAL-206).
pub const FILE_TOO_LONG: &str = "jimhillr::file_too_long";

/// The longest a Rust file may be (QUAL-206).
pub const MAX_FILE_LINES: usize = 800;

/// Path prefixes (repo-relative) of generated code, left out of every
/// check (QUAL-407). Files that say `@generated` near the top are left out
/// too.
pub const GENERATED_PREFIXES: &[&str] = &["infra/src/tfschema/"];

/// The strict profile's lint flags (QUAL-402), passed to clippy after `--`.
/// Its thresholds are in `standards/quality/clippy.toml`.
pub const STRICT_FLAGS: &[&str] = &[
    "-W",
    "clippy::pedantic",
    "-W",
    "clippy::cognitive_complexity",
    "-A",
    "clippy::must_use_candidate",
    "-A",
    "clippy::module_name_repetitions",
];

/// One finding: a lint at a place in a repo-relative file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    pub lint: String,
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub message: String,
}

impl Finding {
    /// The finding as `file:line:column: lint: message`.
    pub fn render(&self) -> String {
        format!(
            "{}:{}:{}: {}: {}",
            self.file, self.line, self.column, self.lint, self.message
        )
    }
}

#[derive(Deserialize)]
struct CargoLine {
    reason: String,
    message: Option<Diagnostic>,
}

#[derive(Deserialize)]
struct Diagnostic {
    message: String,
    level: String,
    code: Option<Code>,
    #[serde(default)]
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Code {
    code: String,
}

#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: u64,
    column_start: u64,
    is_primary: bool,
}

/// True when `path` (repo-relative) is generated code by its prefix.
pub fn is_generated_path(path: &str) -> bool {
    GENERATED_PREFIXES.iter().any(|p| path.starts_with(p))
}

/// True when the start of a file marks it `@generated`.
pub fn is_generated_text(text: &str) -> bool {
    text.lines().take(5).any(|l| l.contains("@generated"))
}

/// The warnings in cargo's `--message-format=json` output, as findings.
///
/// `prefix` is the project folder from the repo root (`cookbook`), which
/// clippy's relative file names are under. A diagnostic counts once even
/// when cargo reports it for several targets (lib and tests). Warnings
/// without a lint code (summaries), spans outside the project (registry
/// crates) and generated code are left out.
///
/// # Errors
///
/// When a line that should be JSON isn't.
pub fn parse_clippy_json(output: &str, prefix: &str) -> Result<Vec<Finding>> {
    let mut seen = BTreeSet::new();
    for (i, line) in output.lines().enumerate() {
        if !line.starts_with('{') {
            continue;
        }
        let parsed: CargoLine = serde_json::from_str(line)
            .with_context(|| format!("line {} of the clippy output", i + 1))?;
        if let Some(finding) = parsed
            .message
            .and_then(|d| to_finding(&parsed.reason, d, prefix))
        {
            seen.insert(finding);
        }
    }
    Ok(seen.into_iter().collect())
}

fn to_finding(reason: &str, d: Diagnostic, prefix: &str) -> Option<Finding> {
    if reason != "compiler-message" || d.level != "warning" {
        return None;
    }
    let lint = d.code?.code;
    let span = d.spans.iter().find(|s| s.is_primary)?;
    if span.file_name.starts_with('/') || span.file_name.starts_with("..") {
        return None;
    }
    let file = join(prefix, &span.file_name);
    if is_generated_path(&file) {
        return None;
    }
    Some(Finding {
        lint,
        file,
        line: span.line_start,
        column: span.column_start,
        message: d.message,
    })
}

fn join(prefix: &str, path: &str) -> String {
    let path = path.trim_start_matches("./");
    if prefix.is_empty() || prefix == "." {
        path.to_string()
    } else {
        format!("{}/{path}", prefix.trim_end_matches('/'))
    }
}

/// A [`FILE_TOO_LONG`] finding for each file over [`MAX_FILE_LINES`].
/// `files` holds each repo-relative path with its text.
pub fn file_length_findings(files: &[(String, String)]) -> Vec<Finding> {
    files
        .iter()
        .filter(|(path, text)| !is_generated_path(path) && !is_generated_text(text))
        .filter_map(|(path, text)| {
            let lines = text.lines().count();
            (lines > MAX_FILE_LINES).then(|| Finding {
                lint: FILE_TOO_LONG.to_string(),
                file: path.clone(),
                line: 1,
                column: 1,
                message: format!("{lines} lines, over the {MAX_FILE_LINES}-line limit (QUAL-206)"),
            })
        })
        .collect()
}

/// The number of findings per lint.
pub fn count(findings: &[Finding]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for f in findings {
        *counts.entry(f.lint.clone()).or_insert(0) += 1;
    }
    counts
}

/// A project's committed `docs/quality.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baseline {
    pub updated: String,
    pub lints: BTreeMap<String, u64>,
}

#[derive(Deserialize)]
struct BaselineFile {
    baseline: BaselineTable,
}

#[derive(Deserialize)]
struct BaselineTable {
    updated: toml::value::Datetime,
    #[serde(default)]
    lints: BTreeMap<String, u64>,
}

impl Baseline {
    /// Total findings over every lint.
    pub fn total(&self) -> u64 {
        self.lints.values().sum()
    }

    /// Parse a `docs/quality.toml`.
    ///
    /// # Errors
    ///
    /// When the text isn't TOML with a `[baseline]` table and an `updated`
    /// date.
    pub fn parse(text: &str) -> Result<Self> {
        let file: BaselineFile = toml::from_str(text).context("reading docs/quality.toml")?;
        Ok(Self {
            updated: file.baseline.updated.to_string(),
            lints: file.baseline.lints,
        })
    }

    /// The file `--update` writes. Lints with no findings are left out.
    pub fn render(&self) -> String {
        let mut s = String::from(
            "# Written by `chores quality --update`. Lower is better; never raise it by hand.\n\
             # The rules are QUAL-501 and QUAL-502 in standards/docs/code-quality.md.\n\
             [baseline]\n",
        );
        let _ = writeln!(s, "updated = {}", self.updated);
        let _ = writeln!(s, "total = {}", self.total());
        s.push_str("\n[baseline.lints]\n");
        for (lint, n) in self.lints.iter().filter(|(_, n)| **n > 0) {
            let _ = writeln!(s, "\"{lint}\" = {n}");
        }
        s
    }
}

/// One lint whose count changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub lint: String,
    pub before: u64,
    pub after: u64,
}

/// How a project's findings compare with its baseline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Comparison {
    pub rose: Vec<Change>,
    pub fell: Vec<Change>,
}

/// Compare the current counts with the baseline's, lint by lint.
pub fn compare(baseline: &BTreeMap<String, u64>, current: &BTreeMap<String, u64>) -> Comparison {
    let lints: BTreeSet<&String> = baseline.keys().chain(current.keys()).collect();
    let mut out = Comparison::default();
    for lint in lints {
        let before = baseline.get(lint).copied().unwrap_or(0);
        let after = current.get(lint).copied().unwrap_or(0);
        let change = Change {
            lint: lint.clone(),
            before,
            after,
        };
        match after.cmp(&before) {
            std::cmp::Ordering::Greater => out.rose.push(change),
            std::cmp::Ordering::Less => out.fell.push(change),
            std::cmp::Ordering::Equal => {}
        }
    }
    out
}

/// The findings to name for the lints that rose. When `changed` (files
/// changed against a base) holds any of them, only those; otherwise all
/// findings of those lints, since the counts can't say which one is new.
pub fn new_findings<'a>(
    comparison: &Comparison,
    findings: &'a [Finding],
    changed: &BTreeSet<String>,
) -> Vec<&'a Finding> {
    let rose: BTreeSet<&str> = comparison.rose.iter().map(|c| c.lint.as_str()).collect();
    let of_rose: Vec<&Finding> = findings
        .iter()
        .filter(|f| rose.contains(f.lint.as_str()))
        .collect();
    let in_changed: Vec<&Finding> = of_rose
        .iter()
        .copied()
        .filter(|f| changed.contains(&f.file))
        .collect();
    if in_changed.is_empty() {
        of_rose
    } else {
        in_changed
    }
}

/// The lints to show first in a report: most findings first.
pub fn top_lints(counts: &BTreeMap<String, u64>, n: usize) -> Vec<(&str, u64)> {
    let mut v: Vec<(&str, u64)> = counts.iter().map(|(l, c)| (l.as_str(), *c)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    v.truncate(n);
    v
}

/// Fail on a date that isn't `YYYY-MM-DD`, so the baseline stays a TOML date.
///
/// # Errors
///
/// When `date` isn't a TOML local date.
pub fn check_date(date: &str) -> Result<()> {
    match date.parse::<toml::value::Datetime>() {
        Ok(d) if d.date.is_some() && d.time.is_none() => Ok(()),
        _ => bail!("'{date}' is not a YYYY-MM-DD date"),
    }
}

#[cfg(test)]
#[path = "quality_tests.rs"]
mod tests;
