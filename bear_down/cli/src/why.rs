//! Plain-language reasons why a result helps the Bears: the rivalry, the
//! tiebreakers it touches (head-to-head, strength of victory and schedule)
//! and how it moves the Bears' division and seeding odds. Jim asked for this
//! on 2026-11-26 so each pick says why, not just by how much.

use bear_down_core::{Conference, Division, Game, GameImpact, Record, Standings, Swing, Team};

/// Which goal a set of reasons explains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// The Bears winning the Super Bowl.
    Champion,
    /// The Bears making the playoffs.
    Playoffs,
    /// All four NFC North teams reaching the divisional round.
    NfcNorth,
}

/// The smallest change in odds worth putting in words: 0.1 points.
const NOTABLE: f64 = 0.001;

/// Playoff odds below which a team isn't called a contender.
const CONTENDER: f64 = 0.05;

/// Why rooting for `swing`'s team helps `goal`, or nothing when neither
/// result matters. `games` is the season's regular-season schedule, played
/// and not.
pub fn reasons(
    impact: &GameImpact,
    goal: Goal,
    swing: &Swing,
    standings: &Standings,
    games: &[Game],
    playoff_odds: &dyn Fn(Team) -> f64,
) -> Vec<String> {
    let Some(root) = impact.root_for(swing) else {
        return Vec::new();
    };
    let other = if root == impact.home {
        impact.away
    } else {
        impact.home
    };
    let bears = Team::BEARS;
    let record = |team: Team| format_record(standings.record(team));
    let mut why = Vec::new();

    if goal == Goal::NfcNorth {
        if root.division() == Division::NFC_NORTH {
            why.push(format!(
                "The {} ({}) are one of the four NFC North teams that all need to reach the divisional round.",
                nickname(root),
                record(root)
            ));
        } else if other.conference() == Conference::Nfc && playoff_odds(other) >= CONTENDER {
            why.push(format!(
                "The {} ({}) compete with the NFC North teams for NFC playoff spots and seeding.",
                nickname(other),
                record(other)
            ));
        }
        return why;
    }

    if root == bears || other == bears {
        let opponent = if root == bears { other } else { root };
        let mut line = format!("The Bears' own game: a win over the {}", nickname(opponent));
        if opponent.division() == Division::NFC_NORTH {
            line.push_str(", a division rival, counts twice in the NFC North race");
        } else if opponent.conference() == Conference::Nfc {
            line.push_str(" also counts in the NFC conference-record tiebreaker");
        }
        line.push('.');
        why.push(line);
    } else if other.division() == Division::NFC_NORTH {
        why.push(format!(
            "The {} ({}) are an NFC North rival of the Bears ({}): every loss helps the Bears win the division.",
            nickname(other),
            record(other),
            record(bears)
        ));
    } else if other.conference() == Conference::Nfc && playoff_odds(other) >= CONTENDER {
        why.push(format!(
            "The {} ({}) compete with the Bears ({}) for NFC playoff spots and seeding.",
            nickname(other),
            record(other),
            record(bears)
        ));
    } else if root.conference() == Conference::Afc {
        why.push(format!(
            "An AFC game, so it only matters through the Bears' likely Super Bowl opponent: the AFC field looks easier with a {} win.",
            nickname(root)
        ));
    }

    if other != bears && root != bears {
        if other.conference() == Conference::Nfc {
            why.extend(head_to_head(other, games));
        }
        why.extend(strength(root, games));
    }

    let gain = |swing: &Swing| {
        if root == impact.home {
            swing.swing()
        } else {
            -swing.swing()
        }
    };
    let mut parts = Vec::new();
    let division = gain(&impact.division);
    if division >= NOTABLE {
        parts.push(format!("winning the NFC North by {}", points(division)));
    }
    let top_seed = gain(&impact.top_seed);
    if top_seed >= NOTABLE {
        parts.push(format!(
            "the No. 1 seed (the only first-round bye) by {}",
            points(top_seed)
        ));
    }
    let playoffs = gain(&impact.playoffs);
    if goal == Goal::Champion && playoffs >= NOTABLE {
        parts.push(format!("making the playoffs by {}", points(playoffs)));
    }
    if !parts.is_empty() {
        let winner = if root == bears {
            "A win".to_string()
        } else {
            format!("A {} win", nickname(root))
        };
        why.push(format!(
            "{winner} raises the Bears' chance of {}.",
            join(&parts)
        ));
    } else if goal == Goal::Champion && root != bears && other != bears {
        why.push(
            "It barely changes the Bears' seeding: the gain comes from who they would likely face in the playoffs."
                .to_string(),
        );
    }
    why
}

/// The Bears' head-to-head with `team` this season, played or to come.
fn head_to_head(team: Team, games: &[Game]) -> Vec<String> {
    let mut why = Vec::new();
    for game in bears_games_with(team, games) {
        match game.winner() {
            Some(winner) if winner == Team::BEARS.abbr() => why.push(format!(
                "The Bears beat the {} in week {}, so they would win a head-to-head tiebreaker with them.",
                nickname(team),
                game.week
            )),
            Some(_) => why.push(format!(
                "The {} beat the Bears in week {}, so the Bears would lose a head-to-head tiebreaker: better that the {} drop games.",
                nickname(team),
                game.week,
                nickname(team)
            )),
            None => why.push(format!(
                "The Bears still play the {} in week {}.",
                nickname(team),
                game.week
            )),
        }
    }
    why
}

/// How a win by `team` feeds the Bears' strength of victory or schedule.
fn strength(team: Team, games: &[Game]) -> Option<String> {
    let mut played = false;
    for game in bears_games_with(team, games) {
        played = true;
        if game.winner() == Some(Team::BEARS.abbr()) {
            return Some(format!(
                "The Bears beat the {} in week {}, so each {} win raises the Bears' strength of victory, a tiebreaker.",
                nickname(team),
                game.week,
                nickname(team)
            ));
        }
    }
    played.then(|| {
        format!(
            "The {} are on the Bears' schedule, so their wins add to the Bears' strength of schedule, a late tiebreaker.",
            nickname(team)
        )
    })
}

fn bears_games_with(team: Team, games: &[Game]) -> impl Iterator<Item = &Game> {
    games
        .iter()
        .filter(move |game| game.involves(Team::BEARS.abbr()) && game.involves(team.abbr()))
}

fn format_record(record: Record) -> String {
    if record.ties > 0 {
        format!("{}-{}-{}", record.wins, record.losses, record.ties)
    } else {
        format!("{}-{}", record.wins, record.losses)
    }
}

fn points(share: f64) -> String {
    format!("{:.1} points", share * 100.0)
}

fn join(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and of {second}"),
        [rest @ .., last] => {
            let head: Vec<String> = rest.to_vec();
            format!("{}, and of {last}", head.join(", of "))
        }
    }
}

/// The team's nickname, as in "Packers".
pub fn nickname(team: Team) -> &'static str {
    match team.abbr() {
        "ARI" => "Cardinals",
        "ATL" => "Falcons",
        "BAL" => "Ravens",
        "BUF" => "Bills",
        "CAR" => "Panthers",
        "CHI" => "Bears",
        "CIN" => "Bengals",
        "CLE" => "Browns",
        "DAL" => "Cowboys",
        "DEN" => "Broncos",
        "DET" => "Lions",
        "GB" => "Packers",
        "HOU" => "Texans",
        "IND" => "Colts",
        "JAX" => "Jaguars",
        "KC" => "Chiefs",
        "LA" | "LAR" => "Rams",
        "LAC" => "Chargers",
        "LV" => "Raiders",
        "MIA" => "Dolphins",
        "MIN" => "Vikings",
        "NE" => "Patriots",
        "NO" => "Saints",
        "NYG" => "Giants",
        "NYJ" => "Jets",
        "PHI" => "Eagles",
        "PIT" => "Steelers",
        "SEA" => "Seahawks",
        "SF" => "49ers",
        "TB" => "Buccaneers",
        "TEN" => "Titans",
        "WAS" => "Commanders",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_parts_in_plain_english() {
        let parts = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(join(&parts(&["a"])), "a");
        assert_eq!(join(&parts(&["a", "b"])), "a and of b");
        assert_eq!(join(&parts(&["a", "b", "c"])), "a, of b, and of c");
    }

    #[test]
    fn formats_records_and_points() {
        let record = Record {
            wins: 8,
            losses: 2,
            ties: 0,
        };
        assert_eq!(format_record(record), "8-2");
        assert_eq!(format_record(Record { ties: 1, ..record }), "8-2-1");
        assert_eq!(points(0.0123), "1.2 points");
    }

    #[test]
    fn every_team_has_a_nickname() {
        for team in Team::all() {
            assert_ne!(nickname(team), team.abbr(), "{team}");
        }
    }
}
