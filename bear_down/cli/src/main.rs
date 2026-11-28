//! `bear_down`: downloads the nflverse schedule, runs the simulation and
//! writes `rooting.json` for the web app (decision 0001). The file's format
//! is described in `docs/rooting-json.md`.
//!
//! Usage: `bear_down [--games <path or URL>] [--out <path>] [--simulations <n>]
//! [--week <n> [--season <year>]] [--team <abbr>] [--teams-dir <dir>]`.
//! `--week` rebuilds a past week's guide from the results before it, for the
//! archive (`docs/plans/2026-11-archive.md`). `--team` writes the guide for
//! another team than the Bears, and `--teams-dir` also writes every other
//! team's guide to `<dir>/<team>.json`, for the team picker
//! (`docs/decisions/0010-any-team.md`).

use std::path::Path;
use std::process::ExitCode;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use bear_down_core::{
    EloParams, Ratings, Schedule, SimConfig, Simulation, Standings, Team, TeamOdds, simulate,
};

mod banner;
mod rooting;
mod why;

/// Where nflverse publishes `games.csv` (decision 0006).
const GAMES_URL: &str = "https://raw.githubusercontent.com/nflverse/nfldata/master/data/games.csv";

/// The command-line options.
#[derive(Debug, PartialEq)]
struct Args {
    /// A local path or an http(s) URL for `games.csv`.
    games: String,
    /// Where to write `rooting.json`.
    out: String,
    /// How many seasons to simulate; `None` uses the weekly default.
    simulations: Option<u32>,
    /// Rebuild this week's guide from the results before it; `None` is the
    /// current week.
    week: Option<u8>,
    /// The season of `week`; `None` is the latest one.
    season: Option<u16>,
    /// The team the guide at `out` is for.
    team: Team,
    /// Where to write every other team's guide, as `<team>.json`; `None`
    /// writes only `out`.
    teams_dir: Option<String>,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        games: GAMES_URL.to_string(),
        out: "rooting.json".to_string(),
        simulations: None,
        week: None,
        season: None,
        team: Team::BEARS,
        teams_dir: None,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--games" => parsed.games = value()?,
            "--out" => parsed.out = value()?,
            "--simulations" => {
                let text = value()?;
                let count = text
                    .parse()
                    .map_err(|_| format!("--simulations {text:?} is not a whole number"))?;
                if count == 0 {
                    return Err("--simulations must be at least 1".to_string());
                }
                parsed.simulations = Some(count);
            }
            "--week" => {
                let text = value()?;
                parsed.week = Some(
                    text.parse()
                        .ok()
                        .filter(|week| *week >= 1)
                        .ok_or_else(|| format!("--week {text:?} is not a week number"))?,
                );
            }
            "--season" => {
                let text = value()?;
                parsed.season = Some(
                    text.parse()
                        .map_err(|_| format!("--season {text:?} is not a year"))?,
                );
            }
            "--team" => {
                let text = value()?;
                parsed.team = Team::from_abbr(&text.to_ascii_uppercase())
                    .ok_or_else(|| format!("--team {text:?} is not an nflverse team"))?;
            }
            "--teams-dir" => parsed.teams_dir = Some(value()?),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument {other:?}\n{}", usage())),
        }
    }
    if parsed.season.is_some() && parsed.week.is_none() {
        return Err("--season needs --week".to_string());
    }
    Ok(parsed)
}

fn usage() -> String {
    "usage: bear_down [--games <path or URL>] [--out <path>] [--simulations <n>] [--week <n> [--season <year>]] [--team <abbr>] [--teams-dir <dir>]"
        .to_string()
}

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)).and_then(|args| run(&args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bear_down: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    eprintln!("bear_down: reading {}", args.games);
    let csv = read_games(&args.games)?;
    let schedule = Schedule::parse(&csv).map_err(|error| format!("games.csv: {error}"))?;
    let schedule = match args.week {
        Some(week) => as_of(&schedule, args.season, week)?,
        None => schedule,
    };
    let generated_at = rfc3339(SystemTime::now());
    write_guide(
        &schedule,
        args.simulations,
        &generated_at,
        args.team,
        &args.out,
    )?;
    if let Some(dir) = &args.teams_dir {
        std::fs::create_dir_all(dir).map_err(|error| format!("couldn't create {dir}: {error}"))?;
        let others: Vec<Team> = Team::all().filter(|&team| team != args.team).collect();
        for_each_in_parallel(&others, |team| {
            let out = Path::new(dir).join(format!("{}.json", team.abbr()));
            write_guide(
                &schedule,
                args.simulations,
                &generated_at,
                team,
                &out.to_string_lossy(),
            )
        })?;
    }
    Ok(())
}

/// Builds `team`'s guide from `schedule` and writes it to `out`.
fn write_guide(
    schedule: &Schedule,
    simulations: Option<u32>,
    generated_at: &str,
    team: Team,
    out: &str,
) -> Result<(), String> {
    let guide = build(schedule, simulations, generated_at.to_string(), team)?;
    let json = serde_json::to_string_pretty(&guide).map_err(|error| error.to_string())?;
    std::fs::write(out, json + "\n").map_err(|error| format!("couldn't write {out}: {error}"))?;
    eprintln!(
        "bear_down: wrote {out} ({team}, season {}, week {}, {} games, {} simulations)",
        guide.season,
        guide
            .week
            .map_or("none".to_string(), |week| week.to_string()),
        guide.games.len(),
        guide.simulations
    );
    Ok(())
}

/// Runs `job` for every team, on as many threads as the machine has cores,
/// and returns the first error.
fn for_each_in_parallel(
    teams: &[Team],
    job: impl Fn(Team) -> Result<(), String> + Sync,
) -> Result<(), String> {
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let queue = Mutex::new(teams.iter().copied());
    let errors = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.min(teams.len()) {
            scope.spawn(|| {
                loop {
                    let next = queue.lock().map_or(None, |mut teams| teams.next());
                    let Some(team) = next else { break };
                    if let Err(error) = job(team)
                        && let Ok(mut errors) = errors.lock()
                    {
                        errors.push(format!("{team}: {error}"));
                    }
                }
            });
        }
    });
    let errors = errors.into_inner().unwrap_or_default();
    match errors.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn read_games(source: &str) -> Result<String, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let response = reqwest::blocking::get(source)
            .and_then(|response| response.error_for_status())
            .map_err(|error| format!("couldn't download {source}: {error}"))?;
        response
            .text()
            .map_err(|error| format!("couldn't read {source}: {error}"))
    } else {
        std::fs::read_to_string(source).map_err(|error| format!("couldn't read {source}: {error}"))
    }
}

/// The schedule just before `week` of `season` (the latest one if `None`).
fn as_of(schedule: &Schedule, season: Option<u16>, week: u8) -> Result<Schedule, String> {
    let year = match season {
        Some(year) => year,
        None => {
            schedule
                .latest_season()
                .ok_or("games.csv has no games")?
                .year
        }
    };
    schedule.as_of(year, week)
}

/// Simulates the schedule's latest season and builds `team`'s guide.
fn build(
    schedule: &Schedule,
    simulations: Option<u32>,
    generated_at: String,
    team: Team,
) -> Result<rooting::Guide, String> {
    let season = schedule.latest_season().ok_or("games.csv has no games")?;
    let simulation = run_simulation(schedule, simulations, team)?;
    let standings = Standings::from_season(&season)?;
    let banner = banner::banner(&banner::Context {
        season: &season,
        standings: &standings,
        focus: simulation.focus,
        odds: simulation.odds(simulation.focus),
        odds_before: odds_before(schedule, simulations, team),
    });
    Ok(rooting::Guide::new(
        &simulation,
        &standings,
        season.latest_played(),
        banner,
        generated_at,
    ))
}

/// Simulates the rest of the schedule's latest season, measuring each
/// game's swing on `team`.
fn run_simulation(
    schedule: &Schedule,
    simulations: Option<u32>,
    team: Team,
) -> Result<Simulation, String> {
    let season = schedule.latest_season().ok_or("games.csv has no games")?;
    let ratings = Ratings::from_schedule(schedule, EloParams::default());
    let week = season.current_week().map_or(0, |week| week.number);
    let mut config = SimConfig::weekly(season.year, week);
    config.focus = team;
    if let Some(simulations) = simulations {
        config.simulations = simulations;
    }
    simulate(&season, &ratings, &config)
}

/// `team`'s odds just before the week of its latest regular-season game,
/// for the banner's "odds up from" line: a second simulation of the
/// schedule as it stood then. `None` when there is no such game, it was a
/// playoff game or it wasn't a win (the banner only follows a win).
fn odds_before(schedule: &Schedule, simulations: Option<u32>, team: Team) -> Option<TeamOdds> {
    let season = schedule.latest_season()?;
    let game = banner::latest_game(&season, team)?;
    if game.winner() != Some(team.abbr()) {
        return None;
    }
    if game.game_type.is_playoff() {
        return None;
    }
    let before = schedule.as_of(season.year, game.week).ok()?;
    let simulation = run_simulation(&before, simulations, team).ok()?;
    Some(simulation.odds(simulation.focus))
}

/// Formats `time` as an RFC 3339 UTC timestamp to the second.
fn rfc3339(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// The calendar date `days` after 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn args(list: &[&str]) -> Result<Args, String> {
        parse_args(list.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn defaults_download_nflverse_and_write_rooting_json() {
        assert_eq!(
            args(&[]),
            Ok(Args {
                games: GAMES_URL.to_string(),
                out: "rooting.json".to_string(),
                simulations: None,
                week: None,
                season: None,
                team: Team::BEARS,
                teams_dir: None,
            })
        );
    }

    #[test]
    fn options_override_the_defaults() {
        assert_eq!(
            args(&[
                "--games",
                "g.csv",
                "--out",
                "web/public/rooting.json",
                "--simulations",
                "500"
            ]),
            Ok(Args {
                games: "g.csv".to_string(),
                out: "web/public/rooting.json".to_string(),
                simulations: Some(500),
                week: None,
                season: None,
                team: Team::BEARS,
                teams_dir: None,
            })
        );
        assert_eq!(
            args(&["--week", "12", "--season", "2024"]).map(|a| (a.week, a.season)),
            Ok((Some(12), Some(2024)))
        );
    }

    #[test]
    fn bad_arguments_are_errors() {
        assert!(args(&["--out"]).is_err());
        assert!(args(&["--simulations", "lots"]).is_err());
        assert!(args(&["--simulations", "0"]).is_err());
        assert!(args(&["--verbose"]).is_err());
        assert!(args(&["--week", "0"]).is_err());
        assert!(args(&["--week", "twelve"]).is_err());
        assert!(args(&["--season", "2024"]).is_err());
        assert!(args(&["--week", "3", "--season", "last"]).is_err());
    }

    #[test]
    fn team_options_pick_the_guides_team() {
        let parsed = args(&["--team", "kc", "--teams-dir", "web/public/teams"]);
        assert_eq!(
            parsed.map(|a| (a.team.abbr(), a.teams_dir)),
            Ok(("KC", Some("web/public/teams".to_string())))
        );
        assert!(args(&["--team", "Bears"]).is_err());
        assert!(args(&["--team"]).is_err());
    }

    #[test]
    fn any_team_gets_its_own_guide() {
        // 2025 before week 18: the Steelers and Ravens play for the AFC
        // North title.
        let Some(steelers) = Team::from_abbr("PIT") else {
            panic!("PIT is a team");
        };
        let guide = match build(&before_week_18(), Some(1_000), String::new(), steelers) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        assert_eq!(guide.team, "PIT");
        assert_eq!(guide.games.len(), 16);
        let own_game = guide
            .games
            .iter()
            .find(|game| game.away == "PIT" || game.home == "PIT");
        let Some(own_game) = own_game else {
            panic!("the Steelers play in week 18");
        };
        // The team's own game is always a pick for it (BEAR-001).
        assert_eq!(
            own_game.champion.why[0].split(':').next(),
            Some("The Steelers' own game")
        );
        assert!(own_game.champion.root_for.is_some());
        assert!(
            own_game
                .champion
                .why
                .iter()
                .any(|line| line.contains("AFC North")),
            "{:?}",
            own_game.champion.why
        );
        for game in &guide.games {
            for swing in [&game.champion, &game.playoffs] {
                assert!(!swing.why.is_empty(), "{}", game.id);
                for line in &swing.why {
                    // Explanations are about the Steelers, never the Bears,
                    // unless the Bears play in the game.
                    if game.away != "CHI" && game.home != "CHI" {
                        assert!(!line.contains("Bears"), "{}: {line}", game.id);
                    }
                    assert!(!line.contains("NFC North title"), "{}: {line}", game.id);
                    assert!(!line.contains("NFC playoff field"), "{}: {line}", game.id);
                }
            }
        }
        let json = match serde_json::to_value(&guide) {
            Ok(json) => json,
            Err(error) => panic!("guide should serialize: {error}"),
        };
        assert_eq!(json["team"], "PIT");
        let record = &json["focus"]["record"];
        assert_eq!(record["wins"].as_u64().map(|w| w > 0), Some(true));
    }

    #[test]
    fn timestamps_are_rfc3339_utc() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let thanksgiving = UNIX_EPOCH + Duration::from_secs(1_795_705_200);
        assert_eq!(rfc3339(thanksgiving), "2026-11-26T15:00:00Z");
        let leap_day = UNIX_EPOCH + Duration::from_secs(951_782_400);
        assert_eq!(rfc3339(leap_day), "2000-02-29T00:00:00Z");
    }

    /// Every 2002 to 2025 game, with 2025's playoffs dropped and its week 18
    /// not yet played.
    fn before_week_18() -> Schedule {
        before_week(2025, 18)
    }

    /// The fixture's seasons up to `year`, with `year`'s playoffs dropped and
    /// its `week` not yet played.
    fn before_week(year: u16, week: u8) -> Schedule {
        let full = include_str!("../../core/fixtures/seasons.csv");
        let (year_text, prefix) = (year.to_string(), format!("{year}_{week:02}_"));
        let mut csv = String::new();
        for (index, line) in full.lines().enumerate() {
            let cols: Vec<&str> = line.split(',').collect();
            if index > 0 && cols[1].parse::<u16>().is_ok_and(|season| season > year) {
                continue;
            }
            if cols[1] == year_text && cols[2] != "REG" {
                continue;
            }
            if line.starts_with(&prefix) {
                let mut cols = cols;
                cols[6] = "";
                cols[8] = "";
                cols[10] = "";
                csv.push_str(&cols.join(","));
            } else {
                csv.push_str(line);
            }
            csv.push('\n');
        }
        match Schedule::parse(&csv) {
            Ok(schedule) => schedule,
            Err(error) => panic!("fixture should parse: {error}"),
        }
    }

    #[test]
    fn writes_the_current_weeks_games_in_the_documented_format() {
        let guide = match build(
            &before_week_18(),
            Some(2_000),
            "2026-01-01T00:00:00Z".into(),
            Team::BEARS,
        ) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        assert_eq!((guide.season, guide.week), (2025, Some(18)));
        assert_eq!(guide.games.len(), 16);
        assert_eq!(guide.simulations, 2_000);
        let json = match serde_json::to_value(&guide) {
            Ok(json) => json,
            Err(error) => panic!("guide should serialize: {error}"),
        };
        for key in [
            "generated_at",
            "season",
            "week",
            "simulations",
            "latest_game",
            "team",
            "focus",
            "banner",
            "nfc_north_divisional",
            "games",
        ] {
            assert!(json.get(key).is_some(), "rooting.json should have {key}");
        }
        assert_eq!(
            json["latest_game"]["gameday"].as_str().map(str::len),
            Some(10)
        );
        let game = &json["games"][0];
        for goal in ["champion", "playoffs", "nfc_north"] {
            for key in [
                "if_home_wins",
                "if_away_wins",
                "swing",
                "root_for",
                "tier",
                "why",
                "channels",
            ] {
                assert!(game[goal].get(key).is_some(), "{goal} should have {key}");
            }
        }
        let odds = &json["focus"]["odds"];
        assert!(odds["playoffs"].as_f64() >= odds["champion"].as_f64());
        let record = &json["focus"]["record"];
        assert_eq!(
            record["wins"].as_u64().unwrap_or(0)
                + record["losses"].as_u64().unwrap_or(0)
                + record["ties"].as_u64().unwrap_or(0),
            16
        );
        // 2025 before week 18: the 11-5 Bears led the 9-6-1 Packers by a
        // game and a half.
        assert_eq!(json["focus"]["division_lead"].as_f64(), Some(1.5));
        assert_eq!(json["focus"]["division_rival"], "GB");
        // Their latest game, week 17 at the 49ers, was a 42-38 loss.
        assert!(json["banner"].is_null());
    }

    #[test]
    fn the_banner_follows_a_bears_win_with_current_context() {
        let full = match Schedule::parse(include_str!("../../core/fixtures/seasons.csv")) {
            Ok(schedule) => schedule,
            Err(error) => panic!("fixture should parse: {error}"),
        };
        let banner_as_of = |week: u8| {
            let schedule = match full.as_of(2025, week) {
                Ok(schedule) => schedule,
                Err(error) => panic!("2025 week {week}: {error}"),
            };
            match build(
                &schedule,
                Some(2_000),
                "2026-01-01T00:00:00Z".into(),
                Team::BEARS,
            ) {
                Ok(guide) => guide.banner,
                Err(error) => panic!("build should succeed: {error}"),
            }
        };
        // 2025 before week 17: the Bears had just beaten the Packers 22-16.
        let Some(banner) = banner_as_of(17) else {
            panic!("a win should get a banner");
        };
        assert_eq!(banner.headline, "Bears win! Chicago 22, Packers 16.");
        assert!(!banner.lines.is_empty() && banner.lines.len() <= banner::MAX_LINES);
        assert!(
            banner.lines[0].contains("NFC North"),
            "the division race leads: {:?}",
            banner.lines
        );
        // After the wild-card win over the Packers, 31-27.
        let Some(banner) = banner_as_of(20) else {
            panic!("a playoff win should get a banner");
        };
        assert_eq!(banner.headline, "Bears win! Chicago 31, Packers 27.");
        assert_eq!(banner.lines, ["On to the divisional round."]);
    }

    #[test]
    fn a_past_week_is_rebuilt_from_the_results_before_it() {
        let full = match Schedule::parse(include_str!("../../core/fixtures/seasons.csv")) {
            Ok(schedule) => schedule,
            Err(error) => panic!("fixture should parse: {error}"),
        };
        let past = match as_of(&full, Some(2024), 12) {
            Ok(schedule) => schedule,
            Err(error) => panic!("2024 week 12: {error}"),
        };
        let guide = match build(&past, Some(500), String::new(), Team::BEARS) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        assert_eq!((guide.season, guide.week), (2024, Some(12)));
        let week_12 = full
            .season(2024)
            .and_then(|season| season.week(12))
            .map_or(0, |week| week.games().len());
        assert_eq!(guide.games.len(), week_12);
        // The latest season is the default.
        let Ok(latest) = as_of(&full, None, 3) else {
            panic!("2025 week 3");
        };
        assert_eq!(latest.latest_season().map(|season| season.year), Some(2025));
        assert!(as_of(&full, Some(2024), 40).is_err());
    }

    #[test]
    fn every_game_is_explained_from_what_was_measured() {
        let guide = match build(&before_week_18(), Some(2_000), String::new(), Team::BEARS) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        let mut negligible = Vec::new();
        for game in &guide.games {
            for (goal, swing) in [("champion", &game.champion), ("playoffs", &game.playoffs)] {
                // BEAR-001, BEAR-003, BEAR-005: a verdict and an explanation
                // for every game.
                assert!(
                    !swing.why.is_empty(),
                    "{} {goal} has no explanation",
                    game.id
                );
                match swing.tier {
                    "matters" => assert!(swing.root_for.is_some(), "{} {goal}", game.id),
                    "negligible" | "none" => {
                        assert!(swing.root_for.is_none(), "{} {goal}", game.id)
                    }
                    "small" => {}
                    other => panic!("unknown tier {other}"),
                }
                // The page applies its A/B arm's line with the same shared
                // rule, so at the CLI's line it must give the CLI's tier.
                if swing.tier != "none" {
                    let tier = bear_down_core::tier_against(
                        swing.swing,
                        swing.std_error,
                        bear_down_core::NEGLIGIBLE_SWING,
                    );
                    assert_eq!(tier.name(), swing.tier, "{} {goal}", game.id);
                }
                if swing.root_for.is_none() && goal == "champion" {
                    negligible.push(swing.why[0].clone());
                }
                // BEAR-006: a tiebreaker step named in the text is one the
                // simulation measured deciding the Bears' seeding.
                for line in &swing.why {
                    let Some((_, after)) = line.split_once("decided by ") else {
                        continue;
                    };
                    let step = after.split(" in ").next().unwrap_or_default();
                    assert!(
                        swing
                            .channels
                            .iter()
                            .any(|channel| channel.cause.as_deref() == Some(step)),
                        "{} {goal}: {line}",
                        game.id
                    );
                }
                for channel in &swing.channels {
                    assert!(channel.share > 2.0 * channel.std_error);
                    assert!(channel.favors == game.home || channel.favors == game.away);
                }
            }
            assert!(game.champion.channels.len() <= game.champion.why.len() + 20);
        }
        // BEAR-005: no two "doesn't matter" games share their headline.
        let mut unique = negligible.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), negligible.len(), "{negligible:#?}");
    }

    /// A golden week (plan step 6): 2018 week 17, the last regular-season
    /// week then. The Bears (11-4) had clinched the NFC North and the Saints
    /// the No. 1 seed. The Bears got the No. 2 seed, the last bye, only by
    /// beating the Vikings while the 49ers beat the Rams (12-3), which would
    /// tie them at 12-4 with the Bears ahead on head-to-head (they beat the
    /// Rams in week 14).
    #[test]
    fn golden_week_2018_week_17() {
        let guide = match build(
            &before_week(2018, 17),
            Some(4_000),
            String::new(),
            Team::BEARS,
        ) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        assert_eq!((guide.season, guide.week), (2018, Some(17)));
        assert_eq!(guide.games.len(), 16);
        assert!((guide.focus.odds.playoffs - 1.0).abs() < 1e-12);
        let game = |away: &str, home: &str| match guide
            .games
            .iter()
            .find(|game| game.away == away && game.home == home)
        {
            Some(game) => game,
            None => panic!("{away} at {home} should be in week 17"),
        };
        for game in &guide.games {
            // Already in: no game moves the Bears' playoff odds at all, and
            // each says so in its own words.
            assert_eq!(game.playoffs.root_for, None, "{}", game.id);
            assert_eq!(game.playoffs.tier, "negligible", "{}", game.id);
            assert!(
                game.playoffs.why[0].contains("no simulated season came out differently"),
                "{}: {:?}",
                game.id,
                game.playoffs.why
            );
            // Already division champions: nothing runs through the division
            // title or a wild card.
            for channel in &game.champion.channels {
                assert!(
                    channel.channel != "division_title" && channel.channel != "wild_card",
                    "{}: {}",
                    game.id,
                    channel.channel
                );
            }
        }
        // The Bears' own game decides the bye on record.
        let own = game("CHI", "MIN");
        assert_eq!(own.champion.root_for.as_deref(), Some("CHI"));
        assert_eq!(own.champion.tier, "matters");
        assert!(own.champion.channels.iter().any(|channel| {
            channel.channel == "seed" && channel.cause.as_deref() == Some("record")
        }));
        // Root for the 49ers: the bye then comes down to head-to-head.
        let rams = game("SF", "LA");
        assert_eq!(rams.champion.root_for.as_deref(), Some("SF"));
        assert_eq!(rams.champion.tier, "matters");
        let Some(biggest) = rams.champion.channels.first() else {
            panic!("49ers at Rams should have a channel");
        };
        assert_eq!(biggest.channel, "seed");
        assert_eq!(biggest.cause.as_deref(), Some("head-to-head"));
        assert_eq!(biggest.favors, "SF");
        assert!(
            rams.champion
                .why
                .iter()
                .any(|line| line.starts_with("Seeding, decided by head-to-head")),
            "{:?}",
            rams.champion.why
        );
    }

    /// A golden week: 2020 week 17. The Packers had clinched the NFC North,
    /// and the 8-7 Bears were in with a win over the Packers or a Cardinals
    /// loss to the Rams (they lost, and got in when the Rams won).
    #[test]
    fn golden_week_2020_week_17() {
        let guide = match build(
            &before_week(2020, 17),
            Some(4_000),
            String::new(),
            Team::BEARS,
        ) {
            Ok(guide) => guide,
            Err(error) => panic!("build should succeed: {error}"),
        };
        assert_eq!((guide.season, guide.week), (2020, Some(17)));
        let game = |away: &str, home: &str| match guide
            .games
            .iter()
            .find(|game| game.away == away && game.home == home)
        {
            Some(game) => game,
            None => panic!("{away} at {home} should be in week 17"),
        };
        let own = game("GB", "CHI");
        assert_eq!(own.playoffs.root_for.as_deref(), Some("CHI"));
        assert_eq!(own.playoffs.tier, "matters");
        let cardinals = game("ARI", "LA");
        assert_eq!(cardinals.playoffs.root_for.as_deref(), Some("LA"));
        assert_eq!(cardinals.playoffs.tier, "matters");
        let Some(biggest) = cardinals.playoffs.channels.first() else {
            panic!("Cardinals at Rams should have a channel");
        };
        assert_eq!(biggest.channel, "wild_card");
        assert_eq!(biggest.favors, "LA");
        assert!(
            cardinals.playoffs.why[1].starts_with("Wild card"),
            "{:?}",
            cardinals.playoffs.why
        );
        // The Packers had the division: nothing runs through it.
        for game in &guide.games {
            for channel in game.champion.channels.iter().chain(&game.playoffs.channels) {
                assert_ne!(channel.channel, "division_title", "{}", game.id);
            }
        }
    }

    #[test]
    fn a_missing_games_file_is_an_error() {
        assert!(read_games("no/such/games.csv").is_err());
    }
}
