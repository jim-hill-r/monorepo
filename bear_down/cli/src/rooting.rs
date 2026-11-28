//! The `rooting.json` the web page reads. The format is described in
//! `docs/rooting-json.md`; change both together.

use bear_down_core::{
    Cause, Channel, Game, GameImpact, Record, Simulation, Standings, Swing, Team, TeamOdds,
};
use serde::Serialize;

use crate::banner::Banner;
use crate::why::{self, Goal};

/// The whole file.
#[derive(Debug, Serialize)]
pub struct Guide {
    /// When the guide was built, as an RFC 3339 UTC timestamp.
    pub generated_at: String,
    /// The season simulated.
    pub season: u16,
    /// The current week, or `None` once the regular season is over.
    pub week: Option<u8>,
    /// How many seasons were simulated.
    pub simulations: u32,
    /// The most recent game with a final score, to show how fresh the data is.
    pub latest_game: Option<LatestGame>,
    /// The team the guide is for, as an nflverse abbreviation: `CHI` for
    /// `rooting.json`, any team for `teams/<team>.json`.
    pub team: String,
    /// That team's record and odds.
    pub focus: Focus,
    /// The celebration banner, worked out from the current results: only
    /// when the team's most recent game was a win, otherwise `None`.
    pub banner: Option<Banner>,
    /// The chance all four NFC North teams reach the divisional round.
    pub nfc_north_divisional: f64,
    /// The current week's remaining games, in schedule order.
    pub games: Vec<GameEntry>,
}

/// The latest played game.
#[derive(Debug, Serialize)]
pub struct LatestGame {
    pub id: String,
    pub gameday: String,
    pub away: String,
    pub home: String,
    pub away_score: u16,
    pub home_score: u16,
}

/// The focus team's record and odds.
#[derive(Debug, Serialize)]
pub struct Focus {
    pub record: RecordEntry,
    pub odds: Odds,
    /// Games ahead of the best division rival, by record alone (no
    /// tiebreakers): positive is alone in first place, 0 is tied for it,
    /// negative is behind.
    pub division_lead: f64,
    /// That best division rival.
    pub division_rival: String,
}

/// How many games `focus` leads its best division rival by (half a game
/// per tie), and that rival.
fn division_lead(standings: &Standings, focus: Team) -> (f64, Team) {
    let margin = |team: Team| {
        let record = standings.record(team);
        f64::from(record.wins) - f64::from(record.losses)
    };
    let rival = focus
        .division()
        .teams()
        .into_iter()
        .filter(|&team| team != focus)
        .max_by(|&a, &b| margin(a).total_cmp(&margin(b)))
        .unwrap_or(focus);
    ((margin(focus) - margin(rival)) / 2.0, rival)
}

/// A win-loss-tie record.
#[derive(Debug, Serialize)]
pub struct RecordEntry {
    pub wins: u32,
    pub losses: u32,
    pub ties: u32,
}

/// How far a team gets, as shares of the simulations.
#[derive(Debug, Serialize)]
pub struct Odds {
    pub playoffs: f64,
    pub divisional_round: f64,
    pub conference_game: f64,
    pub super_bowl: f64,
    pub champion: f64,
}

/// One remaining game and its swing on each goal.
#[derive(Debug, Serialize)]
pub struct GameEntry {
    pub id: String,
    pub gameday: String,
    pub away: String,
    pub home: String,
    pub home_win_probability: f64,
    /// The swing on the team winning the Super Bowl (the default goal).
    pub champion: SwingEntry,
    /// The swing on the team making the playoffs.
    pub playoffs: SwingEntry,
    /// The swing on all four NFC North teams reaching the divisional round.
    pub nfc_north: SwingEntry,
}

/// One goal's odds if each team wins.
#[derive(Debug, Serialize)]
pub struct SwingEntry {
    pub if_home_wins: f64,
    pub if_away_wins: f64,
    /// `if_home_wins - if_away_wins`.
    pub swing: f64,
    /// The standard error of `swing`, so the page can apply its own
    /// "doesn't matter" line with the same rule (`bear_down_core::tier_against`).
    pub std_error: f64,
    /// The team to root for, or `None` for "doesn't matter".
    pub root_for: Option<String>,
    /// How much the game matters: `matters`, `small`, `negligible` or
    /// `none` (it can't reach the team).
    pub tier: &'static str,
    /// The plain-language explanation, never empty: the channels through
    /// which the game reaches the team, or why it doesn't.
    pub why: Vec<String>,
    /// The measured channels behind `why`, biggest first. Empty for the
    /// NFC North goal.
    pub channels: Vec<ChannelEntry>,
}

/// One channel's share of a swing.
#[derive(Debug, Serialize)]
pub struct ChannelEntry {
    /// `division_title`, `wild_card`, `seed`, `playoff_path` or
    /// `super_bowl_opponent`.
    pub channel: &'static str,
    /// For the seeding channels, what decided it: `record`, `coin flip` or a
    /// tiebreaker step such as `strength of victory`. Null otherwise.
    pub cause: Option<String>,
    /// The team whose win this channel favors.
    pub favors: String,
    /// The size of the share, as a probability.
    pub share: f64,
    /// Its standard error.
    pub std_error: f64,
    /// The share of simulated seasons in this channel and cause.
    pub seasons: f64,
}

impl Guide {
    /// Builds the guide from a simulation, the season's current standings and
    /// its latest played game.
    pub fn new(
        simulation: &Simulation,
        standings: &Standings,
        latest: Option<&Game>,
        banner: Option<Banner>,
        generated_at: String,
    ) -> Self {
        let focus = simulation.focus;
        Guide {
            generated_at,
            season: simulation.season,
            week: simulation.week,
            simulations: simulation.simulations,
            latest_game: latest.and_then(LatestGame::new),
            team: team(focus),
            focus: Focus {
                record: record(standings.record(focus)),
                odds: odds(simulation.odds(focus)),
                division_lead: division_lead(standings, focus).0,
                division_rival: team(division_lead(standings, focus).1),
            },
            banner,
            nfc_north_divisional: simulation.nfc_north,
            games: simulation
                .games
                .iter()
                .map(|impact| {
                    GameEntry::new(focus, impact, standings, &|team| {
                        simulation.odds(team).playoffs
                    })
                })
                .collect(),
        }
    }
}

impl LatestGame {
    fn new(game: &Game) -> Option<Self> {
        let score = game.score?;
        Some(LatestGame {
            id: game.id.clone(),
            gameday: game.gameday.to_string(),
            away: game.away_team.clone(),
            home: game.home_team.clone(),
            away_score: score.away,
            home_score: score.home,
        })
    }
}

impl GameEntry {
    fn new(
        focus: Team,
        impact: &GameImpact,
        standings: &Standings,
        playoff_odds: &dyn Fn(Team) -> f64,
    ) -> Self {
        let swing = |swing: &Swing, goal: Goal| {
            let (root, tier) = why::verdict(impact, goal, swing);
            SwingEntry {
                if_home_wins: swing.if_home_wins,
                if_away_wins: swing.if_away_wins,
                swing: swing.swing(),
                std_error: swing.std_error,
                root_for: root.map(team),
                tier: tier.name(),
                why: why::reasons(focus, impact, goal, swing, standings, playoff_odds),
                channels: why::shown_channels(impact, goal)
                    .into_iter()
                    .filter_map(|channel| {
                        let (share, std_error) = why::share(channel, goal)?;
                        Some(ChannelEntry {
                            channel: channel_name(channel.channel),
                            cause: channel.cause.map(cause_name),
                            favors: team(if share > 0.0 {
                                impact.home
                            } else {
                                impact.away
                            }),
                            share: share.abs(),
                            std_error,
                            seasons: if impact.pairs == 0 {
                                0.0
                            } else {
                                f64::from(channel.pairs) / f64::from(impact.pairs)
                            },
                        })
                    })
                    .collect(),
            }
        };
        GameEntry {
            id: impact.game_id.clone(),
            gameday: impact.gameday.to_string(),
            away: team(impact.away),
            home: team(impact.home),
            home_win_probability: impact.home_win_probability,
            champion: swing(&impact.champion, Goal::Champion),
            playoffs: swing(&impact.playoffs, Goal::Playoffs),
            nfc_north: swing(&impact.nfc_north, Goal::NfcNorth),
        }
    }
}

fn team(team: Team) -> String {
    team.abbr().to_string()
}

fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::DivisionTitle => "division_title",
        Channel::WildCard => "wild_card",
        Channel::Seed => "seed",
        Channel::PlayoffPath => "playoff_path",
        Channel::SuperBowlOpponent => "super_bowl_opponent",
        Channel::Chance => "chance",
    }
}

fn cause_name(cause: Cause) -> String {
    match cause {
        Cause::Record => "record".to_string(),
        Cause::Tiebreaker(step) => step.name().to_string(),
        Cause::CoinFlip => "coin flip".to_string(),
    }
}

fn record(record: Record) -> RecordEntry {
    RecordEntry {
        wins: record.wins,
        losses: record.losses,
        ties: record.ties,
    }
}

fn odds(odds: TeamOdds) -> Odds {
    Odds {
        playoffs: odds.playoffs,
        divisional_round: odds.divisional_round,
        conference_game: odds.conference_game,
        super_bowl: odds.super_bowl,
        champion: odds.champion,
    }
}
