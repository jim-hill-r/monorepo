//! Shared data types for pane.
//!
//! `pane_snapshot` builds a [`Snapshot`] from a checkout of the repo and writes
//! it as JSON; `pane_web` reads it back with [`Snapshot::from_json`]. Dates are
//! ISO 8601 strings (`YYYY-MM-DD`, or RFC 3339 for times) so the crate has no
//! date library and compiles to wasm unchanged.

use std::fmt;

pub mod health;
pub mod signals;

use serde::{Deserialize, Serialize};

/// The repo pane reviews, for links to files and releases on GitHub.
pub const REPO_URL: &str = "https://github.com/jimhillr/monorepo";

/// Version of the snapshot JSON format. Bump it for any change that an older
/// reader could not parse.
pub const SNAPSHOT_VERSION: u32 = 1;

/// Everything pane knows from one checkout of the repo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The format version; always [`SNAPSHOT_VERSION`] when written.
    pub version: u32,
    /// The commit the snapshot was taken from.
    pub commit: String,
    /// When the snapshot was taken, RFC 3339.
    pub taken_at: String,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub work_items: Vec<WorkItem>,
    #[serde(default)]
    pub human_issues: Vec<HumanIssue>,
    #[serde(default)]
    pub deliverables: Vec<Deliverable>,
    #[serde(default)]
    pub reviews: Vec<Review>,
    #[serde(default)]
    pub retros: Vec<Retro>,
    /// What every project's `docs/cost.toml` says, with the radar's budgets
    /// and the free limits, for the month the snapshot was taken in.
    #[serde(default)]
    pub costs: costs::Costs,
    /// The traffic-light thresholds from `pane/health.toml`, so the web app
    /// applies the same rules the snapshot was built with.
    #[serde(default)]
    pub thresholds: health::Thresholds,
    /// The repo audit's findings (`mise run //:audit`), or `None` when the
    /// snapshot was built without them (the Standards light is then grey).
    #[serde(default)]
    pub audit: Option<audit::Audit>,
}

impl Snapshot {
    /// An empty snapshot of `commit`, taken at `taken_at`, at the current
    /// format version.
    pub fn new(commit: impl Into<String>, taken_at: impl Into<String>) -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            commit: commit.into(),
            taken_at: taken_at.into(),
            projects: Vec::new(),
            work_items: Vec::new(),
            human_issues: Vec::new(),
            deliverables: Vec::new(),
            reviews: Vec::new(),
            retros: Vec::new(),
            costs: costs::Costs::default(),
            thresholds: health::Thresholds::default(),
            audit: None,
        }
    }

    /// Parse a snapshot, refusing one written with a different format version.
    pub fn from_json(json: &str) -> Result<Self, SnapshotError> {
        #[derive(Deserialize)]
        struct VersionOnly {
            version: u32,
        }
        let VersionOnly { version } =
            serde_json::from_str(json).map_err(|e| SnapshotError::Parse(e.to_string()))?;
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::Version {
                found: version,
                expected: SNAPSHOT_VERSION,
            });
        }
        serde_json::from_str(json).map_err(|e| SnapshotError::Parse(e.to_string()))
    }

    /// Serialize the snapshot as pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, SnapshotError> {
        serde_json::to_string_pretty(self).map_err(|e| SnapshotError::Parse(e.to_string()))
    }
}

/// Why a snapshot could not be read or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// The JSON was malformed or did not match the types.
    Parse(String),
    /// The snapshot was written with another format version.
    Version { found: u32, expected: u32 },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "snapshot is not valid: {e}"),
            Self::Version { found, expected } => write!(
                f,
                "snapshot format version {found} is not supported (expected {expected})"
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// One project from `[monorepo].config_roots`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Path from the repo root, such as `pane/web`.
    pub path: String,
    /// The project type: the `<type>` of its `standards/tasks/<type>.toml`.
    pub kind: String,
    /// Production URL, from `DEPLOY_URL` in its `mise.toml`.
    #[serde(default)]
    pub deploy_url: Option<String>,
    /// First paragraph of its README.
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub status: ProjectStatus,
}

/// Where a project stands in `standards/docs/issues/README.md`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Listed under `# Priority Projects`.
    Priority,
    /// Listed under `# On Hold Projects`.
    OnHold,
    #[default]
    Active,
}

/// One `TODO` or `FIX`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkItem {
    pub kind: WorkKind,
    /// The text after the marker and tags.
    pub text: String,
    /// The project it belongs to, or `None` for repo-wide files.
    #[serde(default)]
    pub project: Option<String>,
    /// Path from the repo root.
    pub file: String,
    /// 1-based line number (where an issue file's text starts).
    pub line: u32,
    pub source: WorkSource,
    /// `Priority Issues` or `Backlog`, from an issue file's `priority`.
    #[serde(default)]
    pub section: Option<String>,
    /// An issue file's `epic`, such as `Epic / Phase 1: start`.
    #[serde(default)]
    pub epic: Option<String>,
    /// An issue file's `tags`, or the tags in parentheses after a code
    /// comment's marker, such as `agent-generated`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// `YYYY-MM-DD` it was added: an issue file's `added`, or the `git
    /// blame` date of a `FIX` comment's line. `None` when unknown, such as
    /// a `TODO` comment or a line from a shallow clone's boundary commit.
    #[serde(default)]
    pub added: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkKind {
    Todo,
    Fix,
}

/// Where a work item was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkSource {
    IssuesFile,
    CodeComment,
}

/// One task for a person: a file in a project's `docs/issues/human/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanIssue {
    pub text: String,
    pub file: String,
    pub line: u32,
    /// `YYYY-MM-DD`, from the file's `added`.
    #[serde(default)]
    pub added: Option<String>,
    /// What it unblocks, from the file's `unblocks`.
    #[serde(default)]
    pub unblocks: Option<String>,
    /// Why only a person can do it, from the file's `## Why it can't be
    /// automated` section.
    #[serde(default)]
    pub why: Option<String>,
    /// Its runbook's path from the repo root, from the file's `runbook`.
    #[serde(default)]
    pub runbook: Option<String>,
    /// The project whose `docs/issues/human/` holds it; every project keeps
    /// its own human tasks and pane gathers them all.
    #[serde(default)]
    pub project: Option<String>,
}

/// Something pane offers to view or download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deliverable {
    pub kind: DeliverableKind,
    pub title: String,
    #[serde(default)]
    pub project: Option<String>,
    /// Path from the repo root, for files in the repo.
    #[serde(default)]
    pub path: Option<String>,
    /// Where to open or download it, for things outside the repo.
    #[serde(default)]
    pub url: Option<String>,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub date: Option<String>,
    /// A document's status line, such as `Accepted`.
    #[serde(default)]
    pub status: Option<String>,
    /// Files to download, such as a release's macOS and Android builds.
    #[serde(default)]
    pub downloads: Vec<Link>,
    /// The Markdown source of a plan, decision or retro, shown in the page.
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableKind {
    LiveSite,
    Plan,
    Decision,
    Retro,
    ReviewDeck,
    Release,
    UatReport,
}

/// An executive review deck: one row of the rotation table in
/// `docs/reviews/README.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub project: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Path to its `deck.md`, such as `docs/reviews/2026-11-25-cookbook/deck.md`.
    pub path: String,
    /// The review's pull request, where Jim leaves comments.
    #[serde(default)]
    pub pr: Option<String>,
    #[serde(default)]
    pub scorecard: Vec<ScorecardRow>,
}

impl Review {
    /// The deck's folder name, such as `2026-11-25-cookbook`: pane's release
    /// serves the built deck under `/reviews/<folder>/`.
    pub fn folder(&self) -> &str {
        review_folder(&self.path)
    }
}

/// The folder of a `docs/reviews/<folder>/deck.md` path.
pub fn review_folder(deck_path: &str) -> &str {
    deck_path.rsplit('/').nth(1).unwrap_or(deck_path)
}

/// One row of a deck's `| Area | Rating | Why |` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorecardRow {
    pub area: String,
    pub rating: String,
    pub why: String,
}

/// A retro and its actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retro {
    /// The project, or `None` for repo-wide retros in `standards/docs/retros/`.
    #[serde(default)]
    pub project: Option<String>,
    pub title: String,
    /// `YYYY-MM-DD`, from the file name.
    pub date: String,
    pub path: String,
    #[serde(default)]
    pub actions: Vec<RetroAction>,
}

/// One `- [ ]` or `- [x]` item under `## Actions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetroAction {
    pub text: String,
    pub done: bool,
}

/// A traffic-light colour.
///
/// Ordered from least to most severe, so the worst of several lights is the
/// maximum: `Grey < Green < Amber < Red`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    /// No data, or the signal does not apply.
    Grey,
    Green,
    Amber,
    Red,
}

/// One health signal for a project or system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    /// The signal's name, such as `Build` or `Open issues`.
    pub name: String,
    pub health: Health,
    /// One line on why it has this colour.
    pub reason: String,
    #[serde(default)]
    pub evidence: Vec<Link>,
}

/// A labelled link to evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

pub mod acks;
pub mod costs;
pub mod money;
pub mod queue;

pub mod audit;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Snapshot {
        let mut s = Snapshot::new("abc1234", "2026-11-26T12:00:00Z");
        s.projects.push(Project {
            path: "pane/web".into(),
            kind: "dioxus-web".into(),
            deploy_url: Some("https://jimhillr-pane.jim-hill-r.workers.dev".into()),
            summary: Some("The web frontend for pane.".into()),
            status: ProjectStatus::Priority,
        });
        s.work_items.push(WorkItem {
            kind: WorkKind::Todo,
            text: "Add a model crate.".into(),
            project: Some("pane".into()),
            file: "pane/docs/issues/x.md".into(),
            line: 14,
            source: WorkSource::IssuesFile,
            section: Some("Priority Issues".into()),
            epic: Some("Phase 0: foundations".into()),
            tags: vec!["agent-generated".into()],
            added: Some("2026-11-26".into()),
        });
        s.human_issues.push(HumanIssue {
            text: "Create the Access application.".into(),
            file: "standards/docs/issues/human/x.md".into(),
            line: 3,
            added: Some("2026-11-20".into()),
            unblocks: Some("pane Phase 3".into()),
            why: None,
            runbook: None,
            project: None,
        });
        s.deliverables.push(Deliverable {
            kind: DeliverableKind::Plan,
            title: "Plan: pane, one place to review the monorepo".into(),
            project: Some("pane".into()),
            path: Some("pane/docs/plans/2026-11-review-dashboard.md".into()),
            url: None,
            date: Some("2026-11-25".into()),
            status: None,
            downloads: vec![],
            body: Some("# Plan\n\nText.".into()),
        });
        s.deliverables.push(Deliverable {
            kind: DeliverableKind::Release,
            title: "game nightly".into(),
            project: Some("game".into()),
            path: None,
            url: Some("https://github.com/jimhillr/monorepo/releases/tag/game-nightly".into()),
            date: None,
            status: None,
            downloads: vec![Link {
                label: "macOS".into(),
                url: "https://github.com/jimhillr/monorepo/releases/download/game-nightly/game-macos.zip".into(),
            }],
            body: None,
        });
        s.reviews.push(Review {
            project: "pane".into(),
            date: "2026-11-24".into(),
            path: "docs/reviews/2026-11-24-pane/deck.md".into(),
            pr: Some("https://github.com/jimhillr/monorepo/pull/1".into()),
            scorecard: vec![ScorecardRow {
                area: "Tests".into(),
                rating: "Good".into(),
                why: "Covered.".into(),
            }],
        });
        s.retros.push(Retro {
            project: None,
            title: "Mise migration".into(),
            date: "2026-11-22".into(),
            path: "standards/docs/retros/2026-11-22-mise-migration.md".into(),
            actions: vec![RetroAction {
                text: "Document the fallback.".into(),
                done: true,
            }],
        });
        s
    }

    #[test]
    fn new_snapshot_has_current_version_and_is_empty() {
        let s = Snapshot::new("abc", "2026-11-26T00:00:00Z");
        assert_eq!(s.version, SNAPSHOT_VERSION);
        assert!(s.projects.is_empty() && s.work_items.is_empty() && s.retros.is_empty());
    }

    #[test]
    fn round_trips_through_json() -> Result<(), SnapshotError> {
        let s = sample();
        assert_eq!(Snapshot::from_json(&s.to_json()?)?, s);
        Ok(())
    }

    #[test]
    fn enums_serialize_as_snake_case() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ProjectStatus::OnHold)?,
            "\"on_hold\""
        );
        assert_eq!(
            serde_json::to_string(&WorkSource::CodeComment)?,
            "\"code_comment\""
        );
        assert_eq!(
            serde_json::to_string(&DeliverableKind::ReviewDeck)?,
            "\"review_deck\""
        );
        assert_eq!(serde_json::to_string(&WorkKind::Fix)?, "\"fix\"");
        assert_eq!(serde_json::to_string(&Health::Amber)?, "\"amber\"");
        Ok(())
    }

    #[test]
    fn missing_lists_and_optionals_default() -> Result<(), SnapshotError> {
        let json = r#"{"version":1,"commit":"abc","taken_at":"t",
            "projects":[{"path":"pane","kind":"rust"}]}"#;
        let s = Snapshot::from_json(json)?;
        assert_eq!(s.projects[0].status, ProjectStatus::Active);
        assert_eq!(s.projects[0].deploy_url, None);
        assert!(s.work_items.is_empty() && s.human_issues.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_other_versions() {
        let json = format!(
            r#"{{"version":{},"commit":"a","taken_at":"t"}}"#,
            SNAPSHOT_VERSION + 1
        );
        assert_eq!(
            Snapshot::from_json(&json),
            Err(SnapshotError::Version {
                found: SNAPSHOT_VERSION + 1,
                expected: SNAPSHOT_VERSION
            })
        );
    }

    #[test]
    fn rejects_missing_version_and_malformed_json() {
        assert!(matches!(
            Snapshot::from_json(r#"{"commit":"a","taken_at":"t"}"#),
            Err(SnapshotError::Parse(_))
        ));
        assert!(matches!(
            Snapshot::from_json("not json"),
            Err(SnapshotError::Parse(_))
        ));
    }

    #[test]
    fn rejects_right_version_with_wrong_shape() {
        let json = format!(r#"{{"version":{SNAPSHOT_VERSION},"commit":"a"}}"#);
        assert!(matches!(
            Snapshot::from_json(&json),
            Err(SnapshotError::Parse(_))
        ));
    }

    #[test]
    fn a_review_knows_its_folder() {
        let s = sample();
        assert_eq!(s.reviews[0].folder(), "2026-11-24-pane");
        assert_eq!(review_folder("deck.md"), "deck.md");
    }

    #[test]
    fn a_review_without_a_pr_still_parses() -> Result<(), SnapshotError> {
        let json = r#"{"version":1,"commit":"a","taken_at":"t","reviews":[
            {"project":"pane","date":"2026-11-24","path":"docs/reviews/x/deck.md"}]}"#;
        let s = Snapshot::from_json(json)?;
        assert_eq!(s.reviews[0].pr, None);
        Ok(())
    }

    #[test]
    fn health_orders_worst_last() {
        let lights = [Health::Green, Health::Red, Health::Grey, Health::Amber];
        assert_eq!(lights.iter().max(), Some(&Health::Red));
        assert!(Health::Grey < Health::Green && Health::Green < Health::Amber);
    }

    #[test]
    fn error_messages_are_readable() {
        let e = SnapshotError::Version {
            found: 2,
            expected: 1,
        };
        assert_eq!(
            e.to_string(),
            "snapshot format version 2 is not supported (expected 1)"
        );
    }

    #[test]
    fn signal_round_trips() -> Result<(), serde_json::Error> {
        let sig = Signal {
            name: "Build".into(),
            health: Health::Green,
            reason: "Passed 3 hours ago.".into(),
            evidence: vec![Link {
                label: "run".into(),
                url: "https://github.com/jimhillr/monorepo/actions/runs/1".into(),
            }],
        };
        let back: Signal = serde_json::from_str(&serde_json::to_string(&sig)?)?;
        assert_eq!(back, sig);
        Ok(())
    }
}
