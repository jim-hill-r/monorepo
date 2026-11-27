//! Scripted mode, for smoke runs without a person at the keyboard.
//!
//! `controlled_chaos --script smoke/smoke.script` plays the game from a
//! script instead of waiting for input. The run is the same every time:
//!
//! - **Fixed timestep:** each frame advances the clock by exactly 1/60 s,
//!   however fast the machine renders.
//! - **Seeded shuffle:** the action deck is shuffled with the script's
//!   `seed`, so both players get the same cards on every run.
//! - **Timed inputs:** each step runs at a numbered frame and does what a
//!   key or a tap would.
//! - **Screenshots** of the window at named frames, saved as PNG files.
//! - **Exit code:** 0 when every step ran and every check passed, 1 when a
//!   check failed or a screenshot couldn't be saved, 101 on a panic.
//!
//! Scripted runs never read or save the player's display settings.
//!
//! # Script format
//!
//! One step per line, `<frame> <command> [argument]`, with frames in
//! order. `#` starts a comment. An optional `seed <n>` line comes first.
//!
//! ```text
//! seed 42
//! 10 screenshot splash      # saves splash.png
//! 20 start                  # presses Start Game
//! 30 expect state in-game
//! 40 advance                # Space: draw, or end the turn
//! 50 pick 1                 # key 1: the first card in hand
//! 60 expect phase end       # draw, play, battle or end
//! 70 expect round 1
//! 80 exit
//! ```
//!
//! After the last step (or `exit`) the run waits for its screenshots to be
//! written, then quits.

// The web and Android builds have no command line, so they never run a
// script.
#![cfg_attr(any(target_arch = "wasm32", target_os = "android"), allow(dead_code))]

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;

use crate::game::{RulesEngineResource, ShuffleSeed};
use crate::input::{HAND_SLOTS, PlayerAction};
use crate::rules::{RulesEngine, TurnPhase};
use crate::state::AppState;

/// How long each frame lasts in a scripted run.
pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// Frames to wait for screenshots to be written after the last step before
/// the run fails.
pub const SCREENSHOT_TIMEOUT_FRAMES: u32 = 600;

/// Where screenshots go when `--screenshots` isn't given.
pub const DEFAULT_SCREENSHOT_DIR: &str = "target/smoke";

/// A parsed script: the shuffle seed and the steps in frame order.
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub seed: u64,
    pub steps: Vec<Step>,
}

/// One line of a script.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// The frame the step runs at, counted from 0 at launch.
    pub frame: u32,
    /// The script line it came from, for error messages.
    pub line: usize,
    pub command: Command,
}

/// What a step does.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Press Start Game on the start screen.
    Start,
    /// Press **Space**: draw a card, or end the turn.
    Advance,
    /// Press the digit key for this hand slot (0-based here, 1-based in the
    /// script, as on the keyboard).
    Pick(usize),
    /// Save the window as `<name>.png`.
    Screenshot(String),
    /// Fail the run unless the game is in this state.
    Expect(Expectation),
    /// End the script here.
    Exit,
}

/// A check on the game's state.
#[derive(Debug, Clone, PartialEq)]
pub enum Expectation {
    State(AppState),
    Phase(TurnPhase),
    Round(u32),
}

/// A script line that couldn't be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Reads a script in the format described in the module docs.
pub fn parse(text: &str) -> Result<Script, ParseError> {
    let mut script = Script {
        seed: 0,
        steps: Vec::new(),
    };
    let mut exited = false;
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let err = |message: String| ParseError { line, message };
        let words: Vec<&str> = raw
            .split('#')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .collect();
        match words.as_slice() {
            [] => {}
            ["seed", value] => {
                if !script.steps.is_empty() {
                    return Err(err("`seed` must come before the first step".into()));
                }
                script.seed = value
                    .parse()
                    .map_err(|_| err(format!("`{value}` is not a seed (a whole number)")))?;
            }
            [frame, rest @ ..] => {
                if exited {
                    return Err(err("no steps may follow `exit`".into()));
                }
                let frame: u32 = frame.parse().map_err(|_| {
                    err(format!(
                        "expected `<frame> <command>` or `seed <n>`, found `{frame}`"
                    ))
                })?;
                if let Some(last) = script.steps.last()
                    && frame < last.frame
                {
                    return Err(err(format!(
                        "frame {frame} comes before the previous step's frame {}",
                        last.frame
                    )));
                }
                let command = parse_command(rest).map_err(err)?;
                exited = command == Command::Exit;
                script.steps.push(Step {
                    frame,
                    line,
                    command,
                });
            }
        }
    }
    Ok(script)
}

fn parse_command(words: &[&str]) -> Result<Command, String> {
    match words {
        ["start"] => Ok(Command::Start),
        ["advance"] => Ok(Command::Advance),
        ["exit"] => Ok(Command::Exit),
        ["pick", slot] => match slot.parse::<usize>() {
            Ok(n @ 1..=HAND_SLOTS) => Ok(Command::Pick(n - 1)),
            _ => Err(format!("`pick` takes a hand slot from 1 to {HAND_SLOTS}")),
        },
        ["screenshot", name] => {
            let valid = name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if valid {
                Ok(Command::Screenshot((*name).to_string()))
            } else {
                Err(format!(
                    "screenshot name `{name}` may only use letters, digits, `-` and `_`"
                ))
            }
        }
        ["expect", "state", value] => match *value {
            "splash" => Ok(Command::Expect(Expectation::State(AppState::Splash))),
            "in-game" => Ok(Command::Expect(Expectation::State(AppState::InGame))),
            _ => Err(format!("unknown state `{value}` (splash or in-game)")),
        },
        ["expect", "phase", value] => {
            let phase = match *value {
                "draw" => TurnPhase::Draw,
                "play" => TurnPhase::Play,
                "battle" => TurnPhase::Battle,
                "end" => TurnPhase::End,
                _ => {
                    return Err(format!(
                        "unknown phase `{value}` (draw, play, battle or end)"
                    ));
                }
            };
            Ok(Command::Expect(Expectation::Phase(phase)))
        }
        ["expect", "round", value] => value
            .parse()
            .map(|round| Command::Expect(Expectation::Round(round)))
            .map_err(|_| format!("`{value}` is not a round number")),
        [] => Err("missing command after the frame number".into()),
        _ => Err(format!(
            "unknown command `{}` (start, advance, pick, screenshot, expect or exit)",
            words.join(" ")
        )),
    }
}

/// Checks `expectation` against the game. `Err` says what was found instead.
pub fn check(
    expectation: &Expectation,
    state: &AppState,
    engine: Option<&RulesEngine>,
) -> Result<(), String> {
    let found = match expectation {
        Expectation::State(want) if want == state => return Ok(()),
        Expectation::State(_) => format!("state {state:?}"),
        Expectation::Phase(want) => match engine {
            Some(engine) if engine.phase == *want => return Ok(()),
            Some(engine) => format!("phase {:?}", engine.phase),
            None => "no game in progress".into(),
        },
        Expectation::Round(want) => match engine {
            Some(engine) if engine.round == *want => return Ok(()),
            Some(engine) => format!("round {}", engine.round),
            None => "no game in progress".into(),
        },
    };
    Err(format!("expected {expectation:?}, found {found}"))
}

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptArgs {
    pub script: PathBuf,
    pub screenshots: PathBuf,
}

/// Reads `--script <file>` and `--screenshots <dir>` from the command line
/// (without the program name). `Ok(None)` means play normally.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<ScriptArgs>, String> {
    let mut script = None;
    let mut screenshots = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let slot = match arg.as_str() {
            "--script" => &mut script,
            "--screenshots" => &mut screenshots,
            _ => return Err(format!("unknown argument `{arg}`")),
        };
        *slot = Some(PathBuf::from(
            args.next()
                .ok_or(format!("`{arg}` needs a path after it"))?,
        ));
    }
    match (script, screenshots) {
        (Some(script), screenshots) => Ok(Some(ScriptArgs {
            script,
            screenshots: screenshots.unwrap_or_else(|| PathBuf::from(DEFAULT_SCREENSHOT_DIR)),
        })),
        (None, Some(_)) => Err("`--screenshots` only works with `--script`".into()),
        (None, None) => Ok(None),
    }
}

/// Runs a [`Script`]: fixes the timestep, seeds the shuffle and plays the
/// steps.
pub struct ScriptPlugin {
    pub script: Script,
    pub screenshots: PathBuf,
}

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME))
            .insert_resource(ShuffleSeed(self.script.seed))
            .insert_resource(ScriptRunner::new(
                self.script.clone(),
                self.screenshots.clone(),
            ))
            // PreUpdate: a step's input is read in Update of the same frame,
            // and `start` takes effect in this frame's state transition.
            .add_systems(PreUpdate, run_script);
    }
}

/// A script in progress.
#[derive(Resource)]
pub struct ScriptRunner {
    script: Script,
    next: usize,
    screenshots: PathBuf,
    /// Screenshots requested but not yet written.
    pending: Arc<AtomicUsize>,
    /// Screenshots that couldn't be saved.
    errors: Arc<Mutex<Vec<String>>>,
    /// The frame the last step ran at, or a step failed at.
    finished_at: Option<u32>,
    /// Whether a step failed.
    failed: bool,
}

impl ScriptRunner {
    pub fn new(script: Script, screenshots: PathBuf) -> Self {
        Self {
            script,
            next: 0,
            screenshots,
            pending: Arc::default(),
            errors: Arc::default(),
            finished_at: None,
            failed: false,
        }
    }

    /// Stops the script at frame `now` with a failure.
    fn fail(&mut self, now: u32, line: usize, message: &str) {
        error!("script line {line} (frame {now}): {message}");
        self.failed = true;
        self.finished_at = Some(now);
    }

    /// Screenshot errors so far.
    fn take_errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|mut errors| std::mem::take(&mut *errors))
            .unwrap_or_default()
    }
}

/// Runs every step due by this frame, then, once the script is done and its
/// screenshots are written, quits with the run's result.
#[allow(clippy::too_many_arguments)]
pub fn run_script(
    frame: Res<FrameCount>,
    mut runner: ResMut<ScriptRunner>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    engine: Option<Res<RulesEngineResource>>,
    mut screenshot_manager: Option<ResMut<ScreenshotManager>>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut actions: EventWriter<PlayerAction>,
    mut exit: EventWriter<AppExit>,
) {
    let now = frame.0;
    if let Some(finished_at) = runner.finished_at {
        let errors = runner.take_errors();
        for error in &errors {
            error!("script: {error}");
        }
        if runner.failed || !errors.is_empty() {
            exit.send(AppExit::error());
        } else if runner.pending.load(Ordering::SeqCst) == 0 {
            info!("script: finished at frame {finished_at}, every check passed");
            exit.send(AppExit::Success);
        } else if now > finished_at + SCREENSHOT_TIMEOUT_FRAMES {
            error!(
                "script: screenshots still not written after {SCREENSHOT_TIMEOUT_FRAMES} frames"
            );
            exit.send(AppExit::error());
        }
        return;
    }

    while let Some(step) = runner.script.steps.get(runner.next).cloned() {
        if step.frame > now {
            return;
        }
        runner.next += 1;
        match &step.command {
            Command::Start => next_state.set(AppState::InGame),
            Command::Advance => {
                actions.send(PlayerAction::Advance);
            }
            Command::Pick(slot) => {
                actions.send(PlayerAction::PickCard(*slot));
            }
            Command::Screenshot(name) => {
                let path = runner.screenshots.join(format!("{name}.png"));
                let result = match (screenshot_manager.as_deref_mut(), window.get_single()) {
                    (Some(manager), Ok(window)) => {
                        request_screenshot(manager, window, &runner, path)
                    }
                    // Headless (the tests): there is nothing to capture.
                    _ => {
                        warn!("script line {}: no window, so no screenshot", step.line);
                        Ok(())
                    }
                };
                if let Err(message) = result {
                    runner.fail(now, step.line, &message);
                    return;
                }
            }
            Command::Expect(expectation) => {
                if let Err(message) = check(expectation, &state, engine.as_deref().map(|e| &e.0)) {
                    runner.fail(now, step.line, &message);
                    return;
                }
            }
            Command::Exit => break,
        }
    }
    runner.finished_at = Some(now);
}

/// Asks Bevy for a screenshot of `window`, written to `path` once the frame
/// renders.
fn request_screenshot(
    manager: &mut ScreenshotManager,
    window: Entity,
    runner: &ScriptRunner,
    path: PathBuf,
) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|err| format!("couldn't create {}: {err}", dir.display()))?;
    }
    let pending = Arc::clone(&runner.pending);
    let errors = Arc::clone(&runner.errors);
    pending.fetch_add(1, Ordering::SeqCst);
    let requested = manager.take_screenshot(window, move |image| {
        match save_png(image, &path) {
            Ok(()) => info!("script: saved {}", path.display()),
            Err(err) => {
                if let Ok(mut errors) = errors.lock() {
                    errors.push(err);
                }
            }
        }
        pending.fetch_sub(1, Ordering::SeqCst);
    });
    requested.map_err(|_| {
        runner.pending.fetch_sub(1, Ordering::SeqCst);
        "a screenshot was already taken this frame; give each one its own frame".to_string()
    })
}

fn save_png(image: Image, path: &Path) -> Result<(), String> {
    let image = image
        .try_into_dynamic()
        .map_err(|err| format!("couldn't read the screenshot for {}: {err}", path.display()))?;
    // Drop alpha, as Bevy's own save_screenshot_to_disk does.
    image
        .to_rgb8()
        .save(path)
        .map_err(|err| format!("couldn't save {}: {err}", path.display()))
}

/// Makes a panic end a scripted run with exit code 101 at once, even when it
/// happens on a worker thread Bevy would otherwise keep running past.
pub fn exit_on_panic() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_hook(info);
        std::process::exit(101);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Player;

    fn args(list: &[&str]) -> Result<Option<ScriptArgs>, String> {
        parse_args(list.iter().map(|s| s.to_string()))
    }

    fn parse_err(text: &str) -> ParseError {
        match parse(text) {
            Err(err) => err,
            Ok(script) => panic!("expected an error, parsed {script:?}"),
        }
    }

    #[test]
    fn parses_every_command() -> Result<(), ParseError> {
        let script = parse(
            "# a comment\n\
             seed 42\n\
             \n\
             5 screenshot splash-1  # trailing comment\n\
             10 start\n\
             12 expect state in-game\n\
             20 advance\n\
             20 pick 8\n\
             25 expect phase battle\n\
             26 expect round 3\n\
             30 exit\n",
        )?;
        assert_eq!(script.seed, 42);
        let commands: Vec<_> = script
            .steps
            .iter()
            .map(|s| (s.frame, s.line, s.command.clone()))
            .collect();
        assert_eq!(
            commands,
            [
                (5, 4, Command::Screenshot("splash-1".into())),
                (10, 5, Command::Start),
                (12, 6, Command::Expect(Expectation::State(AppState::InGame))),
                (20, 7, Command::Advance),
                (20, 8, Command::Pick(7)),
                (
                    25,
                    9,
                    Command::Expect(Expectation::Phase(TurnPhase::Battle))
                ),
                (26, 10, Command::Expect(Expectation::Round(3))),
                (30, 11, Command::Exit),
            ]
        );
        Ok(())
    }

    #[test]
    fn seed_defaults_to_zero() -> Result<(), ParseError> {
        assert_eq!(parse("1 start")?.seed, 0);
        Ok(())
    }

    #[test]
    fn rejects_frames_out_of_order() {
        let err = parse_err("10 start\n5 advance");
        assert_eq!(err.line, 2);
        assert!(err.message.contains("before"), "{err}");
    }

    #[test]
    fn rejects_steps_after_exit() {
        assert_eq!(parse_err("1 exit\n2 start").line, 2);
    }

    #[test]
    fn rejects_seed_after_a_step() {
        assert_eq!(parse_err("1 start\nseed 3").line, 2);
    }

    #[test]
    fn rejects_bad_arguments() {
        for text in [
            "1 pick 0",
            "1 pick 9",
            "1 pick x",
            "1 screenshot ../escape",
            "1 expect state paused",
            "1 expect phase lunch",
            "1 expect round x",
            "1 dance",
            "1",
            "start",
            "seed x",
        ] {
            assert_eq!(parse_err(text).line, 1, "{text}");
        }
    }

    #[test]
    fn check_compares_state_phase_and_round() {
        let engine = RulesEngine::new(Player::new("A", 20), Player::new("B", 20));
        let in_game = AppState::InGame;
        assert!(check(&Expectation::State(AppState::InGame), &in_game, None).is_ok());
        assert!(check(&Expectation::State(AppState::Splash), &in_game, None).is_err());
        assert!(
            check(
                &Expectation::Phase(TurnPhase::Draw),
                &in_game,
                Some(&engine)
            )
            .is_ok()
        );
        assert!(check(&Expectation::Round(1), &in_game, Some(&engine)).is_ok());
        let err = check(&Expectation::Round(2), &in_game, Some(&engine));
        assert_eq!(err, Err("expected Round(2), found round 1".into()));
        assert!(check(&Expectation::Phase(TurnPhase::Draw), &in_game, None).is_err());
    }

    /// Runs `text` headless (no window, so no screenshots) through the real
    /// game plugins and returns how the app asked to exit.
    fn run_headless(text: &str) -> AppExit {
        let script = match parse(text) {
            Ok(script) => script,
            Err(err) => panic!("script doesn't parse: {err}"),
        };
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::input::InputPlugin,
        ))
        .init_resource::<crate::theme::GameTheme>()
        .add_plugins(ScriptPlugin {
            script,
            screenshots: PathBuf::from(DEFAULT_SCREENSHOT_DIR),
        })
        .init_state::<AppState>()
        .add_plugins(crate::game::GamePlugin);
        for _ in 0..2_000 {
            app.update();
            if let Some(exit) = app.should_exit() {
                return exit;
            }
        }
        panic!("the script never finished");
    }

    #[test]
    fn the_smoke_script_passes() {
        assert_eq!(
            run_headless(include_str!("../smoke/smoke.script")),
            AppExit::Success
        );
    }

    #[test]
    fn a_failed_check_exits_with_an_error() {
        assert_eq!(
            run_headless("5 start\n10 expect phase battle\n20 exit"),
            AppExit::error()
        );
    }

    #[test]
    fn a_script_without_exit_finishes_after_its_last_step() {
        assert_eq!(run_headless("5 start\n10 advance"), AppExit::Success);
    }

    #[test]
    fn no_arguments_plays_normally() {
        assert_eq!(args(&[]), Ok(None));
    }

    #[test]
    fn script_argument_uses_the_default_screenshot_folder() {
        assert_eq!(
            args(&["--script", "s.script"]),
            Ok(Some(ScriptArgs {
                script: "s.script".into(),
                screenshots: DEFAULT_SCREENSHOT_DIR.into(),
            }))
        );
    }

    #[test]
    fn screenshots_argument_sets_the_folder() {
        assert_eq!(
            args(&["--screenshots", "out", "--script", "s.script"]),
            Ok(Some(ScriptArgs {
                script: "s.script".into(),
                screenshots: "out".into(),
            }))
        );
    }

    #[test]
    fn bad_arguments_are_errors() {
        assert!(args(&["--script"]).is_err());
        assert!(args(&["--screenshots", "out"]).is_err());
        assert!(args(&["--fast"]).is_err());
    }
}
