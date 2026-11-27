//! Gameplay events from the web build to the repo's `telemetry` Worker
//! (`../telemetry/README.md`), so we can see whether people play and where
//! the game breaks.
//!
//! The web build sends, with `navigator.sendBeacon`:
//!
//! | Event | When | `value` |
//! | --- | --- | --- |
//! | `game` `session_start` | the page loads the game | |
//! | `game` `game_start` | Start Game is pressed | |
//! | `game` `round` | every [`ROUND_STEP`]th round is reached | the round |
//! | `game` `game_over` | the game ends | the last round |
//! | `game` `quit` | the page is closed or left (`pagehide`) | the round reached, 0 before a game |
//! | `error` | the game panics | (the panic message) |
//!
//! Nothing identifies the player: no ids, cookies or storage. The Worker
//! drops events from browsers that send Do Not Track or Global Privacy
//! Control and never stores IP addresses. Desktop and Android builds send
//! nothing.

// Only the web build has a transport; the event logic is tested natively.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;

use crate::game::RulesEngineResource;
use crate::state::AppState;

/// The Worker's event endpoint.
pub const ENDPOINT: &str = "https://jimhillr-telemetry.jim-hill-r.workers.dev/event";

/// The `project` every event carries.
pub const PROJECT: &str = "controlled_chaos";

/// A `round` event is sent every this many rounds, so a long game stays
/// well under the Worker's rate limit (about one event a second).
pub const ROUND_STEP: u32 = 5;

/// Panic messages are cut to this many characters (the Worker keeps 500).
const MAX_MESSAGE: usize = 500;

/// Something worth counting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    SessionStart,
    GameStart,
    Round(u32),
    GameOver(u32),
    Quit(u32),
    Panic(String),
}

impl GameEvent {
    /// The event as the JSON body the Worker accepts. `build` is the commit
    /// the game was built from, when known.
    pub fn to_json(&self, build: Option<&str>) -> String {
        let mut json = String::from("{");
        let (name, value) = match self {
            GameEvent::Panic(message) => {
                let message: String = message.chars().take(MAX_MESSAGE).collect();
                let _ = write!(
                    json,
                    r#""kind":"error","project":"{PROJECT}","page":"game","message":{}"#,
                    quote(&message)
                );
                (None, None)
            }
            GameEvent::SessionStart => (Some("session_start"), None),
            GameEvent::GameStart => (Some("game_start"), None),
            GameEvent::Round(round) => (Some("round"), Some(*round)),
            GameEvent::GameOver(round) => (Some("game_over"), Some(*round)),
            GameEvent::Quit(round) => (Some("quit"), Some(*round)),
        };
        if let Some(name) = name {
            let _ = write!(
                json,
                r#""kind":"game","project":"{PROJECT}","name":"{name}""#
            );
        }
        if let Some(value) = value {
            let _ = write!(json, r#","value":{value}"#);
        }
        if let Some(build) = build {
            let _ = write!(json, r#","build":{}"#, quote(build));
        }
        json.push('}');
        json
    }
}

/// `text` as a JSON string literal.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(quoted, "\\u{:04x}", c as u32);
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// The commit to tag events with: the first 12 characters of `sha` when it
/// looks like a git commit (CI sets `GITHUB_SHA` when it builds the game).
pub fn build_id(sha: Option<&str>) -> Option<String> {
    let sha = sha?;
    (sha.len() >= 12 && sha.chars().all(|c| c.is_ascii_hexdigit())).then(|| sha[..12].to_string())
}

/// Sends one event body. The web build posts it to [`ENDPOINT`]; tests
/// record it.
#[derive(Resource, Clone)]
pub struct TelemetrySink(pub Arc<dyn Fn(&str) + Send + Sync>);

/// Sends the gameplay events in the module docs through `sink`.
pub struct TelemetryPlugin {
    pub sink: TelemetrySink,
    pub build: Option<String>,
}

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Telemetry {
            sink: self.sink.clone(),
            build: self.build.clone(),
            round: Arc::default(),
            game_over_sent: false,
        })
        .add_systems(Startup, send_session_start)
        .add_systems(OnEnter(AppState::InGame), send_game_start)
        .add_systems(
            PostUpdate,
            track_progress.run_if(in_state(AppState::InGame)),
        );
    }
}

/// What has been sent so far.
#[derive(Resource)]
pub struct Telemetry {
    sink: TelemetrySink,
    build: Option<String>,
    /// The round reached, shared with the `pagehide` handler.
    round: Arc<AtomicU32>,
    game_over_sent: bool,
}

impl Telemetry {
    pub fn send(&self, event: &GameEvent) {
        (self.sink.0)(&event.to_json(self.build.as_deref()));
    }

    /// The round reached, kept up to date for the `quit` event.
    pub fn round(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.round)
    }
}

fn send_session_start(telemetry: Res<Telemetry>) {
    telemetry.send(&GameEvent::SessionStart);
}

fn send_game_start(mut telemetry: ResMut<Telemetry>) {
    telemetry.round.store(0, Ordering::Relaxed);
    telemetry.game_over_sent = false;
    telemetry.send(&GameEvent::GameStart);
}

/// Sends `round` every [`ROUND_STEP`] rounds and `game_over` once.
fn track_progress(mut telemetry: ResMut<Telemetry>, engine: Option<Res<RulesEngineResource>>) {
    let Some(engine) = engine else {
        return;
    };
    let round = engine.0.round;
    let previous = telemetry.round.swap(round, Ordering::Relaxed);
    if round > previous && previous > 0 && round % ROUND_STEP == 0 {
        telemetry.send(&GameEvent::Round(round));
    }
    if engine.0.is_game_over() && !telemetry.game_over_sent {
        telemetry.game_over_sent = true;
        telemetry.send(&GameEvent::GameOver(round));
    }
}

/// The web build's telemetry: posts with `navigator.sendBeacon`, sends
/// `quit` on `pagehide` and `error` on a panic.
#[cfg(target_arch = "wasm32")]
pub fn web_plugin() -> TelemetryPlugin {
    fn beacon(body: &str) {
        if let Some(window) = web_sys::window() {
            // Best effort: a dropped event is fine.
            let _ = window
                .navigator()
                .send_beacon_with_opt_str(ENDPOINT, Some(body));
        }
    }

    let build = build_id(option_env!("GITHUB_SHA"));
    let plugin = TelemetryPlugin {
        sink: TelemetrySink(Arc::new(beacon)),
        build: build.clone(),
    };

    let panic_build = build.clone();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        beacon(&GameEvent::Panic(info.to_string()).to_json(panic_build.as_deref()));
        default_hook(info);
    }));

    plugin
}

/// Registers the `pagehide` handler that sends `quit`. Runs at startup,
/// once [`Telemetry`] exists.
#[cfg(target_arch = "wasm32")]
pub fn send_quit_on_pagehide(telemetry: Res<Telemetry>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let Some(window) = web_sys::window() else {
        return;
    };
    let round = telemetry.round();
    let sink = telemetry.sink.clone();
    let build = telemetry.build.clone();
    let on_pagehide = Closure::<dyn FnMut()>::new(move || {
        let event = GameEvent::Quit(round.load(Ordering::Relaxed));
        (sink.0)(&event.to_json(build.as_deref()));
    });
    let _ =
        window.add_event_listener_with_callback("pagehide", on_pagehide.as_ref().unchecked_ref());
    // The handler lives as long as the page.
    on_pagehide.forget();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn game_events_name_the_project_and_carry_a_value() {
        assert_eq!(
            GameEvent::SessionStart.to_json(None),
            r#"{"kind":"game","project":"controlled_chaos","name":"session_start"}"#
        );
        assert_eq!(
            GameEvent::Round(10).to_json(Some("0123456789ab")),
            r#"{"kind":"game","project":"controlled_chaos","name":"round","value":10,"build":"0123456789ab"}"#
        );
        assert_eq!(
            GameEvent::Quit(0).to_json(None),
            r#"{"kind":"game","project":"controlled_chaos","name":"quit","value":0}"#
        );
    }

    #[test]
    fn a_panic_is_an_error_event_with_the_message_escaped() {
        assert_eq!(
            GameEvent::Panic("bad \"card\"\n\\ at x.rs:1\u{1}".into()).to_json(None),
            r#"{"kind":"error","project":"controlled_chaos","page":"game","message":"bad \"card\"\n\\ at x.rs:1\u0001"}"#
        );
    }

    #[test]
    fn a_long_panic_message_is_cut() {
        let json = GameEvent::Panic("x".repeat(5_000)).to_json(None);
        assert!(json.len() < 4 * 1024, "the Worker refuses bodies over 4 KB");
        assert!(json.contains(&"x".repeat(MAX_MESSAGE)));
        assert!(!json.contains(&"x".repeat(MAX_MESSAGE + 1)));
    }

    #[test]
    fn build_id_keeps_twelve_characters_of_a_commit() {
        let sha = "4ca9b3d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f70";
        assert_eq!(build_id(Some(sha)), Some("4ca9b3d0e1f2".into()));
        assert_eq!(build_id(Some("not a sha at all")), None);
        assert_eq!(build_id(Some("abc")), None);
        assert_eq!(build_id(None), None);
    }

    /// Plays `script` headless with telemetry recording into a list, and
    /// returns the events sent.
    fn events_from(script: &str) -> Vec<String> {
        let sent: Arc<Mutex<Vec<String>>> = Arc::default();
        let recorder = Arc::clone(&sent);
        let script = match crate::script::parse(script) {
            Ok(script) => script,
            Err(err) => panic!("bad test script: {err}"),
        };
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::input::InputPlugin,
        ))
        .init_resource::<crate::theme::GameTheme>()
        .add_plugins(crate::script::ScriptPlugin {
            script,
            screenshots: "target/smoke".into(),
        })
        .add_plugins(TelemetryPlugin {
            sink: TelemetrySink(Arc::new(move |body| {
                if let Ok(mut sent) = recorder.lock() {
                    sent.push(body.to_string());
                }
            })),
            build: None,
        })
        .init_state::<AppState>()
        .add_plugins(crate::game::GamePlugin);
        for _ in 0..2_000 {
            app.update();
            if app.should_exit().is_some() {
                break;
            }
        }
        sent.lock().map(|sent| sent.clone()).unwrap_or_default()
    }

    fn names(events: &[String]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| e.split(r#""name":""#).nth(1)?.split('"').next())
            .collect()
    }

    #[test]
    fn a_session_sends_session_start_then_game_start() {
        let events = events_from(include_str!("../smoke/smoke.script"));
        assert_eq!(names(&events), ["session_start", "game_start"]);
    }

    #[test]
    fn no_game_start_before_start_is_pressed() {
        let events = events_from("10 expect state splash");
        assert_eq!(names(&events), ["session_start"]);
    }

    #[test]
    fn game_over_is_sent_once_with_the_round() {
        let sent: Arc<Mutex<Vec<String>>> = Arc::default();
        let recorder = Arc::clone(&sent);
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::input::InputPlugin,
        ))
        .init_resource::<crate::theme::GameTheme>()
        .add_plugins(TelemetryPlugin {
            sink: TelemetrySink(Arc::new(move |body| {
                if let Ok(mut sent) = recorder.lock() {
                    sent.push(body.to_string());
                }
            })),
            build: None,
        })
        .init_state::<AppState>()
        .add_plugins(crate::game::GamePlugin);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();
        app.world_mut()
            .resource_mut::<RulesEngineResource>()
            .0
            .signal_deck_exhausted();
        app.update();
        app.update();
        let events = sent.lock().map(|s| s.clone()).unwrap_or_default();
        let over: Vec<&String> = events.iter().filter(|e| e.contains("game_over")).collect();
        assert_eq!(
            over,
            [r#"{"kind":"game","project":"controlled_chaos","name":"game_over","value":1}"#]
        );
    }

    #[test]
    fn round_events_come_every_round_step() {
        // Ten full rounds (five turns each side) of draw, attack, defend, end.
        let mut script = String::from("seed 1\n5 start\n");
        let mut frame = 10;
        for _ in 0..10 {
            for command in ["advance", "pick 1", "advance", "pick 1"] {
                script.push_str(&format!("{frame} {command}\n"));
                frame += 5;
            }
        }
        let events = events_from(&script);
        let rounds: Vec<&String> = events.iter().filter(|e| e.contains(r#""round""#)).collect();
        assert!(!rounds.is_empty(), "no round events in {events:?}");
        for event in rounds {
            let value: u32 = event
                .split(r#""value":"#)
                .nth(1)
                .and_then(|v| v.trim_end_matches('}').parse().ok())
                .unwrap_or(1);
            assert_eq!(value % ROUND_STEP, 0, "{event}");
        }
    }
}
