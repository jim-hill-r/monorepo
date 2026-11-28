//! The celebration banner at the top of the page. The nightly build works it
//! out from the current results, so the page shows only what this file
//! wrote: a banner right after a win by the guide's team (the Bears, or the
//! team a fan picked), and nothing otherwise.
//!
//! The headline is the score of that win. The lines under it are the most
//! notable facts of the moment, picked from a fixed list in priority order
//! (the playoff round they reached, the division race, how much their odds
//! moved, a winning streak and their record), at most [`MAX_LINES`] of them.

use bear_down_core::{Conference, Game, GameType, Season, Standings, Team, TeamOdds};
use serde::Serialize;

use crate::why::nickname;

/// How many context lines the banner shows under its headline.
pub const MAX_LINES: usize = 2;

/// The smallest move in Super Bowl odds worth a line, in probability (one
/// point). Smaller moves are within a few standard errors of the noise.
const SUPER_BOWL_MOVE: f64 = 0.01;

/// The smallest move in playoff odds worth a line, in probability.
const PLAYOFF_MOVE: f64 = 0.02;

/// A winning streak shorter than this isn't worth a line.
const STREAK: usize = 3;

/// The banner: a headline and the lines under it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Banner {
    pub headline: String,
    pub lines: Vec<String>,
}

/// What the banner is worked out from.
pub struct Context<'a> {
    /// The season so far.
    pub season: &'a Season<'a>,
    /// The season's standings now.
    pub standings: &'a Standings,
    /// The team the page is for (the Bears, or the team a fan picked).
    pub focus: Team,
    /// Its odds now.
    pub odds: TeamOdds,
    /// Its odds just before the week of its latest game, if they could be
    /// simulated (regular season only).
    pub odds_before: Option<TeamOdds>,
}

/// The focus team's most recent game with a final score, if any.
pub fn latest_game<'a>(season: &Season<'a>, focus: Team) -> Option<&'a Game> {
    season
        .played()
        .filter(|game| game.involves(focus.abbr()))
        .max_by_key(|game| (game.gameday, game.week))
}

/// The banner, or `None` unless the focus team's most recent game was a win.
pub fn banner(context: &Context<'_>) -> Option<Banner> {
    let us = context.focus.abbr();
    let game = latest_game(context.season, context.focus)?;
    if game.winner() != Some(us) {
        return None;
    }
    let score = game.score?;
    let (ours, theirs) = if game.home_team == us {
        (score.home, score.away)
    } else {
        (score.away, score.home)
    };
    let opponent = game
        .opponent(us)
        .and_then(Team::from_abbr)
        .map_or("opponent", nickname);
    let city = city(context.focus);
    let headline = if game.game_type == GameType::SuperBowl {
        format!("Super Bowl champions! {city} {ours}, {opponent} {theirs}.")
    } else {
        format!(
            "{} win! {city} {ours}, {opponent} {theirs}.",
            nickname(context.focus)
        )
    };
    let lines = [
        playoff_line(game.game_type, context.focus.conference()),
        division_line(context, game),
        odds_line(context, game),
        streak_line(context),
        record_line(context, game),
    ]
    .into_iter()
    .flatten()
    .take(MAX_LINES)
    .collect();
    Some(Banner { headline, lines })
}

/// Where a playoff win in `conference` sends them.
fn playoff_line(game_type: GameType, conference: Conference) -> Option<String> {
    let next = match game_type {
        GameType::Regular | GameType::SuperBowl => return None,
        GameType::WildCard => "the divisional round".to_string(),
        GameType::Divisional => format!("the {conference} Championship"),
        GameType::Conference => "the Super Bowl".to_string(),
    };
    Some(format!("On to {next}."))
}

/// Their place in the division, when they lead it or share the lead.
fn division_line(context: &Context<'_>, game: &Game) -> Option<String> {
    if game.game_type.is_playoff() {
        return None;
    }
    let focus = context.focus;
    let name = focus.division();
    let division = name.teams();
    let margin = |team: Team| {
        let record = context.standings.record(team);
        f64::from(record.wins) - f64::from(record.losses)
    };
    let left = |team: Team| {
        context
            .season
            .regular_season()
            .filter(|game| !game.is_played() && game.involves(team.abbr()))
            .count() as f64
    };
    let rivals: Vec<Team> = division.into_iter().filter(|&t| t != focus).collect();
    // Clinched when even losing out leaves them ahead of every rival that
    // wins out, by record alone (no tiebreakers needed).
    if rivals
        .iter()
        .all(|&rival| margin(focus) - 2.0 * left(focus) > margin(rival) + 2.0 * left(rival))
    {
        return Some(format!("{name} champions."));
    }
    let rival = rivals
        .into_iter()
        .max_by(|&a, &b| margin(a).total_cmp(&margin(b)))?;
    let lead = (margin(focus) - margin(rival)) / 2.0;
    if lead < 0.0 {
        None
    } else if lead == 0.0 {
        Some(format!("Tied for first place in the {name}."))
    } else {
        let games = if lead == 1.0 {
            "a game".to_string()
        } else {
            format!("{lead} games")
        };
        Some(format!(
            "Alone in first place in the {name}, {games} ahead of the {}.",
            nickname(rival)
        ))
    }
}

/// How far the week moved their odds, when it moved them clearly.
fn odds_line(context: &Context<'_>, game: &Game) -> Option<String> {
    if game.game_type.is_playoff() {
        return None;
    }
    let before = context.odds_before?;
    let now = context.odds;
    let moved = |label: &str, before: f64, now: f64, line: f64| {
        let change = now - before;
        (change.abs() >= line).then(|| {
            let way = if change > 0.0 { "up" } else { "down" };
            format!(
                "{label} odds {way} from {} to {}.",
                percent(before),
                percent(now)
            )
        })
    };
    moved("Super Bowl", before.champion, now.champion, SUPER_BOWL_MOVE)
        .or_else(|| moved("Playoff", before.playoffs, now.playoffs, PLAYOFF_MOVE))
}

/// Their winning streak, once it is long enough to mention.
fn streak_line(context: &Context<'_>) -> Option<String> {
    let us = context.focus.abbr();
    let mut games: Vec<&Game> = context
        .season
        .played()
        .filter(|game| game.involves(us))
        .collect();
    games.sort_by_key(|game| (game.gameday, game.week));
    let streak = games
        .iter()
        .rev()
        .take_while(|game| game.winner() == Some(us))
        .count();
    (streak >= STREAK).then(|| format!("{} straight wins.", capitalized_count(streak)))
}

/// Their record, as a regular-season fallback.
fn record_line(context: &Context<'_>, game: &Game) -> Option<String> {
    if game.game_type.is_playoff() {
        return None;
    }
    let record = context.standings.record(context.focus);
    Some(if record.ties == 0 {
        format!("Now {}-{}.", record.wins, record.losses)
    } else {
        format!("Now {}-{}-{}.", record.wins, record.losses, record.ties)
    })
}

/// Where the team plays, as a scoreline names it: "Chicago", "Green Bay".
fn city(team: Team) -> &'static str {
    match team.abbr() {
        "ARI" => "Arizona",
        "ATL" => "Atlanta",
        "BAL" => "Baltimore",
        "BUF" => "Buffalo",
        "CAR" => "Carolina",
        "CHI" => "Chicago",
        "CIN" => "Cincinnati",
        "CLE" => "Cleveland",
        "DAL" => "Dallas",
        "DEN" => "Denver",
        "DET" => "Detroit",
        "GB" => "Green Bay",
        "HOU" => "Houston",
        "IND" => "Indianapolis",
        "JAX" => "Jacksonville",
        "KC" => "Kansas City",
        "LA" | "LAR" | "LAC" => "Los Angeles",
        "LV" => "Las Vegas",
        "MIA" => "Miami",
        "MIN" => "Minnesota",
        "NE" => "New England",
        "NO" => "New Orleans",
        "NYG" | "NYJ" => "New York",
        "PHI" => "Philadelphia",
        "PIT" => "Pittsburgh",
        "SEA" => "Seattle",
        "SF" => "San Francisco",
        "TB" => "Tampa Bay",
        "TEN" => "Tennessee",
        "WAS" => "Washington",
        _ => nickname(team),
    }
}

/// "Three", "Four", ... up to "Ten", then digits.
fn capitalized_count(count: usize) -> String {
    const WORDS: [&str; 11] = [
        "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
    ];
    WORDS
        .get(count)
        .map_or_else(|| count.to_string(), |word| (*word).to_string())
}

/// A probability as a percentage, as the page shows the odds: whole points
/// from 10%, one decimal below.
fn percent(share: f64) -> String {
    let pct = share * 100.0;
    if pct >= 10.0 {
        format!("{pct:.0}%")
    } else {
        format!("{pct:.1}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bear_down_core::Schedule;

    /// A 2026 season of `rows`: `(week, game type, away, home, away score,
    /// home score)`, with empty scores for a game not yet played.
    fn schedule(rows: &[(u8, &str, &str, &str, &str, &str)]) -> Schedule {
        let mut csv = String::from(
            "game_id,season,game_type,week,gameday,away_team,away_score,home_team,home_score,location,result,spread_line,div_game\n",
        );
        for (week, game_type, away, home, away_score, home_score) in rows {
            let id = format!("2026_{week:02}_{away}_{home}");
            let day = 1 + u32::from(*week);
            csv.push_str(&format!(
                "{id},2026,{game_type},{week},2026-10-{day:02},{away},{away_score},{home},{home_score},Home,,,0\n"
            ));
        }
        match Schedule::parse(&csv) {
            Ok(schedule) => schedule,
            Err(error) => panic!("test schedule should parse: {error}"),
        }
    }

    fn odds(champion: f64, playoffs: f64) -> TeamOdds {
        TeamOdds {
            champion,
            playoffs,
            ..TeamOdds::default()
        }
    }

    fn banner_for(
        rows: &[(u8, &str, &str, &str, &str, &str)],
        before: Option<TeamOdds>,
    ) -> Option<Banner> {
        banner_for_team(rows, before, Team::BEARS)
    }

    fn banner_for_team(
        rows: &[(u8, &str, &str, &str, &str, &str)],
        before: Option<TeamOdds>,
        focus: Team,
    ) -> Option<Banner> {
        let schedule = schedule(rows);
        let Some(season) = schedule.latest_season() else {
            panic!("test schedule has a season");
        };
        let standings = match Standings::from_season(&season) {
            Ok(standings) => standings,
            Err(error) => panic!("standings: {error}"),
        };
        banner(&Context {
            season: &season,
            standings: &standings,
            focus,
            odds: odds(0.11, 0.97),
            odds_before: before,
        })
    }

    #[test]
    fn no_banner_after_a_loss_a_tie_or_before_any_game() {
        assert_eq!(banner_for(&[(1, "REG", "CHI", "DET", "", "")], None), None);
        assert_eq!(
            banner_for(&[(1, "REG", "CHI", "DET", "20", "27")], None),
            None
        );
        assert_eq!(
            banner_for(&[(1, "REG", "CHI", "DET", "20", "20")], None),
            None
        );
        // A win, then a loss: the latest game decides.
        assert_eq!(
            banner_for(
                &[
                    (1, "REG", "CHI", "DET", "27", "20"),
                    (2, "REG", "GB", "CHI", "30", "10")
                ],
                None
            ),
            None
        );
    }

    #[test]
    fn a_win_leads_with_the_score_and_the_most_notable_context() {
        let rows = [
            (1, "REG", "GB", "MIN", "17", "24"),
            (2, "REG", "CHI", "DET", "39", "35"),
            (3, "REG", "DET", "CHI", "", ""),
        ];
        assert_eq!(
            banner_for(&rows, Some(odds(0.08, 0.95))),
            Some(Banner {
                headline: "Bears win! Chicago 39, Lions 35.".to_string(),
                lines: vec![
                    "Tied for first place in the NFC North.".to_string(),
                    "Super Bowl odds up from 8.0% to 11%.".to_string(),
                ],
            })
        );
        // A small move gives way to the record.
        assert_eq!(
            banner_for(&rows, Some(odds(0.105, 0.96))).map(|b| b.lines[1].clone()),
            Some("Now 1-0.".to_string())
        );
    }

    #[test]
    fn a_long_winning_streak_and_a_clinched_division_are_mentioned() {
        let rows = [
            (1, "REG", "CHI", "DET", "21", "14"),
            (2, "REG", "GB", "CHI", "3", "10"),
            (3, "REG", "CHI", "MIN", "28", "7"),
        ];
        assert_eq!(
            banner_for(&rows, None).map(|b| b.lines),
            Some(vec![
                "NFC North champions.".to_string(),
                "Three straight wins.".to_string(),
            ])
        );
    }

    #[test]
    fn a_playoff_win_says_where_they_go_next() {
        let rows = [
            (1, "REG", "CHI", "DET", "21", "14"),
            (19, "WC", "GB", "CHI", "17", "24"),
        ];
        assert_eq!(
            banner_for(&rows, None),
            Some(Banner {
                headline: "Bears win! Chicago 24, Packers 17.".to_string(),
                lines: vec!["On to the divisional round.".to_string()],
            })
        );
        let rows = [(22, "SB", "CHI", "BUF", "31", "20")];
        assert_eq!(
            banner_for(&rows, None).map(|b| b.headline),
            Some("Super Bowl champions! Chicago 31, Bills 20.".to_string())
        );
    }

    #[test]
    fn any_team_gets_its_own_banner() {
        let Some(chiefs) = Team::from_abbr("KC") else {
            panic!("KC is a team");
        };
        let rows = [
            (1, "REG", "KC", "LV", "27", "20"),
            (2, "REG", "DEN", "LAC", "10", "13"),
            (3, "REG", "LV", "KC", "", ""),
        ];
        assert_eq!(
            banner_for_team(&rows, None, chiefs),
            Some(Banner {
                headline: "Chiefs win! Kansas City 27, Raiders 20.".to_string(),
                lines: vec![
                    "Tied for first place in the AFC West.".to_string(),
                    "Now 1-0.".to_string(),
                ],
            })
        );
        assert_eq!(banner_for_team(&rows, None, Team::BEARS), None);
        let rows = [(20, "DIV", "BUF", "KC", "20", "23")];
        assert_eq!(
            banner_for_team(&rows, None, chiefs).map(|b| b.lines),
            Some(vec!["On to the AFC Championship.".to_string()])
        );
    }

    #[test]
    fn every_team_has_a_city() {
        for team in Team::all() {
            assert_ne!(city(team), nickname(team), "{team}");
        }
    }
}
