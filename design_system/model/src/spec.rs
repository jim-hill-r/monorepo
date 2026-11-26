//! The token files as written: `tokens/base.toml`, `tokens/modes.toml`,
//! `tokens/pairs.toml`, `tokens/themes/*.toml` and apps' `design/theme.toml`.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Every semantic colour token, in the order the outputs list them.
pub const COLOR_TOKENS: &[&str] = &[
    "bg",
    "surface",
    "surface-raised",
    "border",
    "border-strong",
    "text",
    "text-muted",
    "link",
    "accent",
    "accent-hover",
    "on-accent",
    "focus",
    "success",
    "on-success",
    "warning",
    "on-warning",
    "danger",
    "on-danger",
    "info",
    "on-info",
    "data-1",
    "data-2",
    "data-3",
    "data-4",
    "data-5",
    "data-6",
    "data-7",
    "data-8",
];

/// Every semantic token that isn't a colour and is written in `base.toml`
/// (fonts come from `[fonts]`, text sizes from `[type]`).
pub const OTHER_TOKENS: &[&str] = &[
    "leading-body",
    "leading-heading",
    "weight-body",
    "weight-strong",
    "space-0",
    "space-1",
    "space-2",
    "space-3",
    "space-4",
    "space-5",
    "space-6",
    "space-7",
    "space-8",
    "radius-sm",
    "radius-md",
    "radius-lg",
    "radius-pill",
    "shadow-1",
    "shadow-2",
    "shadow-3",
    "border-width",
    "focus-width",
    "focus-offset",
    "target-min",
    "motion-fast",
    "motion-slow",
    "ease",
];

/// The type scale's step names, from two steps below the base to four above.
pub const TEXT_STEPS: &[(&str, i32)] = &[
    ("text-xs", -2),
    ("text-sm", -1),
    ("text-base", 0),
    ("text-lg", 1),
    ("text-xl", 2),
    ("text-2xl", 3),
    ("text-3xl", 4),
];

/// The tokens a custom app theme may change besides colours and fonts.
pub fn custom_may_override(token: &str) -> bool {
    token.starts_with("radius-") || token.starts_with("shadow-")
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    pub heading: Option<String>,
    pub body: Option<String>,
    pub mono: Option<String>,
}

impl Fonts {
    pub fn tokens(&self) -> impl Iterator<Item = (&'static str, &String)> {
        [
            ("font-heading", &self.heading),
            ("font-body", &self.body),
            ("font-mono", &self.mono),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeScale {
    /// Base font size in rem.
    pub base: Option<f64>,
    /// Ratio between steps.
    pub ratio: Option<f64>,
}

/// `tokens/base.toml`: the values every theme starts from.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Base {
    #[serde(rename = "type")]
    pub type_scale: TypeScale,
    pub fonts: Fonts,
    pub tokens: BTreeMap<String, String>,
}

/// `tokens/modes.toml`: what accessible mode changes in every theme.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modes {
    pub accessible: AccessibleRules,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessibleRules {
    #[serde(rename = "type", default)]
    pub type_scale: TypeScale,
    #[serde(default)]
    pub fonts: Fonts,
    #[serde(default)]
    pub tokens: BTreeMap<String, String>,
}

/// `tokens/pairs.toml`: which foreground sits on which background.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairs {
    pub pair: Vec<PairRule>,
    pub data: DataRule,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairRule {
    pub fg: Vec<String>,
    pub bg: Vec<String>,
    /// Minimum WCAG ratio in standard mode.
    pub min: f64,
    /// Minimum WCAG ratio in accessible mode.
    pub accessible: f64,
    pub why: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataRule {
    pub tokens: Vec<String>,
    pub on: Vec<String>,
    pub min: f64,
    pub accessible: f64,
    /// Smallest OKLab distance between any two data colours, normal vision.
    pub min_distance: f64,
    /// In accessible mode, the smallest distance under every simulated
    /// form of colour blindness too.
    pub accessible_distance: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    Light,
    Dark,
}

impl Scheme {
    pub const ALL: [Scheme; 2] = [Scheme::Light, Scheme::Dark];
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeColors {
    #[serde(default)]
    pub color: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessibleColors {
    #[serde(default)]
    pub light: SchemeColors,
    #[serde(default)]
    pub dark: SchemeColors,
}

impl AccessibleColors {
    pub fn scheme(&self, s: Scheme) -> &SchemeColors {
        match s {
            Scheme::Light => &self.light,
            Scheme::Dark => &self.dark,
        }
    }
}

/// A theme file: one of the five standard themes, or an app's custom theme
/// (which names the standard theme it `extends`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeFile {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Custom app themes only: the standard theme this one builds on.
    #[serde(default)]
    pub extends: Option<String>,
    /// Custom app themes only: the schemes it has (default both).
    #[serde(default)]
    pub schemes: Option<Vec<Scheme>>,
    /// Custom app themes only: whether the app also offers the standard themes.
    #[serde(default)]
    pub offer_standard: Option<bool>,
    /// Custom app themes only: also write `design/theme.rs` (for games).
    #[serde(default)]
    pub rust: Option<bool>,
    #[serde(rename = "type", default)]
    pub type_scale: TypeScale,
    #[serde(default)]
    pub fonts: Fonts,
    #[serde(default)]
    pub tokens: BTreeMap<String, String>,
    #[serde(default)]
    pub primitives: BTreeMap<String, String>,
    #[serde(default)]
    pub light: SchemeColors,
    #[serde(default)]
    pub dark: SchemeColors,
    #[serde(default)]
    pub accessible: AccessibleColors,
}

impl ThemeFile {
    pub fn scheme(&self, s: Scheme) -> &SchemeColors {
        match s {
            Scheme::Light => &self.light,
            Scheme::Dark => &self.dark,
        }
    }

    pub fn schemes(&self) -> Vec<Scheme> {
        self.schemes.clone().unwrap_or_else(|| Scheme::ALL.to_vec())
    }
}
