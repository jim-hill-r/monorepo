//! The settings panel, behind the top-right button as on the live app:
//! Student (the learner's name), Sequence (the letters or words to
//! practise) and Technique.
//!
//! Each is kept in this browser's `localStorage`: the name under
//! `progress::LEARNER_KEY`, the technique under `technique::TECHNIQUE_KEY`
//! and the sequence under `SEQUENCE_KEY`. Nothing leaves the device, so the
//! live app's Upload Practice switch, which sends tracings to S3, is left
//! out (SPRK-006).

use crate::progress::{DEFAULT_LEARNER, LEARNER_KEY, js_string};
use crate::state::known_expressions;
use crate::technique::{TECHNIQUE_KEY, Technique};

/// `localStorage` key that holds the sequence as typed.
pub const SEQUENCE_KEY: &str = "blueeel.sequence";

/// The longest sequence kept, in characters.
const MAX_SEQUENCE: usize = 200;

/// What the settings panel holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The learner's name as typed ("" for the default learner).
    pub student: String,
    /// The letters and words to practise, as typed ("" for the whole course).
    pub sequence: String,
    pub technique: Technique,
}

impl Settings {
    /// The settings `load_js` returned, with defaults for anything missing.
    pub fn parse(loaded: &str) -> Settings {
        #[derive(serde::Deserialize)]
        struct Loaded {
            student: Option<String>,
            sequence: Option<String>,
            technique: Option<String>,
        }
        let Ok(loaded) = serde_json::from_str::<Loaded>(loaded) else {
            return Settings::default();
        };
        Settings {
            student: loaded.student.unwrap_or_default(),
            sequence: loaded.sequence.unwrap_or_default(),
            technique: Technique::parse(loaded.technique.as_deref().unwrap_or(""))
                .unwrap_or_default(),
        }
    }

    /// JavaScript that saves these settings. A blank name or sequence is
    /// removed, so the default learner and the whole course come back.
    pub fn save_js(&self) -> String {
        let student = self.student.trim();
        let sequence: String = self.sequence.trim().chars().take(MAX_SEQUENCE).collect();
        let mut js = String::from("try {");
        js.push_str(&set_or_remove(
            LEARNER_KEY,
            &crate::progress::learner_name(student),
            student.is_empty(),
        ));
        js.push_str(&set_or_remove(SEQUENCE_KEY, &sequence, sequence.is_empty()));
        js.push_str(&format!(
            " localStorage.setItem({}, {});",
            js_string(TECHNIQUE_KEY),
            js_string(self.technique.as_str())
        ));
        js.push_str(" } catch (e) {}");
        js
    }
}

/// One `localStorage` call: set `key` to `value`, or remove it.
fn set_or_remove(key: &str, value: &str, remove: bool) -> String {
    if remove {
        format!(" localStorage.removeItem({});", js_string(key))
    } else {
        format!(
            " localStorage.setItem({}, {});",
            js_string(key),
            js_string(value)
        )
    }
}

/// JavaScript that returns the saved settings as JSON
/// (`{"student": ..., "sequence": ..., "technique": ...}`), with empty
/// values when storage is blocked (private windows).
pub fn load_js() -> String {
    format!(
        r#"let student = '', sequence = '', technique = '';
try {{
  student = localStorage.getItem({learner}) || '';
  sequence = localStorage.getItem({sequence}) || '';
  technique = localStorage.getItem({technique}) || '';
}} catch (e) {{}}
return JSON.stringify({{ student, sequence, technique }});"#,
        learner = js_string(LEARNER_KEY),
        sequence = js_string(SEQUENCE_KEY),
        technique = js_string(TECHNIQUE_KEY),
    )
}

/// The learner the settings name: the name typed, or the default learner.
pub fn learner(settings: &Settings) -> String {
    if settings.student.trim().is_empty() {
        DEFAULT_LEARNER.to_string()
    } else {
        crate::progress::learner_name(&settings.student)
    }
}

/// The part of `sequence` that `level` practises, or `None` when it has
/// none, so the level's own groups are used. Entries are split on commas
/// and spaces; single lowercase letters go to the letter level, single
/// capitals to the capital level, and longer entries to the word level.
/// Anything without a pattern is ignored.
pub fn sequence_for_level(sequence: &str, level: &str) -> Option<Vec<String>> {
    let entries = known_expressions(sequence.split(|c: char| c == ',' || c.is_whitespace()));
    let belongs = |expr: &String| {
        let single = expr.chars().count() == 1;
        match level {
            "word" => !single,
            "capital" => single && expr.chars().all(|c| c.is_ascii_uppercase()),
            _ => single && expr.chars().all(|c| c.is_ascii_lowercase()),
        }
    };
    let chosen: Vec<String> = entries.into_iter().filter(belongs).collect();
    (!chosen.is_empty()).then_some(chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_sequence_is_split_between_the_levels() {
        let typed = "b, d hi,T fun ,B";
        assert_eq!(
            sequence_for_level(typed, "letter"),
            Some(strings(&["b", "d"]))
        );
        assert_eq!(
            sequence_for_level(typed, "word"),
            Some(strings(&["hi", "fun"]))
        );
        assert_eq!(
            sequence_for_level(typed, "capital"),
            Some(strings(&["T", "B"]))
        );
    }

    #[test]
    fn a_level_the_sequence_leaves_out_practises_its_own_groups() {
        assert_eq!(sequence_for_level("", "letter"), None);
        assert_eq!(sequence_for_level("b, c", "word"), None);
        assert_eq!(sequence_for_level("hi", "capital"), None);
    }

    #[test]
    fn a_sequence_ignores_repeats_and_anything_without_a_pattern() {
        assert_eq!(
            sequence_for_level("b,b, 3, é, <x>, c", "letter"),
            Some(strings(&["b", "c"]))
        );
        assert_eq!(sequence_for_level("1, !, ??", "letter"), None);
    }

    #[test]
    fn missing_or_bad_settings_are_the_defaults() {
        assert_eq!(Settings::parse("garbage"), Settings::default());
        assert_eq!(Settings::parse("{}"), Settings::default());
        let settings =
            Settings::parse(r#"{"student":"Ada","sequence":"b, c","technique":"Pattern"}"#);
        assert_eq!(settings.student, "Ada");
        assert_eq!(settings.sequence, "b, c");
        assert_eq!(settings.technique, Technique::Pattern);
        assert_eq!(
            Settings::parse(r#"{"technique":"<script>"}"#).technique,
            Technique::Tracing
        );
    }

    #[test]
    fn a_blank_name_is_the_default_learner() {
        assert_eq!(learner(&Settings::default()), DEFAULT_LEARNER);
        let ada = Settings {
            student: " Ada ".to_string(),
            ..Settings::default()
        };
        assert_eq!(learner(&ada), "Ada");
    }

    #[test]
    fn saving_stays_on_the_device_and_quotes_safely() {
        let settings = Settings {
            student: "a'); alert(1); ('".to_string(),
            sequence: "b, c".to_string(),
            technique: Technique::Freeform,
        };
        let js = settings.save_js();
        assert!(js.contains(r#"localStorage.setItem("blueeel.learner", "a'); alert(1); ('")"#));
        assert!(js.contains(r#"localStorage.setItem("blueeel.sequence", "b, c")"#));
        assert!(js.contains(r#"localStorage.setItem("blueeel.technique", "Freeform")"#));
        assert!(!js.contains("fetch"));
        assert!(!js.contains("XMLHttpRequest"));
    }

    #[test]
    fn a_blank_name_or_sequence_is_removed() {
        let js = Settings::default().save_js();
        assert!(js.contains(r#"localStorage.removeItem("blueeel.learner")"#));
        assert!(js.contains(r#"localStorage.removeItem("blueeel.sequence")"#));
    }

    #[test]
    fn a_long_sequence_is_cut_short() {
        let settings = Settings {
            sequence: "b,".repeat(500),
            ..Settings::default()
        };
        let js = settings.save_js();
        let saved = js
            .split(r#"setItem("blueeel.sequence", ""#)
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the sequence is saved");
        assert_eq!(saved.chars().count(), MAX_SEQUENCE);
    }
}
