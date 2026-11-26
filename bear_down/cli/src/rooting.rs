//! The `rooting.json` the web page reads. The format is described in
//! `docs/rooting-json.md`; change both together.

use bear_down_core::{Game, GameImpact, Record, Simulation, Standings, Swing, Team, TeamOdds};
use serde::Serialize;

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
    /// The Bears' record and odds.
    pub bears: Bears,
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
pub struct Bears {
    pub record: RecordEntry,
    pub odds: Odds,
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
    /// The swing on the Bears winning the Super Bowl (the default goal).
    pub champion: SwingEntry,
    /// The swing on the Bears making the playoffs.
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
    /// The team to root for, or `None` when the swing is within the noise.
    pub root_for: Option<String>,
}

impl Guide {
    /// Builds the guide from a simulation, the season's current standings and
    /// its latest played game.
    pub fn new(
        simulation: &Simulation,
        standings: &Standings,
        latest: Option<&Game>,
        generated_at: String,
    ) -> Self {
        let focus = simulation.focus;
        Guide {
            generated_at,
            season: simulation.season,
            week: simulation.week,
            simulations: simulation.simulations,
            latest_game: latest.and_then(LatestGame::new),
            bears: Bears {
                record: record(standings.record(focus)),
                odds: odds(simulation.odds(focus)),
            },
            nfc_north_divisional: simulation.nfc_north,
            games: simulation.games.iter().map(GameEntry::new).collect(),
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
    fn new(impact: &GameImpact) -> Self {
        let swing = |swing: &Swing| SwingEntry {
            if_home_wins: swing.if_home_wins,
            if_away_wins: swing.if_away_wins,
            swing: swing.swing(),
            root_for: impact.root_for(swing).map(|team| team.abbr().to_string()),
        };
        GameEntry {
            id: impact.game_id.clone(),
            gameday: impact.gameday.to_string(),
            away: team(impact.away),
            home: team(impact.home),
            home_win_probability: impact.home_win_probability,
            champion: swing(&impact.champion),
            playoffs: swing(&impact.playoffs),
            nfc_north: swing(&impact.nfc_north),
        }
    }
}

fn team(team: Team) -> String {
    team.abbr().to_string()
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
