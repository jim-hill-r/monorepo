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
//! the theme script in the page head, so no app copies a file. In the
//! browser it links them as CSS files rather than inlining a `<style>`, so
//! an app's Content Security Policy needs no `style-src 'unsafe-inline'`. For no flash
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

/// The self-hosted fonts (both SIL Open Font License): Atkinson Hyperlegible
/// for accessible mode and Press Start 2P for gamey headings, as
/// `(family, file, weight, style)`. Browsers fetch a file only when a page
/// uses that face.
const FONTS: [(&str, &str, u16, &str); 4] = [
    (
        "Atkinson Hyperlegible",
        "atkinson-hyperlegible-latin-400-normal.woff2",
        400,
        "normal",
    ),
    (
        "Atkinson Hyperlegible",
        "atkinson-hyperlegible-latin-400-italic.woff2",
        400,
        "italic",
    ),
    (
        "Atkinson Hyperlegible",
        "atkinson-hyperlegible-latin-700-normal.woff2",
        700,
        "normal",
    ),
    (
        "Press Start 2P",
        "press-start-2p-latin-400-normal.woff2",
        400,
        "normal",
    ),
];

/// The `@font-face` rules for [`FONTS`], with `url` giving each file's URL.
fn faces(url: impl Fn(&str) -> String) -> String {
    FONTS
        .iter()
        .map(|(family, file, weight, style)| {
            format!(
                "@font-face {{\n  font-family: '{family}';\n  src: url(\"{}\") format(\"woff2\");\n  font-weight: {weight};\n  font-style: {style};\n  font-display: swap;\n}}\n",
                url(file)
            )
        })
        .collect()
}

/// The `@font-face` rules for the self-hosted fonts, pointing at each font's
/// own asset, for inlining in a `<style>`.
pub fn font_faces() -> String {
    const ATKINSON_400: Asset =
        asset!("/assets/design-system-fonts/atkinson-hyperlegible-latin-400-normal.woff2");
    const ATKINSON_400_ITALIC: Asset =
        asset!("/assets/design-system-fonts/atkinson-hyperlegible-latin-400-italic.woff2");
    const ATKINSON_700: Asset =
        asset!("/assets/design-system-fonts/atkinson-hyperlegible-latin-700-normal.woff2");
    const PRESS_START: Asset =
        asset!("/assets/design-system-fonts/press-start-2p-latin-400-normal.woff2");
    faces(|file| {
        let asset = match file {
            "atkinson-hyperlegible-latin-400-normal.woff2" => ATKINSON_400,
            "atkinson-hyperlegible-latin-400-italic.woff2" => ATKINSON_400_ITALIC,
            "atkinson-hyperlegible-latin-700-normal.woff2" => ATKINSON_700,
            _ => PRESS_START,
        };
        asset.to_string()
    })
}

/// How [`Stylesheets`] puts the CSS in the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// One `<style>` element with everything in it. Desktop, mobile and the
    /// server-side render use this.
    Inline,
    /// `<link rel="stylesheet">` to CSS files, and no `<style>` element
    /// (unless the app passes `app_css`), so the page works under a Content
    /// Security Policy without `style-src 'unsafe-inline'`. The default in
    /// the browser (wasm).
    Files,
}

impl Delivery {
    /// The default for this target: [`Delivery::Files`] in the browser,
    /// [`Delivery::Inline`] everywhere else.
    pub const fn for_target() -> Self {
        if cfg!(target_arch = "wasm32") {
            Delivery::Files
        } else {
            Delivery::Inline
        }
    }
}

/// One element [`Stylesheets`] puts in the head, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Sheet {
    /// A `<style>` element with this CSS.
    Style(String),
    /// A `<link rel="stylesheet">` to this URL.
    Link(String),
}

/// The head elements for `delivery`: the font faces, the tokens, the app's
/// theme (a file, then any inline CSS) and the base sheet, in that order.
fn sheets(
    delivery: Delivery,
    app_css: Option<String>,
    app_stylesheet: Option<String>,
) -> Vec<Sheet> {
    match delivery {
        Delivery::Inline => {
            let mut out = Vec::new();
            let mut css = format!("{}\n{TOKENS_CSS}\n", font_faces());
            match app_stylesheet {
                // A file the app links: keep it between the tokens and the
                // base sheet by splitting the style in two.
                Some(href) => {
                    out.push(Sheet::Style(css));
                    out.push(Sheet::Link(href));
                    css = format!("{}\n{BASE_CSS}", app_css.unwrap_or_default());
                }
                None => css.push_str(&format!("{}\n{BASE_CSS}", app_css.unwrap_or_default())),
            }
            out.push(Sheet::Style(css));
            out
        }
        Delivery::Files => {
            const FONTS_DIR: Asset = asset!("/assets/design-system-fonts", AssetOptions::folder());
            const TOKENS: Asset = asset!("/assets/tokens.css");
            const BASE: Asset = asset!("/assets/base.css");
            let mut out = vec![
                Sheet::Link(format!("{FONTS_DIR}/fonts.css")),
                Sheet::Link(TOKENS.to_string()),
            ];
            out.extend(app_stylesheet.map(Sheet::Link));
            out.extend(app_css.filter(|c| !c.is_empty()).map(Sheet::Style));
            out.push(Sheet::Link(BASE.to_string()));
            out
        }
    }
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
/// - `app_stylesheet`: the URL of the app's generated `design/theme.css`,
///   if it has a custom theme, as an `asset!` the app serves. Linked as a
///   file, so it needs no `style-src 'unsafe-inline'`.
/// - `app_css`: the same CSS as a string instead
///   (`include_str!("../design/theme.css")`). Always inlined in a `<style>`,
///   so an app with a strict style policy uses `app_stylesheet`.
/// - `custom` and `offer_standard`: as for [`ThemePicker`], so a saved
///   theme the app no longer offers falls back to the default.
/// - `delivery`: inline `<style>` or linked files; see [`Delivery`]. The
///   default, [`Delivery::for_target`], links files in the browser.
#[component]
pub fn Stylesheets(
    default_theme: String,
    #[props(default)] app_css: Option<String>,
    #[props(default, into)] app_stylesheet: Option<String>,
    #[props(default)] custom: Option<(String, String)>,
    #[props(default = true)] offer_standard: bool,
    #[props(default = Delivery::for_target())] delivery: Delivery,
) -> Element {
    let themes: Vec<String> = offered(&custom, offer_standard)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let names: Vec<&str> = themes.iter().map(String::as_str).collect();
    let head = rsx! {
        for sheet in sheets(delivery, app_css, app_stylesheet) {
            match sheet {
                Sheet::Style(css) => rsx! { document::Style { {css} } },
                Sheet::Link(href) => rsx! { document::Stylesheet { href } },
            }
        }
    };
    #[cfg(target_arch = "wasm32")]
    {
        // In the browser, apply the saved choice from Rust instead of an
        // inline script, so apps whose Content Security Policy blocks
        // inline scripts and `eval` (cookbook, cahokia) still switch theme.
        let joined = names.join(" ");
        use_hook(move || web::init(&default_theme, &joined));
        head
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let script = head_script(&default_theme, &names);
        rsx! {
            {head}
            document::Script { {script} }
        }
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

/// Reads the choice, waiting up to two seconds for the theme script: the
/// `Script` that `Stylesheets` adds can run after the first effects.
#[cfg(not(target_arch = "wasm32"))]
const READ_CHOICE: &str = r#"
for (let i = 0; i < 40 && !window.dsTheme; i++) {
  await new Promise((r) => setTimeout(r, 50));
}
return window.dsTheme ? window.dsTheme.get() : null;
"#;

/// The current choice, read from the page.
#[cfg(target_arch = "wasm32")]
pub fn use_theme() -> Signal<Option<Choice>> {
    use_signal(|| Some(web::get()))
}

/// The current choice, read from the page once the theme script has run.
/// `None` until then.
#[cfg(not(target_arch = "wasm32"))]
pub fn use_theme() -> Signal<Option<Choice>> {
    let mut choice = use_signal(|| None::<Choice>);
    use_future(move || async move {
        let read = document::eval(READ_CHOICE).join::<Option<Choice>>().await;
        if let Ok(Some(c)) = read {
            choice.set(Some(c));
        }
    });
    choice
}

/// Save and apply a choice.
pub fn set_theme(mut signal: Signal<Option<Choice>>, choice: Choice) {
    #[cfg(target_arch = "wasm32")]
    web::set(&choice);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = document::eval(&choice.script());
    signal.set(Some(choice));
}

/// A saved choice as `js/theme.js` stores it: any field may be missing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
struct Saved {
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    a11y: Option<String>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl Saved {
    /// Parses what is in storage; anything unreadable is no choice.
    fn parse(raw: Option<&str>) -> Self {
        raw.and_then(|r| serde_json::from_str(r).ok())
            .unwrap_or_default()
    }

    /// `scheme` as the picker shows it: `light`, `dark` or `system`.
    fn scheme(&self) -> &str {
        match self.scheme.as_deref() {
            Some(s @ ("light" | "dark")) => s,
            _ => "system",
        }
    }

    /// `a11y` as the picker shows it: `on`, `off` or `system`.
    fn a11y(&self) -> &str {
        match self.a11y.as_deref() {
            Some(s @ ("on" | "off")) => s,
            _ => "system",
        }
    }
}

/// The `<html>` attributes for a saved choice, as `apply` in `js/theme.js`
/// computes them: `(data-theme, data-scheme, data-a11y)`, where `None` means
/// remove the attribute. `offered` is `data-ds-themes`, space separated, and
/// empty for every theme; `fallback` is the app's default theme.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn attributes<'a>(
    saved: &'a Saved,
    offered: &str,
    fallback: &'a str,
) -> (&'a str, Option<&'a str>, Option<&'a str>) {
    let theme = match saved.theme.as_deref() {
        Some(t)
            if !t.is_empty()
                && (offered.trim().is_empty() || offered.split_whitespace().any(|o| o == t)) =>
        {
            t
        }
        _ => fallback,
    };
    let scheme = Some(saved.scheme()).filter(|s| *s != "system");
    let a11y = Some(saved.a11y()).filter(|s| *s != "system");
    (theme, scheme, a11y)
}

/// The browser side of `js/theme.js`, in Rust: the same `localStorage` key
/// and `<html>` attributes, with no inline script or `eval`.
#[cfg(target_arch = "wasm32")]
mod web {
    use super::{Choice, Saved, attributes};

    const KEY: &str = "ds-theme";

    fn root() -> Option<web_sys::Element> {
        web_sys::window()?.document()?.document_element()
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn read() -> Saved {
        let raw = storage().and_then(|s| s.get_item(KEY).ok().flatten());
        Saved::parse(raw.as_deref())
    }

    fn set_or_remove(root: &web_sys::Element, name: &str, value: Option<&str>) {
        let _ = match value {
            Some(v) => root.set_attribute(name, v),
            None => root.remove_attribute(name),
        };
    }

    fn apply(saved: &Saved) {
        let Some(root) = root() else { return };
        let offered = root.get_attribute("data-ds-themes").unwrap_or_default();
        let fallback = root
            .get_attribute("data-ds-default")
            .or_else(|| root.get_attribute("data-theme"))
            .unwrap_or_else(|| "professional".into());
        let (theme, scheme, a11y) = attributes(saved, &offered, &fallback);
        let _ = root.set_attribute("data-theme", theme);
        set_or_remove(&root, "data-scheme", scheme);
        set_or_remove(&root, "data-a11y", a11y);
    }

    /// Records the app's default and offered themes, then applies the saved
    /// choice.
    pub fn init(default_theme: &str, themes: &str) {
        if let Some(root) = root() {
            let _ = root.set_attribute("data-ds-default", default_theme);
            let _ = root.set_attribute("data-ds-themes", themes);
        }
        apply(&read());
    }

    pub fn get() -> Choice {
        let saved = read();
        Choice {
            theme: root()
                .and_then(|r| r.get_attribute("data-theme"))
                .unwrap_or_default(),
            scheme: saved.scheme().into(),
            a11y: saved.a11y().into(),
        }
    }

    pub fn set(choice: &Choice) {
        if let (Some(s), Ok(json)) = (storage(), serde_json::to_string(choice)) {
            // No storage: the choice lasts until the page reloads.
            let _ = s.set_item(KEY, &json);
        }
        apply(&Saved {
            theme: Some(choice.theme.clone()),
            scheme: Some(choice.scheme.clone()),
            a11y: Some(choice.a11y.clone()),
        });
    }
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
    fn attributes_match_theme_js() {
        let saved = Saved::parse(Some(r#"{"theme":"gamey","scheme":"dark","a11y":"on"}"#));
        assert_eq!(
            attributes(&saved, "", "earthy"),
            ("gamey", Some("dark"), Some("on"))
        );
        // A theme the app doesn't offer falls back to its default.
        assert_eq!(attributes(&saved, "earthy airy", "earthy").0, "earthy");
        // "system", missing and unknown values leave the attributes off.
        let saved = Saved::parse(Some(r#"{"scheme":"system","a11y":"loud"}"#));
        assert_eq!(attributes(&saved, "", "earthy"), ("earthy", None, None));
        // Unreadable storage is no choice.
        assert_eq!(Saved::parse(Some("not json")), Saved::default());
        assert_eq!(Saved::parse(None), Saved::default());
    }

    #[test]
    fn fonts_css_matches_the_fonts() {
        let file = include_str!("../assets/design-system-fonts/fonts.css");
        assert!(file.ends_with(&faces(|f| f.to_string())));
        for (_, name, _, _) in FONTS {
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("assets/design-system-fonts")
                    .join(name)
                    .exists()
            );
        }
    }

    #[test]
    fn asset_copies_match_dist() {
        assert_eq!(include_str!("../assets/tokens.css"), TOKENS_CSS);
        assert_eq!(include_str!("../assets/base.css"), BASE_CSS);
    }

    #[test]
    fn inline_is_one_style_as_before() {
        let sheets = sheets(Delivery::Inline, Some("/* app */".into()), None);
        assert_eq!(
            sheets,
            vec![Sheet::Style(format!(
                "{}\n{TOKENS_CSS}\n/* app */\n{BASE_CSS}",
                font_faces()
            ))]
        );
        assert_eq!(Delivery::for_target(), Delivery::Inline);
    }

    #[test]
    fn files_emit_no_style_element() {
        let sheets = sheets(Delivery::Files, None, Some("/assets/theme.css".into()));
        assert_eq!(sheets.len(), 4);
        assert!(sheets.iter().all(|s| matches!(s, Sheet::Link(_))));
        let Sheet::Link(fonts) = &sheets[0] else {
            unreachable!()
        };
        assert!(fonts.ends_with("design-system-fonts/fonts.css"));
        // The app's theme sits between the tokens and the base sheet.
        assert_eq!(sheets[2], Sheet::Link("/assets/theme.css".into()));
        // `app_css` is a string, so it can only be inlined.
        let with_css = super::sheets(Delivery::Files, Some("/* app */".into()), None);
        assert_eq!(with_css[2], Sheet::Style("/* app */".into()));
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
