//! The design system for Dioxus 0.7 apps (web, desktop and mobile).
//!
//! ```ignore
//! use design_system_dioxus::{Stylesheets, ThemePicker};
//!
//! rsx! {
//!     Stylesheets { default_theme: "professional" }
//!     ThemePicker {}
//! }
//! ```
//!
//! `Stylesheets` puts the tokens, the base sheet, the self-hosted fonts and
//! the theme script in the page head, so no app copies a file. For no flash
//! of the wrong theme on first paint, also inline [`head_script`] in the
//! app's `index.html` (see the README).

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// `dist/tokens.css`: every standard theme in every combination.
pub const TOKENS_CSS: &str = include_str!("../../dist/tokens.css");
/// `dist/base.css`: the reset and element defaults.
pub const BASE_CSS: &str = include_str!("../../dist/base.css");
/// `js/theme.js`: applies the saved choice and exposes `window.dsTheme`.
pub const THEME_JS: &str = include_str!("../../js/theme.js");

/// The standard themes, `(name, label)`, in the order the picker lists them.
pub const STANDARD_THEMES: [(&str, &str); 5] = [
    ("earthy", "Earthy"),
    ("airy", "Airy"),
    ("professional", "Professional"),
    ("colorful", "Colorful"),
    ("gamey", "Gamey (8-bit)"),
];

const ATKINSON_400: Asset = asset!("/fonts/atkinson-hyperlegible-latin-400-normal.woff2");
const ATKINSON_400_ITALIC: Asset = asset!("/fonts/atkinson-hyperlegible-latin-400-italic.woff2");
const ATKINSON_700: Asset = asset!("/fonts/atkinson-hyperlegible-latin-700-normal.woff2");
const PRESS_START: Asset = asset!("/fonts/press-start-2p-latin-400-normal.woff2");

/// `@font-face` rules for the two self-hosted fonts (both SIL Open Font
/// License): Atkinson Hyperlegible for accessible mode and Press Start 2P for
/// gamey headings. Browsers fetch a file only when a page uses that face.
pub fn font_faces() -> String {
    let face = |family: &str, url: &Asset, weight: u16, style: &str| {
        format!(
            "@font-face {{\n  font-family: '{family}';\n  src: url(\"{url}\") format(\"woff2\");\n  font-weight: {weight};\n  font-style: {style};\n  font-display: swap;\n}}\n"
        )
    };
    [
        face("Atkinson Hyperlegible", &ATKINSON_400, 400, "normal"),
        face("Atkinson Hyperlegible", &ATKINSON_400_ITALIC, 400, "italic"),
        face("Atkinson Hyperlegible", &ATKINSON_700, 700, "normal"),
        face("Press Start 2P", &PRESS_START, 400, "normal"),
    ]
    .concat()
}

/// The script to inline in `<head>`: records the app's default theme and
/// the themes it offers on `<html>`, then applies the saved choice.
pub fn head_script(default_theme: &str, themes: &[&str]) -> String {
    let attr = |v: &str| serde_json::to_string(v).unwrap_or_else(|_| "\"\"".into());
    format!(
        "(function(){{var r=document.documentElement;r.setAttribute(\"data-ds-default\",{});r.setAttribute(\"data-ds-themes\",{});}})();\n{THEME_JS}",
        attr(default_theme),
        attr(&themes.join(" "))
    )
}

/// The themes an app offers: its own custom theme first, if it has one, then
/// the standard themes unless it turns them off.
fn offered(custom: &Option<(String, String)>, offer_standard: bool) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = custom.iter().cloned().collect();
    if offer_standard || custom.is_none() {
        out.extend(
            STANDARD_THEMES
                .iter()
                .map(|(n, l)| (n.to_string(), l.to_string())),
        );
    }
    out
}

/// Put the design system in the page head.
///
/// - `default_theme`: the theme before the user picks one.
/// - `app_css`: the app's generated `design/theme.css`, if it has a custom
///   theme (pass `include_str!("../design/theme.css")`).
/// - `custom` and `offer_standard`: as for [`ThemePicker`], so a saved
///   theme the app no longer offers falls back to the default.
#[component]
pub fn Stylesheets(
    default_theme: String,
    #[props(default)] app_css: Option<String>,
    #[props(default)] custom: Option<(String, String)>,
    #[props(default = true)] offer_standard: bool,
) -> Element {
    let themes: Vec<String> = offered(&custom, offer_standard)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let names: Vec<&str> = themes.iter().map(String::as_str).collect();
    let css = format!(
        "{}\n{TOKENS_CSS}\n{}\n{BASE_CSS}",
        font_faces(),
        app_css.unwrap_or_default()
    );
    let script = head_script(&default_theme, &names);
    rsx! {
        document::Style { {css} }
        document::Script { {script} }
    }
}

/// What the user chose. `scheme` is `system`, `light` or `dark`; `a11y` is
/// `system`, `on` or `off`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub theme: String,
    pub scheme: String,
    pub a11y: String,
}

impl Choice {
    /// The JavaScript that saves and applies this choice.
    pub fn script(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "{}".into());
        format!("if (window.dsTheme) {{ window.dsTheme.set({json}); }}")
    }
}

/// The current choice, read from the page once the theme script has run.
/// `None` until then.
pub fn use_theme() -> Signal<Option<Choice>> {
    let mut choice = use_signal(|| None::<Choice>);
    use_future(move || async move {
        let read = document::eval("return window.dsTheme ? window.dsTheme.get() : null;")
            .join::<Option<Choice>>()
            .await;
        if let Ok(Some(c)) = read {
            choice.set(Some(c));
        }
    });
    choice
}

/// Save and apply a choice.
pub fn set_theme(mut signal: Signal<Option<Choice>>, choice: Choice) {
    let _ = document::eval(&choice.script());
    signal.set(Some(choice));
}

/// A small appearance form: theme, colours (system, light, dark) and
/// accessible mode (system, on, off). Labelled controls in a fieldset, so it
/// works with a keyboard and screen readers.
#[component]
pub fn ThemePicker(
    #[props(default)] custom: Option<(String, String)>,
    #[props(default = true)] offer_standard: bool,
) -> Element {
    let choice = use_theme();
    let themes = offered(&custom, offer_standard);
    let current = choice.read().clone().unwrap_or(Choice {
        theme: themes.first().map(|t| t.0.clone()).unwrap_or_default(),
        scheme: "system".into(),
        a11y: "system".into(),
    });
    let (c1, c2, c3) = (current.clone(), current.clone(), current.clone());
    rsx! {
        fieldset { class: "ds-theme-picker",
            legend { "Appearance" }
            label {
                "Theme "
                select {
                    value: "{current.theme}",
                    onchange: move |e| set_theme(choice, Choice { theme: e.value(), ..c1.clone() }),
                    for (name, label) in themes {
                        option { value: "{name}", selected: name == current.theme, "{label}" }
                    }
                }
            }
            label {
                "Colours "
                select {
                    value: "{current.scheme}",
                    onchange: move |e| set_theme(choice, Choice { scheme: e.value(), ..c2.clone() }),
                    for (value, label) in [("system", "Match system"), ("light", "Light"), ("dark", "Dark")] {
                        option { value, selected: value == current.scheme, "{label}" }
                    }
                }
            }
            label {
                "Accessible mode "
                select {
                    value: "{current.a11y}",
                    onchange: move |e| set_theme(choice, Choice { a11y: e.value(), ..c3.clone() }),
                    for (value, label) in [("system", "Match system"), ("on", "On"), ("off", "Off")] {
                        option { value, selected: value == current.a11y, "{label}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_themes_match_the_tokens() {
        let json: serde_json::Value =
            serde_json::from_str(include_str!("../../dist/tokens.json")).unwrap();
        let names: Vec<&str> = json["$extensions"]["design_system"]["themes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let ours: Vec<&str> = STANDARD_THEMES.iter().map(|t| t.0).collect();
        assert_eq!(names, ours);
        for (name, label) in STANDARD_THEMES {
            assert_eq!(json[name]["$description"], label);
        }
    }

    #[test]
    fn a_custom_theme_comes_first() {
        let bears = Some(("bears".to_string(), "Chicago Bears".to_string()));
        let all = offered(&bears, true);
        assert_eq!(all.len(), 6);
        assert_eq!(all[0].0, "bears");
        assert_eq!(offered(&bears, false).len(), 1);
        assert_eq!(offered(&None, false).len(), 5);
    }

    #[test]
    fn head_script_records_the_default() {
        let s = head_script("bears", &["bears", "earthy"]);
        assert!(s.contains(r#""data-ds-default","bears""#));
        assert!(s.contains(r#""data-ds-themes","bears earthy""#));
        assert!(s.contains("window.dsTheme"));
    }

    #[test]
    fn choice_script_saves_json() {
        let c = Choice {
            theme: "gamey".into(),
            scheme: "dark".into(),
            a11y: "on".into(),
        };
        assert_eq!(
            c.script(),
            r#"if (window.dsTheme) { window.dsTheme.set({"theme":"gamey","scheme":"dark","a11y":"on"}); }"#
        );
    }
}
