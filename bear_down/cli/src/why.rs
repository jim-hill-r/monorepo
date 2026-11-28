//! Plain-language explanations of how a game reaches the focus team (the
//! Bears by default, or any team a fan picks), written from what the
//! simulation measured (step 5 of
//! `docs/plans/2026-11-robust-recommendations.md`): each channel's share of
//! the swing (division title, wild card, seeding by record or by a named
//! tiebreaker step, the conference's playoff field, the Super Bowl opponent), which
//! team it favors and how big it is, the net when they disagree, and for a
//! "doesn't matter" game why it doesn't (BEAR-003 to BEAR-006).

use bear_down_core::{
    Cause, Channel, ChannelImpact, Conference, Division, GameImpact, Link, Record, Standings,
    Swing, Team, Tier,
};

/// Which goal a set of reasons explains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// The focus team winning the Super Bowl.
    Champion,
    /// The focus team making the playoffs.
    Playoffs,
    /// All four NFC North teams reaching the divisional round.
    NfcNorth,
}

impl Goal {
    /// The unit a swing on this goal is told in, as in "1.2 Super Bowl
    /// points".
    fn unit(self) -> &'static str {
        match self {
            Goal::Champion => "Super Bowl",
            Goal::Playoffs => "playoff",
            Goal::NfcNorth => "NFC North",
        }
    }
}

/// Playoff odds below which a team isn't called a contender.
const CONTENDER: f64 = 0.05;

/// The most channel lines an explanation lists.
const MAX_CHANNELS: usize = 4;

/// The verdict for one goal: the team to root for (`None` for "doesn't
/// matter") and the tier. A pick needs a clear direction and a swing that
/// isn't confidently under the line (BEAR-005); the NFC North goal has no
/// line, so any clear direction is a pick there.
pub fn verdict(impact: &GameImpact, goal: Goal, swing: &Swing) -> (Option<Team>, Tier) {
    if !impact.relevance.can_matter() {
        return (None, Tier::None);
    }
    let root = impact.root_for(swing);
    if goal == Goal::NfcNorth {
        let tier = if root.is_some() {
            Tier::Matters
        } else {
            Tier::Negligible
        };
        return (root, tier);
    }
    let tier = impact.tier(swing);
    match tier {
        Tier::Matters | Tier::Small => (root, tier),
        Tier::Negligible | Tier::None => (None, tier),
    }
}

/// The channels of `goal`'s swing worth showing: clearly bigger than their
/// noise, without the leftover simulation noise ([`Channel::Chance`]),
/// biggest first.
pub fn shown_channels(impact: &GameImpact, goal: Goal) -> Vec<&ChannelImpact> {
    let mut shown: Vec<&ChannelImpact> = impact
        .channels
        .iter()
        .filter(|channel| channel.channel != Channel::Chance)
        .filter(|channel| share(channel, goal).is_some_and(|(swing, se)| swing.abs() > 2.0 * se))
        .collect();
    shown.sort_by(|a, b| {
        let size =
            |channel: &ChannelImpact| share(channel, goal).map_or(0.0, |(swing, _)| swing.abs());
        size(b).total_cmp(&size(a))
    });
    shown
}

/// A channel's share of `goal`'s swing and its standard error.
pub fn share(channel: &ChannelImpact, goal: Goal) -> Option<(f64, f64)> {
    match goal {
        Goal::Champion => Some((channel.champion.swing, channel.champion.std_error)),
        Goal::Playoffs => Some((channel.playoffs.swing, channel.playoffs.std_error)),
        Goal::NfcNorth => None,
    }
}

/// The plain-language explanation of `goal`'s verdict for one game, for a
/// fan of `focus`. Never empty: every game says how it reaches the focus
/// team, or why it can't.
pub fn reasons(
    focus: Team,
    impact: &GameImpact,
    goal: Goal,
    swing: &Swing,
    standings: &Standings,
    playoff_odds: &dyn Fn(Team) -> f64,
) -> Vec<String> {
    let (root, tier) = verdict(impact, goal, swing);
    let record = |team: Team| format_record(standings.record(team));
    let (away, home) = (impact.away, impact.home);

    if tier == Tier::None {
        return vec![closed(focus, impact, standings)];
    }

    if goal == Goal::NfcNorth {
        return nfc_north(impact, root, standings, playoff_odds);
    }

    let mut why = Vec::new();
    match root {
        Some(root) => {
            if let Some(line) = rivalry(focus, impact, root, standings, playoff_odds) {
                why.push(line);
            }
            if tier == Tier::Small {
                why.push(format!(
                    "A small edge: {} is better by {}, close to the 0.05-point line.",
                    a_win(root),
                    give_or_take(swing, goal)
                ));
            }
        }
        None if tier == Tier::Small => why.push(format!(
            "Too close to call: {} at {} moves {} {} odds by {}, near the 0.05-point line.",
            nickname(away),
            nickname(home),
            possessive(focus),
            goal.unit(),
            give_or_take(swing, goal)
        )),
        None => {
            let lean = if swing.swing() == 0.0 && swing.std_error == 0.0 {
                format!(
                    "no simulated season came out differently for {}",
                    the(focus)
                )
            } else if swing.matters() {
                let better = if swing.swing() > 0.0 { home } else { away };
                format!(
                    "{} is better by just {}",
                    a_win(better),
                    points(swing.swing().abs(), goal)
                )
            } else {
                format!(
                    "the difference, {}, is within the simulation's noise",
                    give_or_take(swing, goal)
                )
            };
            why.push(format!(
                "Doesn't matter: {} at {} moves {} {} odds by under 0.05 points ({lean}).",
                nickname(away),
                nickname(home),
                possessive(focus),
                goal.unit()
            ));
        }
    }

    let shown = shown_channels(impact, goal);
    for channel in shown.iter().take(MAX_CHANNELS) {
        why.push(channel_line(focus, impact, channel, goal));
    }
    let favors =
        |channel: &&ChannelImpact| share(channel, goal).is_some_and(|(swing, _)| swing > 0.0);
    if shown.iter().any(favors) && !shown.iter().all(favors) {
        let net = if swing.matters() {
            let winner = if swing.swing() > 0.0 { home } else { away };
            format!(
                "Net: these pull opposite ways, and {} comes out ahead by {}.",
                a_win(winner),
                points(swing.swing().abs(), goal)
            )
        } else {
            "Net: these pull opposite ways and about cancel out.".to_string()
        };
        why.push(net);
    }
    if shown.is_empty() {
        why.push(links(focus, impact, &record));
    }
    why
}

/// One channel's line, as in "Seeding, decided by strength of victory in
/// 1.2% of seasons: a Saints win helps the Bears by 0.010 Super Bowl
/// points."
fn channel_line(focus: Team, impact: &GameImpact, channel: &ChannelImpact, goal: Goal) -> String {
    let (swing, _) = share(channel, goal).unwrap_or_default();
    let favored = if swing > 0.0 {
        impact.home
    } else {
        impact.away
    };
    let (ours, theirs) = (focus.conference(), other_conference(focus.conference()));
    let label = match channel.channel {
        Channel::DivisionTitle => format!("{} title", focus.division()),
        Channel::WildCard => "Wild card".to_string(),
        Channel::Seed => "Seeding".to_string(),
        Channel::PlayoffPath => format!("{ours} playoff field (who else gets in, and where)"),
        Channel::SuperBowlOpponent => format!("Super Bowl opponent (who wins the {theirs})"),
        Channel::Chance => "Chance".to_string(),
    };
    let seasons = if impact.pairs == 0 {
        0.0
    } else {
        f64::from(channel.pairs) / f64::from(impact.pairs)
    };
    let cause = match channel.cause {
        None => String::new(),
        Some(Cause::Record) => ", on record".to_string(),
        Some(Cause::Tiebreaker(step)) => {
            format!(", decided by {step} in {} of seasons", percent(seasons))
        }
        Some(Cause::CoinFlip) => format!(", by coin flip in {} of seasons", percent(seasons)),
    };
    let winner = a_win(favored);
    format!(
        "{label}{cause}: {winner} helps {} by {}.",
        the(focus),
        points(swing.abs(), goal)
    )
}

/// Why a game with every link closed can't matter.
fn closed(focus: Team, impact: &GameImpact, standings: &Standings) -> String {
    if !impact.relevance.focus_alive {
        return format!(
            "{} can no longer make the playoffs, so no result this week can change their odds.",
            capitalize(&the(focus))
        );
    }
    let record = |team: Team| format_record(standings.record(team));
    format!(
        "Can't matter: the {} ({}) and the {} ({}) are both out of the playoff race, and neither plays a team still in it, so no result changes anything for {}.",
        nickname(impact.away),
        record(impact.away),
        nickname(impact.home),
        record(impact.home),
        the(focus)
    )
}

/// The open links in words, for a game none of whose channels showed up in
/// the simulation: each team's place in the race, and whether it is on the
/// focus team's schedule.
fn links(focus: Team, impact: &GameImpact, record: &dyn Fn(Team) -> String) -> String {
    let mut parts = Vec::new();
    if impact.relevance.links.contains(&Link::FocusPlays) {
        parts.push(format!("{} play in it", the(focus)));
    }
    let race = format!("are still in the {} race", focus.conference());
    for team in [impact.away, impact.home] {
        if team == focus {
            continue;
        }
        let has = |wanted: &dyn Fn(&Link) -> bool| impact.relevance.links.iter().any(wanted);
        let status = if has(&|link| *link == Link::Race(team)) {
            race.as_str()
        } else if has(&|link| *link == Link::SuperBowlRace(team)) {
            "could still be the Super Bowl opponent"
        } else {
            "are out of the playoff race"
        };
        let schedule = impact.relevance.links.iter().find_map(|link| match link {
            Link::Schedule {
                team: linked,
                contenders,
            } if *linked == team => Some(contenders),
            _ => None,
        });
        let tail = match schedule {
            Some(contenders) if contenders.contains(&focus) => {
                format!(
                    " and are on {} schedule (strength of victory and schedule)",
                    possessive(focus)
                )
            }
            Some(contenders)
                if !has(&|link| {
                    *link == Link::Race(team) || *link == Link::SuperBowlRace(team)
                }) =>
            {
                format!(
                    " but play {} contenders (their strength of schedule)",
                    contenders.len()
                )
            }
            _ => String::new(),
        };
        parts.push(format!(
            "the {} ({}) {status}{tail}",
            nickname(team),
            record(team)
        ));
    }
    format!(
        "How it could reach {}, though too slightly to show up in the simulation: {}.",
        the(focus),
        list(&parts)
    )
}

/// The lead line for a pick: why the team rooted against matters.
fn rivalry(
    focus: Team,
    impact: &GameImpact,
    root: Team,
    standings: &Standings,
    playoff_odds: &dyn Fn(Team) -> f64,
) -> Option<String> {
    let (division, conference) = (focus.division(), focus.conference());
    let other = if root == impact.home {
        impact.away
    } else {
        impact.home
    };
    let record = |team: Team| format_record(standings.record(team));
    if root == focus || other == focus {
        let opponent = if root == focus { other } else { root };
        let tail = if opponent.division() == division {
            format!(", a division rival, counts twice in the {division} race")
        } else if opponent.conference() == conference {
            format!(" also counts in the {conference} conference-record tiebreaker")
        } else {
            String::new()
        };
        Some(format!(
            "{} own game: a win over the {}{tail}.",
            capitalize(&possessive(focus)),
            nickname(opponent)
        ))
    } else if other.division() == division {
        Some(format!(
            "The {} ({}) are {} rival of {} ({}).",
            nickname(other),
            record(other),
            a_or_an(&division.to_string()),
            the(focus),
            record(focus)
        ))
    } else if other.conference() == conference && playoff_odds(other) >= CONTENDER {
        Some(format!(
            "The {} ({}) compete with {} ({}) for {conference} playoff spots and seeding.",
            nickname(other),
            record(other),
            the(focus),
            record(focus)
        ))
    } else if root.conference() != conference && other.conference() != conference {
        Some(format!(
            "{} {} game, so it reaches {} mostly through the Super Bowl opponent and the tiebreakers of teams on their schedule.",
            capitalize(a_or_an(&root.conference().to_string())),
            root.conference(),
            the(focus)
        ))
    } else {
        None
    }
}

/// The NFC North goal's explanation (decision 0007).
fn nfc_north(
    impact: &GameImpact,
    root: Option<Team>,
    standings: &Standings,
    playoff_odds: &dyn Fn(Team) -> f64,
) -> Vec<String> {
    let record = |team: Team| format_record(standings.record(team));
    let Some(root) = root else {
        return vec![format!(
            "Doesn't matter: neither result of {} at {} measurably moves the chance that all four NFC North teams reach the divisional round.",
            nickname(impact.away),
            nickname(impact.home)
        )];
    };
    let other = if root == impact.home {
        impact.away
    } else {
        impact.home
    };
    if root.division() == Division::NFC_NORTH {
        vec![format!(
            "The {} ({}) are one of the four NFC North teams that all need to reach the divisional round.",
            nickname(root),
            record(root)
        )]
    } else if other.conference() == Conference::Nfc && playoff_odds(other) >= CONTENDER {
        vec![format!(
            "The {} ({}) compete with the NFC North teams for NFC playoff spots and seeding.",
            nickname(other),
            record(other)
        )]
    } else {
        vec![format!(
            "{} helps the NFC North teams' seeding or playoff path.",
            capitalize(&a_win(root))
        )]
    }
}

fn format_record(record: Record) -> String {
    if record.ties > 0 {
        format!("{}-{}-{}", record.wins, record.losses, record.ties)
    } else {
        format!("{}-{}", record.wins, record.losses)
    }
}

/// A share of a goal's odds in percentage points, with more decimals as it
/// gets smaller, as in "1.2 Super Bowl points" or "0.015 playoff points".
fn points(share: f64, goal: Goal) -> String {
    let points = share * 100.0;
    let number = if points >= 1.0 {
        format!("{points:.1}")
    } else if points >= 0.1 {
        format!("{points:.2}")
    } else if points >= 0.001 {
        format!("{points:.3}")
    } else if points > 0.0 {
        "under 0.001".to_string()
    } else {
        "0".to_string()
    };
    format!("{number} {} points", goal.unit())
}

/// A swing and its noise, as in "0.015 give or take 0.016 Super Bowl
/// points".
fn give_or_take(swing: &Swing, goal: Goal) -> String {
    let suffix = format!(" {} points", goal.unit());
    let size = points(swing.swing().abs(), goal);
    let size = size.strip_suffix(&suffix).unwrap_or(&size);
    format!("{size} give or take {}", points(swing.std_error, goal))
}

/// "a Saints win", or "an Eagles win".
fn a_win(team: Team) -> String {
    let name = nickname(team);
    format!("{} {name} win", a_or_an(name))
}

/// The article for a word or name: "an" before a vowel sound ("an
/// Eagles", "an AFC", "an NFC North"), "a" otherwise.
fn a_or_an(word: &str) -> &'static str {
    // Spelled-out letters such as NFC ("en-eff-cee") sound like a vowel.
    let initialism = word.len() >= 2 && word.chars().take(3).all(|c| c.is_ascii_uppercase());
    let vowel = word.starts_with(['A', 'E', 'I', 'O', 'U'])
        || (initialism && word.starts_with(['F', 'H', 'L', 'M', 'N', 'R', 'S', 'X']));
    if vowel { "an" } else { "a" }
}

/// The team as a sentence names it: "the Bears".
pub fn the(team: Team) -> String {
    format!("the {}", nickname(team))
}

/// The team's possessive: "the Bears'". Every nickname is a plural ending
/// in "s".
pub fn possessive(team: Team) -> String {
    format!("the {}'", nickname(team))
}

/// The other conference: the one a Super Bowl opponent comes from.
pub fn other_conference(conference: Conference) -> Conference {
    match conference {
        Conference::Afc => Conference::Nfc,
        Conference::Nfc => Conference::Afc,
    }
}

/// The phrase with its first letter in upper case.
fn capitalize(phrase: &str) -> String {
    let mut chars = phrase.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// A share as a percentage of seasons, as in "1.2%".
fn percent(share: f64) -> String {
    let pct = share * 100.0;
    if pct >= 1.0 {
        format!("{pct:.1}%")
    } else if pct >= 0.01 {
        format!("{pct:.2}%")
    } else {
        "under 0.01%".to_string()
    }
}

/// Joins phrases in plain English: "a", "a and b", "a, b and c".
fn list(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
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
    fn lists_parts_in_plain_english() {
        let parts = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(list(&parts(&["a"])), "a");
        assert_eq!(list(&parts(&["a", "b"])), "a and b");
        assert_eq!(list(&parts(&["a", "b", "c"])), "a, b and c");
    }

    #[test]
    fn formats_records_points_and_percents() {
        let record = Record {
            wins: 8,
            losses: 2,
            ties: 0,
        };
        assert_eq!(format_record(record), "8-2");
        assert_eq!(format_record(Record { ties: 1, ..record }), "8-2-1");
        assert_eq!(points(0.0123, Goal::Champion), "1.2 Super Bowl points");
        assert_eq!(points(0.0029, Goal::Playoffs), "0.29 playoff points");
        assert_eq!(points(0.00015, Goal::Champion), "0.015 Super Bowl points");
        assert_eq!(percent(0.123), "12.3%");
        assert_eq!(percent(0.0012), "0.12%");
        assert_eq!(percent(0.00001), "under 0.01%");
        assert_eq!(
            points(0.000001, Goal::Champion),
            "under 0.001 Super Bowl points"
        );
        assert_eq!(points(0.0, Goal::Playoffs), "0 playoff points");
        let swing = Swing {
            if_home_wins: 0.50015,
            if_away_wins: 0.5,
            std_error: 0.00016,
        };
        assert_eq!(
            give_or_take(&swing, Goal::Champion),
            "0.015 give or take 0.016 Super Bowl points"
        );
        assert_eq!(a_win(Team::BEARS), "a Bears win");
        let eagles = Team::from_abbr("PHI").unwrap_or(Team::BEARS);
        assert_eq!(a_win(eagles), "an Eagles win");
    }

    #[test]
    fn every_team_has_a_nickname() {
        for team in Team::all() {
            assert_ne!(nickname(team), team.abbr(), "{team}");
        }
    }
}
