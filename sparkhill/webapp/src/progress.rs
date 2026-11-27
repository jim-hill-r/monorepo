//! Each learner's progress, kept in the browser's `localStorage`.
//!
//! The live app keeps practice per learner name, with `Standard` when no name
//! is given. Here a learner types their name on the home page, and each level's
//! `AppState` is saved under that name after every tracing, so a reload, or a
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

/// `localStorage` key for `learner`'s progress in `level`.
pub fn progress_key(learner: &str, level: &str) -> String {
    format!("blueeel.progress.{level}.{learner}")
}

/// `value` as a JavaScript string literal.
fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

/// JavaScript that returns `{"learner": ..., "saved": ...}` as JSON: the
/// current learner and their saved progress in `level` (or null). Storage
/// can be blocked (private windows), so it falls back to the default learner.
pub fn load_js(level: &str) -> String {
    format!(
        r#"let learner = {default}, saved = null;
try {{
  learner = localStorage.getItem({learner_key}) || {default};
  saved = localStorage.getItem('blueeel.progress.' + {level} + '.' + learner);
}} catch (e) {{}}
return JSON.stringify({{ learner, saved }});"#,
        default = js_string(DEFAULT_LEARNER),
        learner_key = js_string(LEARNER_KEY),
        level = js_string(level),
    )
}

/// JavaScript that saves `state` as `learner`'s progress in `level`.
pub fn save_js(learner: &str, level: &str, state: &AppState) -> Option<String> {
    let json = serde_json::to_string(state).ok()?;
    Some(format!(
        "try {{ localStorage.setItem({}, {}); }} catch (e) {{}}",
        js_string(&progress_key(learner, level)),
        js_string(&json)
    ))
}

/// JavaScript that makes `typed` the current learner (the default learner
/// when it is blank).
pub fn set_learner_js(typed: &str) -> String {
    if typed.trim().is_empty() {
        return format!(
            "try {{ localStorage.removeItem({}); }} catch (e) {{}}",
            js_string(LEARNER_KEY)
        );
    }
    format!(
        "try {{ localStorage.setItem({}, {}); }} catch (e) {{}}",
        js_string(LEARNER_KEY),
        js_string(&learner_name(typed))
    )
}

/// JavaScript that returns the learner's name as typed last time, or "".
pub fn get_learner_js() -> String {
    format!(
        "try {{ return localStorage.getItem({}) || ''; }} catch (e) {{ return ''; }}",
        js_string(LEARNER_KEY)
    )
}

/// What `load_js` returned: the learner, and their saved progress if it can
/// be carried on (it parses and the level isn't finished).
pub fn parse_loaded(loaded: &str) -> (String, Option<AppState>) {
    #[derive(serde::Deserialize)]
    struct Loaded {
        learner: Option<String>,
        saved: Option<String>,
    }
    let Ok(loaded) = serde_json::from_str::<Loaded>(loaded) else {
        return (DEFAULT_LEARNER.to_string(), None);
    };
    let learner = learner_name(loaded.learner.as_deref().unwrap_or(""));
    let state = loaded
        .saved
        .and_then(|json| serde_json::from_str::<AppState>(&json).ok())
        .filter(|state| state.expression.is_some() && !state.is_complete());
    (learner, state)
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
    fn progress_is_kept_per_learner_and_level() {
        assert_ne!(progress_key("Ada", "letter"), progress_key("Bo", "letter"));
        assert_ne!(progress_key("Ada", "letter"), progress_key("Ada", "word"));
    }

    fn loaded(learner: &str, state: &AppState) -> String {
        let saved = serde_json::to_string(state).expect("state serializes");
        serde_json::json!({ "learner": learner, "saved": saved }).to_string()
    }

    #[test]
    fn saved_progress_carries_on_where_the_learner_was() {
        let mut state = AppState::new("letter");
        state.start_practice();
        state.record_attempt(true);
        state.record_attempt(false);
        let (learner, restored) = parse_loaded(&loaded("Ada", &state));
        assert_eq!(learner, "Ada");
        let restored = restored.expect("progress restored");
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
        assert!(parse_loaded(&loaded("Ada", &done)).1.is_none());
        assert!(
            parse_loaded(r#"{"learner":"Ada","saved":"not json"}"#)
                .1
                .is_none()
        );
        assert!(parse_loaded(r#"{"learner":null,"saved":null}"#).1.is_none());
        assert_eq!(parse_loaded("garbage").0, DEFAULT_LEARNER);
    }

    #[test]
    fn names_are_quoted_safely_in_javascript() {
        let js = set_learner_js("a'); alert(1); ('");
        assert!(js.contains(r#""a'); alert(1); ('""#));
        let state = AppState::new("letter");
        let js = save_js("x\"y", "letter", &state).expect("saves");
        assert!(js.contains(r#""blueeel.progress.letter.x\"y""#));
    }
}
