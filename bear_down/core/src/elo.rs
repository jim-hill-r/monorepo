//! Elo ratings built from past results, and single-game win probabilities.
//!
//! [`Ratings::from_schedule`] replays every played game in a [`Schedule`],
//! oldest first, moving each team's rating one third of the way back to the
//! average between seasons. [`Ratings::home_win_probability`] then turns any
//! matchup into a probability for the simulator. See
//! `docs/decisions/0004-elo-win-probabilities.md`.

use std::collections::HashMap;

use crate::games::{Game, Outcome, Schedule};

/// The tunable numbers behind the ratings. [`Default`] gives the starting
/// values from decision 0004.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EloParams {
    /// How far one game can move a rating, before the margin multiplier.
    pub k: f64,
    /// Elo points added to the home team's rating, except at neutral sites.
    pub home_field: f64,
    /// The rating every team starts at and regresses toward.
    pub mean: f64,
    /// The share of the gap to [`EloParams::mean`] each rating loses between
    /// seasons, from 0 (keep it all) to 1 (everyone starts even).
    pub season_regression: f64,
}

impl Default for EloParams {
    fn default() -> Self {
        EloParams {
            k: 20.0,
            home_field: 48.0,
            mean: 1505.0,
            season_regression: 1.0 / 3.0,
        }
    }
}

/// The chance that a team rated `diff` Elo points above its opponent wins,
/// with home-field advantage already counted in `diff`.
pub fn win_probability(diff: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-diff / 400.0))
}

/// How much a result's margin scales the rating change.
///
/// This is FiveThirtyEight's NFL multiplier,
/// `ln(margin + 1) * 2.2 / (winner_diff * 0.001 + 2.2)`, where `winner_diff`
/// is the winner's rating edge over the loser, home field included. Blowouts
/// count for more, but less so when the favorite wins them, which keeps
/// strong teams' ratings from running away. A tie has no winner and uses 1.
pub fn margin_multiplier(margin: u16, winner_diff: f64) -> f64 {
    if margin == 0 {
        return 1.0;
    }
    (f64::from(margin) + 1.0).ln() * 2.2 / (winner_diff * 0.001 + 2.2)
}

/// The franchise a team abbreviation belongs to today. nflverse keeps the old
/// abbreviation for games played before a move, so a relocated team's rating
/// carries over only if both are mapped to one name.
pub fn franchise(team: &str) -> &str {
    match team {
        "OAK" => "LV",
        "SD" => "LAC",
        "STL" => "LA",
        other => other,
    }
}

/// Every team's current Elo rating.
#[derive(Debug, Clone, PartialEq)]
pub struct Ratings {
    params: EloParams,
    ratings: HashMap<String, f64>,
    season: Option<u16>,
}

impl Ratings {
    /// Ratings with no games played: every team is at the mean.
    pub fn new(params: EloParams) -> Self {
        Ratings {
            params,
            ratings: HashMap::new(),
            season: None,
        }
    }

    /// Replays every played game in `schedule`, oldest first, and moves the
    /// ratings into the schedule's latest season, even when it has no results
    /// yet. Unplayed games are skipped.
    pub fn from_schedule(schedule: &Schedule, params: EloParams) -> Self {
        let mut ratings = Ratings::new(params);
        for season in schedule.seasons() {
            ratings.start_season(season.year);
            for game in season.played() {
                ratings.record(game);
            }
        }
        ratings
    }

    /// The parameters the ratings are built with.
    pub fn params(&self) -> EloParams {
        self.params
    }

    /// The season the ratings are for, or `None` before any game.
    pub fn season(&self) -> Option<u16> {
        self.season
    }

    /// `team`'s rating. A team with no games yet is at the mean.
    pub fn rating(&self, team: &str) -> f64 {
        self.ratings
            .get(franchise(team))
            .copied()
            .unwrap_or(self.params.mean)
    }

    /// Every rated team and its rating, highest first.
    pub fn ranked(&self) -> Vec<(&str, f64)> {
        let mut ranked: Vec<_> = self
            .ratings
            .iter()
            .map(|(team, rating)| (team.as_str(), *rating))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
        ranked
    }

    /// Moves into season `year`. Each rating moves
    /// [`EloParams::season_regression`] of the way back to the mean once for
    /// every season boundary crossed. Going back to an earlier or the same
    /// season does nothing.
    pub fn start_season(&mut self, year: u16) {
        let seasons = match self.season {
            Some(current) if year > current => i32::from(year - current),
            Some(_) => return,
            None => 0,
        };
        let keep = (1.0 - self.params.season_regression).powi(seasons);
        let mean = self.params.mean;
        for rating in self.ratings.values_mut() {
            *rating = mean + (*rating - mean) * keep;
        }
        self.season = Some(year);
    }

    /// The home team's rating edge in `game`, home field included.
    pub fn home_edge(&self, game: &Game) -> f64 {
        let home_field = if game.neutral_site {
            0.0
        } else {
            self.params.home_field
        };
        self.rating(&game.home_team) - self.rating(&game.away_team) + home_field
    }

    /// The chance the home team wins `game`, ignoring ties.
    pub fn home_win_probability(&self, game: &Game) -> f64 {
        win_probability(self.home_edge(game))
    }

    /// The chance `team` wins `game`, or `None` if it doesn't play in it.
    pub fn win_probability(&self, game: &Game, team: &str) -> Option<f64> {
        let home = self.home_win_probability(game);
        if game.home_team == team {
            Some(home)
        } else if game.away_team == team {
            Some(1.0 - home)
        } else {
            None
        }
    }

    /// Updates both teams' ratings with `game`'s result. Moves into the
    /// game's season first if it is a later one. An unplayed game changes
    /// nothing.
    pub fn record(&mut self, game: &Game) {
        let (Some(outcome), Some(score)) = (game.outcome(), game.score) else {
            return;
        };
        self.start_season(game.season);
        let edge = self.home_edge(game);
        let expected = win_probability(edge);
        let (actual, winner_diff) = match outcome {
            Outcome::HomeWin => (1.0, edge),
            Outcome::AwayWin => (0.0, -edge),
            Outcome::Tie => (0.5, 0.0),
        };
        let margin = score.home.abs_diff(score.away);
        let shift = self.params.k * margin_multiplier(margin, winner_diff) * (actual - expected);
        let home = self.rating(&game.home_team) + shift;
        let away = self.rating(&game.away_team) - shift;
        self.ratings
            .insert(franchise(&game.home_team).to_owned(), home);
        self.ratings
            .insert(franchise(&game.away_team).to_owned(), away);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::{Date, GameType, Score};

    const FIXTURE: &str = include_str!("../fixtures/games.csv");

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn game(season: u16, away: &str, home: &str, score: Option<(u16, u16)>) -> Game {
        Game {
            id: format!("{season}_01_{away}_{home}"),
            season,
            game_type: GameType::Regular,
            week: 1,
            gameday: Date {
                year: season,
                month: 9,
                day: 10,
            },
            away_team: away.into(),
            home_team: home.into(),
            neutral_site: false,
            division_game: false,
            score: score.map(|(away, home)| Score { away, home }),
            spread_line: None,
        }
    }

    fn neutral(mut game: Game) -> Game {
        game.neutral_site = true;
        game
    }

    #[test]
    fn default_params_match_the_decision() {
        let params = EloParams::default();
        close(params.k, 20.0);
        close(params.home_field, 48.0);
        close(params.mean, 1505.0);
        close(params.season_regression, 1.0 / 3.0);
    }

    #[test]
    fn win_probability_follows_the_elo_curve() {
        close(win_probability(0.0), 0.5);
        close(win_probability(400.0), 10.0 / 11.0);
        close(win_probability(-400.0), 1.0 / 11.0);
        close(win_probability(48.0) + win_probability(-48.0), 1.0);
        assert!(win_probability(48.0) > 0.56 && win_probability(48.0) < 0.57);
    }

    #[test]
    fn margin_multiplier_grows_with_margin_and_shrinks_for_favorites() {
        close(margin_multiplier(0, 100.0), 1.0);
        close(margin_multiplier(7, 0.0), 8f64.ln());
        assert!(margin_multiplier(21, 0.0) > margin_multiplier(3, 0.0));
        assert!(margin_multiplier(14, 200.0) < margin_multiplier(14, 0.0));
        assert!(margin_multiplier(14, -200.0) > margin_multiplier(14, 0.0));
    }

    #[test]
    fn unknown_teams_start_at_the_mean() {
        let ratings = Ratings::new(EloParams::default());
        close(ratings.rating("CHI"), 1505.0);
        assert_eq!(ratings.season(), None);
        assert!(ratings.ranked().is_empty());
    }

    #[test]
    fn even_teams_favor_the_home_side_by_home_field() {
        let ratings = Ratings::new(EloParams::default());
        let at_home = game(2026, "GB", "CHI", None);
        close(ratings.home_edge(&at_home), 48.0);
        close(
            ratings.home_win_probability(&at_home),
            win_probability(48.0),
        );
        close(ratings.home_edge(&neutral(at_home)), 0.0);
    }

    #[test]
    fn win_probability_for_either_team_sums_to_one() {
        let ratings = Ratings::new(EloParams::default());
        let matchup = game(2026, "GB", "CHI", None);
        let (Some(bears), Some(packers)) = (
            ratings.win_probability(&matchup, "CHI"),
            ratings.win_probability(&matchup, "GB"),
        ) else {
            panic!("both teams play");
        };
        close(bears, win_probability(48.0));
        close(bears + packers, 1.0);
        assert_eq!(ratings.win_probability(&matchup, "DET"), None);
    }

    #[test]
    fn a_win_moves_both_ratings_by_the_same_amount() {
        let mut ratings = Ratings::new(EloParams::default());
        // An even neutral-site game won by 7: shift = 20 * ln(8) * 0.5.
        ratings.record(&neutral(game(2026, "GB", "CHI", Some((10, 17)))));
        let shift = 20.0 * 8f64.ln() * 0.5;
        close(ratings.rating("CHI"), 1505.0 + shift);
        close(ratings.rating("GB"), 1505.0 - shift);
        assert_eq!(ratings.season(), Some(2026));
        assert_eq!(ratings.ranked()[0].0, "CHI");
    }

    #[test]
    fn an_away_win_counts_home_field_against_the_winner() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&game(2026, "CHI", "GB", Some((24, 21))));
        // The Bears were 48-point underdogs on the road, so the win counts
        // for more than an even game.
        let expected = win_probability(48.0);
        let shift = 20.0 * margin_multiplier(3, -48.0) * expected;
        close(ratings.rating("CHI"), 1505.0 + shift);
        close(ratings.rating("GB"), 1505.0 - shift);
        assert!(shift > 20.0 * margin_multiplier(3, 0.0) * 0.5);
    }

    #[test]
    fn a_tie_moves_the_favorite_down() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&game(2026, "NYG", "WAS", Some((20, 20))));
        let shift = 20.0 * (0.5 - win_probability(48.0));
        close(ratings.rating("WAS"), 1505.0 + shift);
        close(ratings.rating("NYG"), 1505.0 - shift);
        assert!(ratings.rating("WAS") < 1505.0);
    }

    #[test]
    fn an_unplayed_game_changes_nothing() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&game(2026, "CHI", "DET", None));
        assert_eq!(ratings, Ratings::new(EloParams::default()));
    }

    #[test]
    fn ratings_regress_a_third_toward_the_mean_between_seasons() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&neutral(game(2025, "GB", "CHI", Some((0, 30)))));
        let bears = ratings.rating("CHI");
        let packers = ratings.rating("GB");

        ratings.start_season(2026);
        close(ratings.rating("CHI"), 1505.0 + (bears - 1505.0) * 2.0 / 3.0);
        close(
            ratings.rating("GB"),
            1505.0 + (packers - 1505.0) * 2.0 / 3.0,
        );

        // Same or earlier season: no change.
        let before = ratings.clone();
        ratings.start_season(2026);
        ratings.start_season(2020);
        assert_eq!(ratings, before);

        // Skipping a season regresses twice.
        ratings.start_season(2028);
        close(
            ratings.rating("CHI"),
            1505.0 + (bears - 1505.0) * 4.0 / 9.0 * 2.0 / 3.0,
        );
    }

    #[test]
    fn recording_a_later_season_regresses_first() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&neutral(game(2025, "GB", "CHI", Some((0, 30)))));
        let mut expected = ratings.clone();
        expected.start_season(2026);
        let next = neutral(game(2026, "DET", "MIN", Some((17, 17))));
        expected.record(&next);
        ratings.record(&next);
        assert_eq!(ratings, expected);
        close(ratings.rating("MIN"), 1505.0);
    }

    #[test]
    fn relocated_teams_keep_their_rating() {
        let mut ratings = Ratings::new(EloParams::default());
        ratings.record(&neutral(game(2015, "STL", "SD", Some((10, 20)))));
        ratings.record(&neutral(game(2015, "OAK", "SD", Some((10, 20)))));
        close(ratings.rating("LAC"), ratings.rating("SD"));
        close(ratings.rating("LA"), ratings.rating("STL"));
        close(ratings.rating("LV"), ratings.rating("OAK"));
        assert!(ratings.rating("LAC") > 1505.0);
        let teams: Vec<_> = ratings.ranked().into_iter().map(|(team, _)| team).collect();
        assert_eq!(teams.len(), 3);
        assert_eq!(teams[0], "LAC");
        assert!(teams.contains(&"LV") && teams.contains(&"LA"));
        assert_eq!(franchise("CHI"), "CHI");
    }

    #[test]
    fn from_schedule_replays_every_played_game_in_order() {
        let schedule = match Schedule::parse(FIXTURE) {
            Ok(schedule) => schedule,
            Err(error) => panic!("fixture should parse: {error}"),
        };
        let params = EloParams::default();
        let ratings = Ratings::from_schedule(&schedule, params);

        let mut expected = Ratings::new(params);
        for game in schedule.games() {
            expected.record(game);
        }
        assert_eq!(ratings, expected);
        assert_eq!(ratings.season(), Some(2026));

        // Ratings stay zero-sum around the mean.
        let total: f64 = ratings
            .ranked()
            .iter()
            .map(|(_, rating)| rating - 1505.0)
            .sum();
        assert!(total.abs() < 1e-9, "total drift {total}");

        // The Bears won both their 2026 games in the fixture.
        assert!(ratings.rating("CHI") > 1505.0);
        let Some(bears_at_lions) = schedule.games().iter().find(|g| g.id == "2026_12_CHI_DET")
        else {
            panic!("fixture has 2026_12_CHI_DET");
        };
        let bears = ratings.win_probability(bears_at_lions, "CHI");
        assert!(bears.is_some_and(|p| p > 0.0 && p < 1.0));
    }

    #[test]
    fn from_schedule_regresses_into_a_season_with_no_results() {
        let games = vec![
            neutral(game(2025, "GB", "CHI", Some((0, 30)))),
            game(2026, "CHI", "DET", None),
        ];
        let ratings =
            Ratings::from_schedule(&Schedule::from_games(games.clone()), EloParams::default());
        let mut expected = Ratings::new(EloParams::default());
        expected.record(&games[0]);
        expected.start_season(2026);
        assert_eq!(ratings, expected);
    }

    #[test]
    fn custom_params_are_used() {
        let params = EloParams {
            k: 40.0,
            home_field: 0.0,
            mean: 1500.0,
            season_regression: 1.0,
        };
        let mut ratings = Ratings::new(params);
        assert_eq!(ratings.params(), params);
        ratings.record(&game(2025, "GB", "CHI", Some((10, 17))));
        close(ratings.rating("CHI"), 1500.0 + 40.0 * 8f64.ln() * 0.5);
        ratings.start_season(2026);
        close(ratings.rating("CHI"), 1500.0);
    }
}
