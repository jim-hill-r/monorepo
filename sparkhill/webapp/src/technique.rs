//! The practice technique: how much of a letter the demonstration shows.
//!
//! The live app (blue.eel.education) has three, picked under Technique in
//! its settings panel (`settings.rs`) or with a `?tech=` query:
//!
//! - **Tracing** animates every stroke, start dot to end dot.
//! - **Pattern** shows only each stroke's start (green) and end (orange)
//!   dots, which stay on the canvas while the learner draws between them.
//! - **Freeform** shows nothing: the learner writes the letter from memory.
//!
//! The choice is kept in this browser's `localStorage` under `TECHNIQUE_KEY`,
//! with the other settings. Nothing leaves the device.

/// How the demonstration shows the letter, and what stays on the canvas
/// while the learner draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Technique {
    #[default]
    Tracing,
    Pattern,
    Freeform,
}

/// `localStorage` key that holds the chosen technique.
pub const TECHNIQUE_KEY: &str = "blueeel.technique";

impl Technique {
    /// Every technique, in the order the picker lists them.
    pub const ALL: [Technique; 3] = [Technique::Tracing, Technique::Pattern, Technique::Freeform];

    /// The name the live app and the canvas script use.
    pub fn as_str(self) -> &'static str {
        match self {
            Technique::Tracing => "Tracing",
            Technique::Pattern => "Pattern",
            Technique::Freeform => "Freeform",
        }
    }

    /// The technique named `name`, ignoring case and spaces around it.
    pub fn parse(name: &str) -> Option<Technique> {
        let name = name.trim();
        Technique::ALL
            .into_iter()
            .find(|t| t.as_str().eq_ignore_ascii_case(name))
    }

    /// The technique a `?tech=Pattern` query asks for, as in the live app.
    pub fn from_query(query: &str) -> Option<Technique> {
        query
            .trim_start_matches('?')
            .split('&')
            .find_map(|pair| pair.strip_prefix("tech="))
            .and_then(Technique::parse)
    }

    /// The technique to practise with: the query's, else the saved one,
    /// else Tracing.
    pub fn choose(query: &str, saved: &str) -> Technique {
        Technique::from_query(query)
            .or_else(|| Technique::parse(saved))
            .unwrap_or_default()
    }

    /// JavaScript that sets the canvas up for the learner's drawing: a clean
    /// canvas, with the stroke dots put back for Pattern.
    pub fn drawing_canvas_js(self) -> &'static str {
        match self {
            Technique::Pattern => "window.eelCanvas.showMarks();",
            Technique::Tracing | Technique::Freeform => "window.eelCanvas.clear();",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for t in Technique::ALL {
            assert_eq!(Technique::parse(t.as_str()), Some(t));
        }
        assert_eq!(Technique::parse(" pattern "), Some(Technique::Pattern));
        assert_eq!(Technique::parse("FREEFORM"), Some(Technique::Freeform));
        assert_eq!(Technique::parse(""), None);
        assert_eq!(Technique::parse("dots"), None);
    }

    #[test]
    fn the_query_names_the_technique() {
        assert_eq!(
            Technique::from_query("?tech=Pattern"),
            Some(Technique::Pattern)
        );
        assert_eq!(
            Technique::from_query("?practice=b&tech=freeform"),
            Some(Technique::Freeform)
        );
        assert_eq!(Technique::from_query("?tech="), None);
        assert_eq!(Technique::from_query("?technique=Pattern"), None);
        assert_eq!(Technique::from_query(""), None);
    }

    #[test]
    fn the_query_wins_over_the_saved_technique_and_tracing_is_the_default() {
        assert_eq!(
            Technique::choose("?tech=Freeform", "Pattern"),
            Technique::Freeform
        );
        assert_eq!(Technique::choose("", "Pattern"), Technique::Pattern);
        assert_eq!(
            Technique::choose("?tech=nope", "Pattern"),
            Technique::Pattern
        );
        assert_eq!(Technique::choose("", ""), Technique::Tracing);
        assert_eq!(Technique::choose("", "<script>"), Technique::Tracing);
    }

    #[test]
    fn only_pattern_keeps_the_dots_while_drawing() {
        assert!(Technique::Pattern.drawing_canvas_js().contains("showMarks"));
        assert!(Technique::Tracing.drawing_canvas_js().contains("clear"));
        assert!(Technique::Freeform.drawing_canvas_js().contains("clear"));
    }
}
