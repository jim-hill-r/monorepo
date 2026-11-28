//! `chores repo-health`: the "Repo health" tables in the root `README.md`
//! and `standards/docs/repo-health.md` are written from one file,
//! `standards/repo-health.toml`, which pane's Repo health summary reads too,
//! so the README badges and pane never disagree (Jim, 2026-11-28: "A top
//! level pane summary should have the same information as those badges").
//!
//!   HLT-001  A generated table differs from `standards/repo-health.toml`:
//!            run `chores repo-health --write`.
//!
//! Each table sits between `<!-- repo-health:start -->` and
//! `<!-- repo-health:end -->` markers, with one `###` heading per domain.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// The one source, from the repo root.
pub const SOURCE: &str = "standards/repo-health.toml";

/// Where the generated tables go, from the repo root.
pub const README: &str = "README.md";
pub const POLICY: &str = "standards/docs/repo-health.md";

const START: &str = "<!-- repo-health:start -->";
const END: &str = "<!-- repo-health:end -->";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoHealth {
    pub domain: Vec<Domain>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub name: String,
    pub summary: String,
    pub row: Vec<Row>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub area: String,
    pub runs: String,
    pub checks: String,
    pub red: String,
    /// pane's drill-down route; pane reads it, the tables don't.
    #[serde(default)]
    #[allow(dead_code)]
    pub pane: Option<String>,
    pub badge: Vec<Badge>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
    pub alt: String,
    pub image: String,
    pub link: String,
}

/// Parse the source file's text.
pub fn parse(text: &str) -> Result<RepoHealth> {
    let health: RepoHealth = toml::from_str(text).context("parse standards/repo-health.toml")?;
    for domain in &health.domain {
        if domain.row.is_empty() {
            bail!("domain {:?} has no rows", domain.name);
        }
        for row in &domain.row {
            if row.badge.is_empty() {
                bail!("row {:?} has no badge", row.area);
            }
        }
    }
    Ok(health)
}

/// A row's badges as Markdown images that link to their runs.
fn badges(row: &Row) -> String {
    row.badge
        .iter()
        .map(|b| format!("[![{}]({})]({})", b.alt, b.image, b.link))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Repo-root-relative Markdown links made relative to `standards/docs/`.
fn from_standards_docs(markdown: &str) -> String {
    let mut out = String::new();
    let mut rest = markdown;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i + 2]);
        rest = &rest[i + 2..];
        if let Some(path) = rest.strip_prefix("standards/docs/") {
            rest = path;
        } else if !(rest.starts_with("http") || rest.starts_with('#')) {
            out.push_str("../../");
        }
    }
    out.push_str(rest);
    out
}

/// The README table: area, badges, when it runs and what it checks.
pub fn readme_table(health: &RepoHealth) -> String {
    let mut out = String::new();
    for domain in &health.domain {
        let _ = write!(
            out,
            "\n### {}\n\n{}\n\n| Area | Status | Runs | Checks |\n| --- | --- | --- | --- |\n",
            domain.name, domain.summary
        );
        for row in &domain.row {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                row.area,
                badges(row),
                row.runs,
                row.checks
            );
        }
    }
    out.push('\n');
    out
}

/// The policy table: area, when it runs and what red means.
pub fn policy_table(health: &RepoHealth) -> String {
    let mut out = String::new();
    for domain in &health.domain {
        let _ = write!(
            out,
            "\n### {}\n\n| Area | Runs | Red means |\n| --- | --- | --- |\n",
            domain.name
        );
        for row in &domain.row {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                row.area,
                row.runs,
                from_standards_docs(&row.red)
            );
        }
    }
    out.push('\n');
    out
}

/// `text` with the part between the markers replaced by `table`.
pub fn splice(text: &str, table: &str, file: &str) -> Result<String> {
    let Some(start) = text.find(START) else {
        bail!("{file} has no {START} marker");
    };
    let Some(end) = text.find(END) else {
        bail!("{file} has no {END} marker");
    };
    if end < start {
        bail!("{file} has {END} before {START}");
    }
    Ok(format!(
        "{}{START}\n{table}{}",
        &text[..start],
        &text[end..]
    ))
}

/// Each generated file and the text it should have.
fn targets(root: &Path) -> Result<Vec<(&'static str, String, String)>> {
    let source = fs::read_to_string(root.join(SOURCE)).with_context(|| format!("read {SOURCE}"))?;
    let health = parse(&source)?;
    let mut out = Vec::new();
    for (file, table) in [
        (README, readme_table(&health)),
        (POLICY, policy_table(&health)),
    ] {
        let text = fs::read_to_string(root.join(file)).with_context(|| format!("read {file}"))?;
        let want = splice(&text, &table, file)?;
        out.push((file, text, want));
    }
    Ok(out)
}

/// The files whose table is out of date (HLT-001).
pub fn check(root: &Path) -> Result<Vec<&'static str>> {
    Ok(targets(root)?
        .into_iter()
        .filter(|(_, have, want)| have != want)
        .map(|(file, _, _)| file)
        .collect())
}

/// Rewrite the tables; returns the files that changed.
pub fn write(root: &Path) -> Result<Vec<&'static str>> {
    let mut changed = Vec::new();
    for (file, have, want) in targets(root)? {
        if have != want {
            fs::write(root.join(file), want).with_context(|| format!("write {file}"))?;
            changed.push(file);
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[domain]]
name = "Build"
summary = "Whether main builds."

[[domain.row]]
area = "Main CI"
runs = "Every push"
checks = "CI ([docs](standards/docs/ci.md))"
red = "CI failed ([evals](standards/docs/evals.md#x), [readme](README.md), [web](https://e.x))"
pane = "/projects"
[[domain.row.badge]]
alt = "Main CI"
image = "https://example.com/a.svg"
link = "https://example.com/a"
[[domain.row.badge]]
alt = "Other"
image = "https://example.com/b.svg"
link = "https://example.com/b"
"#;

    #[test]
    fn readme_table_lists_each_badge() {
        let health = parse(SAMPLE).unwrap();
        let table = readme_table(&health);
        assert!(table.contains("### Build\n\nWhether main builds.\n"));
        assert!(table.contains(
            "| Main CI | [![Main CI](https://example.com/a.svg)](https://example.com/a) [![Other](https://example.com/b.svg)](https://example.com/b) | Every push | CI ([docs](standards/docs/ci.md)) |"
        ));
    }

    #[test]
    fn policy_table_rebases_links() {
        let health = parse(SAMPLE).unwrap();
        let table = policy_table(&health);
        assert!(table.contains(
            "| Main CI | Every push | CI failed ([evals](evals.md#x), [readme](../../README.md), [web](https://e.x)) |"
        ));
    }

    #[test]
    fn splice_replaces_between_markers() {
        let text = format!("a\n{START}\nold\n{END}\nb\n");
        let out = splice(&text, "\nnew\n\n", "f").unwrap();
        assert_eq!(out, format!("a\n{START}\n\nnew\n\n{END}\nb\n"));
        assert!(splice("no markers", "x", "f").is_err());
    }

    #[test]
    fn rejects_unknown_fields_and_empty_rows() {
        assert!(parse("[[domain]]\nname = \"a\"\nsummary = \"b\"\nrow = []\n").is_err());
        assert!(parse(&SAMPLE.replace("pane =", "nope =")).is_err());
    }
}
