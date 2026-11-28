//! A project's committed baseline, `<project>/docs/quality.toml`: its
//! strict-profile findings counted per lint (QUAL-501). Counts may only go
//! down.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use anyhow::{Context, Result, bail};
use serde::Serialize;

/// Where the baseline lives, from the project folder.
pub const FILE: &str = "docs/quality.toml";

/// The counts per lint and the day they were written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Baseline {
    pub updated: String,
    pub lints: BTreeMap<String, u64>,
}

impl Baseline {
    pub fn total(&self) -> u64 {
        self.lints.values().sum()
    }
}

/// Reads a baseline file's text.
pub fn parse(text: &str) -> Result<Baseline> {
    let table: toml::Table = toml::from_str(text).context("not valid TOML")?;
    let Some(baseline) = table.get("baseline").and_then(toml::Value::as_table) else {
        bail!("no [baseline] table");
    };
    let updated = baseline
        .get("updated")
        .map(ToString::to_string)
        .unwrap_or_default();
    let mut lints = BTreeMap::new();
    if let Some(table) = baseline.get("lints").and_then(toml::Value::as_table) {
        for (lint, count) in table {
            let count = count
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .with_context(|| format!("[baseline.lints] {lint} is not a count"))?;
            lints.insert(lint.clone(), count);
        }
    }
    Ok(Baseline {
        updated: updated.trim_matches('"').to_string(),
        lints,
    })
}

/// The file text for a baseline. Lints with no findings are left out.
pub fn render(baseline: &Baseline) -> String {
    let mut out = String::from(
        "# Written by `chores quality --update` (QUAL-501 in standards/docs/code-quality.md).\n\
         # Lower is better; never raise it by hand.\n\
         [baseline]\n",
    );
    let _ = writeln!(out, "updated = {}", baseline.updated);
    let _ = writeln!(out, "total = {}", baseline.total());
    out.push_str("\n[baseline.lints]\n");
    for (lint, count) in baseline.lints.iter().filter(|(_, n)| **n > 0) {
        let _ = writeln!(out, "{} = {count}", toml::Value::String(lint.clone()));
    }
    out
}

/// One lint whose count differs from the baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub lint: String,
    pub before: u64,
    pub after: u64,
}

impl Change {
    pub fn rose(&self) -> bool {
        self.after > self.before
    }
}

/// Every lint whose count differs between the baseline and now, by name.
pub fn compare(before: &BTreeMap<String, u64>, after: &BTreeMap<String, u64>) -> Vec<Change> {
    let lints: std::collections::BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    lints
        .into_iter()
        .filter_map(|lint| {
            let b = before.get(lint).copied().unwrap_or(0);
            let a = after.get(lint).copied().unwrap_or(0);
            (a != b).then(|| Change {
                lint: lint.clone(),
                before: b,
                after: a,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lints(pairs: &[(&str, u64)]) -> BTreeMap<String, u64> {
        pairs.iter().map(|(l, n)| ((*l).to_string(), *n)).collect()
    }

    #[test]
    fn render_and_parse_round_trip() {
        let baseline = Baseline {
            updated: "2026-11-28".into(),
            lints: lints(&[
                ("clippy::format_push_string", 30),
                ("jimhillr::file_too_long", 1),
            ]),
        };
        let text = render(&baseline);
        assert!(
            text.contains("updated = 2026-11-28\ntotal = 31\n"),
            "{text}"
        );
        assert!(
            text.contains("\"clippy::format_push_string\" = 30\n"),
            "{text}"
        );
        assert_eq!(parse(&text).unwrap(), baseline);
    }

    #[test]
    fn zero_counts_are_left_out() {
        let baseline = Baseline {
            updated: "2026-11-28".into(),
            lints: lints(&[("clippy::a", 0), ("clippy::b", 2)]),
        };
        assert!(!render(&baseline).contains("clippy::a"));
    }

    #[test]
    fn parses_the_plan_example() {
        let text = "[baseline]\nupdated = 2026-11-27\ntotal = 118\n\n[baseline.lints]\n\
                    \"clippy::format_push_string\" = 30\n\"clippy::too_many_lines\" = 8\n";
        let baseline = parse(text).unwrap();
        assert_eq!(baseline.updated, "2026-11-27");
        assert_eq!(baseline.total(), 38);
    }

    #[test]
    fn bad_files_are_errors() {
        assert!(parse("not = [toml").is_err());
        assert!(parse("[other]\n").is_err());
        assert!(parse("[baseline.lints]\n\"clippy::a\" = -1\n").is_err());
        assert!(parse("[baseline.lints]\n\"clippy::a\" = \"many\"\n").is_err());
    }

    #[test]
    fn compare_finds_rises_falls_new_and_gone_lints() {
        let before = lints(&[
            ("clippy::a", 3),
            ("clippy::b", 2),
            ("clippy::gone", 1),
            ("clippy::same", 4),
        ]);
        let after = lints(&[
            ("clippy::a", 4),
            ("clippy::b", 1),
            ("clippy::new", 2),
            ("clippy::same", 4),
        ]);
        let changes = compare(&before, &after);
        let summary: Vec<_> = changes
            .iter()
            .map(|c| (c.lint.as_str(), c.before, c.after, c.rose()))
            .collect();
        assert_eq!(
            summary,
            [
                ("clippy::a", 3, 4, true),
                ("clippy::b", 2, 1, false),
                ("clippy::gone", 1, 0, false),
                ("clippy::new", 0, 2, true),
            ]
        );
    }
}
