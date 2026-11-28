//! Post-merge review (Jim, 2026-11-27, decision 0014): low-risk PRs merge
//! without his approval, so pane lists every merged PR for him to
//! acknowledge after the fact, and to flag one for a follow-up issue.
//!
//! The browser reads merged PRs from GitHub's public search with no token.
//! Which ones Jim has acknowledged is kept in the browser's `localStorage`
//! as an [`AckStore`]. This module does no I/O.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::queue::{Attention, Label, User, badges, summary};

/// The `localStorage` key for the [`AckStore`].
pub const STORAGE_KEY: &str = "pane.acks.v1";

/// Search results per page, GitHub's maximum.
pub const PER_PAGE: usize = 100;

/// The most search pages read in one load (500 merged PRs).
pub const MAX_PAGES: usize = 5;

/// The search for PRs merged on or after `since` (`YYYY-MM-DD`), page
/// `page` (from 1).
#[must_use]
pub fn merged_search_url(since: &str, page: usize) -> String {
    format!(
        "https://api.github.com/search/issues?q=repo%3Ajimhillr%2Fmonorepo+is%3Apr+is%3Amerged+merged%3A%3E%3D{since}&per_page={PER_PAGE}&page={page}"
    )
}

/// What the browser keeps: the first day it lists PRs from, and the PRs
/// acknowledged since then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckStore {
    /// `YYYY-MM-DD`: merged PRs from this day on are listed.
    pub since: String,
    #[serde(default)]
    pub acked: BTreeSet<u64>,
}

impl AckStore {
    /// The stored value, or a new store listing PRs from `today` when there
    /// is none or it doesn't parse.
    #[must_use]
    pub fn parse(stored: Option<&str>, today: &str) -> AckStore {
        stored
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| AckStore {
                since: today.to_owned(),
                acked: BTreeSet::new(),
            })
    }
}

#[derive(Deserialize)]
struct Search {
    items: Vec<MergedItem>,
}

#[derive(Deserialize)]
struct MergedItem {
    number: u64,
    title: String,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    labels: Vec<Label>,
    user: User,
    pull_request: MergedInfo,
}

#[derive(Deserialize)]
struct MergedInfo {
    #[serde(default)]
    merged_at: Option<String>,
}

/// One merged PR, as the Acknowledge page shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Merged {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub summary: String,
    pub attention: Attention,
    pub badges: Vec<String>,
    /// RFC 3339.
    pub merged_at: String,
}

/// Parse one page of [`merged_search_url`].
///
/// # Errors
///
/// When the JSON is not a search result.
pub fn parse_merged(json: &str) -> Result<Vec<Merged>, String> {
    let search: Search = serde_json::from_str(json).map_err(|e| format!("GitHub search: {e}"))?;
    Ok(search
        .items
        .into_iter()
        .map(|i| {
            let (badges, attention) = badges(&i.labels, &i.user.login);
            Merged {
                number: i.number,
                summary: summary(i.body.as_deref().unwrap_or_default()),
                title: i.title,
                url: i.html_url,
                author: i.user.login,
                attention,
                badges,
                merged_at: i.pull_request.merged_at.unwrap_or_default(),
            }
        })
        .collect())
}

/// The merged PRs not yet acknowledged, newest merge first.
#[must_use]
pub fn unacknowledged(merged: &[Merged], store: &AckStore) -> Vec<Merged> {
    let mut out: Vec<Merged> = merged
        .iter()
        .filter(|m| !store.acked.contains(&m.number))
        .cloned()
        .collect();
    out.sort_by(|a, b| b.merged_at.cmp(&a.merged_at).then(b.number.cmp(&a.number)));
    out
}

/// The project a PR title names, from its `project:` or `project/sub:`
/// prefix, when that is one of `projects` (the snapshot's paths).
#[must_use]
pub fn project_of<'a>(title: &str, projects: &'a [String]) -> Option<&'a str> {
    let prefix = title.split_once(':')?.0.trim();
    projects
        .iter()
        .find(|p| p.as_str() == prefix)
        .map(String::as_str)
}

/// A link to GitHub's new-file editor, filled in with a `fix` issue file
/// for a follow-up on `m`, in its project's `docs/issues/` folder (or the
/// repo-wide `standards/docs/issues/` when the title names no project). Committing it from GitHub
/// opens a PR from Jim's own account, so pane needs no token to flag.
#[must_use]
pub fn flag_url(m: &Merged, projects: &[String], today: &str) -> String {
    let dir = project_of(&m.title, projects).map_or_else(
        || "standards/docs/issues".to_owned(),
        |p| format!("{p}/docs/issues"),
    );
    let filename = format!("{dir}/follow-up-pr-{}.md", m.number);
    let title = m.title.replace(['\n', '\r'], " ");
    let value = format!(
        "---\nkind: fix\npriority: priority\nadded: {today}\n---\n\nFollow up on #{} \"{title}\" ({}), found in post-merge review: <say what is wrong>.\n",
        m.number, m.url
    );
    format!(
        "{}/new/main?filename={}&value={}",
        crate::REPO_URL,
        encode(&filename),
        encode(&value)
    )
}

/// Percent-encode everything but unreserved characters (RFC 3986).
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"{"total_count":3,"items":[
        {"number":5,"title":"pane: a thing","html_url":"https://github.com/jimhillr/monorepo/pull/5",
         "body":"**Summary.** Adds a thing.","labels":[],"user":{"login":"jimhillr-agt"},
         "pull_request":{"merged_at":"2026-11-27T10:00:00Z"}},
        {"number":6,"title":"infra: a risky thing","html_url":"https://github.com/jimhillr/monorepo/pull/6",
         "body":null,"labels":[{"name":"security"}],"user":{"login":"jimhillr-agt"},
         "pull_request":{"merged_at":"2026-11-27T12:00:00Z"}},
        {"number":7,"title":"Tidy","html_url":"https://github.com/jimhillr/monorepo/pull/7",
         "body":"Tidy.","labels":[],"user":{"login":"someone"},
         "pull_request":{"merged_at":"2026-11-26T12:00:00Z"}}]}"#;

    #[test]
    fn lists_unacknowledged_merged_prs_newest_first() -> Result<(), String> {
        let merged = parse_merged(PAGE)?;
        let store = AckStore {
            since: "2026-11-26".to_owned(),
            acked: [5].into(),
        };
        let left = unacknowledged(&merged, &store);
        assert_eq!(
            left.iter().map(|m| m.number).collect::<Vec<_>>(),
            vec![6, 7]
        );
        assert_eq!(left[0].attention, Attention::Look);
        assert_eq!(left[0].badges, vec!["security".to_owned()]);
        assert_eq!(left[1].badges, vec!["outside author".to_owned()]);
        assert_eq!(merged[0].summary, "Adds a thing.");
        Ok(())
    }

    #[test]
    fn a_missing_or_broken_store_starts_today() {
        let fresh = AckStore::parse(None, "2026-11-27");
        assert_eq!(fresh.since, "2026-11-27");
        assert!(fresh.acked.is_empty());
        assert_eq!(AckStore::parse(Some("not json"), "2026-11-27"), fresh);
        let kept = AckStore::parse(
            Some(r#"{"since":"2026-11-20","acked":[3,4]}"#),
            "2026-11-27",
        );
        assert_eq!(kept.since, "2026-11-20");
        assert_eq!(kept.acked, [3, 4].into());
    }

    #[test]
    fn flag_links_a_prefilled_issue_file_in_the_project() -> Result<(), String> {
        let merged = parse_merged(PAGE)?;
        let projects = vec!["pane".to_owned(), "pane/web".to_owned()];
        let url = flag_url(&merged[0], &projects, "2026-11-28");
        assert!(url.starts_with("https://github.com/jimhillr/monorepo/new/main?filename=pane%2Fdocs%2Fissues%2Ffollow-up-pr-5.md&value="));
        assert!(url.contains("kind%3A%20fix"));
        assert!(url.contains("added%3A%202026-11-28"));
        // No project prefix: the repo-wide standards/docs/issues/ folder.
        let root = flag_url(&merged[2], &projects, "2026-11-28");
        assert!(root.contains("filename=standards%2Fdocs%2Fissues%2Ffollow-up-pr-7.md"));
        Ok(())
    }

    #[test]
    fn the_search_url_filters_by_merge_date() {
        let url = merged_search_url("2026-11-27", 2);
        assert!(url.contains("is%3Amerged+merged%3A%3E%3D2026-11-27"));
        assert!(url.ends_with("&per_page=100&page=2"));
    }
}
