//! The repo health badges from `standards/repo-health.toml`, the one file
//! the root README's "Repo health" tables are written from (`chores
//! repo-health`). Pane's Repo health summary on Today and the Health page
//! shows the same badges, grouped into the same domains, so the README and
//! pane never disagree (Jim, 2026-11-28). The badge images are GitHub's own,
//! loaded live, so each shows the latest run on `main` exactly as the README
//! does.

use serde::{Deserialize, Serialize};

/// The file, from the repo root.
pub const SOURCE: &str = "standards/repo-health.toml";

/// Every domain, in the file's order. The field names match the TOML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoHealth {
    #[serde(default)]
    pub domain: Vec<Domain>,
}

/// A group of checks, such as "Build and deploy" or "Threats".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub name: String,
    /// One sentence on what the domain's badges say together.
    pub summary: String,
    pub row: Vec<Row>,
}

/// One check: a workflow (or a stored result) and its badges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub area: String,
    /// When it runs.
    pub runs: String,
    /// What it checks, Markdown with repo-root-relative links.
    pub checks: String,
    /// What a red badge means, Markdown with repo-root-relative links.
    pub red: String,
    /// The pane page that holds the detail behind it, such as `/money`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pane: Option<String>,
    pub badge: Vec<Badge>,
}

/// A status badge: GitHub's image and the page of runs it links to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
    pub alt: String,
    pub image: String,
    pub link: String,
}

impl RepoHealth {
    /// How many rows there are in all.
    pub fn rows(&self) -> usize {
        self.domain.iter().map(|d| d.row.len()).sum()
    }
}

/// The page anchor for a domain or row name: lower case, with each run of
/// other characters as one `-` (`Threats: CodeQL` is `threats-codeql`).
pub fn anchor(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_are_slugs() {
        assert_eq!(anchor("Threats: CodeQL"), "threats-codeql");
        assert_eq!(anchor("Main CI/CD"), "main-ci-cd");
        assert_eq!(anchor("tofu_state"), "tofu-state");
        assert_eq!(anchor("Build and deploy"), "build-and-deploy");
    }

    #[test]
    fn counts_rows_and_round_trips() {
        let row = Row {
            area: "A".into(),
            runs: "Daily".into(),
            checks: "c".into(),
            red: "r".into(),
            pane: None,
            badge: vec![Badge {
                alt: "A".into(),
                image: "https://e.x/a.svg".into(),
                link: "https://e.x/a".into(),
            }],
        };
        let health = RepoHealth {
            domain: vec![Domain {
                name: "D".into(),
                summary: "s".into(),
                row: vec![row.clone(), row],
            }],
        };
        assert_eq!(health.rows(), 2);
        let json = serde_json::to_string(&health).unwrap();
        assert!(!json.contains("pane"));
        assert_eq!(serde_json::from_str::<RepoHealth>(&json).unwrap(), health);
    }
}
