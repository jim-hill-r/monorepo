//! `palette/src/generated.rs` and an app's `design/theme.rs`.

use design_system_model::spec::COLOR_TOKENS;
use design_system_model::{Combo, Resolved, Scheme};

/// A CSS length in px (`rem` counts as 16 px). `None` if it isn't a length.
pub fn px(v: &str) -> Option<f64> {
    if v == "0" {
        return Some(0.0);
    }
    if let Some(n) = v.strip_suffix("rem") {
        return n.parse::<f64>().ok().map(|n| n * 16.0);
    }
    v.strip_suffix("px").and_then(|n| n.parse().ok())
}

fn field(token: &str) -> String {
    token.replace('-', "_")
}

fn float(v: f64) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0');
    if s.ends_with('.') {
        format!("{s}0")
    } else {
        s.to_string()
    }
}

fn lengths(r: &Resolved, names: &[&str]) -> String {
    let v: Vec<String> = names
        .iter()
        .map(|n| float(r.token(n).and_then(px).unwrap_or(0.0)))
        .collect();
    format!("[{}]", v.join(", "))
}

const SPACE: [&str; 9] = [
    "space-0", "space-1", "space-2", "space-3", "space-4", "space-5", "space-6", "space-7",
    "space-8",
];
const RADIUS: [&str; 4] = ["radius-sm", "radius-md", "radius-lg", "radius-pill"];

/// The `Tokens` struct, generated so it always has every colour token.
fn tokens_struct() -> String {
    let mut out = String::from(
        "/// Every token's value in one theme, scheme and mode.\n#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Tokens {\n",
    );
    for k in COLOR_TOKENS.iter().filter(|k| !k.starts_with("data-")) {
        out.push_str(&format!(
            "    /// `--ds-color-{k}`\n    pub {}: Rgba,\n",
            field(k)
        ));
    }
    out.push_str(
        "    /// `--ds-color-data-1` to `--ds-color-data-8`, the categorical palette.\n    pub data: [Rgba; 8],\n",
    );
    out.push_str("    /// `--ds-text-xs` to `--ds-text-3xl` in px.\n    pub text_px: [f32; 7],\n");
    out.push_str("    /// `--ds-space-0` to `--ds-space-8` in px.\n    pub space_px: [f32; 9],\n");
    out.push_str(
        "    /// `--ds-radius-sm`, `-md`, `-lg` and `-pill` in px.\n    pub radius_px: [f32; 4],\n",
    );
    out.push_str("    /// `--ds-border-width` in px.\n    pub border_width_px: f32,\n");
    out.push_str("    /// `--ds-focus-width` in px.\n    pub focus_width_px: f32,\n");
    out.push_str("    /// `--ds-target-min` in px.\n    pub target_min_px: f32,\n");
    out.push_str("}\n");
    out
}

fn rgba(c: design_system_model::Rgba, krate: &str) -> String {
    format!(
        "{krate}::Rgba({:#04x}, {:#04x}, {:#04x}, {:#04x})",
        c.r, c.g, c.b, c.a
    )
}

fn tokens_const(name: &str, r: &Resolved, krate: &str) -> String {
    let mut out = format!(
        "/// {} {}.\npub const {name}: {krate}::Tokens = {krate}::Tokens {{\n",
        r.theme,
        r.combo.label().to_lowercase()
    );
    let mut data = Vec::new();
    for (k, v) in &r.colors {
        if k.starts_with("data-") {
            data.push(rgba(*v, krate));
        } else {
            out.push_str(&format!("    {}: {},\n", field(k), rgba(*v, krate)));
        }
    }
    out.push_str(&format!(
        "    data: [\n        {},\n    ],\n",
        data.join(",\n        ")
    ));
    let text: Vec<String> = r.text_rem.iter().map(|v| float(v * 16.0)).collect();
    out.push_str(&format!("    text_px: [{}],\n", text.join(", ")));
    out.push_str(&format!("    space_px: {},\n", lengths(r, &SPACE)));
    out.push_str(&format!("    radius_px: {},\n", lengths(r, &RADIUS)));
    for (f, t) in [
        ("border_width_px", "border-width"),
        ("focus_width_px", "focus-width"),
        ("target_min_px", "target-min"),
    ] {
        out.push_str(&format!(
            "    {f}: {},\n",
            float(r.token(t).and_then(px).unwrap_or(0.0))
        ));
    }
    out.push_str("};\n");
    out
}

fn const_name(theme: &str, combo: Combo) -> String {
    format!("{}_{}", theme, combo.slug())
        .replace('-', "_")
        .to_uppercase()
}

/// The constants for one theme, and a `Theme` tying them together.
fn theme_consts(name: &str, label: &str, all: &[Resolved], krate: &str) -> String {
    let mut out = String::new();
    for r in all {
        out.push_str(&tokens_const(&const_name(name, r.combo), r, krate));
        out.push('\n');
    }
    // A light-only custom theme uses its light values for dark too.
    let pick = |scheme: Scheme, accessible: bool| {
        let want = Combo { scheme, accessible };
        let have = all.iter().any(|r| r.combo == want);
        let c = if have {
            want
        } else {
            Combo {
                scheme: Scheme::Light,
                accessible,
            }
        };
        format!("&{}", const_name(name, c))
    };
    out.push_str(&format!(
        "/// The {label} theme.\npub const {}: {krate}::Theme = {krate}::Theme {{\n    name: {name:?},\n    label: {label:?},\n    light: {},\n    dark: {},\n    accessible_light: {},\n    accessible_dark: {},\n}};\n",
        name.replace('-', "_").to_uppercase(),
        pick(Scheme::Light, false),
        pick(Scheme::Dark, false),
        pick(Scheme::Light, true),
        pick(Scheme::Dark, true),
    ));
    out
}

const RUST_HEADER: &str =
    "// @generated by design_system_cli render from design_system/tokens. Do not edit.\n";

/// `palette/src/generated.rs`.
pub fn palette_rs(themes: &[(String, String, Vec<Resolved>)]) -> String {
    let mut out = format!("{RUST_HEADER}\nuse crate::Rgba;\n\n{}\n", tokens_struct());
    for (name, label, all) in themes {
        out.push_str(&theme_consts(name, label, all, "crate"));
        out.push('\n');
    }
    let names: Vec<String> = themes.iter().map(|(n, _, _)| n.to_uppercase()).collect();
    out.push_str(&format!(
        "/// The standard themes, in the order pickers list them.\npub const THEMES: [crate::Theme; {}] = [{}];\n",
        names.len(),
        names.join(", ")
    ));
    out
}

/// `<app>/design/theme.rs`, for a game with a custom theme.
pub fn app_rs(app: &str, name: &str, label: &str, all: &[Resolved]) -> String {
    format!(
        "// @generated by design_system_cli render from {app}/design/theme.toml. Do not edit.\n// Needs the design_system_palette crate as a dependency.\n\n{}",
        theme_consts(name, label, all, "design_system_palette")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_in_px() {
        assert_eq!(px("0"), Some(0.0));
        assert_eq!(px("0.25rem"), Some(4.0));
        assert_eq!(px("44px"), Some(44.0));
        assert_eq!(px("none"), None);
        assert_eq!(float(4.0), "4.0");
        assert_eq!(float(13.333), "13.33");
    }
}
