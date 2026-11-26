//! The Monte Carlo season and playoff simulator, and each game's swing on the
//! Bears' odds.
//!
//! [`simulate`] starts from a season's actual results, plays every remaining
//! regular-season game with the Elo win probability, seeds both conferences
//! with [`Standings::seeds`] and plays the bracket game by game, many times
//! over. Simulated games never tie, and ratings don't move inside a simulated
//! season. Playoff games that were already played keep their real winner.
//!
//! For every regular-season game left in the current week, the simulations
//! are split by who won it, which gives the swing on the focus team's playoff
//! and Super Bowl odds and on the NFC North divisional-round goal without a
//! separate run per game. See `docs/decisions/0003-own-monte-carlo-simulator.md`
//! and `docs/decisions/0007-nfc-north-divisional-mode.md`.

use std::collections::HashMap;

use crate::NFC_NORTH;
use crate::elo::{Ratings, win_probability};
use crate::games::Outcome;
use crate::games::{Game, Season};
use crate::standings::{Conference, Standings, Team, wild_cards};

/// How many seasons a weekly build simulates (decision 0003).
pub const DEFAULT_SIMULATIONS: u32 = 100_000;

/// How a simulation run is set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimConfig {
    /// How many seasons to simulate.
    pub simulations: u32,
    /// The random seed. The same seed and input give the same result.
    pub seed: u64,
    /// The team whose odds each game's swing is measured on.
    pub focus: Team,
}

impl SimConfig {
    /// The weekly build's setup for `season`'s week `week`:
    /// [`DEFAULT_SIMULATIONS`] runs, a seed fixed by season and week, and the
    /// Bears as the focus team.
    pub fn weekly(season: u16, week: u8) -> Self {
        SimConfig {
            simulations: DEFAULT_SIMULATIONS,
            seed: (u64::from(season) << 8) | u64::from(week),
            focus: Team::BEARS,
        }
    }
}

/// The share of simulations in which a team got at least this far.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TeamOdds {
    /// Made the playoffs.
    pub playoffs: f64,
    /// Reached the divisional round, by a bye or a wild-card win.
    pub divisional_round: f64,
    /// Reached the conference championship game.
    pub conference_game: f64,
    /// Played in the Super Bowl.
    pub super_bowl: f64,
    /// Won the Super Bowl.
    pub champion: f64,
}

/// The odds of one goal if each team wins a game, with the Monte Carlo
/// standard error of their difference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swing {
    /// The goal's probability in the simulations the home team won.
    pub if_home_wins: f64,
    /// The goal's probability in the simulations the away team won.
    pub if_away_wins: f64,
    /// The standard error of [`Swing::swing`]. Infinite when one team never
    /// won the game in any simulation.
    pub std_error: f64,
}

impl Swing {
    /// How much a home win raises the goal's probability over an away win.
    /// Negative when the away team is the one to root for.
    pub fn swing(&self) -> f64 {
        self.if_home_wins - self.if_away_wins
    }

    /// Whether the swing is bigger than the simulation's noise: more than
    /// two standard errors. Decision 0002 shows the rest as "doesn't matter".
    pub fn matters(&self) -> bool {
        self.swing().abs() > 2.0 * self.std_error
    }

    /// Builds a swing from counts: how often the goal happened among the
    /// `home_runs` simulations the home team won, and among the `away_runs`
    /// it lost.
    fn from_counts(home_hits: u32, home_runs: u32, away_hits: u32, away_runs: u32) -> Self {
        let share = |hits: u32, runs: u32| {
            if runs == 0 {
                0.0
            } else {
                f64::from(hits) / f64::from(runs)
            }
        };
        let variance = |p: f64, runs: u32| p * (1.0 - p) / f64::from(runs);
        let (home, away) = (share(home_hits, home_runs), share(away_hits, away_runs));
        let std_error = if home_runs == 0 || away_runs == 0 {
            f64::INFINITY
        } else {
            (variance(home, home_runs) + variance(away, away_runs)).sqrt()
        };
        Swing {
            if_home_wins: home,
            if_away_wins: away,
            std_error,
        }
    }
}

/// One remaining game this week and how its result moves the goals.
#[derive(Debug, Clone, PartialEq)]
pub struct GameImpact {
    /// The nflverse game id.
    pub game_id: String,
    /// The home team.
    pub home: Team,
    /// The away team.
    pub away: Team,
    /// The model's chance that the home team wins.
    pub home_win_probability: f64,
    /// The swing on the focus team's playoff odds.
    pub playoffs: Swing,
    /// The swing on the focus team's odds of winning the Super Bowl.
    pub champion: Swing,
    /// The swing on the odds that all four NFC North teams reach the
    /// divisional round (decision 0007).
    pub nfc_north: Swing,
}

impl GameImpact {
    /// The team to root for to help `swing`'s goal, or `None` when the swing
    /// is within the noise.
    pub fn root_for(&self, swing: &Swing) -> Option<Team> {
        if !swing.matters() {
            None
        } else if swing.swing() > 0.0 {
            Some(self.home)
        } else {
            Some(self.away)
        }
    }
}

/// The result of a simulation run.
#[derive(Debug, Clone, PartialEq)]
pub struct Simulation {
    /// The season simulated.
    pub season: u16,
    /// The current week: the first with a game left, if any.
    pub week: Option<u8>,
    /// How many seasons were simulated.
    pub simulations: u32,
    /// The team the swings are measured for.
    pub focus: Team,
    /// Every team's odds, indexed by [`Team::index`].
    odds: [TeamOdds; 32],
    /// The share of simulations in which all four NFC North teams reached
    /// the divisional round.
    pub nfc_north: f64,
    /// The current week's remaining regular-season games, in schedule order.
    pub games: Vec<GameImpact>,
}

impl Simulation {
    /// `team`'s odds.
    pub fn odds(&self, team: Team) -> TeamOdds {
        self.odds[team.index()]
    }

    /// The focus team's odds.
    pub fn focus_odds(&self) -> TeamOdds {
        self.odds(self.focus)
    }
}

/// A remaining regular-season game, ready to play.
struct Remaining {
    home: Team,
    away: Team,
    home_win_probability: f64,
}

/// Per-game counts for the swings, split by who won.
#[derive(Default, Clone, Copy)]
struct ImpactCounts {
    runs: [u32; 2],
    playoffs: [u32; 2],
    champion: [u32; 2],
    nfc_north: [u32; 2],
}

/// How far each team got in one simulated postseason.
#[derive(Default)]
struct Postseason {
    /// The furthest round reached: 0 missed the playoffs, 1 made them,
    /// 2 divisional round, 3 conference game, 4 Super Bowl, 5 champion.
    reached: [u8; 32],
}

const PLAYOFFS: u8 = 1;
const DIVISIONAL: u8 = 2;
const CONFERENCE: u8 = 3;
const SUPER_BOWL: u8 = 4;
const CHAMPION: u8 = 5;

/// Simulates the rest of `season` with `ratings` as set up by `config`.
///
/// Fails for seasons before today's divisions and for unknown teams.
pub fn simulate(
    season: &Season<'_>,
    ratings: &Ratings,
    config: &SimConfig,
) -> Result<Simulation, String> {
    let base = Standings::from_season(season)?;
    let team = |game: &Game, abbr: &str| {
        Team::from_abbr(abbr).ok_or_else(|| format!("{} has unknown team {abbr:?}", game.id))
    };
    let mut remaining = Vec::new();
    for game in season.regular_season().filter(|game| !game.is_played()) {
        remaining.push(Remaining {
            home: team(game, &game.home_team)?,
            away: team(game, &game.away_team)?,
            home_win_probability: ratings.home_win_probability(game),
        });
    }
    let mut played_playoffs = HashMap::new();
    for game in season.playoffs() {
        if let Some(winner) = game.winner() {
            let (home, away) = (team(game, &game.home_team)?, team(game, &game.away_team)?);
            played_playoffs.insert(pair(home, away), team(game, winner)?);
        }
    }

    // The current week's games to measure, as indexes into `remaining`.
    let week = season.current_week();
    let week_number = week.map(|week| week.number);
    let mut tracked = Vec::new();
    let mut tracked_games = Vec::new();
    for (index, game) in season
        .regular_season()
        .filter(|game| !game.is_played())
        .enumerate()
    {
        if Some(game.week) == week_number {
            tracked.push(index);
            tracked_games.push(game);
        }
    }

    let mut rating = [0.0; 32];
    for team in Team::all() {
        rating[team.index()] = ratings.rating(team.abbr());
    }
    let home_field = ratings.params().home_field;
    let playoff_seeds = wild_cards(season.year) + 4;
    let north: Vec<usize> = NFC_NORTH
        .iter()
        .filter_map(|abbr| Team::from_abbr(abbr))
        .map(Team::index)
        .collect();

    let mut rng = Rng::new(config.seed);
    let mut reached_counts = [[0u32; 6]; 32];
    let mut nfc_north_count = 0u32;
    let mut impacts = vec![ImpactCounts::default(); tracked.len()];
    let mut home_won = vec![false; remaining.len()];
    for _ in 0..config.simulations {
        let mut standings = base.clone();
        for (game, won) in remaining.iter().zip(home_won.iter_mut()) {
            *won = rng.chance(game.home_win_probability);
            let outcome = if *won {
                Outcome::HomeWin
            } else {
                Outcome::AwayWin
            };
            standings.add_result(game.home, game.away, outcome);
        }
        let mut coin = |tied: usize| rng.below(tied);
        let seeds = Conference::ALL
            .map(|conference| standings.seeds(conference, playoff_seeds - 4, &mut coin));
        let mut play = |home: Team, away: Team, neutral: bool| {
            if let Some(&winner) = played_playoffs.get(&pair(home, away)) {
                return winner;
            }
            let edge = rating[home.index()] - rating[away.index()]
                + if neutral { 0.0 } else { home_field };
            if rng.chance(win_probability(edge)) {
                home
            } else {
                away
            }
        };
        let postseason = play_postseason(&seeds, &mut play);

        for (counts, &reached) in reached_counts.iter_mut().zip(&postseason.reached) {
            for round in counts.iter_mut().take(usize::from(reached) + 1) {
                *round += 1;
            }
        }
        let all_north = north
            .iter()
            .all(|&team| postseason.reached[team] >= DIVISIONAL);
        nfc_north_count += u32::from(all_north);
        let focus = postseason.reached[config.focus.index()];
        for (counts, &index) in impacts.iter_mut().zip(&tracked) {
            let side = usize::from(!home_won[index]);
            counts.runs[side] += 1;
            counts.playoffs[side] += u32::from(focus >= PLAYOFFS);
            counts.champion[side] += u32::from(focus >= CHAMPION);
            counts.nfc_north[side] += u32::from(all_north);
        }
    }

    let runs = config.simulations;
    let share = |count: u32| {
        if runs == 0 {
            0.0
        } else {
            f64::from(count) / f64::from(runs)
        }
    };
    let odds = reached_counts.map(|counts| TeamOdds {
        playoffs: share(counts[usize::from(PLAYOFFS)]),
        divisional_round: share(counts[usize::from(DIVISIONAL)]),
        conference_game: share(counts[usize::from(CONFERENCE)]),
        super_bowl: share(counts[usize::from(SUPER_BOWL)]),
        champion: share(counts[usize::from(CHAMPION)]),
    });
    let games = tracked
        .iter()
        .zip(&tracked_games)
        .zip(&impacts)
        .map(|((&index, game), counts)| {
            let swing = |hits: [u32; 2]| {
                Swing::from_counts(hits[0], counts.runs[0], hits[1], counts.runs[1])
            };
            GameImpact {
                game_id: game.id.clone(),
                home: remaining[index].home,
                away: remaining[index].away,
                home_win_probability: remaining[index].home_win_probability,
                playoffs: swing(counts.playoffs),
                champion: swing(counts.champion),
                nfc_north: swing(counts.nfc_north),
            }
        })
        .collect();
    Ok(Simulation {
        season: season.year,
        week: week_number,
        simulations: runs,
        focus: config.focus,
        odds,
        nfc_north: share(nfc_north_count),
        games,
    })
}

/// Plays both conferences' brackets and the Super Bowl. `seeds` holds the
/// AFC and NFC seeds, best first. `play(home, away, neutral)` returns the
/// winner of one game.
fn play_postseason(
    seeds: &[Vec<Team>; 2],
    play: &mut impl FnMut(Team, Team, bool) -> Team,
) -> Postseason {
    let mut postseason = Postseason::default();
    let mut champions = Vec::with_capacity(2);
    for conference in seeds {
        for &team in conference {
            postseason.reached[team.index()] = PLAYOFFS;
        }
        // The top seed has the only bye with seven teams; with six, the top
        // two do.
        let byes = if conference.len() >= 7 { 1 } else { 2 };
        let mut alive: Vec<Team> = conference[..byes.min(conference.len())].to_vec();
        alive.extend(play_round(&conference[byes.min(conference.len())..], play));
        // Survivors are re-seeded: keep them in seed order.
        alive.sort_by_key(|team| conference.iter().position(|seeded| seeded == team));
        for &team in &alive {
            postseason.reached[team.index()] = DIVISIONAL;
        }
        for _ in 0..2 {
            let mut winners = play_round(&alive, play);
            winners.sort_by_key(|team| conference.iter().position(|seeded| seeded == team));
            alive = winners;
            for &team in &alive {
                postseason.reached[team.index()] += 1;
            }
        }
        champions.extend(alive);
    }
    if let [afc, nfc] = champions[..] {
        let winner = play(afc, nfc, true);
        postseason.reached[winner.index()] = CHAMPION;
    }
    postseason
}

/// Plays one round: the best seed hosts the worst, the second best hosts
/// the second worst, and so on. `teams` is in seed order. Returns the
/// winners in the same order as their games.
fn play_round(teams: &[Team], play: &mut impl FnMut(Team, Team, bool) -> Team) -> Vec<Team> {
    let games = teams.len() / 2;
    (0..games)
        .map(|index| play(teams[index], teams[teams.len() - 1 - index], false))
        .collect()
}

/// An order-free key for two teams' game.
fn pair(a: Team, b: Team) -> (Team, Team) {
    if a < b { (a, b) } else { (b, a) }
}

/// A small, fast, deterministic random number generator (SplitMix64). Its
/// output never changes between builds or dependency versions, so a fixed
/// seed always gives the same page.
#[derive(Debug, Clone)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `[0, 1)`.
    pub(crate) fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// `true` with probability `p`.
    pub(crate) fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// A number in `0..n`. `n` must be at least 1.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elo::EloParams;
    use crate::games::{GameType, Schedule};

    /// Every 2002 to 2025 game; see `fixtures/README.md`.
    const SEASONS: &str = include_str!("../fixtures/seasons.csv");

    fn team(abbr: &str) -> Team {
        match Team::from_abbr(abbr) {
            Some(team) => team,
            None => panic!("{abbr} should be a team"),
        }
    }

    fn teams(abbrs: &[&str]) -> Vec<Team> {
        abbrs.iter().map(|abbr| team(abbr)).collect()
    }

    /// The fixture's seasons up to `year`, with `year`'s games changed by
    /// `edit`: return `None` to drop a game, or the game, possibly unplayed.
    fn schedule(year: u16, edit: impl Fn(Game) -> Option<Game>) -> Schedule {
        let full = match Schedule::parse(SEASONS) {
            Ok(schedule) => schedule,
            Err(error) => panic!("seasons fixture should parse: {error}"),
        };
        let games = full
            .games()
            .iter()
            .filter(|game| game.season <= year)
            .cloned()
            .filter_map(|game| {
                if game.season == year {
                    edit(game)
                } else {
                    Some(game)
                }
            })
            .collect();
        Schedule::from_games(games)
    }

    fn run(schedule: &Schedule, year: u16, config: &SimConfig) -> Simulation {
        let Some(season) = schedule.season(year) else {
            panic!("schedule should have {year}");
        };
        let ratings = Ratings::from_schedule(schedule, EloParams::default());
        match simulate(&season, &ratings, config) {
            Ok(simulation) => simulation,
            Err(error) => panic!("{year} should simulate: {error}"),
        }
    }

    fn config(simulations: u32, focus: &str) -> SimConfig {
        SimConfig {
            simulations,
            seed: 7,
            focus: team(focus),
        }
    }

    fn without_playoffs(game: Game) -> Option<Game> {
        (game.game_type == GameType::Regular).then_some(game)
    }

    /// 2023 with week 18 unplayed and no playoffs.
    fn before_week_18(game: Game) -> Option<Game> {
        let mut game = without_playoffs(game)?;
        if game.week == 18 {
            game.score = None;
        }
        Some(game)
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} should be {expected}"
        );
    }

    #[test]
    fn weekly_config_is_fixed_by_season_and_week() {
        let config = SimConfig::weekly(2026, 12);
        assert_eq!(config.simulations, DEFAULT_SIMULATIONS);
        assert_eq!(config.focus, Team::BEARS);
        assert_eq!(config, SimConfig::weekly(2026, 12));
        assert_ne!(config.seed, SimConfig::weekly(2026, 13).seed);
        assert_ne!(config.seed, SimConfig::weekly(2025, 12).seed);
    }

    #[test]
    fn rng_repeats_for_the_same_seed() {
        let draws = |seed| {
            let mut rng = Rng::new(seed);
            (0..5).map(|_| rng.next_u64()).collect::<Vec<_>>()
        };
        assert_eq!(draws(1), draws(1));
        assert_ne!(draws(1), draws(2));
    }

    #[test]
    fn rng_stays_in_range_and_matches_probabilities() {
        let mut rng = Rng::new(42);
        let mut hits = 0;
        let mut faces = [0; 3];
        for _ in 0..30_000 {
            let x = rng.next_f64();
            assert!((0.0..1.0).contains(&x));
            hits += u32::from(rng.chance(0.3));
            faces[rng.below(3)] += 1;
        }
        let share = f64::from(hits) / 30_000.0;
        assert!((share - 0.3).abs() < 0.01, "{share}");
        for count in faces {
            assert!((9_500..10_500).contains(&count), "{faces:?}");
        }
        assert!(!rng.chance(0.0));
        assert!(rng.chance(1.0));
    }

    #[test]
    fn a_round_pairs_the_best_seed_with_the_worst() {
        let seeded = teams(&["KC", "BUF", "BAL", "HOU"]);
        let mut games = Vec::new();
        let winners = play_round(&seeded, &mut |home, away, neutral| {
            games.push((home.abbr(), away.abbr(), neutral));
            away
        });
        assert_eq!(games, [("KC", "HOU", false), ("BUF", "BAL", false)]);
        assert_eq!(winners, teams(&["HOU", "BAL"]));
    }

    const AFC: [&str; 7] = ["BAL", "BUF", "KC", "HOU", "CLE", "MIA", "PIT"];
    const NFC: [&str; 7] = ["SF", "DAL", "DET", "TB", "PHI", "LA", "GB"];

    fn reached(postseason: &Postseason, abbrs: &[&str]) -> Vec<u8> {
        abbrs
            .iter()
            .map(|abbr| postseason.reached[team(abbr).index()])
            .collect()
    }

    #[test]
    fn seven_team_bracket_gives_only_the_top_seed_a_bye() {
        let seeds = [teams(&AFC), teams(&NFC)];
        let mut neutral_games = Vec::new();
        let postseason = play_postseason(&seeds, &mut |home, away, neutral| {
            if neutral {
                neutral_games.push((home, away));
            }
            home
        });
        // Home teams always win: the top two seeds meet in each conference
        // game, and the AFC's top seed wins the neutral-site Super Bowl.
        assert_eq!(reached(&postseason, &AFC), [5, 3, 2, 2, 1, 1, 1]);
        assert_eq!(reached(&postseason, &NFC), [4, 3, 2, 2, 1, 1, 1]);
        assert_eq!(neutral_games, [(team("BAL"), team("SF"))]);
        assert_eq!(postseason.reached[team("CHI").index()], 0);
    }

    #[test]
    fn survivors_are_reseeded_each_round() {
        let seeds = [teams(&AFC), teams(&NFC)];
        let mut games = Vec::new();
        let postseason = play_postseason(&seeds, &mut |home, away, neutral| {
            if !neutral && home.conference() == Conference::Nfc {
                games.push((home.abbr(), away.abbr()));
            }
            away
        });
        assert_eq!(
            games,
            [
                // Wild card: 2-7, 3-6, 4-5.
                ("DAL", "GB"),
                ("DET", "LA"),
                ("TB", "PHI"),
                // Divisional: the top seed hosts the lowest seed left.
                ("SF", "GB"),
                ("PHI", "LA"),
                // Conference: the better seed left hosts.
                ("LA", "GB"),
            ]
        );
        assert_eq!(reached(&postseason, &NFC), [2, 1, 1, 1, 2, 3, 5]);
    }

    #[test]
    fn six_team_bracket_gives_the_top_two_seeds_byes() {
        let afc = teams(&["NE", "DEN", "IND", "BAL", "KC", "LAC"]);
        let nfc = teams(&["SEA", "NO", "PHI", "GB", "SF", "CAR"]);
        let postseason = play_postseason(&[afc, nfc], &mut |home, _, _| home);
        assert_eq!(
            reached(&postseason, &["SEA", "NO", "PHI", "GB", "SF", "CAR"]),
            [4, 3, 2, 2, 1, 1]
        );
    }

    #[test]
    fn a_finished_season_keeps_its_real_results() {
        let schedule = schedule(2023, Some);
        let simulation = run(&schedule, 2023, &config(200, "CHI"));
        assert_eq!(simulation.week, None);
        assert!(simulation.games.is_empty());
        let champion = simulation.odds(team("KC"));
        close(champion.champion, 1.0);
        close(simulation.odds(team("SF")).super_bowl, 1.0);
        for abbr in AFC.iter().chain(&NFC) {
            close(simulation.odds(team(abbr)).playoffs, 1.0);
        }
        close(simulation.focus_odds().playoffs, 0.0);
        // 2023's divisional round: GB was the only NFC North team besides DET.
        close(simulation.odds(team("GB")).divisional_round, 1.0);
        close(simulation.nfc_north, 0.0);
    }

    #[test]
    fn every_simulated_postseason_is_a_full_bracket() {
        let schedule = schedule(2023, without_playoffs);
        let simulation = run(&schedule, 2023, &config(2_000, "CHI"));
        let total = |field: fn(TeamOdds) -> f64| -> f64 {
            Team::all().map(|team| field(simulation.odds(team))).sum()
        };
        close(total(|odds| odds.playoffs), 14.0);
        close(total(|odds| odds.divisional_round), 8.0);
        close(total(|odds| odds.conference_game), 4.0);
        close(total(|odds| odds.super_bowl), 2.0);
        close(total(|odds| odds.champion), 1.0);
        // Top seeds have the bye.
        close(simulation.odds(team("BAL")).divisional_round, 1.0);
        close(simulation.odds(team("SF")).divisional_round, 1.0);
        for team in Team::all() {
            let odds = simulation.odds(team);
            assert!(odds.playoffs >= odds.divisional_round);
            assert!(odds.divisional_round >= odds.conference_game);
            assert!(odds.conference_game >= odds.super_bowl);
            assert!(odds.super_bowl >= odds.champion);
        }
    }

    #[test]
    fn same_seed_same_result() {
        let schedule = schedule(2023, before_week_18);
        let first = run(&schedule, 2023, &config(500, "GB"));
        assert_eq!(first, run(&schedule, 2023, &config(500, "GB")));
        let other = SimConfig {
            seed: 8,
            ..config(500, "GB")
        };
        assert_ne!(first, run(&schedule, 2023, &other));
    }

    #[test]
    fn measures_each_game_left_this_week() {
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(4_000, "GB"));
        assert_eq!(simulation.week, Some(18));
        assert_eq!(simulation.games.len(), 16);
        // 2023 week 18: the Packers clinched by beating the Bears.
        let Some(game) = simulation.games.iter().find(|game| game.home == team("GB")) else {
            panic!("GB should host a week 18 game");
        };
        assert_eq!(game.away, Team::BEARS);
        close(game.playoffs.if_home_wins, 1.0);
        assert!(game.playoffs.if_away_wins < 1.0);
        assert_eq!(game.root_for(&game.playoffs), Some(team("GB")));
        assert!(game.home_win_probability > 0.5);
        // The overall odds sit between the two conditional ones.
        let overall = simulation.focus_odds().playoffs;
        assert!(game.playoffs.if_away_wins <= overall && overall <= 1.0);
    }

    #[test]
    fn an_eliminated_team_has_no_swings() {
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(1_000, "CHI"));
        // The 7-9 Bears were out before week 18.
        close(simulation.focus_odds().playoffs, 0.0);
        for game in &simulation.games {
            close(game.playoffs.swing(), 0.0);
            close(game.champion.swing(), 0.0);
            assert_eq!(game.root_for(&game.playoffs), None);
        }
    }

    #[test]
    fn swing_noise_comes_from_the_counts() {
        let even = Swing::from_counts(50, 100, 50, 100);
        close(even.swing(), 0.0);
        close(even.std_error, (2.0 * 0.25 / 100.0f64).sqrt());
        assert!(!even.matters());

        let clear = Swing::from_counts(90, 100, 10, 100);
        close(clear.swing(), 0.8);
        assert!(clear.matters());

        let one_sided = Swing::from_counts(5, 10, 0, 0);
        assert!(one_sided.std_error.is_infinite());
        assert!(!one_sided.matters());

        let certain = Swing::from_counts(100, 100, 100, 100);
        close(certain.std_error, 0.0);
        assert!(!certain.matters());
    }

    #[test]
    fn root_for_follows_the_sign() {
        let game = GameImpact {
            game_id: "x".to_owned(),
            home: team("DET"),
            away: team("GB"),
            home_win_probability: 0.5,
            playoffs: Swing::from_counts(10, 100, 90, 100),
            champion: Swing::from_counts(90, 100, 10, 100),
            nfc_north: Swing::from_counts(1, 100, 1, 100),
        };
        assert_eq!(game.root_for(&game.playoffs), Some(team("GB")));
        assert_eq!(game.root_for(&game.champion), Some(team("DET")));
        assert_eq!(game.root_for(&game.nfc_north), None);
    }

    #[test]
    fn rejects_old_seasons_and_unknown_teams() {
        let ratings = Ratings::new(EloParams::default());
        let old = schedule(2023, |mut game| {
            game.season = 2001;
            Some(game)
        });
        let Some(season) = old.season(2001) else {
            panic!("the edited games should be in 2001");
        };
        assert!(simulate(&season, &ratings, &config(1, "CHI")).is_err());

        let schedule = schedule(2023, |mut game| {
            if game.id == "2023_18_CHI_GB" {
                game.away_team = "XYZ".to_owned();
                game.score = None;
            }
            Some(game)
        });
        let Some(season) = schedule.season(2023) else {
            panic!("2023 should be in the fixture");
        };
        let error = simulate(&season, &ratings, &config(1, "CHI"));
        assert!(matches!(error, Err(message) if message.contains("XYZ")));
    }

    #[test]
    fn zero_simulations_give_zero_odds() {
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(0, "CHI"));
        assert_eq!(simulation.focus_odds(), TeamOdds::default());
        close(simulation.nfc_north, 0.0);
    }
}
