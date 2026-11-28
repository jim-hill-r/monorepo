//! Each learner's progress, kept in the browser's `localStorage`.
//!
//! The live app keeps practice per learner name, with `Standard` when no name
//! is given. Here the learner's name is typed under Student in the settings
//! panel (`settings.rs`), and each level's `AppState` is saved under that
//! name after every tracing, so a reload, or a
//! second learner on the same device, carries on where they were. Nothing
//! leaves the device.

use crate::state::AppState;

/// The learner when no name is given, as in the live app.
pub const DEFAULT_LEARNER: &str = "Standard";

/// `localStorage` key that holds the current learner's name.
pub const LEARNER_KEY: &str = "blueeel.learner";

/// The longest name kept, in characters.
const MAX_NAME: usize = 40;

/// The learner's name as typed, trimmed and cut to `MAX_NAME` characters,
/// or `DEFAULT_LEARNER` when it is blank.
pub fn learner_name(typed: &str) -> String {
    let name: String = typed.trim().chars().take(MAX_NAME).collect();
    if name.is_empty() {
        DEFAULT_LEARNER.to_string()
    } else {
        name
    }
}

/// `localStorage` key for `learner`'s progress in `level`. Practising a
/// sequence from the settings panel keeps its own progress, apart from the
/// whole level's, so changing the sequence never mixes the two up.
pub fn progress_key(learner: &str, level: &str, sequence: Option<&[String]>) -> String {
    match sequence {
        Some(expressions) => format!(
            "blueeel.progress.{level}.{learner}#{}",
            expressions.join(",")
        ),
        None => format!("blueeel.progress.{level}.{learner}"),
    }
}

/// `value` as a JavaScript string literal.
pub fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

/// JavaScript that returns the progress saved under `key`, or "" when there
/// is none or storage is blocked (private windows).
pub fn load_js(key: &str) -> String {
    format!(
        "try {{ return localStorage.getItem({}) || ''; }} catch (e) {{ return ''; }}",
        js_string(key)
    )
}

/// JavaScript that saves `state` under `key`.
pub fn save_js(key: &str, state: &AppState) -> Option<String> {
    let json = serde_json::to_string(state).ok()?;
    Some(format!(
        "try {{ localStorage.setItem({}, {}); }} catch (e) {{}}",
        js_string(key),
        js_string(&json)
    ))
}

/// The saved progress `load_js` returned, if it can be carried on (it
/// parses and the level isn't finished).
pub fn restore(saved: &str) -> Option<AppState> {
    serde_json::from_str::<AppState>(saved)
        .ok()
        .filter(|state| state.expression.is_some() && !state.is_complete())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_name_is_the_default_learner() {
        assert_eq!(learner_name("  "), DEFAULT_LEARNER);
        assert_eq!(learner_name(" Ada "), "Ada");
        assert_eq!(learner_name(&"x".repeat(100)).chars().count(), MAX_NAME);
    }

    #[test]
    fn progress_is_kept_per_learner_level_and_sequence() {
        assert_ne!(
            progress_key("Ada", "letter", None),
            progress_key("Bo", "letter", None)
        );
        assert_ne!(
            progress_key("Ada", "letter", None),
            progress_key("Ada", "word", None)
        );
        let bc = ["b".to_string(), "c".to_string()];
        assert_eq!(
            progress_key("Ada", "letter", Some(&bc)),
            "blueeel.progress.letter.Ada#b,c"
        );
        assert_eq!(
            progress_key("Ada", "letter", None),
            "blueeel.progress.letter.Ada",
            "the whole level's key is unchanged, so saved progress carries on"
        );
    }

    #[test]
    fn saved_progress_carries_on_where_the_learner_was() {
        let mut state = AppState::new("letter");
        state.start_practice();
        state.record_attempt(true);
        state.record_attempt(false);
        let saved = serde_json::to_string(&state).expect("state serializes");
        let restored = restore(&saved).expect("progress restored");
        assert_eq!(restored.expression, state.expression);
        assert_eq!(restored.active_queue, state.active_queue);
        assert_eq!(restored.history["b"].single_attempt_successes, 1);
        assert_eq!(restored.history["c"].attempts, 1);
    }

    #[test]
    fn a_finished_level_or_bad_data_starts_afresh() {
        let mut done = AppState::new("word");
        done.start_practice();
        while !done.is_complete() {
            done.record_attempt(true);
        }
        let saved = serde_json::to_string(&done).expect("state serializes");
        assert!(restore(&saved).is_none());
        assert!(restore("not json").is_none());
        assert!(restore("").is_none());
        let unstarted = serde_json::to_string(&AppState::new("letter")).expect("serializes");
        assert!(restore(&unstarted).is_none());
    }

    #[test]
    fn keys_are_quoted_safely_in_javascript() {
        let state = AppState::new("letter");
        let js = save_js(&progress_key("x\"y", "letter", None), &state).expect("saves");
        assert!(js.contains(r#""blueeel.progress.letter.x\"y""#));
        assert!(load_js("a'b").contains(r#"getItem("a'b")"#));
        assert!(!js.contains("fetch"));
    }
}
