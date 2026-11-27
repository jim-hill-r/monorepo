//! Letter stroke patterns for the tracing demonstrations, bundled with the app.
//!
//! `patterns.json` holds one pattern per lowercase letter, in the format the
//! canvas's `drawLetter` takes: the letter, its guide-line `boundary` and a
//! `path` of points where `type` marks each stroke's `start` and `end`. The
//! strokes come from the public-domain Hershey Simplex Roman font, mapped onto
//! the canvas guide lines and put into handwriting stroke order (README.md).
//! They used to be fetched at runtime from Blue Eel's AWS S3 bucket.

use serde_json::{Map, Value};
use std::sync::LazyLock;

const PATTERNS_JSON: &str = include_str!("patterns.json");

static PATTERNS: LazyLock<Map<String, Value>> =
    LazyLock::new(|| serde_json::from_str(PATTERNS_JSON).unwrap_or_default());

/// The patterns for each letter of `expression` (a letter or a word) as a
/// JSON array, or `None` when a letter has no pattern.
pub fn patterns_json(expression: &str) -> Option<String> {
    let patterns = expression
        .chars()
        .map(|c| PATTERNS.get(&c.to_string()).cloned())
        .collect::<Option<Vec<Value>>>()?;
    if patterns.is_empty() {
        return None;
    }
    serde_json::to_string(&patterns).ok()
}

/// JavaScript that animates the demonstration of `expression` on the canvas,
/// or marks the animation done at once when there is no pattern for it.
pub fn demonstration_js(expression: &str) -> String {
    match patterns_json(expression) {
        // A JSON array is a valid JavaScript literal.
        Some(json) => format!("window.eelCanvas.drawLetter(JSON.stringify({json}), 'Tracing');"),
        None => "window._eelAnimDone = true;".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{DEFAULT_LETTER_SEQUENCES, DEFAULT_WORD_SEQUENCES};

    #[test]
    fn every_letter_has_a_pattern() {
        for c in 'a'..='z' {
            assert!(
                patterns_json(&c.to_string()).is_some(),
                "no pattern for {c}"
            );
        }
    }

    #[test]
    fn every_practice_expression_has_patterns() {
        for expr in DEFAULT_LETTER_SEQUENCES
            .iter()
            .chain(DEFAULT_WORD_SEQUENCES)
            .flat_map(|s| s.iter())
        {
            assert!(patterns_json(expr).is_some(), "no pattern for {expr}");
        }
    }

    #[test]
    fn each_stroke_starts_and_ends() {
        for (letter, pattern) in PATTERNS.iter() {
            let path = pattern["path"].as_array().expect("path is an array");
            let kinds: Vec<&str> = path
                .iter()
                .filter_map(|p| p.get("type").and_then(Value::as_str))
                .collect();
            assert!(!kinds.is_empty(), "{letter} has no strokes");
            for pair in kinds.chunks(2) {
                assert_eq!(pair, ["start", "end"], "{letter} strokes don't pair up");
            }
            assert_eq!(pattern["letter"], Value::String(letter.clone()));
            assert!(pattern["boundary"]["capLine"].is_number());
            assert!(pattern["boundary"]["baseLine"].is_number());
        }
    }

    #[test]
    fn a_word_combines_its_letters() {
        let json = patterns_json("hi").expect("patterns for hi");
        let parsed: Vec<Value> = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[1]["letter"], "i");
    }

    #[test]
    fn unknown_characters_fall_back() {
        assert_eq!(patterns_json("A"), None);
        assert_eq!(patterns_json(""), None);
        assert_eq!(demonstration_js("?"), "window._eelAnimDone = true;");
        assert!(demonstration_js("a").starts_with("window.eelCanvas.drawLetter("));
    }

    #[test]
    fn no_runtime_fetch() {
        assert!(!demonstration_js("a").contains("fetch"));
    }
}
