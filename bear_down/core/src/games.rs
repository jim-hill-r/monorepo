//! The NFL schedule and results, parsed from nflverse `games.csv`.
//!
//! [`Schedule::parse`] reads the CSV text (the CLI downloads it; this crate
//! does no I/O) into [`Game`]s, which [`Schedule`] groups into [`Season`]s
//! and [`Week`]s. See `docs/decisions/0006-nflverse-data.md`.

use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

/// The kind of game, from the `game_type` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameType {
    /// A regular-season game (`REG`).
    Regular,
    /// A wild-card playoff game (`WC`).
    WildCard,
    /// A divisional-round playoff game (`DIV`).
    Divisional,
    /// A conference championship game (`CON`).
    Conference,
    /// The Super Bowl (`SB`).
    SuperBowl,
}

impl GameType {
    /// Whether this is a playoff game.
    pub fn is_playoff(self) -> bool {
        self != GameType::Regular
    }
}

impl FromStr for GameType {
    type Err = String;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code {
            "REG" => Ok(GameType::Regular),
            "WC" => Ok(GameType::WildCard),
            "DIV" => Ok(GameType::Divisional),
            "CON" => Ok(GameType::Conference),
            "SB" => Ok(GameType::SuperBowl),
            other => Err(format!("unknown game_type {other:?}")),
        }
    }
}

/// A calendar date, from the `gameday` column (`YYYY-MM-DD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    /// The year, for example 2026.
    pub year: u16,
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month, 1 to 31.
    pub day: u8,
}

impl FromStr for Date {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid date {text:?}, expected YYYY-MM-DD");
        let mut parts = text.split('-');
        let (Some(year), Some(month), Some(day), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(invalid());
        };
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return Err(invalid());
        }
        let date = Date {
            year: year.parse().map_err(|_| invalid())?,
            month: month.parse().map_err(|_| invalid())?,
            day: day.parse().map_err(|_| invalid())?,
        };
        if !(1..=12).contains(&date.month) || !(1..=31).contains(&date.day) {
            return Err(invalid());
        }
        Ok(date)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A final score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Score {
    /// Points scored by the away team.
    pub away: u16,
    /// Points scored by the home team.
    pub home: u16,
}

/// How a played game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// The home team won.
    HomeWin,
    /// The away team won.
    AwayWin,
    /// The game ended tied (regular season only).
    Tie,
}

/// One game on the schedule, played or not.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    /// The nflverse game id, for example `2026_12_CHI_DET`.
    pub id: String,
    /// The season the game belongs to. Playoff games played in January and
    /// February belong to the previous calendar year's season.
    pub season: u16,
    /// Regular season or which playoff round.
    pub game_type: GameType,
    /// The week number. Playoff rounds continue the regular-season count.
    pub week: u8,
    /// The date the game is or was played.
    pub gameday: Date,
    /// The away team's nflverse abbreviation.
    pub away_team: String,
    /// The home team's nflverse abbreviation.
    pub home_team: String,
    /// Whether the game is at a neutral site (international games, the Super
    /// Bowl), so neither team gets home-field advantage.
    pub neutral_site: bool,
    /// Whether both teams are in the same division.
    pub division_game: bool,
    /// The final score, or `None` if the game hasn't been played.
    pub score: Option<Score>,
    /// The betting line as the home team's expected margin of victory
    /// (negative when the away team is favored), if one is published.
    pub spread_line: Option<f64>,
}

impl Game {
    /// Whether the game has a final score.
    pub fn is_played(&self) -> bool {
        self.score.is_some()
    }

    /// How the game ended, or `None` if it hasn't been played.
    pub fn outcome(&self) -> Option<Outcome> {
        let score = self.score?;
        Some(match score.home.cmp(&score.away) {
            std::cmp::Ordering::Greater => Outcome::HomeWin,
            std::cmp::Ordering::Less => Outcome::AwayWin,
            std::cmp::Ordering::Equal => Outcome::Tie,
        })
    }

    /// The winning team, or `None` for a tie or an unplayed game.
    pub fn winner(&self) -> Option<&str> {
        match self.outcome()? {
            Outcome::HomeWin => Some(&self.home_team),
            Outcome::AwayWin => Some(&self.away_team),
            Outcome::Tie => None,
        }
    }

    /// The losing team, or `None` for a tie or an unplayed game.
    pub fn loser(&self) -> Option<&str> {
        match self.outcome()? {
            Outcome::HomeWin => Some(&self.away_team),
            Outcome::AwayWin => Some(&self.home_team),
            Outcome::Tie => None,
        }
    }

    /// Whether `team` plays in this game.
    pub fn involves(&self, team: &str) -> bool {
        self.home_team == team || self.away_team == team
    }

    /// The team `team` plays in this game, or `None` if it doesn't play.
    pub fn opponent(&self, team: &str) -> Option<&str> {
        if self.home_team == team {
            Some(&self.away_team)
        } else if self.away_team == team {
            Some(&self.home_team)
        } else {
            None
        }
    }
}

/// Why `games.csv` couldn't be parsed.
#[derive(Debug)]
pub struct ParseError {
    /// The 1-based line of the file the problem is on, when known.
    pub line: Option<u64>,
    /// What went wrong.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "games.csv line {line}: {}", self.message),
            None => write!(f, "games.csv: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<csv::Error> for ParseError {
    fn from(error: csv::Error) -> Self {
        let line = error.position().map(csv::Position::line);
        let message = match error.kind() {
            csv::ErrorKind::Deserialize { err, .. } => err.to_string(),
            _ => error.to_string(),
        };
        ParseError { line, message }
    }
}

/// The columns of `games.csv` that bear_down uses. The file has many more,
/// which are ignored.
#[derive(Deserialize)]
struct Row {
    game_id: String,
    season: u16,
    game_type: String,
    week: u8,
    gameday: String,
    away_team: String,
    away_score: Option<u16>,
    home_team: String,
    home_score: Option<u16>,
    location: String,
    result: Option<i32>,
    spread_line: Option<f64>,
    div_game: u8,
}

impl Row {
    fn into_game(self) -> Result<Game, String> {
        let score = match (self.away_score, self.home_score) {
            (Some(away), Some(home)) => Some(Score { away, home }),
            (None, None) => None,
            _ => return Err(format!("{} has only one team's score", self.game_id)),
        };
        if let (Some(score), Some(result)) = (score, self.result)
            && i32::from(score.home) - i32::from(score.away) != result
        {
            return Err(format!(
                "{} result {result} doesn't match the score {}-{}",
                self.game_id, score.away, score.home
            ));
        }
        let neutral_site = match self.location.as_str() {
            "Home" => false,
            "Neutral" => true,
            other => return Err(format!("{} has unknown location {other:?}", self.game_id)),
        };
        let division_game = match self.div_game {
            0 => false,
            1 => true,
            other => return Err(format!("{} has unknown div_game {other}", self.game_id)),
        };
        if self.away_team.is_empty() || self.home_team.is_empty() {
            return Err(format!("{} is missing a team", self.game_id));
        }
        Ok(Game {
            game_type: self.game_type.parse()?,
            gameday: self.gameday.parse()?,
            id: self.game_id,
            season: self.season,
            week: self.week,
            away_team: self.away_team,
            home_team: self.home_team,
            neutral_site,
            division_game,
            score,
            spread_line: self.spread_line,
        })
    }
}

/// Every game in `games.csv`, sorted by season, week, date and id.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Schedule {
    games: Vec<Game>,
}

impl Schedule {
    /// Parses the text of nflverse `games.csv`.
    pub fn parse(csv_text: &str) -> Result<Self, ParseError> {
        let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
        let headers = reader.headers()?.clone();
        let mut games = Vec::new();
        for record in reader.records() {
            let record = record?;
            let line = record.position().map(csv::Position::line);
            let row: Row = record.deserialize(Some(&headers)).map_err(|error| {
                let mut error = ParseError::from(error);
                error.line = error.line.or(line);
                error
            })?;
            let game = row
                .into_game()
                .map_err(|message| ParseError { line, message })?;
            games.push(game);
        }
        Ok(Self::from_games(games))
    }

    /// Builds a schedule from games in any order.
    pub fn from_games(mut games: Vec<Game>) -> Self {
        games.sort_by(|a, b| {
            (a.season, a.week, a.gameday, &a.id).cmp(&(b.season, b.week, b.gameday, &b.id))
        });
        games.dedup_by(|a, b| a.id == b.id);
        Schedule { games }
    }

    /// Every game, sorted by season, week, date and id.
    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// The seasons, oldest first.
    pub fn seasons(&self) -> impl Iterator<Item = Season<'_>> {
        self.games
            .chunk_by(|a, b| a.season == b.season)
            .map(|games| Season {
                year: games[0].season,
                games,
            })
    }

    /// The season that started in `year`, if the schedule has it.
    pub fn season(&self, year: u16) -> Option<Season<'_>> {
        self.seasons().find(|season| season.year == year)
    }

    /// The most recent season in the schedule.
    pub fn latest_season(&self) -> Option<Season<'_>> {
        self.seasons().last()
    }

    /// The most recently played game: the latest data the schedule includes.
    pub fn latest_played(&self) -> Option<&Game> {
        latest_played(&self.games)
    }
}

/// One season's games, regular season and playoffs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Season<'a> {
    /// The year the season started.
    pub year: u16,
    games: &'a [Game],
}

impl<'a> Season<'a> {
    /// Every game in the season, sorted by week, date and id.
    pub fn games(&self) -> &'a [Game] {
        self.games
    }

    /// The weeks, in order. Playoff rounds are weeks too.
    pub fn weeks(&self) -> impl Iterator<Item = Week<'a>> + use<'a> {
        let season = self.year;
        self.games
            .chunk_by(|a, b| a.week == b.week)
            .map(move |games| Week {
                season,
                number: games[0].week,
                games,
            })
    }

    /// Week `number` of the season, if it has one.
    pub fn week(&self, number: u8) -> Option<Week<'a>> {
        self.weeks().find(|week| week.number == number)
    }

    /// The regular-season games.
    pub fn regular_season(&self) -> impl Iterator<Item = &'a Game> + use<'a> {
        self.games
            .iter()
            .filter(|game| game.game_type == GameType::Regular)
    }

    /// The playoff games.
    pub fn playoffs(&self) -> impl Iterator<Item = &'a Game> + use<'a> {
        self.games.iter().filter(|game| game.game_type.is_playoff())
    }

    /// The games with a final score.
    pub fn played(&self) -> impl Iterator<Item = &'a Game> + use<'a> {
        self.games.iter().filter(|game| game.is_played())
    }

    /// The games still to be played.
    pub fn remaining(&self) -> impl Iterator<Item = &'a Game> + use<'a> {
        self.games.iter().filter(|game| !game.is_played())
    }

    /// The most recently played game of the season.
    pub fn latest_played(&self) -> Option<&'a Game> {
        latest_played(self.games)
    }

    /// The first week with a game still to be played, if any.
    pub fn current_week(&self) -> Option<Week<'a>> {
        self.weeks().find(|week| week.remaining().next().is_some())
    }
}

/// One week's games.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Week<'a> {
    /// The season the week belongs to.
    pub season: u16,
    /// The week number.
    pub number: u8,
    games: &'a [Game],
}

impl<'a> Week<'a> {
    /// The week's games, sorted by date and id.
    pub fn games(&self) -> &'a [Game] {
        self.games
    }

    /// The week's games still to be played.
    pub fn remaining(&self) -> impl Iterator<Item = &'a Game> + use<'a> {
        self.games.iter().filter(|game| !game.is_played())
    }
}

fn latest_played(games: &[Game]) -> Option<&Game> {
    games
        .iter()
        .filter(|game| game.is_played())
        .max_by(|a, b| (a.gameday, a.season, a.week).cmp(&(b.gameday, b.season, b.week)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rows cut from the real nflverse file; see `fixtures/README.md`.
    const FIXTURE: &str = include_str!("../fixtures/games.csv");

    const HEADER: &str = "game_id,season,game_type,week,gameday,away_team,away_score,home_team,home_score,location,result,spread_line,div_game";

    fn fixture() -> Schedule {
        match Schedule::parse(FIXTURE) {
            Ok(schedule) => schedule,
            Err(error) => panic!("fixture should parse: {error}"),
        }
    }

    fn game<'a>(schedule: &'a Schedule, id: &str) -> &'a Game {
        match schedule.games().iter().find(|game| game.id == id) {
            Some(game) => game,
            None => panic!("{id} should be in the fixture"),
        }
    }

    fn parse_error(rows: &str) -> ParseError {
        match Schedule::parse(&format!("{HEADER}\n{rows}\n")) {
            Ok(schedule) => panic!("expected an error, parsed {schedule:?}"),
            Err(error) => error,
        }
    }

    #[test]
    fn parses_every_fixture_row() {
        assert_eq!(fixture().games().len(), 13);
    }

    #[test]
    fn parses_a_played_game() {
        let schedule = fixture();
        let game = game(&schedule, "2026_11_NO_CHI");
        assert_eq!(
            game,
            &Game {
                id: "2026_11_NO_CHI".into(),
                season: 2026,
                game_type: GameType::Regular,
                week: 11,
                gameday: Date {
                    year: 2026,
                    month: 11,
                    day: 22
                },
                away_team: "NO".into(),
                home_team: "CHI".into(),
                neutral_site: false,
                division_game: false,
                score: Some(Score { away: 13, home: 21 }),
                spread_line: Some(11.5),
            }
        );
        assert_eq!(game.outcome(), Some(Outcome::HomeWin));
        assert_eq!(game.winner(), Some("CHI"));
        assert_eq!(game.loser(), Some("NO"));
    }

    #[test]
    fn parses_an_unplayed_game() {
        let schedule = fixture();
        let game = game(&schedule, "2026_12_CHI_DET");
        assert!(!game.is_played());
        assert_eq!(game.score, None);
        assert_eq!(game.outcome(), None);
        assert_eq!(game.winner(), None);
        assert!(game.division_game);
        assert_eq!(game.spread_line, Some(-1.5));
    }

    #[test]
    fn parses_a_tie() {
        let schedule = fixture();
        let game = game(&schedule, "2026_05_NYG_WAS");
        assert_eq!(game.outcome(), Some(Outcome::Tie));
        assert_eq!(game.winner(), None);
        assert_eq!(game.loser(), None);
    }

    #[test]
    fn parses_an_away_win() {
        let schedule = fixture();
        let game = game(&schedule, "2026_01_CHI_CAR");
        assert_eq!(game.outcome(), Some(Outcome::AwayWin));
        assert_eq!(game.winner(), Some("CHI"));
        assert_eq!(game.loser(), Some("CAR"));
    }

    #[test]
    fn parses_neutral_sites() {
        let schedule = fixture();
        assert!(game(&schedule, "2026_01_SF_LA").neutral_site);
        assert!(game(&schedule, "2025_22_SEA_NE").neutral_site);
        assert!(!game(&schedule, "2026_01_NE_SEA").neutral_site);
    }

    #[test]
    fn parses_every_game_type() {
        let schedule = fixture();
        let types = [
            ("2026_01_NE_SEA", GameType::Regular),
            ("2025_20_LA_CHI", GameType::Divisional),
            ("2025_21_NE_DEN", GameType::Conference),
            ("2025_22_SEA_NE", GameType::SuperBowl),
        ];
        for (id, game_type) in types {
            assert_eq!(game(&schedule, id).game_type, game_type, "{id}");
        }
        assert_eq!("WC".parse(), Ok(GameType::WildCard));
        assert!(GameType::WildCard.is_playoff());
        assert!(!GameType::Regular.is_playoff());
    }

    #[test]
    fn parses_quoted_empty_columns() {
        // 2022_04_MIN_NO has `""` in its surface column.
        let schedule = fixture();
        assert_eq!(
            game(&schedule, "2022_04_MIN_NO").score,
            Some(Score { away: 28, home: 25 })
        );
    }

    #[test]
    fn playoff_games_belong_to_the_season_they_finish() {
        let schedule = fixture();
        let super_bowl = game(&schedule, "2025_22_SEA_NE");
        assert_eq!(super_bowl.season, 2025);
        assert_eq!(super_bowl.gameday.year, 2026);
        let season = schedule.season(2025);
        let playoffs: Vec<_> = season
            .iter()
            .flat_map(|season| season.playoffs())
            .map(|game| game.id.as_str())
            .collect();
        assert_eq!(
            playoffs,
            [
                "2025_20_LA_CHI",
                "2025_21_LA_SEA",
                "2025_21_NE_DEN",
                "2025_22_SEA_NE"
            ]
        );
    }

    #[test]
    fn groups_games_into_seasons_and_weeks() {
        let schedule = fixture();
        let years: Vec<_> = schedule.seasons().map(|season| season.year).collect();
        assert_eq!(years, [1999, 2022, 2025, 2026]);

        let Some(season) = schedule.latest_season() else {
            panic!("fixture has seasons");
        };
        assert_eq!(season.year, 2026);
        let weeks: Vec<_> = season
            .weeks()
            .map(|week| (week.number, week.games().len()))
            .collect();
        assert_eq!(weeks, [(1, 3), (5, 1), (11, 1), (12, 2)]);
        assert_eq!(season.regular_season().count(), 7);
        assert_eq!(season.playoffs().count(), 0);
        assert_eq!(season.played().count(), 5);
        assert_eq!(season.remaining().count(), 2);
        assert_eq!(season.week(3), None);
    }

    #[test]
    fn games_are_sorted_by_season_week_and_date() {
        let schedule = fixture();
        let Some(week) = schedule.season(2026).and_then(|season| season.week(1)) else {
            panic!("fixture has 2026 week 1");
        };
        let ids: Vec<_> = week.games().iter().map(|game| game.id.as_str()).collect();
        assert_eq!(ids, ["2026_01_NE_SEA", "2026_01_SF_LA", "2026_01_CHI_CAR"]);

        // The fixture lists week 11 after week 12.
        let all: Vec<_> = schedule
            .games()
            .iter()
            .map(|game| game.id.as_str())
            .collect();
        assert_eq!(
            all[all.len() - 3..],
            ["2026_11_NO_CHI", "2026_12_GB_LA", "2026_12_CHI_DET"]
        );
    }

    #[test]
    fn current_week_is_the_first_with_games_left() {
        let schedule = fixture();
        let current = schedule
            .latest_season()
            .and_then(|season| season.current_week());
        assert_eq!(current.map(|week| week.number), Some(12));
        assert_eq!(current.map(|week| week.remaining().count()), Some(2));
        let finished = schedule
            .season(2025)
            .and_then(|season| season.current_week());
        assert_eq!(finished, None);
    }

    #[test]
    fn latest_played_is_the_most_recent_result() {
        let schedule = fixture();
        assert_eq!(
            schedule.latest_played().map(|game| game.id.as_str()),
            Some("2026_11_NO_CHI")
        );
        assert_eq!(
            schedule
                .season(2025)
                .and_then(|season| season.latest_played())
                .map(|game| game.id.as_str()),
            Some("2025_22_SEA_NE")
        );
    }

    #[test]
    fn finds_a_teams_opponent() {
        let schedule = fixture();
        let game = game(&schedule, "2026_12_CHI_DET");
        assert!(game.involves(crate::BEARS));
        assert_eq!(game.opponent(crate::BEARS), Some("DET"));
        assert_eq!(game.opponent("DET"), Some("CHI"));
        assert_eq!(game.opponent("GB"), None);
        assert!(!game.involves("GB"));
    }

    #[test]
    fn duplicate_rows_are_kept_once() {
        let row = "2026_18_DAL_WAS,2026,REG,18,2027-01-10,DAL,,WAS,,Home,,,1";
        let Ok(schedule) = Schedule::parse(&format!("{HEADER}\n{row}\n{row}\n")) else {
            panic!("duplicate rows should parse");
        };
        assert_eq!(schedule.games().len(), 1);
    }

    #[test]
    fn empty_file_is_an_empty_schedule() {
        let Ok(schedule) = Schedule::parse(&format!("{HEADER}\n")) else {
            panic!("a header-only file should parse");
        };
        assert!(schedule.games().is_empty());
        assert_eq!(schedule.latest_season(), None);
        assert_eq!(schedule.latest_played(), None);
    }

    #[test]
    fn rejects_an_unknown_game_type() {
        let error = parse_error("2026_01_A_B,2026,PRE,1,2026-08-01,A,1,B,2,Home,1,,0");
        assert_eq!(error.line, Some(2));
        assert!(error.message.contains("PRE"), "{error}");
    }

    #[test]
    fn rejects_a_half_score() {
        let error = parse_error("2026_01_A_B,2026,REG,1,2026-09-01,A,10,B,,Home,,,0");
        assert!(error.message.contains("one team's score"), "{error}");
    }

    #[test]
    fn rejects_a_result_that_disagrees_with_the_score() {
        let error = parse_error("2026_01_A_B,2026,REG,1,2026-09-01,A,10,B,17,Home,-7,,0");
        assert!(error.message.contains("doesn't match"), "{error}");
    }

    #[test]
    fn rejects_an_unknown_location() {
        let error = parse_error("2026_01_A_B,2026,REG,1,2026-09-01,A,10,B,17,Away,7,,0");
        assert!(error.message.contains("Away"), "{error}");
    }

    #[test]
    fn rejects_a_bad_number_with_its_line() {
        let error = parse_error(
            "2026_01_A_B,2026,REG,1,2026-09-01,A,10,B,17,Home,7,,0\n\
             2026_01_C_D,2026,REG,one,2026-09-01,C,10,D,17,Home,7,,0",
        );
        assert_eq!(error.line, Some(3));
        assert!(
            error.to_string().starts_with("games.csv line 3:"),
            "{error}"
        );
    }

    #[test]
    fn rejects_a_missing_column() {
        let error = match Schedule::parse("game_id,season\n2026_01_A_B,2026\n") {
            Ok(schedule) => panic!("expected an error, parsed {schedule:?}"),
            Err(error) => error,
        };
        assert!(error.message.contains("game_type"), "{error}");
    }

    #[test]
    fn parses_and_formats_dates() {
        let date: Result<Date, _> = "2027-01-10".parse();
        assert_eq!(
            date,
            Ok(Date {
                year: 2027,
                month: 1,
                day: 10
            })
        );
        assert_eq!(date.map(|date| date.to_string()), Ok("2027-01-10".into()));
        for bad in [
            "",
            "2026-1-10",
            "2026-13-01",
            "2026-01-00",
            "2026-01-10-1",
            "26-01-10",
        ] {
            assert!(bad.parse::<Date>().is_err(), "{bad:?} should not parse");
        }
    }
}
