//! Which ways a game could reach the focus team at all, worked out from the
//! schedule and standings alone, with no simulation (step 3 of
//! `docs/plans/2026-11-robust-recommendations.md`).
//!
//! The simulator measures how much a game matters. This pass says which
//! links are open, and when none is, the game can't matter in any possible
//! season, which the simulator can never prove. Every closed link is closed
//! for sure: the elimination check only counts teams that are already ahead
//! of a team's best possible finish. That errs toward "still alive": a team
//! out only on tiebreakers still counts as alive here, and the simulator
//! then measures its games as they are.

use crate::games::Season;
use crate::standings::{Standings, Team, wild_cards};

/// One way a game can reach the focus team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// The focus team plays in the game.
    FocusPlays,
    /// A team in the game is in the focus team's conference and can still
    /// make the playoffs, so its record is in the race.
    Race(Team),
    /// A team in the game is in the other conference and can still make the
    /// playoffs there, so it could be the Super Bowl opponent.
    SuperBowlRace(Team),
    /// A team in the game is on these playoff contenders' schedules, so its
    /// record moves their strength of victory or schedule, which can break
    /// a tie that reaches the focus team. `contenders` is in team order and
    /// may include the focus team itself.
    Schedule {
        /// The team in the game.
        team: Team,
        /// The contenders that play `team` this season.
        contenders: Vec<Team>,
    },
}

/// The open links for one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relevance {
    /// The home team.
    pub home: Team,
    /// The away team.
    pub away: Team,
    /// Whether the focus team can still make the playoffs. If not, nothing
    /// can matter and `links` is empty.
    pub focus_alive: bool,
    /// Whether each team can still make the playoffs, home first.
    pub alive: [bool; 2],
    /// Every open link. Empty means the game can't matter to the focus team.
    pub links: Vec<Link>,
}

impl Relevance {
    /// Whether the game can reach the focus team in any possible season.
    pub fn can_matter(&self) -> bool {
        !self.links.is_empty()
    }
}

/// Teams' standing in the race: current points and the most they can
/// still reach, in half wins (a tie is one half win).
struct Race {
    points: [u32; 32],
    max_points: [u32; 32],
    seeds: usize,
}

impl Race {
    fn new(season: &Season<'_>, standings: &Standings) -> Self {
        let mut points = [0; 32];
        let mut max_points = [0; 32];
        for team in Team::all() {
            let record = standings.record(team);
            points[team.index()] = 2 * record.wins + record.ties;
        }
        max_points.copy_from_slice(&points);
        for game in season.regular_season().filter(|game| !game.is_played()) {
            for abbr in [&game.home_team, &game.away_team] {
                if let Some(team) = Team::from_abbr(abbr) {
                    max_points[team.index()] += 2;
                }
            }
        }
        Race {
            points,
            max_points,
            seeds: wild_cards(season.year) + 4,
        }
    }

    /// Whether `team` can still make the playoffs. It can't once its
    /// division has a team already ahead of its best finish (so it can't win
    /// the division) and its conference has at least as many such teams as
    /// there are seeds: at most four of them win divisions, so the rest take
    /// every wild card.
    fn alive(&self, team: Team) -> bool {
        let best = self.max_points[team.index()];
        let ahead = |other: &Team| *other != team && self.points[other.index()] > best;
        let division_led = team.division().teams().iter().any(ahead);
        let conference_ahead = team.conference().teams().filter(ahead).count();
        !(division_led && conference_ahead >= self.seeds)
    }
}

/// The open links for every game left in `season`'s current week, in
/// schedule order, measured for `focus`.
///
/// Fails for unknown teams.
pub fn relevance(
    season: &Season<'_>,
    standings: &Standings,
    focus: Team,
) -> Result<Vec<Relevance>, String> {
    let race = Race::new(season, standings);
    let team = |abbr: &str| Team::from_abbr(abbr).ok_or_else(|| format!("unknown team {abbr:?}"));
    // Each team's opponents this season, played or not.
    let mut opponents: [Vec<Team>; 32] = std::array::from_fn(|_| Vec::new());
    for game in season.regular_season() {
        let (home, away) = (team(&game.home_team)?, team(&game.away_team)?);
        opponents[home.index()].push(away);
        opponents[away.index()].push(home);
    }
    let focus_alive = race.alive(focus);
    let Some(week) = season.current_week() else {
        return Ok(Vec::new());
    };
    week.remaining()
        .map(|game| {
            let (home, away) = (team(&game.home_team)?, team(&game.away_team)?);
            let alive = [race.alive(home), race.alive(away)];
            let mut links = Vec::new();
            if focus_alive {
                if home == focus || away == focus {
                    links.push(Link::FocusPlays);
                }
                for (&playing, &playing_alive) in [home, away].iter().zip(&alive) {
                    if playing == focus {
                        continue;
                    }
                    if playing_alive {
                        links.push(if playing.conference() == focus.conference() {
                            Link::Race(playing)
                        } else {
                            Link::SuperBowlRace(playing)
                        });
                    }
                    let mut contenders: Vec<Team> = opponents[playing.index()]
                        .iter()
                        .copied()
                        .filter(|&other| other != home && other != away && race.alive(other))
                        .collect();
                    contenders.sort();
                    contenders.dedup();
                    if !contenders.is_empty() {
                        links.push(Link::Schedule {
                            team: playing,
                            contenders,
                        });
                    }
                }
            }
            Ok(Relevance {
                home,
                away,
                focus_alive,
                alive,
                links,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::{Game, GameType, Schedule};

    /// Every 2002 to 2025 game; see `fixtures/README.md`.
    const SEASONS: &str = include_str!("../fixtures/seasons.csv");

    fn team(abbr: &str) -> Team {
        match Team::from_abbr(abbr) {
            Some(team) => team,
            None => panic!("{abbr} should be a team"),
        }
    }

    /// `year`'s regular season with weeks from `week` on unplayed.
    fn schedule(year: u16, week: u8) -> Schedule {
        let full = match Schedule::parse(SEASONS) {
            Ok(schedule) => schedule,
            Err(error) => panic!("seasons fixture should parse: {error}"),
        };
        let games: Vec<Game> = full
            .games()
            .iter()
            .filter(|game| game.season == year && game.game_type == GameType::Regular)
            .cloned()
            .map(|mut game| {
                if game.week >= week {
                    game.score = None;
                }
                game
            })
            .collect();
        Schedule::from_games(games)
    }

    fn links(year: u16, week: u8, focus: &str) -> Vec<Relevance> {
        let schedule = schedule(year, week);
        let Some(season) = schedule.season(year) else {
            panic!("schedule should have {year}");
        };
        let standings = match Standings::from_season(&season) {
            Ok(standings) => standings,
            Err(error) => panic!("{year} should have standings: {error}"),
        };
        match relevance(&season, &standings, team(focus)) {
            Ok(relevance) => relevance,
            Err(error) => panic!("{year} should have relevance: {error}"),
        }
    }

    fn game<'a>(games: &'a [Relevance], home: &str) -> &'a Relevance {
        match games.iter().find(|game| game.home == team(home)) {
            Some(game) => game,
            None => panic!("{home} should host a game"),
        }
    }

    #[test]
    fn an_eliminated_focus_team_has_no_links() {
        // The 2-14 Panthers were out long before 2023 week 18. (The 7-9
        // Bears were out too, but only on tiebreakers, which this records-only
        // check doesn't see: it errs toward "still alive".)
        let games = links(2023, 18, "CAR");
        assert_eq!(games.len(), 16);
        for game in &games {
            assert!(!game.focus_alive);
            assert!(!game.can_matter());
        }
    }

    #[test]
    fn a_contender_sees_its_own_game_and_the_race() {
        // 2023 week 18: the 8-8 Packers hosted the Bears for a wild card.
        let games = links(2023, 18, "GB");
        let packers = game(&games, "GB");
        assert!(packers.focus_alive);
        assert_eq!(packers.links.first(), Some(&Link::FocusPlays));
        // Every NFC game with a team still alive is in the race.
        let Some(rams) = games.iter().find(|game| game.away == team("LA")) else {
            panic!("LA should play away in week 18");
        };
        assert!(rams.links.contains(&Link::Race(team("LA"))));
    }

    #[test]
    fn elimination_never_counts_a_team_out_that_made_it() {
        // For every season and every week from 10 on, each team that made
        // the playoffs was alive at that point.
        let full = match Schedule::parse(SEASONS) {
            Ok(schedule) => schedule,
            Err(error) => panic!("seasons fixture should parse: {error}"),
        };
        for year in 2002..=2025 {
            let playoff_teams: Vec<Team> = full
                .games()
                .iter()
                .filter(|game| game.season == year && game.game_type != GameType::Regular)
                .flat_map(|game| [team(&game.home_team), team(&game.away_team)])
                .collect();
            for week in 10..=18 {
                let schedule = schedule(year, week);
                let Some(season) = schedule.season(year) else {
                    continue;
                };
                let Ok(standings) = Standings::from_season(&season) else {
                    continue;
                };
                let race = Race::new(&season, &standings);
                for &made_it in &playoff_teams {
                    assert!(race.alive(made_it), "{year} week {week}: {made_it}");
                }
            }
        }
    }

    #[test]
    fn some_teams_are_out_late_in_the_season() {
        let games = links(2023, 18, "GB");
        let out = games
            .iter()
            .flat_map(|game| game.alive)
            .filter(|alive| !alive)
            .count();
        assert!(out > 0, "some teams should be eliminated before week 18");
    }
}
