//! Fan votes on each verdict (BEAR-010, `docs/plans/2026-11-fan-feedback.md`
//! step 3): the events the page posts to the `telemetry` Worker, and the key
//! that keeps one vote per game, tab and week in this browser. Pure logic,
//! tested natively; `main.rs` does the browser calls.

use bear_down_core::Tier;
use serde::Serialize;

use crate::data::Goal;

/// Where votes go: the `telemetry` Worker (`../../telemetry/README.md`).
pub const TELEMETRY_URL: &str = "https://jimhillr-telemetry.jim-hill-r.workers.dev/event";

/// The most characters a note may have; the Worker cuts longer ones.
pub const MAX_NOTE: usize = 200;

/// A fan's answer to "Does this verdict look right?".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
    Confusing,
}

impl Vote {
    pub const ALL: [Vote; 3] = [Vote::Yes, Vote::No, Vote::Confusing];

    pub fn label(self) -> &'static str {
        match self {
            Vote::Yes => "Yes",
            Vote::No => "No",
            Vote::Confusing => "Confusing",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Vote::Yes => "yes",
            Vote::No => "no",
            Vote::Confusing => "confusing",
        }
    }
}

impl Goal {
    /// The tab's name in events: `super_bowl`, `playoffs` or `nfc_north`.
    pub fn tab(self) -> &'static str {
        match self {
            Goal::SuperBowl => "super_bowl",
            Goal::Playoffs => "playoffs",
            Goal::NfcNorth => "nfc_north",
        }
    }
}

/// What the page showed for a game: the team to root for, or why there is
/// none.
pub fn verdict(root: Option<&str>, tier: Tier) -> String {
    match (root, tier) {
        (Some(team), _) => team.to_string(),
        (None, Tier::None) => "cant_matter".to_string(),
        (None, Tier::Small) => "too_close".to_string(),
        (None, Tier::Matters | Tier::Negligible) => "doesnt_matter".to_string(),
    }
}

/// The `localStorage` key that records a vote, so each browser votes once
/// per game, tab and week.
pub fn vote_key(season: u16, week: u8, goal: Goal, game: &str) -> String {
    format!("bear_down.vote.{season}.{week}.{}.{game}", goal.tab())
}

/// One `feedback` event, as the Worker takes it.
#[derive(Debug, Serialize)]
struct Feedback<'a> {
    kind: &'static str,
    project: &'static str,
    arm: &'a str,
    season: u16,
    week: u8,
    tab: &'static str,
    game: &'a str,
    verdict: &'a str,
    vote: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<&'a str>,
}

/// Where a vote was cast: the game and what the page showed for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Ballot {
    pub arm: String,
    pub season: u16,
    pub week: u8,
    pub goal: Goal,
    pub game: String,
    pub verdict: String,
}

impl Ballot {
    /// The event for a vote.
    pub fn vote(&self, vote: Vote) -> String {
        self.event(vote.name(), None)
    }

    /// The follow-up event for the optional note on a vote just cast. Its
    /// `vote` is `note`, so the weekly export counts each vote once.
    pub fn note(&self, note: &str) -> Option<String> {
        let note = note.trim();
        if note.is_empty() {
            return None;
        }
        let end = note
            .char_indices()
            .nth(MAX_NOTE)
            .map_or(note.len(), |(index, _)| index);
        Some(self.event("note", Some(&note[..end])))
    }

    fn event(&self, vote: &'static str, note: Option<&str>) -> String {
        serde_json::to_string(&Feedback {
            kind: "feedback",
            project: "bear_down",
            arm: &self.arm,
            season: self.season,
            week: self.week,
            tab: self.goal.tab(),
            game: &self.game,
            verdict: &self.verdict,
            vote,
            note,
        })
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ballot() -> Ballot {
        Ballot {
            arm: "a".to_string(),
            season: 2026,
            week: 12,
            goal: Goal::SuperBowl,
            game: "2026_12_PHI_DAL".to_string(),
            verdict: verdict(None, Tier::Negligible),
        }
    }

    #[test]
    fn a_vote_is_one_feedback_event() {
        assert_eq!(
            ballot().vote(Vote::No),
            r#"{"kind":"feedback","project":"bear_down","arm":"a","season":2026,"week":12,"tab":"super_bowl","game":"2026_12_PHI_DAL","verdict":"doesnt_matter","vote":"no"}"#
        );
    }

    #[test]
    fn a_note_follows_its_vote_and_is_cut() {
        assert_eq!(ballot().note("   "), None);
        let Some(note) = ballot().note(" Felt like it mattered ") else {
            panic!("a note should make an event");
        };
        assert!(note.ends_with(r#""vote":"note","note":"Felt like it mattered"}"#));
        let Some(long) = ballot().note(&"é".repeat(300)) else {
            panic!("a long note should make an event");
        };
        assert_eq!(long.matches('é').count(), MAX_NOTE);
    }

    #[test]
    fn verdicts_name_the_pick_or_why_there_is_none() {
        assert_eq!(verdict(Some("CHI"), Tier::Matters), "CHI");
        assert_eq!(verdict(None, Tier::Small), "too_close");
        assert_eq!(verdict(None, Tier::None), "cant_matter");
        assert_eq!(verdict(None, Tier::Negligible), "doesnt_matter");
    }

    #[test]
    fn each_game_tab_and_week_has_its_own_vote() {
        assert_eq!(
            vote_key(2026, 12, Goal::Playoffs, "2026_12_CHI_DET"),
            "bear_down.vote.2026.12.playoffs.2026_12_CHI_DET"
        );
        assert_ne!(
            vote_key(2026, 12, Goal::Playoffs, "g"),
            vote_key(2026, 12, Goal::SuperBowl, "g")
        );
    }
}
