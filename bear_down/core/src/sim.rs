//! The Monte Carlo season and playoff simulator, and each game's swing on the
//! Bears' odds.
//!
//! [`simulate`] starts from a season's actual results, plays every remaining
//! regular-season game with the Elo win probability, seeds both conferences
//! with [`Standings::seeds`] and plays the bracket game by game, many times
//! over. Simulated games never tie, and ratings don't move inside a simulated
//! season. Playoff games that were already played keep their real winner.
//!
//! For every regular-season game left in the current week, each simulated
//! season is played twice from the same seed, once with each result (common
//! random numbers). That gives the swing on the focus team's playoff and
//! Super Bowl odds and on the NFC North divisional-round goal with little
//! noise, so even a small swing from a far-off game shows. See `docs/decisions/0003-own-monte-carlo-simulator.md`
//! and `docs/decisions/0007-nfc-north-divisional-mode.md`.
//!
//! Each pair is also sorted into a [`Channel`] (how the result reached the
//! focus team) and, when its seeding changed, a [`Cause`], so the playoff
//! and Super Bowl swings split into shares that add up to the whole. A game
//! too close to the "doesn't matter" line ([`NEGLIGIBLE_SWING`]) to call
//! gets more pairs, and a game the relevance pass shows can't reach the focus
//! team isn't simulated at all; [`GameImpact::tier`] sorts the result.

use std::collections::HashMap;

use crate::NFC_NORTH;
use crate::elo::{Ratings, win_probability};
use crate::games::Outcome;
use crate::games::{Date, Game, Season};
use crate::relevance::{Relevance, relevance};
use crate::standings::{Conference, Procedure, Record, Standings, Step, Team, wild_cards};

/// How many seasons a weekly build simulates (decision 0003).
pub const DEFAULT_SIMULATIONS: u32 = 100_000;

/// The "doesn't matter" line on a game's Super Bowl swing: 0.05 percentage
/// points (BEAR-005). A game is negligible only when its swing is below this
/// with confidence.
pub const NEGLIGIBLE_SWING: f64 = 0.0005;

/// The most pairs a game gets, as a multiple of the simulations, when its
/// swing is too close to [`NEGLIGIBLE_SWING`] to call.
pub const MAX_PAIR_FACTOR: u32 = 4;

/// Whether `swing` is too close to [`NEGLIGIBLE_SWING`] to call: its
/// two-standard-error range spans the line and is still wider than the
/// plan's target (a standard error over a fifth of the line).
fn near_the_line(swing: &Swing) -> bool {
    let (size, margin) = (swing.swing().abs(), 2.0 * swing.std_error);
    size - margin < NEGLIGIBLE_SWING
        && size + margin > NEGLIGIBLE_SWING
        && swing.std_error > NEGLIGIBLE_SWING / 5.0
}

/// How much a game matters for one goal, from its swing and whether it can
/// reach the focus team at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// The swing is at least [`NEGLIGIBLE_SWING`], with confidence.
    Matters,
    /// Neither clearly above nor clearly below the line: a small effect.
    Small,
    /// Below [`NEGLIGIBLE_SWING`] with confidence: doesn't matter.
    Negligible,
    /// The game can't reach the focus team in any season.
    None,
}

impl Tier {
    /// The tier's name in `rooting.json`: `matters`, `small`, `negligible`
    /// or `none`.
    pub fn name(self) -> &'static str {
        match self {
            Tier::Matters => "matters",
            Tier::Small => "small",
            Tier::Negligible => "negligible",
            Tier::None => "none",
        }
    }

    /// The tier with this `rooting.json` name, if any.
    pub fn from_name(name: &str) -> Option<Tier> {
        [Tier::Matters, Tier::Small, Tier::Negligible, Tier::None]
            .into_iter()
            .find(|tier| tier.name() == name)
    }
}

/// The tier of a `swing` with standard error `std_error` against a
/// "doesn't matter" `line`, for a game that can reach the focus team. The
/// CLI uses it with [`NEGLIGIBLE_SWING`]; the page uses it with its A/B
/// arm's line (`docs/plans/2026-11-fan-feedback.md`), so both sort games the
/// same way.
pub fn tier_against(swing: f64, std_error: f64, line: f64) -> Tier {
    let (size, margin) = (swing.abs(), 2.0 * std_error);
    if size + margin < line {
        Tier::Negligible
    } else if size - margin >= line {
        Tier::Matters
    } else {
        Tier::Small
    }
}

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
    /// The goal's probability when the home team wins.
    pub if_home_wins: f64,
    /// The goal's probability when the away team wins.
    pub if_away_wins: f64,
    /// The standard error of [`Swing::swing`]. Infinite with no
    /// simulations.
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

    /// No swing at all, known exactly (no noise).
    fn none() -> Self {
        Swing {
            if_home_wins: 0.0,
            if_away_wins: 0.0,
            std_error: 0.0,
        }
    }

    /// The swing's tier against [`NEGLIGIBLE_SWING`], for a game that can
    /// reach the focus team.
    pub fn tier(&self) -> Tier {
        tier_against(self.swing(), self.std_error, NEGLIGIBLE_SWING)
    }

    /// Builds a swing from `runs` paired seasons: how often the goal
    /// happened with a home win and with an away win, and how many pairs
    /// reached it only with a home win or only with an away win. The
    /// standard error is that of the mean paired difference.
    fn from_pairs(home: u32, away: u32, home_only: u32, away_only: u32, runs: u32) -> Self {
        if runs == 0 {
            return Swing {
                if_home_wins: 0.0,
                if_away_wins: 0.0,
                std_error: f64::INFINITY,
            };
        }
        let n = f64::from(runs);
        let mean = (f64::from(home_only) - f64::from(away_only)) / n;
        // Each pair's difference is 1, -1 or 0, so its square is 1 exactly
        // when the two results disagree.
        let mean_square = f64::from(home_only + away_only) / n;
        let variance = (mean_square - mean * mean).max(0.0) / n;
        Swing {
            if_home_wins: f64::from(home) / n,
            if_away_wins: f64::from(away) / n,
            std_error: variance.sqrt(),
        }
    }
}

/// One remaining game this week and how its result moves the goals.
#[derive(Debug, Clone, PartialEq)]
pub struct GameImpact {
    /// The nflverse game id.
    pub game_id: String,
    /// The day the game is played.
    pub gameday: Date,
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
    /// The swing on the focus team's odds of winning its division.
    pub division: Swing,
    /// The swing on the focus team's odds of the conference's top seed.
    pub top_seed: Swing,
    /// How the playoff and Super Bowl swings split by channel and cause,
    /// biggest Super Bowl share first. Only pairs whose seasons differ in
    /// some way are counted, so a game that changes nothing has none.
    pub channels: Vec<ChannelImpact>,
    /// How many paired seasons measured this game: the simulations, more
    /// for a game near the "doesn't matter" line, or none for a game that
    /// can't reach the focus team.
    pub pairs: u32,
    /// Which links the game has to the focus team, from the schedule alone.
    pub relevance: Relevance,
}

impl GameImpact {
    /// How much the game matters for `swing`'s goal: [`Tier::None`] when no
    /// link can reach the focus team, otherwise [`Swing::tier`].
    pub fn tier(&self, swing: &Swing) -> Tier {
        if self.relevance.can_matter() {
            swing.tier()
        } else {
            Tier::None
        }
    }

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

/// How one game's result reaches the focus team, read from a pair of
/// seasons that differ only in that game. Each pair is put in the first
/// channel, in this order, where its two seasons differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Channel {
    /// The focus team wins its division in one season and not the other.
    DivisionTitle,
    /// The focus team makes the playoffs in one season and not the other,
    /// as a wild card.
    WildCard,
    /// The focus team makes the playoffs in both, with different seeds.
    Seed,
    /// Same seed, but a different field in its conference: who else made
    /// the playoffs, or their seeds.
    PlayoffPath,
    /// Same conference field, but a different champion of the other
    /// conference: the likely Super Bowl opponent.
    SuperBowlOpponent,
    /// Both conferences were seeded the same, so the seasons differ only in
    /// the playoff draws (a tiebreaker coin flip used one more draw). This is
    /// simulation noise and averages out.
    Chance,
}

impl Channel {
    /// Every channel, in attribution order.
    pub const ALL: [Channel; 6] = [
        Channel::DivisionTitle,
        Channel::WildCard,
        Channel::Seed,
        Channel::PlayoffPath,
        Channel::SuperBowlOpponent,
        Channel::Chance,
    ];

    /// Whether the focus team's own seeding differs in this channel, so a
    /// [`Cause`] applies.
    pub fn is_seeding(self) -> bool {
        matches!(
            self,
            Channel::DivisionTitle | Channel::WildCard | Channel::Seed
        )
    }
}

/// Why the focus team's seeding differs between a pair of seasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cause {
    /// A record changed: the focus team's own (its game), or another team
    /// passing it or falling behind it outright.
    Record,
    /// A tiebreaker step: the first tiebreak the focus team was in that went
    /// differently in the two seasons (lost in one and won in the other, or
    /// only needed in one).
    Tiebreaker(Step),
    /// A coin flip decided the tie.
    CoinFlip,
}

/// One goal's share of a game's swing: the paired difference counted only
/// over the pairs in one channel. The shares of all channels add up to the
/// whole swing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contribution {
    /// The share of the swing, home win minus away win.
    pub swing: f64,
    /// Its standard error.
    pub std_error: f64,
}

impl Contribution {
    fn from_pairs(home_only: u32, away_only: u32, runs: u32) -> Self {
        let swing = Swing::from_pairs(home_only, away_only, home_only, away_only, runs);
        Contribution {
            swing: swing.swing(),
            std_error: swing.std_error,
        }
    }

    /// Whether the share is bigger than its noise: more than two standard
    /// errors.
    pub fn matters(&self) -> bool {
        self.swing.abs() > 2.0 * self.std_error
    }
}

/// One channel's part in a game's swing (plan step 2 in
/// `docs/plans/2026-11-robust-recommendations.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelImpact {
    /// How the result reaches the focus team.
    pub channel: Channel,
    /// For seeding channels, what decided the seeding; `None` otherwise.
    pub cause: Option<Cause>,
    /// Pairs of seasons in this channel and cause.
    pub pairs: u32,
    /// For seeding channels, the pairs where a home win gave the focus team
    /// the better seeding.
    pub home_better: u32,
    /// For seeding channels, the pairs where an away win did.
    pub away_better: u32,
    /// The share of the playoff-odds swing.
    pub playoffs: Contribution,
    /// The share of the Super Bowl-odds swing.
    pub champion: Contribution,
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

/// One goal's counts over paired seasons that differ only in one game.
#[derive(Default, Clone, Copy)]
struct PairedCounts {
    /// Seasons the goal happened when the home team won the game.
    home: u32,
    /// Seasons the goal happened when the away team won the game.
    away: u32,
    /// Pairs where only the home win reached the goal.
    home_only: u32,
    /// Pairs where only the away win reached the goal.
    away_only: u32,
}

impl PairedCounts {
    fn add(&mut self, home: bool, away: bool) {
        self.home += u32::from(home);
        self.away += u32::from(away);
        self.home_only += u32::from(home && !away);
        self.away_only += u32::from(away && !home);
    }

    fn swing(self, runs: u32) -> Swing {
        Swing::from_pairs(self.home, self.away, self.home_only, self.away_only, runs)
    }
}

/// How far each team got in one simulated postseason.
#[derive(Default)]
struct Postseason {
    /// The furthest round reached: 0 missed the playoffs, 1 made them,
    /// 2 divisional round, 3 conference game, 4 Super Bowl, 5 champion.
    reached: [u8; 32],
}

/// One simulated season, as far as the focus team's channels need it.
struct Played {
    postseason: Postseason,
    /// Both conferences' seeds, AFC first.
    seeds: [Vec<Team>; 2],
    /// The focus team's seed index, if it made the playoffs.
    focus_seed: Option<usize>,
    /// The focus team's final regular-season record.
    focus_record: Record,
    /// The first tiebreaks the focus team was in, in order.
    ties: FocusTies,
}

/// The tiebreaks one team was in during seeding, in order: each step (`None`
/// for a coin flip) and whether the team survived it. Fixed-size, so the
/// simulator's hot path doesn't allocate; ties past the first few are
/// dropped.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct FocusTies {
    events: [(Option<Step>, bool); FocusTies::CAPACITY],
    len: usize,
}

impl FocusTies {
    const CAPACITY: usize = 8;

    fn push(&mut self, step: Option<Step>, survived: bool) {
        if self.len < Self::CAPACITY {
            self.events[self.len] = (step, survived);
            self.len += 1;
        }
    }

    fn events(&self) -> &[(Option<Step>, bool)] {
        &self.events[..self.len]
    }

    /// The step of the first tiebreak that went differently in `self` than
    /// in `other`: one the team lost here and won there, or one that only
    /// happened in one of them. `self`'s event wins when both have one.
    fn first_difference(&self, other: &FocusTies) -> Option<Option<Step>> {
        let (mine, theirs) = (self.events(), other.events());
        let index = (0..mine.len().max(theirs.len()))
            .find(|&index| mine.get(index) != theirs.get(index))?;
        mine.get(index).or(theirs.get(index)).map(|&(step, _)| step)
    }
}

impl Played {
    /// The champion of `conference`, if the bracket got that far.
    fn conference_champion(&self, conference: Conference) -> Option<Team> {
        self.seeds[conference as usize]
            .iter()
            .copied()
            .find(|team| self.postseason.reached[team.index()] >= SUPER_BOWL)
    }
}

/// Sorts a pair of seasons into a [`Channel`], with a [`Cause`] for the
/// seeding channels and whether the home-win season gave the better seeding.
/// `None` when the two seasons don't differ in any way the focus team sees.
fn classify(home: &Played, away: &Played, focus: Team) -> Option<(Channel, Option<Cause>, bool)> {
    let conference = focus.conference() as usize;
    // A lower seed index is better; missing the playoffs is worst.
    let rank = |played: &Played| played.focus_seed.unwrap_or(usize::MAX);
    if home.focus_seed != away.focus_seed {
        let home_better = rank(home) < rank(away);
        let (better, worse) = if home_better {
            (home, away)
        } else {
            (away, home)
        };
        let division = |played: &Played| played.focus_seed.is_some_and(|seed| seed < 4);
        let channel = if division(home) != division(away) {
            Channel::DivisionTitle
        } else if home.focus_seed.is_none() || away.focus_seed.is_none() {
            Channel::WildCard
        } else {
            Channel::Seed
        };
        let cause = if home.focus_record != away.focus_record {
            Cause::Record
        } else {
            match worse.ties.first_difference(&better.ties) {
                Some(Some(step)) => Cause::Tiebreaker(step),
                Some(None) => Cause::CoinFlip,
                None => Cause::Record,
            }
        };
        return Some((channel, Some(cause), home_better));
    }
    if home.seeds[conference] != away.seeds[conference] {
        return Some((Channel::PlayoffPath, None, false));
    }
    let other = Conference::ALL[1 - conference];
    if home.conference_champion(other) != away.conference_champion(other) {
        return Some((Channel::SuperBowlOpponent, None, false));
    }
    let focus_reached = |played: &Played| played.postseason.reached[focus.index()];
    if focus_reached(home) != focus_reached(away) {
        return Some((Channel::Chance, None, false));
    }
    None
}

/// One channel and cause's counts over a game's pairs.
struct ChannelCounts {
    channel: Channel,
    cause: Option<Cause>,
    pairs: u32,
    home_better: u32,
    away_better: u32,
    playoffs: PairedCounts,
    champion: PairedCounts,
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
            home_win_probability: game
                .spread_line
                .map(spread_win_probability)
                .unwrap_or_else(|| ratings.home_win_probability(game)),
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

    // Plays one season from `seed`: the remaining regular-season games (with
    // `forced`'s game given to the home team if `true`, the away team if
    // `false`), then the playoffs. The same seed gives the same draws, so two
    // seasons that differ only in the forced game share everything else.
    let play_season = |seed: u64, forced: Option<(usize, bool)>| -> Played {
        let mut rng = Rng::new(seed);
        let mut standings = base.clone();
        for (index, game) in remaining.iter().enumerate() {
            let mut home_won = rng.chance(game.home_win_probability);
            if let Some((forced_index, home_wins)) = forced
                && forced_index == index
            {
                home_won = home_wins;
            }
            let outcome = if home_won {
                Outcome::HomeWin
            } else {
                Outcome::AwayWin
            };
            standings.add_result(game.home, game.away, outcome);
        }
        let mut coin = |tied: usize| rng.below(tied);
        let mut ties = FocusTies::default();
        let mut trace = |_: Procedure, step: Option<Step>, group: &[Team], kept: &[Team]| {
            if group.contains(&config.focus) {
                ties.push(step, kept.contains(&config.focus));
            }
        };
        let seeds = Conference::ALL.map(|conference| {
            standings.seeds_inner(conference, playoff_seeds - 4, &mut coin, &mut trace)
        });
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
        let focus_seed = seeds[config.focus.conference() as usize]
            .iter()
            .position(|&team| team == config.focus);
        let postseason = play_postseason(&seeds, &mut play);
        Played {
            postseason,
            seeds,
            focus_seed,
            focus_record: standings.record(config.focus),
            ties,
        }
    };
    // The goals each game's swing is measured on: the focus team's playoffs,
    // title, division title and top seed, and all four NFC North teams in
    // the divisional round.
    let goals = |played: &Played| {
        let (postseason, focus_seed) = (&played.postseason, &played.focus_seed);
        let focus = postseason.reached[config.focus.index()];
        let all_north = north
            .iter()
            .all(|&team| postseason.reached[team] >= DIVISIONAL);
        [
            focus >= PLAYOFFS,
            focus >= CHAMPION,
            all_north,
            focus_seed.is_some_and(|seed| seed < 4),
            *focus_seed == Some(0),
        ]
    };
    let run_seed =
        |run: u32| Rng::new(config.seed ^ (u64::from(run) << 32 | u64::from(run))).next_u64();

    let mut reached_counts = [[0u32; 6]; 32];
    let mut nfc_north_count = 0u32;
    for run in 0..config.simulations {
        let Played { postseason, .. } = play_season(run_seed(run), None);
        for (counts, &reached) in reached_counts.iter_mut().zip(&postseason.reached) {
            for round in counts.iter_mut().take(usize::from(reached) + 1) {
                *round += 1;
            }
        }
        nfc_north_count += u32::from(
            north
                .iter()
                .all(|&team| postseason.reached[team] >= DIVISIONAL),
        );
    }

    // Each game's swing: every season is played twice, once with each
    // result, from the same seed (common random numbers). The difference
    // then comes from this game alone, which makes even small swings clear.
    // Games whose Super Bowl swing is too close to the "doesn't matter" line
    // to call get more pairs, a batch at a time, up to a cap.
    let runs = config.simulations;
    let pair_cap = runs.saturating_mul(MAX_PAIR_FACTOR);
    let links = relevance(season, &base, config.focus)?;
    let games = tracked
        .iter()
        .zip(&tracked_games)
        .map(|(&index, game)| -> Result<GameImpact, String> {
            let (home_team, away_team) = (remaining[index].home, remaining[index].away);
            let Some(relevance) = links
                .iter()
                .find(|relevance| relevance.home == home_team && relevance.away == away_team)
                .cloned()
            else {
                return Err(format!("{} is missing from the relevance pass", game.id));
            };
            let mut pairs = [PairedCounts::default(); 5];
            let mut channels: Vec<ChannelCounts> = Vec::new();
            // A game that can't reach the focus team in any season can't
            // move anyone's seeding, so its swings are exactly zero.
            let mut target = if relevance.can_matter() { runs } else { 0 };
            let mut done = 0;
            while done < target {
                for run in done..target {
                    let seed = run_seed(run);
                    let home_played = play_season(seed, Some((index, true)));
                    let away_played = play_season(seed, Some((index, false)));
                    let home = goals(&home_played);
                    let away = goals(&away_played);
                    for ((counts, home), away) in pairs.iter_mut().zip(home).zip(away) {
                        counts.add(home, away);
                    }
                    let Some((channel, cause, home_better)) =
                        classify(&home_played, &away_played, config.focus)
                    else {
                        continue;
                    };
                    let position = channels
                        .iter()
                        .position(|counts| counts.channel == channel && counts.cause == cause);
                    let counts = match position {
                        Some(position) => &mut channels[position],
                        None => {
                            channels.push(ChannelCounts {
                                channel,
                                cause,
                                pairs: 0,
                                home_better: 0,
                                away_better: 0,
                                playoffs: PairedCounts::default(),
                                champion: PairedCounts::default(),
                            });
                            let last = channels.len() - 1;
                            &mut channels[last]
                        }
                    };
                    counts.pairs += 1;
                    if channel.is_seeding() {
                        counts.home_better += u32::from(home_better);
                        counts.away_better += u32::from(!home_better);
                    }
                    counts.playoffs.add(home[0], away[0]);
                    counts.champion.add(home[1], away[1]);
                }
                done = target;
                if near_the_line(&pairs[1].swing(done)) {
                    target = done.saturating_add(runs).min(pair_cap);
                }
            }
            let runs = done;
            let exact = |counts: PairedCounts| {
                if runs == 0 {
                    Swing::none()
                } else {
                    counts.swing(runs)
                }
            };
            let [playoffs, champion, nfc_north, division, top_seed] = pairs.map(exact);
            let mut channels: Vec<ChannelImpact> = channels
                .into_iter()
                .map(|counts| ChannelImpact {
                    channel: counts.channel,
                    cause: counts.cause,
                    pairs: counts.pairs,
                    home_better: counts.home_better,
                    away_better: counts.away_better,
                    playoffs: Contribution::from_pairs(
                        counts.playoffs.home_only,
                        counts.playoffs.away_only,
                        runs,
                    ),
                    champion: Contribution::from_pairs(
                        counts.champion.home_only,
                        counts.champion.away_only,
                        runs,
                    ),
                })
                .collect();
            channels.sort_by(|a, b| {
                b.champion
                    .swing
                    .abs()
                    .total_cmp(&a.champion.swing.abs())
                    .then(b.playoffs.swing.abs().total_cmp(&a.playoffs.swing.abs()))
                    .then(a.channel.cmp(&b.channel))
                    .then(b.pairs.cmp(&a.pairs))
            });
            Ok(GameImpact {
                game_id: game.id.clone(),
                gameday: game.gameday,
                home: remaining[index].home,
                away: remaining[index].away,
                home_win_probability: remaining[index].home_win_probability,
                playoffs,
                champion,
                nfc_north,
                division,
                top_seed,
                channels,
                pairs: runs,
                relevance,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

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

/// The standard deviation of NFL game margins around the betting line, in
/// points. With it, a line becomes a win probability.
pub const SPREAD_SIGMA: f64 = 13.45;

/// The home team's chance of winning from the betting line (nflverse
/// `spread_line`: the home team's expected margin, positive when it is
/// favored). Games with a line use it instead of Elo, since the market knows
/// about injuries, rest and form that Elo can't see.
pub fn spread_win_probability(spread: f64) -> f64 {
    normal_cdf(spread / SPREAD_SIGMA)
}

/// The standard normal CDF, from Abramowitz and Stegun 7.1.26 (error under
/// 1.5e-7).
fn normal_cdf(x: f64) -> f64 {
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * z);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let erf = 1.0 - poly * (-z * z).exp();
    if x >= 0.0 {
        0.5 * (1.0 + erf)
    } else {
        0.5 * (1.0 - erf)
    }
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
    fn channel_shares_add_up_to_the_swing() {
        // 2023 before week 18, measured for the Packers, who were in a
        // wild-card race decided by records and tiebreakers.
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(2_000, "GB"));
        let mut tiebreakers = 0;
        for game in &simulation.games {
            let sum =
                |share: fn(&ChannelImpact) -> f64| -> f64 { game.channels.iter().map(share).sum() };
            close(sum(|c| c.playoffs.swing), game.playoffs.swing());
            close(sum(|c| c.champion.swing), game.champion.swing());
            for channel in &game.channels {
                assert!(channel.pairs > 0);
                assert!(channel.pairs <= game.pairs);
                assert_eq!(channel.cause.is_some(), channel.channel.is_seeding());
                if channel.channel.is_seeding() {
                    assert_eq!(channel.home_better + channel.away_better, channel.pairs);
                } else {
                    assert_eq!(channel.home_better + channel.away_better, 0);
                }
                tiebreakers += u32::from(matches!(channel.cause, Some(Cause::Tiebreaker(_))));
            }
            // Each channel and cause appears once.
            for (i, a) in game.channels.iter().enumerate() {
                for b in &game.channels[i + 1..] {
                    assert!(a.channel != b.channel || a.cause != b.cause);
                }
            }
        }
        assert!(
            tiebreakers > 0,
            "some seedings should hinge on a tiebreaker"
        );

        // The Packers clinched by beating the Bears: a Packers loss costs
        // them a wild card in many seasons, by record, never by noise alone.
        let Some(game) = simulation.games.iter().find(|game| game.home == team("GB")) else {
            panic!("GB should host a week 18 game");
        };
        let Some(biggest) = game.channels.first() else {
            panic!("GB vs CHI should change something");
        };
        assert_eq!(biggest.channel, Channel::WildCard);
        assert_eq!(biggest.cause, Some(Cause::Record));
        assert_eq!(biggest.away_better, 0);
    }

    #[test]
    fn the_cause_is_the_first_tiebreak_that_went_differently() {
        let ties = |events: &[(Option<Step>, bool)]| {
            let mut ties = FocusTies::default();
            for &(step, survived) in events {
                ties.push(step, survived);
            }
            ties
        };
        let h2h = Some(Step::HeadToHead);
        let sov = Some(Step::StrengthOfVictory);
        // Same first tie, then strength of victory went the other way.
        let worse = ties(&[(h2h, true), (sov, false)]);
        let better = ties(&[(h2h, true), (sov, true)]);
        assert_eq!(worse.first_difference(&better), Some(sov));
        // A tie only the worse season had.
        assert_eq!(
            ties(&[(None, false)]).first_difference(&ties(&[])),
            Some(None)
        );
        // A tie only the better season had.
        assert_eq!(ties(&[]).first_difference(&ties(&[(sov, true)])), Some(sov));
        // The same ties both ways: the records decided it.
        assert_eq!(worse.first_difference(&worse), None);
        // Past the capacity, ties are dropped rather than stored.
        let many = ties(&[(h2h, true); 20]);
        assert_eq!(many.events().len(), FocusTies::CAPACITY);
    }

    #[test]
    fn a_game_that_changes_nothing_has_no_channels() {
        // The 7-9 Bears were out before week 18: no game can reach them.
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(500, "CHI"));
        for game in &simulation.games {
            assert!(
                game.channels
                    .iter()
                    .all(|c| c.playoffs.swing == 0.0 && c.champion.swing == 0.0),
                "{}: {:?}",
                game.game_id,
                game.channels
            );
        }
    }

    /// A game with every link open, for hand-built impacts.
    fn open_game(home: &str, away: &str) -> Relevance {
        Relevance {
            home: team(home),
            away: team(away),
            focus_alive: true,
            alive: [true, true],
            links: vec![crate::relevance::Link::Race(team(home))],
        }
    }

    #[test]
    fn tiers_follow_the_line_with_confidence() {
        let swing = |home_only: u32, away_only: u32, runs: u32| {
            Swing::from_pairs(home_only, away_only, home_only, away_only, runs)
        };
        // 1 point, clearly above the 0.05-point line.
        assert_eq!(swing(1_000, 0, 100_000).tier(), Tier::Matters);
        // 0.01 points, clearly below it.
        assert_eq!(swing(10, 0, 100_000).tier(), Tier::Negligible);
        assert_eq!(swing(0, 0, 100_000).tier(), Tier::Negligible);
        // 0.05 points from 100,000 pairs: too close to call.
        assert_eq!(swing(50, 0, 100_000).tier(), Tier::Small);
        // Too few pairs to say anything.
        assert_eq!(swing(1, 0, 100).tier(), Tier::Small);
        // The same swing from 10,000 pairs is near the line and gets more;
        // from 100,000 it is already as precise as the plan asks.
        assert!(near_the_line(&swing(5, 0, 10_000)));
        assert!(!near_the_line(&swing(50, 0, 100_000)));
        assert!(!near_the_line(&swing(1_000, 0, 100_000)));
        assert!(!near_the_line(&swing(10, 0, 100_000)));
        // A game that can't reach the focus team is in no tier but None,
        // whatever its swing says.
        let mut game = GameImpact {
            game_id: "x".to_owned(),
            gameday: Date {
                year: 2026,
                month: 11,
                day: 26,
            },
            home: team("DET"),
            away: team("GB"),
            home_win_probability: 0.5,
            playoffs: swing(1_000, 0, 100_000),
            champion: swing(1_000, 0, 100_000),
            nfc_north: Swing::none(),
            division: Swing::none(),
            top_seed: Swing::none(),
            channels: Vec::new(),
            pairs: 100_000,
            relevance: open_game("DET", "GB"),
        };
        assert_eq!(game.tier(&game.champion), Tier::Matters);
        game.relevance.links.clear();
        assert_eq!(game.tier(&game.champion), Tier::None);
    }

    #[test]
    fn close_calls_get_more_pairs_and_closed_games_none() {
        let schedule = schedule(2023, before_week_18);
        let simulation = run(&schedule, 2023, &config(2_000, "GB"));
        let cap = 2_000 * MAX_PAIR_FACTOR;
        for game in &simulation.games {
            assert!(game.relevance.can_matter());
            assert!((2_000..=cap).contains(&game.pairs), "{}", game.pairs);
            assert!(game.pairs % 2_000 == 0 || game.pairs == cap);
            // A game that got extra pairs was near the line before them.
            if game.pairs < cap {
                assert!(!near_the_line(&game.champion), "{}", game.game_id);
            }
        }
        assert!(
            simulation.games.iter().any(|game| game.pairs > 2_000),
            "some game should be close enough to the line for more pairs"
        );

        // The 2-14 Panthers were out: no game can reach them, so none is
        // simulated and every swing is exactly zero.
        let simulation = run(&schedule, 2023, &config(500, "CAR"));
        for game in &simulation.games {
            assert_eq!(game.pairs, 0);
            assert_eq!(game.champion, Swing::none());
            assert_eq!(game.playoffs, Swing::none());
            assert!(game.channels.is_empty());
            assert_eq!(game.tier(&game.champion), Tier::None);
        }
    }

    #[test]
    fn swing_noise_comes_from_the_pairs() {
        // Every pair agrees: no swing and no noise.
        let none = Swing::from_pairs(50, 50, 0, 0, 100);
        close(none.swing(), 0.0);
        close(none.std_error, 0.0);
        assert!(!none.matters());

        // One pair in a hundred: a small swing, but clearly not noise once
        // there are enough pairs.
        let small = Swing::from_pairs(5_100, 5_000, 100, 0, 10_000);
        close(small.swing(), 0.01);
        assert!(small.matters());

        // Disagreements both ways that cancel out.
        let cancels = Swing::from_pairs(50, 50, 10, 10, 100);
        close(cancels.swing(), 0.0);
        assert!(!cancels.matters());

        let empty = Swing::from_pairs(0, 0, 0, 0, 0);
        assert!(empty.std_error.is_infinite());
        assert!(!empty.matters());
    }

    #[test]
    fn betting_lines_become_win_probabilities() {
        close(spread_win_probability(0.0), 0.5);
        // A 3-point favorite wins about 59% of the time.
        assert!((spread_win_probability(3.0) - 0.588).abs() < 0.002);
        close(
            spread_win_probability(-7.0) + spread_win_probability(7.0),
            1.0,
        );
        assert!(spread_win_probability(14.5) > 0.85);
    }

    #[test]
    fn root_for_follows_the_sign() {
        let game = GameImpact {
            game_id: "x".to_owned(),
            gameday: Date {
                year: 2026,
                month: 11,
                day: 26,
            },
            home: team("DET"),
            away: team("GB"),
            home_win_probability: 0.5,
            playoffs: Swing::from_pairs(10, 90, 0, 80, 100),
            champion: Swing::from_pairs(90, 10, 80, 0, 100),
            nfc_north: Swing::from_pairs(1, 1, 0, 0, 100),
            division: Swing::from_pairs(1, 1, 0, 0, 100),
            top_seed: Swing::from_pairs(1, 1, 0, 0, 100),
            channels: Vec::new(),
            pairs: 100,
            relevance: open_game("DET", "GB"),
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
