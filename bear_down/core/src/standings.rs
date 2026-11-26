//! NFL standings, division winners, wild cards and playoff seeding.
//!
//! [`Standings`] keeps every team's record against every other team, built
//! from played games ([`Standings::from_season`]) or one result at a time
//! ([`Standings::add_result`], which the simulator uses). [`Standings::seeds`]
//! then ranks a conference the way the NFL does, with the tiebreaker steps in
//! `docs/decisions/0005-tiebreakers.md`: head-to-head, division, common-games
//! and conference records, then strength of victory and of schedule, and a
//! coin flip for anything still tied. The points-based steps are skipped.
//!
//! Divisions are the ones the NFL has used since 2002, so earlier seasons are
//! rejected.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign};

use crate::elo::franchise;
use crate::games::{Game, GameType, Outcome, Season};

/// The first season with today's eight divisions.
pub const FIRST_SEASON: u16 = 2002;

/// A conference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Conference {
    /// The American Football Conference.
    Afc,
    /// The National Football Conference.
    Nfc,
}

impl Conference {
    /// Both conferences, AFC first.
    pub const ALL: [Conference; 2] = [Conference::Afc, Conference::Nfc];

    /// The conference's four divisions: East, North, South, West.
    pub fn divisions(self) -> [Division; 4] {
        Region::ALL.map(|region| Division {
            conference: self,
            region,
        })
    }

    /// The conference's sixteen teams, division by division.
    pub fn teams(self) -> impl Iterator<Item = Team> {
        self.divisions().into_iter().flat_map(Division::teams)
    }
}

impl fmt::Display for Conference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Conference::Afc => "AFC",
            Conference::Nfc => "NFC",
        })
    }
}

/// Which of a conference's four divisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Region {
    /// East.
    East,
    /// North.
    North,
    /// South.
    South,
    /// West.
    West,
}

impl Region {
    /// The four regions, in the NFL's usual order.
    pub const ALL: [Region; 4] = [Region::East, Region::North, Region::South, Region::West];
}

/// A division, such as the NFC North.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Division {
    /// The division's conference.
    pub conference: Conference,
    /// Which of the conference's divisions it is.
    pub region: Region,
}

impl Division {
    /// The NFC North, the Bears' division.
    pub const NFC_NORTH: Division = Division {
        conference: Conference::Nfc,
        region: Region::North,
    };

    fn index(self) -> usize {
        self.conference as usize * 4 + self.region as usize
    }

    /// The division's four teams, in alphabetical order of abbreviation.
    pub fn teams(self) -> [Team; 4] {
        let first = self.index() * 4;
        [0, 1, 2, 3].map(|offset| Team((first + offset) as u8))
    }
}

impl fmt::Display for Division {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let region = match self.region {
            Region::East => "East",
            Region::North => "North",
            Region::South => "South",
            Region::West => "West",
        };
        write!(f, "{} {region}", self.conference)
    }
}

/// Every team's abbreviation, four per division, in [`Division::index`]
/// order.
const ABBREVIATIONS: [&str; 32] = [
    "BUF", "MIA", "NE", "NYJ", // AFC East
    "BAL", "CIN", "CLE", "PIT", // AFC North
    "HOU", "IND", "JAX", "TEN", // AFC South
    "DEN", "KC", "LAC", "LV", // AFC West
    "DAL", "NYG", "PHI", "WAS", // NFC East
    "CHI", "DET", "GB", "MIN", // NFC North
    "ATL", "CAR", "NO", "TB", // NFC South
    "ARI", "LA", "SEA", "SF", // NFC West
];

/// One of the 32 teams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Team(u8);

impl Team {
    /// The Chicago Bears ([`crate::BEARS`]).
    pub const BEARS: Team = Team(20);

    /// Every team, division by division.
    pub fn all() -> impl Iterator<Item = Team> {
        (0..32).map(Team)
    }

    /// The team with nflverse abbreviation `abbr`. A relocated team's old
    /// abbreviation (`OAK`, `SD`, `STL`) gives the franchise's team today.
    pub fn from_abbr(abbr: &str) -> Option<Team> {
        let abbr = franchise(abbr);
        ABBREVIATIONS
            .iter()
            .position(|known| *known == abbr)
            .map(|index| Team(index as u8))
    }

    /// The team's nflverse abbreviation today.
    pub fn abbr(self) -> &'static str {
        ABBREVIATIONS[self.index()]
    }

    /// The team's division.
    pub fn division(self) -> Division {
        let index = self.index() / 4;
        Division {
            conference: Conference::ALL[index / 4],
            region: Region::ALL[index % 4],
        }
    }

    /// The team's conference.
    pub fn conference(self) -> Conference {
        self.division().conference
    }

    /// A number from 0 to 31, for indexing per-team arrays.
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbr())
    }
}

/// Wins, losses and ties.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Record {
    /// Games won.
    pub wins: u32,
    /// Games lost.
    pub losses: u32,
    /// Games tied.
    pub ties: u32,
}

impl Record {
    /// The number of games played.
    pub fn games(self) -> u32 {
        self.wins + self.losses + self.ties
    }

    /// The winning percentage, counting a tie as half a win, from 0 to 1. A
    /// record with no games is 0.
    pub fn pct(self) -> f64 {
        let (numerator, denominator) = self.fraction();
        numerator as f64 / denominator as f64
    }

    /// Compares two winning percentages exactly, without rounding.
    pub fn cmp_pct(self, other: Record) -> Ordering {
        let (a, b) = self.fraction();
        let (c, d) = other.fraction();
        (a * d).cmp(&(c * b))
    }

    /// The winning percentage as (half-wins, half-games).
    fn fraction(self) -> (u64, u64) {
        let games = u64::from(self.games());
        if games == 0 {
            return (0, 1);
        }
        (2 * u64::from(self.wins) + u64::from(self.ties), 2 * games)
    }

    /// `self` repeated `times` times, for weighting a record by games played.
    fn times(self, times: u32) -> Record {
        Record {
            wins: self.wins * times,
            losses: self.losses * times,
            ties: self.ties * times,
        }
    }
}

impl Add for Record {
    type Output = Record;

    fn add(self, other: Record) -> Record {
        Record {
            wins: self.wins + other.wins,
            losses: self.losses + other.losses,
            ties: self.ties + other.ties,
        }
    }
}

impl AddAssign for Record {
    fn add_assign(&mut self, other: Record) {
        *self = *self + other;
    }
}

impl std::iter::Sum for Record {
    fn sum<I: Iterator<Item = Record>>(iter: I) -> Record {
        iter.fold(Record::default(), Add::add)
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.wins, self.losses)?;
        if self.ties > 0 {
            write!(f, "-{}", self.ties)?;
        }
        Ok(())
    }
}

/// The number of wild cards per conference in `season`: three since 2020,
/// two before.
pub fn wild_cards(season: u16) -> usize {
    if season >= 2020 { 3 } else { 2 }
}

/// Which set of tiebreaker steps to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Procedure {
    /// Teams in the same division.
    Division,
    /// Teams in different divisions: wild cards and seeding.
    WildCard,
}

/// One tiebreaker step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    HeadToHead,
    DivisionRecord,
    CommonGames,
    ConferenceRecord,
    StrengthOfVictory,
    StrengthOfSchedule,
}

impl Step {
    /// The step's name, as fans know it.
    pub fn name(self) -> &'static str {
        match self {
            Step::HeadToHead => "head-to-head",
            Step::DivisionRecord => "division record",
            Step::CommonGames => "record in common games",
            Step::ConferenceRecord => "conference record",
            Step::StrengthOfVictory => "strength of victory",
            Step::StrengthOfSchedule => "strength of schedule",
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One decision while breaking a tie: `step` (or a coin flip, when `None`)
/// kept `kept` out of `group`, teams with the same winning percentage.
/// [`Standings::seeds_traced`] reports these in the order they happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiebreak {
    /// Division or wild-card procedure.
    pub procedure: Procedure,
    /// The step that separated the teams, or `None` for a coin flip.
    pub step: Option<Step>,
    /// The tied teams the step looked at.
    pub group: Vec<Team>,
    /// The teams it kept: the rest lost the tie at this step.
    pub kept: Vec<Team>,
}

impl Tiebreak {
    /// Whether `team` was in this tie and lost it here.
    pub fn dropped(&self, team: Team) -> bool {
        self.group.contains(&team) && !self.kept.contains(&team)
    }

    /// Whether `team` was in this tie and survived it while others dropped.
    pub fn survived(&self, team: Team) -> bool {
        self.kept.contains(&team)
    }
}

const DIVISION_STEPS: [Step; 6] = [
    Step::HeadToHead,
    Step::DivisionRecord,
    Step::CommonGames,
    Step::ConferenceRecord,
    Step::StrengthOfVictory,
    Step::StrengthOfSchedule,
];

const WILD_CARD_STEPS: [Step; 5] = [
    Step::HeadToHead,
    Step::ConferenceRecord,
    Step::CommonGames,
    Step::StrengthOfVictory,
    Step::StrengthOfSchedule,
];

/// The fewest common games a wild-card tie needs before that step applies.
const MIN_WILD_CARD_COMMON_GAMES: u32 = 4;

/// Every team's record against every other team.
#[derive(Debug, Clone, PartialEq)]
pub struct Standings {
    /// `versus[a][b]` is team `a`'s record against team `b`.
    versus: [[Record; 32]; 32],
}

impl Default for Standings {
    fn default() -> Self {
        Standings::new()
    }
}

impl Standings {
    /// Standings with no games played.
    pub fn new() -> Self {
        Standings {
            versus: [[Record::default(); 32]; 32],
        }
    }

    /// The standings from `season`'s played regular-season games. Fails for
    /// seasons before [`FIRST_SEASON`] and for unknown teams.
    pub fn from_season(season: &Season<'_>) -> Result<Self, String> {
        if season.year < FIRST_SEASON {
            return Err(format!(
                "{} is before {FIRST_SEASON}, when today's divisions began",
                season.year
            ));
        }
        Standings::from_games(season.regular_season())
    }

    /// The standings from the played regular-season games in `games`.
    /// Unplayed and playoff games are skipped. Fails for unknown teams.
    pub fn from_games<'a>(games: impl IntoIterator<Item = &'a Game>) -> Result<Self, String> {
        let mut standings = Standings::new();
        for game in games {
            let Some(outcome) = game.outcome() else {
                continue;
            };
            if game.game_type != GameType::Regular {
                continue;
            }
            let team = |abbr: &str| {
                Team::from_abbr(abbr)
                    .ok_or_else(|| format!("{} has unknown team {abbr:?}", game.id))
            };
            standings.add_result(team(&game.home_team)?, team(&game.away_team)?, outcome);
        }
        Ok(standings)
    }

    /// Records one game's result.
    pub fn add_result(&mut self, home: Team, away: Team, outcome: Outcome) {
        let (home_result, away_result) = match outcome {
            Outcome::HomeWin => (WIN, LOSS),
            Outcome::AwayWin => (LOSS, WIN),
            Outcome::Tie => (TIE, TIE),
        };
        self.versus[home.index()][away.index()] += home_result;
        self.versus[away.index()][home.index()] += away_result;
    }

    /// `team`'s overall record.
    pub fn record(&self, team: Team) -> Record {
        self.versus[team.index()].iter().copied().sum()
    }

    /// `team`'s record in games against `opponent`.
    pub fn record_against(&self, team: Team, opponent: Team) -> Record {
        self.versus[team.index()][opponent.index()]
    }

    /// `team`'s record against the teams in `opponents`.
    fn record_against_all(&self, team: Team, opponents: impl IntoIterator<Item = Team>) -> Record {
        opponents
            .into_iter()
            .map(|opponent| self.record_against(team, opponent))
            .sum()
    }

    /// `team`'s record against its own division.
    pub fn division_record(&self, team: Team) -> Record {
        self.record_against_all(team, team.division().teams())
    }

    /// `team`'s record against its own conference.
    pub fn conference_record(&self, team: Team) -> Record {
        self.record_against_all(team, team.conference().teams())
    }

    /// The combined record of the teams `team` beat, counted once per win.
    pub fn strength_of_victory(&self, team: Team) -> Record {
        Team::all()
            .map(|opponent| {
                let wins = self.record_against(team, opponent).wins;
                self.record(opponent).times(wins)
            })
            .sum()
    }

    /// The combined record of every team `team` played, counted once per
    /// game.
    pub fn strength_of_schedule(&self, team: Team) -> Record {
        Team::all()
            .map(|opponent| {
                let games = self.record_against(team, opponent).games();
                self.record(opponent).times(games)
            })
            .sum()
    }

    /// `division`'s teams, best first. Ties still standing after every step
    /// are broken by `coin`, which gets the number of tied teams and returns
    /// the index of the one to rank first.
    pub fn division_ranking(
        &self,
        division: Division,
        coin: &mut impl FnMut(usize) -> usize,
    ) -> [Team; 4] {
        self.division_ranking_traced(division, coin, &mut |_, _, _, _| {})
    }

    fn division_ranking_traced(
        &self,
        division: Division,
        coin: &mut impl FnMut(usize) -> usize,
        trace: &mut impl FnMut(Procedure, Option<Step>, &[Team], &[Team]),
    ) -> [Team; 4] {
        let ranked = self.rank(&division.teams(), Procedure::Division, None, 4, coin, trace);
        [ranked[0], ranked[1], ranked[2], ranked[3]]
    }

    /// `conference`'s playoff seeds: the four division winners, then
    /// `wild_cards` wild cards (see [`wild_cards`]). Seed 1 is first.
    /// `coin` breaks ties as in [`Standings::division_ranking`].
    pub fn seeds(
        &self,
        conference: Conference,
        wild_cards: usize,
        coin: &mut impl FnMut(usize) -> usize,
    ) -> Vec<Team> {
        self.seeds_inner(conference, wild_cards, coin, &mut |_, _, _, _| {})
    }

    /// [`Standings::seeds`], also reporting every tiebreaker decision to
    /// `trace`: division ties first, then the division winners' seeding,
    /// then the wild cards. Ties with no tiebreaker step (teams with
    /// different records) report nothing.
    pub fn seeds_traced(
        &self,
        conference: Conference,
        wild_cards: usize,
        coin: &mut impl FnMut(usize) -> usize,
        trace: &mut impl FnMut(Tiebreak),
    ) -> Vec<Team> {
        self.seeds_inner(
            conference,
            wild_cards,
            coin,
            &mut |procedure, step, group, kept| {
                trace(Tiebreak {
                    procedure,
                    step,
                    group: group.to_vec(),
                    kept: kept.to_vec(),
                })
            },
        )
    }

    fn seeds_inner(
        &self,
        conference: Conference,
        wild_cards: usize,
        coin: &mut impl FnMut(usize) -> usize,
        trace: &mut impl FnMut(Procedure, Option<Step>, &[Team], &[Team]),
    ) -> Vec<Team> {
        let mut division_rank = [0u8; 32];
        let mut winners = Vec::with_capacity(4);
        let mut others = Vec::with_capacity(12);
        for division in conference.divisions() {
            let ranking = self.division_ranking_traced(division, coin, trace);
            for (rank, team) in ranking.iter().enumerate() {
                division_rank[team.index()] = rank as u8;
            }
            winners.push(ranking[0]);
            others.extend_from_slice(&ranking[1..]);
        }
        let ranks = Some(&division_rank);
        let mut seeds = self.rank(&winners, Procedure::WildCard, ranks, 4, coin, trace);
        seeds.extend(self.rank(&others, Procedure::WildCard, ranks, wild_cards, coin, trace));
        seeds
    }

    /// The best `limit` of `teams`, in order: by winning percentage, then by
    /// tiebreakers among teams with the same percentage.
    fn rank(
        &self,
        teams: &[Team],
        procedure: Procedure,
        division_rank: Option<&[u8; 32]>,
        limit: usize,
        coin: &mut impl FnMut(usize) -> usize,
        trace: &mut impl FnMut(Procedure, Option<Step>, &[Team], &[Team]),
    ) -> Vec<Team> {
        let mut by_pct: Vec<(Team, Record)> = teams
            .iter()
            .map(|&team| (team, self.record(team)))
            .collect();
        by_pct.sort_by(|a, b| b.1.cmp_pct(a.1));
        let mut ranked = Vec::with_capacity(limit);
        for group in by_pct.chunk_by(|a, b| a.1.cmp_pct(b.1) == Ordering::Equal) {
            let mut tied: Vec<Team> = group.iter().map(|(team, _)| *team).collect();
            while !tied.is_empty() && ranked.len() < limit {
                let best = self.break_tie(&tied, procedure, division_rank, coin, trace);
                tied.retain(|&team| team != best);
                ranked.push(best);
            }
        }
        ranked
    }

    /// The best of `tied`, teams with the same winning percentage.
    fn break_tie(
        &self,
        tied: &[Team],
        procedure: Procedure,
        division_rank: Option<&[u8; 32]>,
        coin: &mut impl FnMut(usize) -> usize,
        trace: &mut impl FnMut(Procedure, Option<Step>, &[Team], &[Team]),
    ) -> Team {
        let mut group = tied.to_vec();
        if procedure == Procedure::WildCard
            && let Some(ranks) = division_rank
        {
            // Division ties are broken first: only each division's
            // best-ranked team takes part.
            group.retain(|team| {
                !tied.iter().any(|other| {
                    other.division() == team.division()
                        && ranks[other.index()] < ranks[team.index()]
                })
            });
        }
        let steps: &[Step] = match procedure {
            Procedure::Division => &DIVISION_STEPS,
            Procedure::WildCard => &WILD_CARD_STEPS,
        };
        'restart: while group.len() > 1 {
            for &step in steps {
                let survivors = self.apply(step, &group, procedure);
                if survivors.len() < group.len() {
                    trace(procedure, Some(step), &group, &survivors);
                    // A step that drops any team starts the procedure again
                    // with the teams that are left.
                    group = survivors;
                    continue 'restart;
                }
            }
            let winner = group[coin(group.len()) % group.len()];
            trace(procedure, None, &group, &[winner]);
            return winner;
        }
        group[0]
    }

    /// The teams in `group` that step `step` keeps: all of them if it
    /// doesn't separate them.
    fn apply(&self, step: Step, group: &[Team], procedure: Procedure) -> Vec<Team> {
        let records: Option<Vec<Record>> = match step {
            Step::HeadToHead => return self.head_to_head(group, procedure),
            Step::DivisionRecord => Some(group.iter().map(|&t| self.division_record(t)).collect()),
            Step::ConferenceRecord => {
                Some(group.iter().map(|&t| self.conference_record(t)).collect())
            }
            Step::CommonGames => self.common_games(group, procedure),
            Step::StrengthOfVictory => {
                Some(group.iter().map(|&t| self.strength_of_victory(t)).collect())
            }
            Step::StrengthOfSchedule => Some(
                group
                    .iter()
                    .map(|&t| self.strength_of_schedule(t))
                    .collect(),
            ),
        };
        match records {
            Some(records) => best(group, &records),
            None => group.to_vec(),
        }
    }

    /// Head-to-head: the combined record in games among `group`. A
    /// wild-card tie of three or more teams uses it only on a sweep: one
    /// team beat every other (it wins) or lost to every other (it drops
    /// out).
    fn head_to_head(&self, group: &[Team], procedure: Procedure) -> Vec<Team> {
        if procedure == Procedure::WildCard && group.len() > 2 {
            let against_each = |team: Team, wanted: fn(Record) -> bool| {
                group
                    .iter()
                    .filter(|&&other| other != team)
                    .all(|&other| wanted(self.record_against(team, other)))
            };
            let swept = |r: Record| r.games() > 0 && r.wins == r.games();
            let was_swept = |r: Record| r.games() > 0 && r.losses == r.games();
            if let Some(&winner) = group.iter().find(|&&team| against_each(team, swept)) {
                return vec![winner];
            }
            return group
                .iter()
                .copied()
                .filter(|&team| !against_each(team, was_swept))
                .collect();
        }
        let records: Vec<Record> = group
            .iter()
            .map(|&team| self.record_against_all(team, group.iter().copied()))
            .collect();
        if records.iter().any(|record| record.games() == 0) {
            return group.to_vec();
        }
        best(group, &records)
    }

    /// Each team's record against the opponents every team in `group`
    /// played, or `None` when a wild-card team has fewer than four such
    /// games.
    fn common_games(&self, group: &[Team], procedure: Procedure) -> Option<Vec<Record>> {
        let common: Vec<Team> = Team::all()
            .filter(|opponent| !group.contains(opponent))
            .filter(|&opponent| {
                group
                    .iter()
                    .all(|&team| self.record_against(team, opponent).games() > 0)
            })
            .collect();
        let records: Vec<Record> = group
            .iter()
            .map(|&team| self.record_against_all(team, common.iter().copied()))
            .collect();
        let too_few = records
            .iter()
            .any(|record| record.games() < MIN_WILD_CARD_COMMON_GAMES);
        if procedure == Procedure::WildCard && too_few {
            return None;
        }
        Some(records)
    }
}

const WIN: Record = Record {
    wins: 1,
    losses: 0,
    ties: 0,
};
const LOSS: Record = Record {
    wins: 0,
    losses: 1,
    ties: 0,
};
const TIE: Record = Record {
    wins: 0,
    losses: 0,
    ties: 1,
};

/// The teams in `group` whose record (at the same index in `records`) has
/// the best winning percentage.
fn best(group: &[Team], records: &[Record]) -> Vec<Team> {
    let Some(top) = records.iter().copied().max_by(|a, b| a.cmp_pct(*b)) else {
        return group.to_vec();
    };
    group
        .iter()
        .zip(records)
        .filter(|(_, record)| record.cmp_pct(top) == Ordering::Equal)
        .map(|(&team, _)| team)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::Schedule;

    /// Every 2002 to 2025 game, regular season and playoffs, from nflverse;
    /// see `fixtures/README.md`.
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

    fn abbrs(teams: &[Team]) -> Vec<&'static str> {
        teams.iter().map(|team| team.abbr()).collect()
    }

    /// Records `winner` beating `loser` at home, `times` times.
    fn beat(standings: &mut Standings, winner: &str, loser: &str, times: u32) {
        for _ in 0..times {
            standings.add_result(team(winner), team(loser), Outcome::HomeWin);
        }
    }

    fn no_coin(tied: usize) -> usize {
        panic!("no coin flip expected, {tied} teams tied")
    }

    fn record(wins: u32, losses: u32, ties: u32) -> Record {
        Record { wins, losses, ties }
    }

    fn schedule() -> Schedule {
        match Schedule::parse(SEASONS) {
            Ok(schedule) => schedule,
            Err(error) => panic!("seasons fixture should parse: {error}"),
        }
    }

    fn standings(schedule: &Schedule, year: u16) -> Standings {
        let Some(season) = schedule.season(year) else {
            panic!("fixture should have {year}");
        };
        match Standings::from_season(&season) {
            Ok(standings) => standings,
            Err(error) => panic!("{year} standings: {error}"),
        }
    }

    #[test]
    fn every_team_has_one_division() {
        let all: Vec<Team> = Team::all().collect();
        assert_eq!(all.len(), 32);
        for conference in Conference::ALL {
            assert_eq!(conference.teams().count(), 16);
            for division in conference.divisions() {
                for team in division.teams() {
                    assert_eq!(team.division(), division, "{team}");
                    assert_eq!(team.conference(), conference, "{team}");
                    assert_eq!(Team::from_abbr(team.abbr()), Some(team));
                }
            }
        }
        assert_eq!(team("CHI").division(), Division::NFC_NORTH);
        assert_eq!(
            abbrs(&Division::NFC_NORTH.teams()),
            ["CHI", "DET", "GB", "MIN"]
        );
        assert_eq!(Division::NFC_NORTH.to_string(), "NFC North");
    }

    #[test]
    fn bears_constant_is_chicago() {
        assert_eq!(Team::BEARS.abbr(), crate::BEARS);
        assert_eq!(Team::BEARS.division(), Division::NFC_NORTH);
    }

    #[test]
    fn relocated_teams_keep_their_division() {
        assert_eq!(Team::from_abbr("OAK"), Some(team("LV")));
        assert_eq!(Team::from_abbr("SD"), Some(team("LAC")));
        assert_eq!(Team::from_abbr("STL"), Some(team("LA")));
        assert_eq!(Team::from_abbr("chi"), None);
        assert_eq!(Team::from_abbr("XYZ"), None);
    }

    #[test]
    fn ties_count_as_half_a_win() {
        assert_eq!(record(8, 8, 1).pct(), 0.5);
        assert_eq!(record(0, 0, 0).pct(), 0.0);
        assert_eq!(record(13, 3, 0).cmp_pct(record(14, 3, 0)), Ordering::Less);
        assert_eq!(record(1, 1, 0).cmp_pct(record(0, 0, 2)), Ordering::Equal);
        assert_eq!(
            record(10, 6, 1).cmp_pct(record(10, 7, 0)),
            Ordering::Greater
        );
        assert_eq!(record(10, 6, 1).to_string(), "10-6-1");
        assert_eq!(record(10, 7, 0).to_string(), "10-7");
    }

    #[test]
    fn records_split_by_division_and_conference() {
        let mut standings = Standings::new();
        beat(&mut standings, "CHI", "GB", 1);
        beat(&mut standings, "DAL", "CHI", 1);
        beat(&mut standings, "CHI", "KC", 2);
        standings.add_result(team("NYJ"), team("CHI"), Outcome::Tie);
        let chi = team("CHI");
        assert_eq!(standings.record(chi), record(3, 1, 1));
        assert_eq!(standings.division_record(chi), record(1, 0, 0));
        assert_eq!(standings.conference_record(chi), record(1, 1, 0));
        assert_eq!(standings.record_against(chi, team("KC")), record(2, 0, 0));
        assert_eq!(standings.record(team("NYJ")), record(0, 0, 1));
        // Beat GB (0-1) once and KC (0-2) twice: 0-5. Played those plus DAL
        // (1-0) and NYJ (0-0-1).
        assert_eq!(standings.strength_of_victory(chi), record(0, 5, 0));
        assert_eq!(standings.strength_of_schedule(chi), record(1, 5, 1));
    }

    #[test]
    fn from_games_skips_unplayed_and_playoff_games() {
        let schedule = schedule();
        let Some(season) = schedule.season(2025) else {
            panic!("fixture should have 2025");
        };
        let Ok(standings) = Standings::from_games(season.games()) else {
            panic!("2025 should load");
        };
        let total: u32 = Team::all().map(|team| standings.record(team).games()).sum();
        assert_eq!(total, 2 * 272);
    }

    #[test]
    fn rejects_seasons_before_today_s_divisions() {
        let csv = "game_id,season,game_type,week,gameday,away_team,away_score,home_team,home_score,location,result,spread_line,div_game\n\
                   2001_01_A_B,2001,REG,1,2001-09-09,MIN,1,CHI,2,Home,1,,1\n";
        let Ok(schedule) = Schedule::parse(csv) else {
            panic!("row should parse");
        };
        let Some(season) = schedule.season(2001) else {
            panic!("2001 should be there");
        };
        let Err(error) = Standings::from_season(&season) else {
            panic!("2001 should be rejected");
        };
        assert!(error.contains("2002"), "{error}");
    }

    #[test]
    fn rejects_an_unknown_team() {
        let schedule = schedule();
        let Some(game) = schedule.games().first() else {
            panic!("fixture has games");
        };
        let mut game = game.clone();
        game.home_team = "XYZ".into();
        let Err(error) = Standings::from_games([&game]) else {
            panic!("XYZ should be rejected");
        };
        assert!(error.contains("XYZ"), "{error}");
    }

    #[test]
    fn division_tie_goes_to_head_to_head() {
        let mut standings = Standings::new();
        beat(&mut standings, "CHI", "GB", 1);
        beat(&mut standings, "NE", "CHI", 1);
        beat(&mut standings, "GB", "NE", 1);
        let ranking = standings.division_ranking(Division::NFC_NORTH, &mut |_| 0);
        // DET and MIN haven't played: a coin flip, which takes the first.
        assert_eq!(abbrs(&ranking), ["CHI", "GB", "DET", "MIN"]);
    }

    #[test]
    fn split_division_tie_goes_to_division_record() {
        let mut standings = Standings::new();
        beat(&mut standings, "CHI", "GB", 1);
        beat(&mut standings, "GB", "CHI", 1);
        beat(&mut standings, "CHI", "DET", 1);
        beat(&mut standings, "KC", "CHI", 1);
        beat(&mut standings, "GB", "KC", 1);
        beat(&mut standings, "DET", "GB", 1);
        beat(&mut standings, "NE", "DET", 1);
        beat(&mut standings, "MIN", "NE", 1);
        beat(&mut standings, "NE", "MIN", 1);
        let ranking = standings.division_ranking(Division::NFC_NORTH, &mut no_coin);
        assert_eq!(abbrs(&ranking), ["CHI", "GB", "MIN", "DET"]);
    }

    #[test]
    fn three_way_division_tie_restarts_when_one_team_drops_out() {
        let mut standings = Standings::new();
        // A rock-paper-scissors split: everyone 2-2 among the three.
        beat(&mut standings, "CHI", "GB", 2);
        beat(&mut standings, "GB", "DET", 2);
        beat(&mut standings, "DET", "CHI", 2);
        // DET has the worst division record, so it drops out at step 2.
        beat(&mut standings, "CHI", "MIN", 1);
        beat(&mut standings, "GB", "MIN", 1);
        beat(&mut standings, "MIN", "DET", 1);
        // Even out the overall records at 3-3.
        beat(&mut standings, "KC", "CHI", 1);
        beat(&mut standings, "KC", "GB", 1);
        beat(&mut standings, "DET", "KC", 1);
        for abbr in ["CHI", "GB", "DET"] {
            assert_eq!(standings.record(team(abbr)), record(3, 3, 0), "{abbr}");
        }
        // CHI and GB start over at head-to-head, which CHI swept. Carrying on
        // to common games instead would pick GB, which swept DET.
        let ranking = standings.division_ranking(Division::NFC_NORTH, &mut no_coin);
        assert_eq!(abbrs(&ranking), ["CHI", "GB", "DET", "MIN"]);
    }

    #[test]
    fn three_way_wild_card_head_to_head_needs_a_sweep() {
        let mut standings = Standings::new();
        beat(&mut standings, "DAL", "ATL", 1);
        beat(&mut standings, "ATL", "SEA", 1);
        beat(&mut standings, "SEA", "DAL", 1);
        beat(&mut standings, "DAL", "SEA", 1);
        beat(&mut standings, "ATL", "NO", 2);
        let tied = teams(&["DAL", "ATL", "SEA"]);
        // DAL is 2-1 against the other two but swept neither, so conference
        // record decides: ATL 3-1, DAL 2-1.
        let wild_card = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(wild_card, team("ATL"));
        let combined = standings.break_tie(
            &tied,
            Procedure::Division,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(combined, team("DAL"));
    }

    #[test]
    fn wild_card_sweeps_win_or_drop_out() {
        let mut standings = Standings::new();
        beat(&mut standings, "DAL", "ATL", 1);
        beat(&mut standings, "DAL", "SEA", 1);
        let tied = teams(&["ATL", "SEA", "DAL"]);
        let best = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(best, team("DAL"));

        // SEA lost to both; ATL and DAL never met, so conference record
        // decides between them.
        let mut standings = Standings::new();
        beat(&mut standings, "DAL", "SEA", 1);
        beat(&mut standings, "ATL", "SEA", 1);
        beat(&mut standings, "NYG", "DAL", 1);
        let best = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(best, team("ATL"));
    }

    #[test]
    fn wild_card_common_games_need_four() {
        let mut standings = Standings::new();
        // Three common games with KC, where ATL is 2-1 and DAL 1-2.
        beat(&mut standings, "DAL", "KC", 1);
        beat(&mut standings, "KC", "DAL", 2);
        beat(&mut standings, "ATL", "KC", 2);
        beat(&mut standings, "KC", "ATL", 1);
        // DAL's other win is over a 5-1 BUF: the better strength of victory.
        beat(&mut standings, "DAL", "BUF", 1);
        beat(&mut standings, "BUF", "MIA", 5);
        beat(&mut standings, "MIA", "ATL", 1);
        let tied = teams(&["DAL", "ATL"]);
        let wild_card = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(wild_card, team("DAL"));
        // Division ties have no minimum, so common games pick ATL.
        let division = standings.break_tie(
            &tied,
            Procedure::Division,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(division, team("ATL"));
        // A fourth common game each brings the step back.
        beat(&mut standings, "ATL", "KC", 1);
        beat(&mut standings, "KC", "DAL", 1);
        let wild_card = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut no_coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(wild_card, team("ATL"));
    }

    #[test]
    fn wild_card_ties_break_division_ties_first() {
        let mut standings = Standings::new();
        beat(&mut standings, "CHI", "MIN", 3);
        beat(&mut standings, "PHI", "NYG", 3);
        // DET beat GB, DAL beat DET, GB beat DAL: all 1-1.
        beat(&mut standings, "DET", "GB", 1);
        beat(&mut standings, "DAL", "DET", 1);
        beat(&mut standings, "GB", "DAL", 1);
        let seeds = standings.seeds(Conference::Nfc, 3, &mut |_| 0);
        assert_eq!(seeds.len(), 7);
        // GB sits out until DET, ahead of it in the division, is placed.
        assert_eq!(abbrs(&seeds[4..]), ["DAL", "DET", "GB"]);
    }

    #[test]
    fn coin_flip_settles_what_is_left() {
        let standings = Standings::new();
        let tied = teams(&["DAL", "ATL"]);
        let mut flips = Vec::new();
        let mut coin = |n: usize| {
            flips.push(n);
            1
        };
        let best = standings.break_tie(
            &tied,
            Procedure::WildCard,
            None,
            &mut coin,
            &mut |_, _, _, _| {},
        );
        assert_eq!(best, team("ATL"));
        assert_eq!(flips, [2]);
    }

    #[test]
    fn division_winners_are_seeded_first() {
        let schedule = schedule();
        // 2010: the 7-9 Seahawks won the NFC West and hosted the 11-5 Saints.
        let seeds = standings(&schedule, 2010).seeds(Conference::Nfc, 2, &mut no_coin);
        assert_eq!(abbrs(&seeds), ["ATL", "CHI", "PHI", "SEA", "NO", "GB"]);
    }

    #[test]
    fn matches_real_seeds() {
        let schedule = schedule();
        let cases: [(u16, Conference, &[&str]); 5] = [
            // Three-way AFC East tie at 9-7 won by the Jets; Raiders as OAK.
            (
                2002,
                Conference::Afc,
                &["LV", "TEN", "PIT", "NYJ", "IND", "CLE"],
            ),
            // 11-5 Dolphins win the AFC East over the 11-5 Patriots, who miss
            // the playoffs behind the 11-5 Ravens.
            (
                2008,
                Conference::Afc,
                &["TEN", "PIT", "MIA", "LAC", "IND", "BAL"],
            ),
            (
                2018,
                Conference::Nfc,
                &["NO", "LA", "CHI", "DAL", "SEA", "PHI"],
            ),
            (
                2023,
                Conference::Afc,
                &["BAL", "BUF", "KC", "HOU", "CLE", "MIA", "PIT"],
            ),
            (
                2023,
                Conference::Nfc,
                &["SF", "DAL", "DET", "TB", "PHI", "LA", "GB"],
            ),
        ];
        for (year, conference, expected) in cases {
            let seeds =
                standings(&schedule, year).seeds(conference, wild_cards(year), &mut no_coin);
            assert_eq!(abbrs(&seeds), expected, "{year} {conference}");
        }
    }

    /// Checks every 2002 to 2025 season's seeds against the real playoff
    /// games: the same teams made it, wild-card games paired seeds adding up
    /// to 9 with the better seed at home, and the better seed hosted every
    /// divisional game.
    #[test]
    fn seeds_match_every_real_playoff_bracket() {
        let schedule = schedule();
        let mut checked = 0;
        for season in schedule.seasons() {
            let year = season.year;
            let standings = standings(&schedule, year);
            for conference in Conference::ALL {
                let seeds = standings.seeds(conference, wild_cards(year), &mut no_coin);
                let seed = |abbr: &str| {
                    let team = team(abbr);
                    match seeds.iter().position(|&seeded| seeded == team) {
                        Some(index) => index + 1,
                        None => panic!("{year} {conference}: {abbr} played but wasn't seeded"),
                    }
                };
                let games: Vec<&Game> = season
                    .playoffs()
                    .filter(|game| {
                        game.game_type <= GameType::Divisional
                            && team(&game.home_team).conference() == conference
                    })
                    .collect();
                let mut in_playoffs = Vec::new();
                for game in &games {
                    let (home, away) = (seed(&game.home_team), seed(&game.away_team));
                    assert!(home < away, "{year} {conference} {}", game.id);
                    if game.game_type == GameType::WildCard {
                        assert_eq!(home + away, 9, "{year} {conference} {}", game.id);
                    }
                    in_playoffs.extend([home, away]);
                }
                in_playoffs.sort_unstable();
                in_playoffs.dedup();
                assert_eq!(in_playoffs.len(), seeds.len(), "{year} {conference}");
                checked += 1;
            }
        }
        assert_eq!(checked, 2 * 24);
    }

    /// Every tiebreaker decision for `conference` in `year`.
    fn traces(schedule: &Schedule, year: u16, conference: Conference) -> Vec<Tiebreak> {
        let mut traces = Vec::new();
        standings(schedule, year).seeds_traced(
            conference,
            wild_cards(year),
            &mut no_coin,
            &mut |tiebreak| traces.push(tiebreak),
        );
        traces
    }

    fn decided(traces: &[Tiebreak], winner: &str, loser: &str) -> Option<Option<Step>> {
        traces
            .iter()
            .find(|t| t.survived(team(winner)) && t.dropped(team(loser)))
            .map(|t| t.step)
    }

    #[test]
    fn traces_real_tiebreaks() {
        let schedule = schedule();
        // 2023: the Bills won the AFC East over the Dolphins and took the
        // No. 2 seed over the Chiefs, both on head-to-head; the Cowboys
        // took the NFC's No. 2 seed over the Lions on head-to-head.
        let afc = traces(&schedule, 2023, Conference::Afc);
        assert_eq!(decided(&afc, "BUF", "MIA"), Some(Some(Step::HeadToHead)));
        assert_eq!(decided(&afc, "BUF", "KC"), Some(Some(Step::HeadToHead)));
        let nfc = traces(&schedule, 2023, Conference::Nfc);
        assert_eq!(decided(&nfc, "DAL", "DET"), Some(Some(Step::HeadToHead)));
        // 2024: the Steelers were seeded over the Broncos on head-to-head.
        let afc = traces(&schedule, 2024, Conference::Afc);
        assert_eq!(decided(&afc, "PIT", "DEN"), Some(Some(Step::HeadToHead)));
    }

    #[test]
    fn tracing_never_changes_the_seeds() {
        let schedule = schedule();
        for year in 2002..=2025 {
            let standings = standings(&schedule, year);
            for conference in Conference::ALL {
                let plain = standings.seeds(conference, wild_cards(year), &mut no_coin);
                let mut traces = Vec::new();
                let traced = standings.seeds_traced(
                    conference,
                    wild_cards(year),
                    &mut no_coin,
                    &mut |tiebreak| traces.push(tiebreak),
                );
                assert_eq!(plain, traced, "{year} {conference}");
                for tiebreak in &traces {
                    assert!(tiebreak.group.len() > 1, "{year}: {tiebreak:?}");
                    assert!(!tiebreak.kept.is_empty(), "{year}: {tiebreak:?}");
                    assert!(tiebreak.kept.len() < tiebreak.group.len());
                    assert!(tiebreak.kept.iter().all(|t| tiebreak.group.contains(t)));
                    let pct = standings.record(tiebreak.group[0]);
                    assert!(
                        tiebreak
                            .group
                            .iter()
                            .all(|&t| standings.record(t).cmp_pct(pct) == Ordering::Equal),
                        "{year}: tied teams share a record: {tiebreak:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_coin_flip_is_traced_as_no_step() {
        // Two teams that never met, with identical results against the
        // same opponents, can only be split by a coin.
        let mut standings = Standings::new();
        beat(&mut standings, "CHI", "NYG", 1);
        beat(&mut standings, "GB", "NYG", 1);
        let mut traces = Vec::new();
        standings.division_ranking_traced(
            Division::NFC_NORTH,
            &mut |_| 0,
            &mut |_, step, group, _| traces.push((step, group.to_vec())),
        );
        assert!(
            traces
                .iter()
                .any(|(step, group)| step.is_none() && group.contains(&team("CHI"))),
            "{traces:?}"
        );
        assert_eq!(Step::StrengthOfVictory.to_string(), "strength of victory");
    }
}
