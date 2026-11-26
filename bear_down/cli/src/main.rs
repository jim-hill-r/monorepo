//! `bear_down`: downloads the nflverse schedule, runs the simulation and
//! writes `rooting.json` for the web app (decision 0001). The file's format
//! is described in `docs/rooting-json.md`.
//!
//! Usage: `bear_down [--games <path or URL>] [--out <path>] [--simulations <n>]`

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bear_down_core::{EloParams, Ratings, Schedule, SimConfig, Standings, simulate};

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
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        games: GAMES_URL.to_string(),
        out: "rooting.json".to_string(),
        simulations: None,
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
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument {other:?}\n{}", usage())),
        }
    }
    Ok(parsed)
}

fn usage() -> String {
    "usage: bear_down [--games <path or URL>] [--out <path>] [--simulations <n>]".to_string()
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
    let generated_at = rfc3339(SystemTime::now());
    let guide = build(&schedule, args.simulations, generated_at)?;
    let json = serde_json::to_string_pretty(&guide).map_err(|error| error.to_string())?;
    std::fs::write(&args.out, json + "\n")
        .map_err(|error| format!("couldn't write {}: {error}", args.out))?;
    eprintln!(
        "bear_down: wrote {} (season {}, week {}, {} games, {} simulations)",
        args.out,
        guide.season,
        guide
            .week
            .map_or("none".to_string(), |week| week.to_string()),
        guide.games.len(),
        guide.simulations
    );
    Ok(())
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

/// Simulates the schedule's latest season and builds the guide.
fn build(
    schedule: &Schedule,
    simulations: Option<u32>,
    generated_at: String,
) -> Result<rooting::Guide, String> {
    let season = schedule.latest_season().ok_or("games.csv has no games")?;
    let ratings = Ratings::from_schedule(schedule, EloParams::default());
    let week = season.current_week().map_or(0, |week| week.number);
    let mut config = SimConfig::weekly(season.year, week);
    if let Some(simulations) = simulations {
        config.simulations = simulations;
    }
    let simulation = simulate(&season, &ratings, &config)?;
    let standings = Standings::from_season(&season)?;
    let games: Vec<_> = season.regular_season().cloned().collect();
    Ok(rooting::Guide::new(
        &simulation,
        &standings,
        season.latest_played(),
        &games,
        generated_at,
    ))
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
            })
        );
    }

    #[test]
    fn bad_arguments_are_errors() {
        assert!(args(&["--out"]).is_err());
        assert!(args(&["--simulations", "lots"]).is_err());
        assert!(args(&["--simulations", "0"]).is_err());
        assert!(args(&["--verbose"]).is_err());
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
        let full = include_str!("../../core/fixtures/seasons.csv");
        let mut csv = String::new();
        for line in full.lines() {
            let cols: Vec<&str> = line.split(',').collect();
            if cols[1] == "2025" && cols[2] != "REG" {
                continue;
            }
            if line.starts_with("2025_18_") {
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
            "bears",
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
            for key in ["if_home_wins", "if_away_wins", "swing", "root_for", "why"] {
                assert!(game[goal].get(key).is_some(), "{goal} should have {key}");
            }
        }
        let odds = &json["bears"]["odds"];
        assert!(odds["playoffs"].as_f64() >= odds["champion"].as_f64());
        let record = &json["bears"]["record"];
        assert_eq!(
            record["wins"].as_u64().unwrap_or(0)
                + record["losses"].as_u64().unwrap_or(0)
                + record["ties"].as_u64().unwrap_or(0),
            16
        );
    }

    #[test]
    fn a_missing_games_file_is_an_error() {
        assert!(read_games("no/such/games.csv").is_err());
    }
}
