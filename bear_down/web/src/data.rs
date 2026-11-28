//! The `rooting.json` model the `cli` writes, and the pure logic the page
//! uses to rank and describe games. Probabilities are 0..1 floats and teams
//! are nflverse abbreviations.

use bear_down_core::{Tier, tier_against};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Rooting {
    pub generated_at: String,
    pub season: u16,
    pub week: Option<u8>,
    pub simulations: u32,
    pub latest_game: Option<LatestGame>,
    /// The team the guide is for (decision 0010). Missing in older files,
    /// which are all the Bears'.
    #[serde(default = "bears")]
    pub team: String,
    /// That team's record and odds; `bears` in older files.
    #[serde(alias = "bears")]
    pub focus: Focus,
    /// The celebration banner the nightly build worked out: only after a
    /// win by the guide's team. Missing in older files.
    #[serde(default)]
    pub banner: Option<Banner>,
    pub nfc_north_divisional: f64,
    pub games: Vec<Game>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LatestGame {
    pub id: String,
    pub gameday: String,
    pub away: String,
    pub home: String,
    pub away_score: u16,
    pub home_score: u16,
}

/// The Bears' abbreviation, the team of files without a `team`.
fn bears() -> String {
    bear_down_core::BEARS.to_string()
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Focus {
    pub record: Record,
    pub odds: Odds,
    /// Games ahead of the best division rival; missing in older files.
    #[serde(default)]
    pub division_lead: Option<f64>,
    /// That rival.
    #[serde(default)]
    pub division_rival: Option<String>,
}

/// The celebration banner: a headline and the lines under it, picked by the
/// CLI from the current results (`cli/src/banner.rs`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Banner {
    pub headline: String,
    #[serde(default)]
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Record {
    pub wins: u8,
    pub losses: u8,
    pub ties: u8,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Odds {
    pub playoffs: f64,
    pub divisional_round: f64,
    pub conference_game: f64,
    pub super_bowl: f64,
    pub champion: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Game {
    pub id: String,
    pub gameday: String,
    pub away: String,
    pub home: String,
    pub home_win_probability: f64,
    pub champion: Swing,
    pub playoffs: Swing,
    pub nfc_north: Swing,
}

/// One goal's odds conditioned on a game's result.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Swing {
    pub if_home_wins: f64,
    pub if_away_wins: f64,
    /// `if_home_wins - if_away_wins`.
    pub swing: f64,
    /// The standard error of `swing`. Missing in older files, and null when
    /// nothing was simulated.
    #[serde(default)]
    pub std_error: Option<f64>,
    /// The team to root for, or `None` when the swing is within noise.
    pub root_for: Option<String>,
    /// How much the game matters: `matters`, `small`, `negligible` or
    /// `none`. Missing in older files.
    #[serde(default)]
    pub tier: String,
    /// The plain-language explanation of the verdict. Missing in older files.
    #[serde(default)]
    pub why: Vec<String>,
}

impl Swing {
    /// The verdict against another "doesn't matter" `line`, for an A/B arm
    /// (`docs/plans/2026-11-fan-feedback.md`): the team to root for and the
    /// tier, by the CLI's rule ([`tier_against`], and a pick needs a swing
    /// more than two standard errors from zero and a tier of matters or
    /// small). A game that can't matter, or a file without `std_error`,
    /// keeps the published verdict. Only for the Super Bowl and playoff
    /// goals; the NFC North goal has no line.
    pub fn verdict_at(&self, game: &Game, line: f64) -> (Option<String>, Tier) {
        let published = Tier::from_name(&self.tier).unwrap_or(Tier::Negligible);
        let std_error = match self.std_error {
            Some(std_error) if published != Tier::None => std_error,
            _ => return (self.root_for.clone(), published),
        };
        let tier = tier_against(self.swing, std_error, line);
        let clear = self.swing.abs() > 2.0 * std_error;
        let root = match tier {
            Tier::Matters | Tier::Small if clear => Some(if self.swing > 0.0 {
                game.home.clone()
            } else {
                game.away.clone()
            }),
            _ => None,
        };
        (root, tier)
    }
}

/// The verdict shown when there is no team to root for.
pub fn no_pick_label(tier: Tier) -> &'static str {
    match tier {
        Tier::None => "Can't matter",
        Tier::Small => "Too close to call",
        Tier::Matters | Tier::Negligible => "Doesn't matter",
    }
}

/// What the rankings optimize.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Goal {
    /// The guide's team winning the Super Bowl (decision 0002).
    #[default]
    SuperBowl,
    /// The guide's team making the playoffs (Jim, 2026-11-26).
    Playoffs,
    /// All four NFC North teams reaching the divisional round (decision 0007).
    NfcNorth,
}

impl Goal {
    pub fn swing(self, game: &Game) -> &Swing {
        match self {
            Goal::SuperBowl => &game.champion,
            Goal::Playoffs => &game.playoffs,
            Goal::NfcNorth => &game.nfc_north,
        }
    }

    /// The label after a swing, as in "+1.8% Super Bowl odds".
    /// The game's verdict for this goal against a "doesn't matter" `line`:
    /// the team to root for and the tier. The NFC North goal has no line, so
    /// it keeps the published verdict.
    pub fn verdict(self, game: &Game, line: f64) -> (Option<String>, Tier) {
        let swing = self.swing(game);
        match self {
            Goal::NfcNorth => (
                swing.root_for.clone(),
                Tier::from_name(&swing.tier).unwrap_or(Tier::Negligible),
            ),
            Goal::SuperBowl | Goal::Playoffs => swing.verdict_at(game, line),
        }
    }

    pub fn odds_label(self) -> &'static str {
        match self {
            Goal::SuperBowl => "Super Bowl odds",
            Goal::Playoffs => "playoff odds",
            Goal::NfcNorth => "NFC North odds",
        }
    }
}

impl Rooting {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("rooting.json is not valid: {e}"))
    }

    /// The week's games, biggest swing for `goal` first. Ties keep schedule
    /// order.
    pub fn ranked(&self, goal: Goal) -> Vec<&Game> {
        let mut games: Vec<&Game> = self.games.iter().collect();
        games.sort_by(|a, b| {
            let (a, b) = (goal.swing(a).swing.abs(), goal.swing(b).swing.abs());
            b.total_cmp(&a)
        });
        games
    }

    /// True when the guide's team can no longer make the playoffs.
    pub fn eliminated(&self) -> bool {
        self.focus.odds.playoffs <= 0.0
    }

    /// Whether the guide's team plays in the NFC North, so the NFC North
    /// goal (decision 0007) is offered.
    pub fn nfc_north_team(&self) -> bool {
        bear_down_core::is_nfc_north(&self.team)
    }
}

impl Record {
    /// "8-4" or "8-3-1".
    pub fn display(&self) -> String {
        if self.ties == 0 {
            format!("{}-{}", self.wins, self.losses)
        } else {
            format!("{}-{}-{}", self.wins, self.losses, self.ties)
        }
    }
}

/// A probability as a percentage, with more decimals as it gets smaller so
/// tiny NFC North odds still show (decision 0007).
pub fn percent(p: f64) -> String {
    let pct = p * 100.0;
    let a = pct.abs();
    if a == 0.0 {
        "0%".to_string()
    } else if a >= 10.0 {
        format!("{pct:.0}%")
    } else if a >= 1.0 {
        format!("{pct:.1}%")
    } else if a >= 0.1 {
        format!("{pct:.2}%")
    } else if a >= 0.001 {
        format!("{pct:.3}%")
    } else if pct > 0.0 {
        "<0.001%".to_string()
    } else {
        "-<0.001%".to_string()
    }
}

/// A swing in favor of the team to root for: always shown as a gain, as in
/// "+1.8%".
pub fn swing_percent(swing: f64) -> String {
    format!("+{}", percent(swing.abs()))
}

/// "2026-11-24T13:05:00Z" as "2026-11-24 13:05 UTC". Anything unexpected is
/// shown as given.
pub fn timestamp(rfc3339: &str) -> String {
    match (rfc3339.get(..10), rfc3339.get(11..16)) {
        (Some(date), Some(time)) if rfc3339.as_bytes().get(10) == Some(&b'T') => {
            let zone = rfc3339.get(19..).unwrap_or("");
            let zone = zone.trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
            let zone = if zone.is_empty() || zone == "Z" || zone == "+00:00" {
                "UTC"
            } else {
                zone
            };
            format!("{date} {time} {zone}")
        }
        _ => rfc3339.to_string(),
    }
}

/// `archive/index.json`: the past weeks whose guides the site has rebuilt
/// ([archive plan](../../docs/plans/2026-11-archive.md)). Each week's guide
/// is `archive/<season>-week-<nn>.json`, in the `rooting.json` format.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ArchiveIndex {
    pub weeks: Vec<ArchiveWeek>,
}

/// One past week in the archive.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveWeek {
    pub season: u16,
    pub week: u8,
}

impl ArchiveIndex {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("archive/index.json is not valid: {e}"))
    }

    /// The weeks to offer, newest first, once each, leaving out `current`
    /// (the week the live guide shows).
    pub fn newest_first(&self, current: Option<ArchiveWeek>) -> Vec<ArchiveWeek> {
        let mut weeks: Vec<ArchiveWeek> = self
            .weeks
            .iter()
            .copied()
            .filter(|week| Some(*week) != current)
            .collect();
        weeks.sort_unstable_by(|a, b| b.cmp(a));
        weeks.dedup();
        weeks
    }
}

impl ArchiveWeek {
    /// The week's guide, as in `/archive/2026-week-07.json`.
    pub fn path(self) -> String {
        format!("/archive/{}-week-{:02}.json", self.season, self.week)
    }

    /// The picker's value for the week, as in `2026-07`.
    pub fn value(self) -> String {
        format!("{}-{:02}", self.season, self.week)
    }

    /// The week a picker value names; `None` for anything else, such as
    /// the empty "this week" value.
    pub fn from_value(value: &str) -> Option<Self> {
        let (season, week) = value.split_once('-')?;
        Some(Self {
            season: season.parse().ok()?,
            week: week.parse().ok().filter(|week| *week >= 1)?,
        })
    }

    /// "Week 7, 2026 season".
    pub fn label(self) -> String {
        format!("Week {}, {} season", self.week, self.season)
    }
}

impl Rooting {
    /// The week this guide is for, as an archive entry; `None` once the
    /// regular season is over.
    pub fn archive_week(&self) -> Option<ArchiveWeek> {
        self.week.map(|week| ArchiveWeek {
            season: self.season,
            week,
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../public/rooting.json");

    #[test]
    fn parses_the_sample() {
        let r = Rooting::parse(SAMPLE).expect("sample parses");
        assert!(!r.games.is_empty());
        // At most one Bears game is left in a week (none once it's played).
        assert!(
            r.games
                .iter()
                .filter(|g| g.home == "CHI" || g.away == "CHI")
                .count()
                <= 1
        );
    }

    #[test]
    fn sample_swings_are_consistent() {
        let r = Rooting::parse(SAMPLE).expect("sample parses");
        for g in &r.games {
            for s in [&g.champion, &g.playoffs, &g.nfc_north] {
                assert!(
                    (s.if_home_wins - s.if_away_wins - s.swing).abs() < 1e-9,
                    "{}",
                    g.id
                );
                if let Some(team) = &s.root_for {
                    let favored = if s.swing > 0.0 { &g.home } else { &g.away };
                    assert_eq!(team, favored, "{}", g.id);
                }
            }
        }
    }

    #[test]
    fn ranks_by_absolute_swing_for_the_goal() {
        let r = Rooting::parse(SAMPLE).expect("sample parses");
        for goal in [Goal::SuperBowl, Goal::Playoffs, Goal::NfcNorth] {
            let swings: Vec<f64> = r
                .ranked(goal)
                .iter()
                .map(|g| goal.swing(g).swing.abs())
                .collect();
            assert!(swings.windows(2).all(|w| w[0] >= w[1]), "{goal:?}");
        }
        // The Bears' own game, while it's still to play, leads the Super
        // Bowl and playoff rankings.
        if r.games.iter().any(|g| g.home == "CHI" || g.away == "CHI") {
            for goal in [Goal::SuperBowl, Goal::Playoffs] {
                let top = r.ranked(goal)[0];
                assert!(top.home == "CHI" || top.away == "CHI", "{goal:?}");
            }
        }
    }

    #[test]
    fn verdicts_at_the_cli_line_match_the_published_ones() {
        let rooting = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting,
            Err(error) => panic!("sample should parse: {error}"),
        };
        let line = bear_down_core::NEGLIGIBLE_SWING;
        for game in &rooting.games {
            for swing in [&game.champion, &game.playoffs] {
                assert!(swing.std_error.is_some(), "{} has no std_error", game.id);
                let (root, tier) = swing.verdict_at(game, line);
                assert_eq!(root, swing.root_for, "{}", game.id);
                assert_eq!(tier.name(), swing.tier, "{}", game.id);
            }
        }
    }

    #[test]
    fn a_lower_line_turns_small_swings_into_picks() {
        let first = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting.games[0].clone(),
            Err(error) => panic!("sample should parse: {error}"),
        };
        let game = |swing: f64, std_error: f64| Game {
            champion: Swing {
                if_home_wins: 0.0,
                if_away_wins: 0.0,
                swing,
                std_error: Some(std_error),
                root_for: None,
                tier: "negligible".to_string(),
                why: Vec::new(),
            },
            ..first.clone()
        };
        // 0.03 points with a 0.002-point standard error: under 0.05, over 0.02.
        let small = game(-0.0003, 0.00002);
        assert_eq!(
            small.champion.verdict_at(&small, 0.0005),
            (None, Tier::Negligible)
        );
        assert_eq!(
            small.champion.verdict_at(&small, 0.0002),
            (Some(small.away.clone()), Tier::Matters)
        );
        // A game that can't matter stays that way at any line.
        let mut closed = game(0.0, 0.0);
        closed.champion.tier = "none".to_string();
        assert_eq!(closed.champion.verdict_at(&closed, 0.0), (None, Tier::None));
    }

    #[test]
    fn no_pick_labels_follow_the_tier() {
        let swing = |tier: &str| Swing {
            if_home_wins: 0.5,
            if_away_wins: 0.5,
            swing: 0.0,
            std_error: None,
            root_for: None,
            tier: tier.to_string(),
            why: Vec::new(),
        };
        let first = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting.games[0].clone(),
            Err(error) => panic!("sample should parse: {error}"),
        };
        let label = |tier: &str| no_pick_label(swing(tier).verdict_at(&first, 0.0005).1);
        assert_eq!(label("none"), "Can't matter");
        assert_eq!(label("small"), "Too close to call");
        assert_eq!(label("negligible"), "Doesn't matter");
        // Older files have no tier.
        assert_eq!(label(""), "Doesn't matter");
    }

    #[test]
    fn formats_percentages() {
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(0.42), "42%");
        assert_eq!(percent(0.018), "1.8%");
        assert_eq!(percent(0.0042), "0.42%");
        assert_eq!(percent(0.00042), "0.042%");
        assert_eq!(percent(0.000_000_1), "<0.001%");
        assert_eq!(swing_percent(-0.018), "+1.8%");
    }

    #[test]
    fn formats_records_and_timestamps() {
        let r = Record {
            wins: 8,
            losses: 3,
            ties: 1,
        };
        assert_eq!(r.display(), "8-3-1");
        assert_eq!(Record { ties: 0, ..r }.display(), "8-3");
        assert_eq!(timestamp("2026-11-24T13:05:00Z"), "2026-11-24 13:05 UTC");
        assert_eq!(
            timestamp("2026-11-24T13:05:00.123+00:00"),
            "2026-11-24 13:05 UTC"
        );
        assert_eq!(
            timestamp("2026-11-24T08:05:00-05:00"),
            "2026-11-24 08:05 -05:00"
        );
        assert_eq!(timestamp("soon"), "soon");
    }

    #[test]
    fn the_banner_is_read_from_the_file() {
        let mut rooting = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting,
            Err(error) => panic!("sample should parse: {error}"),
        };
        let banner = rooting.banner.take();
        assert_eq!(
            banner.as_ref().map(|b| b.headline.as_str()),
            Some("Bears win! Chicago 39, Lions 35.")
        );
        assert!(banner.is_some_and(|b| !b.lines.is_empty()));
        // No banner after a loss (null), or in an older file (missing).
        let mut json: serde_json::Value = match serde_json::from_str(SAMPLE) {
            Ok(json) => json,
            Err(error) => panic!("sample is JSON: {error}"),
        };
        json["banner"] = serde_json::Value::Null;
        assert_eq!(
            Rooting::parse(&json.to_string()).map(|r| r.banner),
            Ok(None)
        );
        if let Some(fields) = json.as_object_mut() {
            fields.remove("banner");
        }
        assert_eq!(
            Rooting::parse(&json.to_string()).map(|r| r.banner),
            Ok(None)
        );
    }

    #[test]
    fn eliminated_when_playoff_odds_are_zero() {
        let mut r = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting,
            Err(error) => panic!("sample should parse: {error}"),
        };
        assert!(!r.eliminated());
        r.focus.odds.playoffs = 0.0;
        assert!(r.eliminated());
    }

    #[test]
    fn a_guide_names_its_team_and_older_files_are_the_bears() {
        let r = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting,
            Err(error) => panic!("sample should parse: {error}"),
        };
        assert_eq!(r.team, "CHI");
        assert!(r.nfc_north_team());
        // An older file: no `team`, and `bears` for the team's odds.
        let mut json: serde_json::Value = match serde_json::from_str(SAMPLE) {
            Ok(json) => json,
            Err(error) => panic!("sample is JSON: {error}"),
        };
        if let Some(fields) = json.as_object_mut() {
            fields.remove("team");
            if let Some(focus) = fields.remove("focus") {
                fields.insert("bears".to_string(), focus);
            }
        }
        let older = Rooting::parse(&json.to_string());
        assert_eq!(older.as_ref().map(|r| r.team.as_str()), Ok("CHI"));
        assert_eq!(older.map(|r| r.focus), Ok(r.focus.clone()));
        json["team"] = "KC".into();
        let chiefs = Rooting::parse(&json.to_string());
        assert_eq!(chiefs.map(|r| r.nfc_north_team()), Ok(false));
    }
    #[test]
    fn archive_weeks_are_offered_newest_first_without_the_current_one() {
        let index = match ArchiveIndex::parse(
            r#"{"weeks":[{"season":2026,"week":2},{"season":2026,"week":11},{"season":2025,"week":18},{"season":2026,"week":2},{"season":2026,"week":12}]}"#,
        ) {
            Ok(index) => index,
            Err(error) => panic!("index should parse: {error}"),
        };
        let week = |season, week| ArchiveWeek { season, week };
        assert_eq!(
            index.newest_first(Some(week(2026, 12))),
            vec![week(2026, 11), week(2026, 2), week(2025, 18)]
        );
        assert_eq!(index.newest_first(None).len(), 4);
        assert!(ArchiveIndex::parse("<!doctype html>").is_err());
    }

    #[test]
    fn archive_weeks_name_their_files_and_picker_values() {
        let week = ArchiveWeek {
            season: 2026,
            week: 7,
        };
        assert_eq!(week.path(), "/archive/2026-week-07.json");
        assert_eq!(week.value(), "2026-07");
        assert_eq!(week.label(), "Week 7, 2026 season");
        assert_eq!(ArchiveWeek::from_value("2026-07"), Some(week));
        assert_eq!(ArchiveWeek::from_value(""), None);
        assert_eq!(ArchiveWeek::from_value("2026-00"), None);
        assert_eq!(ArchiveWeek::from_value("week"), None);
        let rooting = match Rooting::parse(SAMPLE) {
            Ok(rooting) => rooting,
            Err(error) => panic!("sample should parse: {error}"),
        };
        assert_eq!(
            rooting.archive_week(),
            Some(ArchiveWeek {
                season: 2026,
                week: 12
            })
        );
    }
}
