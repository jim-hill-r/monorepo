//! The events the Worker accepts, how it checks them, and the Analytics
//! Engine data point each one becomes. Pure logic with no Workers types, so
//! it is tested natively. The format is in `../README.md`.

use serde::Deserialize;

/// The largest request body accepted, in bytes.
pub const MAX_BODY: usize = 4096;

/// The most characters kept from a fan's note.
pub const MAX_NOTE: usize = 200;

/// The most characters kept from an error message.
pub const MAX_MESSAGE: usize = 500;

/// One event, as the client posts it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    /// A page showed an A/B arm (`bear_down/docs/plans/2026-11-fan-feedback.md`).
    Exposure {
        project: String,
        #[serde(default)]
        build: Option<String>,
        arm: String,
        season: u16,
        week: u8,
        tab: String,
    },
    /// A fan voted on a verdict.
    Feedback {
        project: String,
        #[serde(default)]
        build: Option<String>,
        arm: String,
        season: u16,
        week: u8,
        tab: String,
        game: String,
        verdict: String,
        vote: String,
        #[serde(default)]
        note: Option<String>,
    },
    /// An uncaught error or panic in a web app or game.
    Error {
        project: String,
        #[serde(default)]
        build: Option<String>,
        page: String,
        message: String,
    },
    /// A gameplay event, such as a level reached.
    Game {
        project: String,
        #[serde(default)]
        build: Option<String>,
        name: String,
        #[serde(default)]
        value: Option<f64>,
    },
    /// An opt-in CLI report: a command's time, and its error if it failed.
    Cli {
        project: String,
        #[serde(default)]
        build: Option<String>,
        command: String,
        ms: f64,
        #[serde(default)]
        error: Option<String>,
    },
    /// A production deploy.
    Deploy {
        project: String,
        #[serde(default)]
        build: Option<String>,
        commit: String,
        #[serde(default)]
        pr: Option<u32>,
    },
}

/// An Analytics Engine data point: one index, then blobs and doubles in the
/// order the README lists them.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub index: String,
    pub blobs: Vec<String>,
    pub doubles: Vec<f64>,
}

/// Parses and checks a request body. The error says what is wrong, for the
/// 400 response.
pub fn parse(body: &str) -> Result<Point, String> {
    if body.len() > MAX_BODY {
        return Err(format!("body is over {MAX_BODY} bytes"));
    }
    let event: Event = serde_json::from_str(body).map_err(|error| error.to_string())?;
    event.point()
}

impl Event {
    fn project(&self) -> &str {
        match self {
            Event::Exposure { project, .. }
            | Event::Feedback { project, .. }
            | Event::Error { project, .. }
            | Event::Game { project, .. }
            | Event::Cli { project, .. }
            | Event::Deploy { project, .. } => project,
        }
    }

    fn build(&self) -> Option<&str> {
        match self {
            Event::Exposure { build, .. }
            | Event::Feedback { build, .. }
            | Event::Error { build, .. }
            | Event::Game { build, .. }
            | Event::Cli { build, .. }
            | Event::Deploy { build, .. } => build.as_deref(),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Event::Exposure { .. } => "exposure",
            Event::Feedback { .. } => "feedback",
            Event::Error { .. } => "error",
            Event::Game { .. } => "game",
            Event::Cli { .. } => "cli",
            Event::Deploy { .. } => "deploy",
        }
    }

    /// The data point for a valid event.
    pub fn point(&self) -> Result<Point, String> {
        let project = name("project", self.project(), 32)?;
        let build = match self.build() {
            Some(build) => name("build", build, 40)?,
            None => String::new(),
        };
        let mut blobs = vec![self.kind().to_string(), project.clone(), build];
        let mut doubles = Vec::new();
        match self {
            Event::Exposure {
                arm,
                season,
                week,
                tab,
                ..
            } => {
                blobs.push(one_of("arm", arm, ARMS)?);
                blobs.push(one_of("tab", tab, TABS)?);
                doubles.extend([f64::from(*season), f64::from(week_number(*week)?)]);
            }
            Event::Feedback {
                arm,
                season,
                week,
                tab,
                game,
                verdict,
                vote,
                note,
                ..
            } => {
                blobs.push(one_of("arm", arm, ARMS)?);
                blobs.push(one_of("tab", tab, TABS)?);
                blobs.push(name("game", game, 32)?);
                blobs.push(name("verdict", verdict, 32)?);
                blobs.push(one_of("vote", vote, VOTES)?);
                blobs.push(
                    note.as_deref()
                        .map(|note| clean(note, MAX_NOTE))
                        .unwrap_or_default(),
                );
                doubles.extend([f64::from(*season), f64::from(week_number(*week)?)]);
            }
            Event::Error { page, message, .. } => {
                blobs.push(text("page", page, 200)?);
                blobs.push(clean(message, MAX_MESSAGE));
            }
            Event::Game {
                name: event, value, ..
            } => {
                blobs.push(name("name", event, 40)?);
                doubles.push(finite("value", value.unwrap_or(0.0))?);
            }
            Event::Cli {
                command, ms, error, ..
            } => {
                blobs.push(name("command", command, 40)?);
                blobs.push(
                    error
                        .as_deref()
                        .map(|error| clean(error, MAX_MESSAGE))
                        .unwrap_or_default(),
                );
                doubles.push(finite("ms", *ms)?);
            }
            Event::Deploy { commit, pr, .. } => {
                if commit.is_empty()
                    || commit.len() > 40
                    || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    return Err("commit must be a hex SHA".to_string());
                }
                blobs.push(commit.to_ascii_lowercase());
                doubles.push(f64::from(pr.unwrap_or(0)));
            }
        }
        Ok(Point {
            index: project,
            blobs,
            doubles,
        })
    }
}

const ARMS: &[&str] = &["a", "b"];
const TABS: &[&str] = &["super_bowl", "playoffs", "nfc_north"];
/// `note` marks the follow-up event that carries the optional note on a
/// vote just cast, so the weekly export counts each vote once.
const VOTES: &[&str] = &["yes", "no", "confusing", "note"];

/// A short identifier: letters, digits, `_`, `-` and `.`, at most `max`
/// bytes.
fn name(field: &str, value: &str, max: usize) -> Result<String, String> {
    let ok = !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if ok {
        Ok(value.to_string())
    } else {
        Err(format!(
            "{field} must be 1 to {max} letters, digits, '_', '-' or '.'"
        ))
    }
}

fn one_of(field: &str, value: &str, allowed: &[&str]) -> Result<String, String> {
    if allowed.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(format!("{field} must be one of {}", allowed.join(", ")))
    }
}

fn text(field: &str, value: &str, max: usize) -> Result<String, String> {
    if value.is_empty() || value.chars().count() > max {
        Err(format!("{field} must be 1 to {max} characters"))
    } else {
        Ok(value.to_string())
    }
}

fn week_number(week: u8) -> Result<u8, String> {
    if (1..=22).contains(&week) {
        Ok(week)
    } else {
        Err("week must be 1 to 22".to_string())
    }
}

fn finite(field: &str, value: f64) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!("{field} must be a finite number"))
    }
}

/// Free text with anything that looks like an email address or a phone
/// number replaced, control characters dropped, and cut to `max`
/// characters.
pub fn clean(value: &str, max: usize) -> String {
    let words: Vec<&str> = value.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        // A run of words made only of phone characters, such as
        // "(312) 555-1234" or "+1 312 555 1234".
        let mut end = i;
        let mut digits = 0;
        while end < words.len() && is_phone_part(words[end]) {
            digits += words[end].bytes().filter(u8::is_ascii_digit).count();
            end += 1;
        }
        if digits >= 7 {
            out.push("[number]".to_string());
            i = end;
            continue;
        }
        let word = words[i];
        if looks_like_email(word) {
            out.push("[email]".to_string());
        } else {
            out.push(word.chars().filter(|c| !c.is_control()).collect());
        }
        i += 1;
    }
    out.join(" ").chars().take(max).collect()
}

fn is_phone_part(word: &str) -> bool {
    word.bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'(' | b')' | b'+' | b'-' | b'.'))
        && word.bytes().any(|byte| byte.is_ascii_digit())
}

fn looks_like_email(word: &str) -> bool {
    match word.split_once('@') {
        Some((user, domain)) => !user.is_empty() && domain.contains('.'),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(body: &str) -> Point {
        match parse(body) {
            Ok(point) => point,
            Err(error) => panic!("{body} should be valid: {error}"),
        }
    }

    fn error(body: &str) -> String {
        match parse(body) {
            Ok(point) => panic!("{body} should be refused, got {point:?}"),
            Err(error) => error,
        }
    }

    #[test]
    fn a_feedback_vote_becomes_one_data_point() {
        let point = point(
            r#"{"kind":"feedback","project":"bear_down","arm":"b","season":2026,"week":12,
                "tab":"super_bowl","game":"2026_12_PHI_DAL","verdict":"doesnt_matter",
                "vote":"no","note":"Feels like it matters, call me 312-555-1234"}"#,
        );
        assert_eq!(point.index, "bear_down");
        assert_eq!(
            point.blobs,
            [
                "feedback",
                "bear_down",
                "",
                "b",
                "super_bowl",
                "2026_12_PHI_DAL",
                "doesnt_matter",
                "no",
                "Feels like it matters, call me [number]",
            ]
        );
        assert_eq!(point.doubles, [2026.0, 12.0]);
    }

    #[test]
    fn an_exposure_names_the_arm_and_tab() {
        let point = point(
            r#"{"kind":"exposure","project":"bear_down","build":"4ca9b3d","arm":"a","season":2026,"week":12,"tab":"playoffs"}"#,
        );
        assert_eq!(
            point.blobs,
            ["exposure", "bear_down", "4ca9b3d", "a", "playoffs"]
        );
        assert_eq!(point.doubles, [2026.0, 12.0]);
    }

    #[test]
    fn the_other_kinds_are_accepted() {
        let error = point(
            r#"{"kind":"error","project":"cookbook","page":"/recipe/1","message":"panicked at x.rs:1"}"#,
        );
        assert_eq!(error.blobs[0], "error");
        let game = point(r#"{"kind":"game","project":"controlled_chaos","name":"level_3"}"#);
        assert_eq!(game.doubles, [0.0]);
        let cli = point(r#"{"kind":"cli","project":"homer","command":"search","ms":120.5}"#);
        assert_eq!(cli.doubles, [120.5]);
        let deploy =
            point(r#"{"kind":"deploy","project":"bear_down","commit":"D459B1B","pr":671}"#);
        assert_eq!(deploy.blobs[3], "d459b1b");
        assert_eq!(deploy.doubles, [671.0]);
    }

    #[test]
    fn bad_events_are_refused() {
        assert!(error(r#"{"kind":"nope","project":"x"}"#).contains("unknown variant"));
        assert!(error(r#"{"kind":"game","project":"has space","name":"x"}"#).contains("project"));
        assert!(error(r#"{"kind":"exposure","project":"bear_down","arm":"c","season":2026,"week":1,"tab":"playoffs"}"#).contains("arm"));
        assert!(error(r#"{"kind":"exposure","project":"bear_down","arm":"a","season":2026,"week":40,"tab":"playoffs"}"#).contains("week"));
        assert!(
            error(r#"{"kind":"deploy","project":"bear_down","commit":"not-hex"}"#)
                .contains("commit")
        );
        // Unknown fields are refused, so nothing extra is ever stored.
        assert!(
            error(r#"{"kind":"game","project":"x","name":"y","email":"a@b.co"}"#)
                .contains("unknown field")
        );
        let big = format!(
            r#"{{"kind":"game","project":"x","name":"{}"}}"#,
            "y".repeat(MAX_BODY)
        );
        assert!(error(&big).contains("bytes"));
        assert!(error("not json").contains("expected"));
    }

    #[test]
    fn notes_lose_emails_and_phone_numbers_and_are_cut() {
        assert_eq!(
            clean("mail me at fan@example.com", 200),
            "mail me at [email]"
        );
        assert_eq!(clean("(312) 555 1234 is me", 200), "[number] is me");
        assert_eq!(clean("+1 312-555-1234", 200), "[number]");
        // Short numbers such as scores stay.
        assert_eq!(clean("lost 24-17 in week 12", 200), "lost 24-17 in week 12");
        assert_eq!(clean("a\u{7}b  c", 200), "ab c");
        assert_eq!(clean(&"é".repeat(300), MAX_NOTE).chars().count(), MAX_NOTE);
    }
}
