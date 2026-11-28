//! "New since your last visit" for the Today page.
//!
//! Work items, human issues and deliverables have no stable ids and most have
//! no date, so pane remembers what it showed instead: a short hash of each
//! item's identity (kind, project and text, or a document's path), with the
//! snapshot's commit and the time. The browser keeps that in `localStorage`
//! (see `main.rs`); everything here is pure so it is tested natively.
//!
//! Two visits are kept. While the snapshot's commit stays the same, the page
//! keeps comparing with the visit before it, so a reload doesn't clear what
//! is new. Once a newer snapshot loads, the remembered one becomes the
//! baseline.

use std::collections::HashSet;

use pane_model::{Deliverable, HumanIssue, Snapshot, WorkItem};
use serde::{Deserialize, Serialize};

/// The `localStorage` key pane keeps its visits under.
pub const STORAGE_KEY: &str = "pane.visits.v1";

/// What one visit saw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visit {
    /// The snapshot's commit.
    pub commit: String,
    /// When the visit was, in Unix seconds.
    pub at: i64,
    /// [`key_hash`] of every item shown.
    pub seen: Vec<String>,
}

/// The latest visit, and the one before it with a different snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visits {
    pub current: Visit,
    #[serde(default)]
    pub previous: Option<Visit>,
}

/// FNV-1a, 64 bits, as 16 hex digits: short and stable across builds.
pub fn key_hash(key: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn work_key(i: &WorkItem) -> String {
    format!(
        "w|{:?}|{}|{}",
        i.kind,
        i.project.as_deref().unwrap_or_default(),
        i.text
    )
}

fn human_key(i: &HumanIssue) -> String {
    format!("h|{}", i.text)
}

fn deliverable_key(d: &Deliverable) -> String {
    let id = d.path.as_deref().or(d.url.as_deref()).unwrap_or(&d.title);
    format!("d|{:?}|{id}", d.kind)
}

/// The hashes of everything in `snapshot` that Today can call new.
pub fn seen(snapshot: &Snapshot) -> Vec<String> {
    let mut keys: Vec<String> = snapshot
        .work_items
        .iter()
        .map(work_key)
        .chain(snapshot.human_issues.iter().map(human_key))
        .chain(snapshot.deliverables.iter().map(deliverable_key))
        .map(|k| key_hash(&k))
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

/// The visits to store after showing `snapshot` at `now`, given what was
/// stored before (`None` on a first visit or without `localStorage`).
pub fn record(stored: Option<Visits>, snapshot: &Snapshot, now: i64) -> Visits {
    let visit = Visit {
        commit: snapshot.commit.clone(),
        at: now,
        seen: seen(snapshot),
    };
    match stored {
        // Same snapshot: keep the baseline, so a reload still shows what's new.
        Some(v) if v.current.commit == snapshot.commit => Visits {
            current: Visit {
                at: now,
                ..v.current
            },
            previous: v.previous,
        },
        Some(v) => Visits {
            current: visit,
            previous: Some(v.current),
        },
        None => Visits {
            current: visit,
            previous: None,
        },
    }
}

/// Parse stored visits, treating anything unreadable as none.
pub fn parse(json: Option<&str>) -> Option<Visits> {
    serde_json::from_str(json?).ok()
}

/// What `snapshot` has that `baseline` didn't see.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NewItems<'a> {
    pub human_issues: Vec<&'a HumanIssue>,
    pub work_items: Vec<&'a WorkItem>,
    pub deliverables: Vec<&'a Deliverable>,
}

impl NewItems<'_> {
    pub fn len(&self) -> usize {
        self.human_issues.len() + self.work_items.len() + self.deliverables.len()
    }
}

/// Items in `snapshot` whose hash isn't in `baseline`, each list in the
/// order the other pages use.
pub fn new_since<'a>(snapshot: &'a Snapshot, baseline: &Visit) -> NewItems<'a> {
    let old: HashSet<&str> = baseline.seen.iter().map(String::as_str).collect();
    let is_new = |key: String| !old.contains(key_hash(&key).as_str());
    NewItems {
        human_issues: crate::view::sorted_human_issues(snapshot)
            .into_iter()
            .filter(|i| is_new(human_key(i)))
            .collect(),
        work_items: crate::view::filter_work(&snapshot.work_items, &Default::default())
            .into_iter()
            .filter(|i| is_new(work_key(i)))
            .collect(),
        deliverables: crate::view::DELIVERABLE_GROUPS
            .iter()
            .flat_map(|(kind, _)| crate::view::deliverables_of(snapshot, *kind))
            .filter(|d| is_new(deliverable_key(d)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pane_model::{DeliverableKind, WorkKind, WorkSource};

    fn todo(project: &str, text: &str, line: u32) -> WorkItem {
        WorkItem {
            kind: WorkKind::Todo,
            text: text.into(),
            project: Some(project.into()),
            file: format!("{project}/issues/x.md"),
            line,
            source: WorkSource::IssuesFile,
            section: Some("Priority Issues".into()),
            epic: None,
            tags: vec![],
            added: None,
        }
    }

    fn human(text: &str) -> HumanIssue {
        HumanIssue {
            text: text.into(),
            file: "standards/docs/issues/human/x.md".into(),
            line: 1,
            added: None,
            unblocks: None,
            why: None,
            runbook: None,
            project: None,
        }
    }

    fn plan(path: &str) -> Deliverable {
        Deliverable {
            kind: DeliverableKind::Plan,
            title: path.into(),
            project: None,
            path: Some(path.into()),
            url: None,
            date: None,
            status: None,
            downloads: vec![],
            body: None,
        }
    }

    fn snapshot(commit: &str) -> Snapshot {
        let mut s = Snapshot::new(commit, "2026-11-26T12:00:00Z");
        s.work_items = vec![todo("pane", "Build Today.", 3)];
        s.human_issues = vec![human("Make a token.")];
        s.deliverables = vec![plan("pane/docs/plans/a.md")];
        s
    }

    #[test]
    fn hashes_are_stable_and_distinct() {
        assert_eq!(key_hash(""), "cbf29ce484222325");
        assert_eq!(key_hash("a"), "af63dc4c8601ec8c");
        assert_ne!(key_hash("w|Todo|a|x"), key_hash("w|Fix|a|x"));
    }

    #[test]
    fn first_visit_has_no_baseline() {
        let s = snapshot("a");
        let v = record(None, &s, 100);
        assert_eq!(v.current.commit, "a");
        assert_eq!(v.current.seen.len(), 3);
        assert_eq!(v.previous, None);
    }

    #[test]
    fn a_new_snapshot_moves_the_last_visit_to_the_baseline() {
        let first = record(None, &snapshot("a"), 100);
        let second = record(Some(first.clone()), &snapshot("b"), 200);
        assert_eq!(second.current.commit, "b");
        assert_eq!(second.current.at, 200);
        assert_eq!(second.previous, Some(first.current));
    }

    #[test]
    fn a_reload_of_the_same_snapshot_keeps_the_baseline() {
        let first = record(None, &snapshot("a"), 100);
        let second = record(Some(first), &snapshot("b"), 200);
        let reload = record(Some(second.clone()), &snapshot("b"), 300);
        assert_eq!(reload.previous, second.previous);
        assert_eq!(reload.current.at, 300);
        assert_eq!(reload.current.seen, second.current.seen);
    }

    #[test]
    fn finds_items_added_since_the_baseline() {
        let old = snapshot("a");
        let baseline = record(None, &old, 100).current;
        let mut s = snapshot("b");
        // Moved to another line: not new.
        s.work_items[0].line = 9;
        s.work_items.push(todo("pane", "Build Project.", 4));
        s.human_issues.push(human("Approve the PR."));
        s.deliverables.push(plan("pane/docs/plans/b.md"));
        let new = new_since(&s, &baseline);
        assert_eq!(new.len(), 3);
        assert_eq!(new.work_items[0].text, "Build Project.");
        assert_eq!(new.human_issues[0].text, "Approve the PR.");
        assert_eq!(
            new.deliverables[0].path.as_deref(),
            Some("pane/docs/plans/b.md")
        );
        assert_eq!(new_since(&old, &baseline).len(), 0);
    }

    #[test]
    fn a_changed_text_counts_as_new() {
        let baseline = record(None, &snapshot("a"), 100).current;
        let mut s = snapshot("b");
        s.work_items[0].text = "Build Today, with visits.".into();
        assert_eq!(new_since(&s, &baseline).work_items.len(), 1);
    }

    #[test]
    fn round_trips_and_ignores_bad_storage() {
        let v = record(None, &snapshot("a"), 100);
        let json = serde_json::to_string(&v).unwrap_or_default();
        assert_eq!(parse(Some(&json)), Some(v));
        assert_eq!(parse(None), None);
        assert_eq!(parse(Some("not json")), None);
        assert_eq!(parse(Some(r#"{"current":1}"#)), None);
    }
}
