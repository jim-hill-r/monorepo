//! Turning theme files into a value for every semantic token in one
//! combination of theme, scheme and accessible mode.

use crate::color::Rgba;
use crate::spec::{COLOR_TOKENS, OTHER_TOKENS, Scheme, TEXT_STEPS, ThemeFile, custom_may_override};
use crate::{DesignSystem, STANDARD_THEMES};
use std::collections::BTreeMap;
use std::fmt;

/// A scheme with accessible mode on or off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Combo {
    pub scheme: Scheme,
    pub accessible: bool,
}

impl Combo {
    pub const ALL: [Combo; 4] = [
        Combo {
            scheme: Scheme::Light,
            accessible: false,
        },
        Combo {
            scheme: Scheme::Dark,
            accessible: false,
        },
        Combo {
            scheme: Scheme::Light,
            accessible: true,
        },
        Combo {
            scheme: Scheme::Dark,
            accessible: true,
        },
    ];

    /// `light`, `dark`, `accessible-light` or `accessible-dark`.
    pub fn slug(self) -> String {
        if self.accessible {
            format!("accessible-{}", self.scheme.name())
        } else {
            self.scheme.name().to_string()
        }
    }

    /// `Light`, `Dark`, `Accessible light` or `Accessible dark`.
    pub fn label(self) -> String {
        match (self.accessible, self.scheme) {
            (false, Scheme::Light) => "Light".into(),
            (false, Scheme::Dark) => "Dark".into(),
            (true, Scheme::Light) => "Accessible light".into(),
            (true, Scheme::Dark) => "Accessible dark".into(),
        }
    }
}

impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.slug())
    }
}

/// Every token's value in one combination.
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    pub theme: String,
    pub combo: Combo,
    /// In `COLOR_TOKENS` order.
    pub colors: Vec<(String, Rgba)>,
    /// Fonts, text sizes, then `OTHER_TOKENS` order, as CSS values.
    pub tokens: Vec<(String, String)>,
    /// Text sizes in rem, in `TEXT_STEPS` order.
    pub text_rem: Vec<f64>,
}

impl Resolved {
    pub fn color(&self, name: &str) -> Option<Rgba> {
        self.colors.iter().find(|(k, _)| k == name).map(|(_, v)| *v)
    }

    pub fn token(&self, name: &str) -> Option<&str> {
        self.tokens
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Format a number of rem as CSS, to three decimals.
pub fn rem(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{s}rem")
}

/// Everything wrong with a theme file, as messages. Empty when it is valid.
pub fn validate(theme: &ThemeFile) -> Vec<String> {
    let mut out = Vec::new();
    let custom = theme.extends.is_some();
    let all_tokens = |k: &str| COLOR_TOKENS.contains(&k) || OTHER_TOKENS.contains(&k);

    if !theme
        .name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || theme.name.is_empty()
    {
        out.push(format!(
            "name {:?} must be lowercase kebab-case",
            theme.name
        ));
    }
    match &theme.extends {
        Some(base) => {
            if !STANDARD_THEMES.contains(&base.as_str()) {
                out.push(format!(
                    "extends {base:?}, which is not a standard theme ({})",
                    STANDARD_THEMES.join(", ")
                ));
            }
            if STANDARD_THEMES.contains(&theme.name.as_str()) {
                out.push(format!(
                    "a custom theme can't reuse the name {:?}",
                    theme.name
                ));
            }
            if theme.type_scale.base.is_some() || theme.type_scale.ratio.is_some() {
                out.push("a custom theme can't change the type scale".into());
            }
            for k in theme.tokens.keys() {
                if !custom_may_override(k) {
                    out.push(format!(
                        "a custom theme may override only colours, fonts, radii and shadows, not {k:?}"
                    ));
                }
            }
            if theme.schemes().is_empty() {
                out.push("schemes must list at least one scheme".into());
            }
            for s in Scheme::ALL {
                if !theme.schemes().contains(&s)
                    && (!theme.scheme(s).color.is_empty()
                        || !theme.accessible.scheme(s).color.is_empty())
                {
                    out.push(format!(
                        "sets [{0}.color] but schemes doesn't include {0:?}",
                        s.name()
                    ));
                }
            }
        }
        None => {
            if theme.schemes.is_some() || theme.offer_standard.is_some() || theme.rust.is_some() {
                out.push("schemes, offer_standard and rust are for custom app themes only".into());
            }
            for s in Scheme::ALL {
                for k in COLOR_TOKENS {
                    if !theme.scheme(s).color.contains_key(*k) {
                        out.push(format!("[{}.color] doesn't define {k:?}", s.name()));
                    }
                }
            }
        }
    }
    for k in theme.tokens.keys() {
        if !custom && !all_tokens(k) {
            out.push(format!("[tokens] has unknown token {k:?}"));
        }
        if COLOR_TOKENS.contains(&k.as_str()) {
            out.push(format!(
                "[tokens] {k:?} is a colour; set it under [light.color] and [dark.color]"
            ));
        }
    }
    for (k, v) in &theme.primitives {
        if all_tokens(k) {
            out.push(format!(
                "primitive {k:?} has the same name as a semantic token"
            ));
        }
        if Rgba::parse_hex(v).is_none() {
            out.push(format!("primitive {k:?} = {v:?} is not a #hex colour"));
        }
    }
    let maps = Scheme::ALL.into_iter().flat_map(|s| {
        [
            (format!("{}.color", s.name()), &theme.scheme(s).color),
            (
                format!("accessible.{}.color", s.name()),
                &theme.accessible.scheme(s).color,
            ),
        ]
    });
    for (section, map) in maps {
        for k in map.keys() {
            if !COLOR_TOKENS.contains(&k.as_str()) {
                out.push(format!("[{section}] has unknown colour token {k:?}"));
            }
        }
    }
    out
}

/// Everything wrong with `base.toml` and `modes.toml`.
pub fn validate_shared(ds: &DesignSystem) -> Vec<String> {
    let mut out = Vec::new();
    for k in OTHER_TOKENS {
        if !ds.base.tokens.contains_key(*k) {
            out.push(format!("base.toml [tokens] doesn't define {k:?}"));
        }
    }
    for k in ds.base.tokens.keys() {
        if !OTHER_TOKENS.contains(&k.as_str()) {
            out.push(format!("base.toml [tokens] has unknown token {k:?}"));
        }
    }
    if ds.base.type_scale.base.is_none() || ds.base.type_scale.ratio.is_none() {
        out.push("base.toml [type] must set base and ratio".into());
    }
    if ds.base.fonts.tokens().count() != 3 {
        out.push("base.toml [fonts] must set heading, body and mono".into());
    }
    for k in ds.modes.accessible.tokens.keys() {
        if !OTHER_TOKENS.contains(&k.as_str()) {
            out.push(format!(
                "modes.toml [accessible.tokens] has unknown token {k:?}"
            ));
        }
    }
    out
}

fn lookup(
    name: &str,
    value: &str,
    primitives: &BTreeMap<String, String>,
    colors: &BTreeMap<String, String>,
    depth: usize,
) -> Result<Rgba, String> {
    if value.starts_with('#') {
        return Rgba::parse_hex(value).ok_or_else(|| format!("{name} = {value:?} is not a colour"));
    }
    if let Some(p) = primitives.get(value) {
        return Rgba::parse_hex(p)
            .ok_or_else(|| format!("primitive {value} = {p:?} is not a colour"));
    }
    if depth < 8
        && let Some(v) = colors.get(value)
    {
        return lookup(value, v, primitives, colors, depth + 1);
    }
    Err(format!(
        "{name} = {value:?} is neither a #hex colour, a primitive nor another colour token"
    ))
}

/// Resolve every token of `theme` in `combo`. `parent` is the standard theme
/// a custom theme extends (`None` for a standard theme).
pub fn resolve(
    ds: &DesignSystem,
    theme: &ThemeFile,
    parent: Option<&ThemeFile>,
    combo: Combo,
) -> Result<Resolved, String> {
    let layers: Vec<&ThemeFile> = parent.into_iter().chain([theme]).collect();
    let s = combo.scheme;

    // Colours: the base theme's scheme, then its accessible overrides, then
    // the custom theme's scheme and its accessible overrides.
    let mut primitives = BTreeMap::new();
    let mut raw = BTreeMap::new();
    for t in &layers {
        primitives.extend(t.primitives.clone());
        raw.extend(t.scheme(s).color.clone());
        if combo.accessible {
            raw.extend(t.accessible.scheme(s).color.clone());
        }
    }
    let mut colors = Vec::new();
    for k in COLOR_TOKENS {
        let v = raw
            .get(*k)
            .ok_or_else(|| format!("{} {combo}: {k} is not defined", theme.name))?;
        let c = lookup(k, v, &primitives, &raw, 0)
            .map_err(|e| format!("{} {combo}: {e}", theme.name))?;
        colors.push((k.to_string(), c));
    }

    // Everything else: base.toml, the theme layers, then accessible mode.
    let mut other: BTreeMap<String, String> = ds.base.tokens.clone();
    let mut fonts: BTreeMap<&str, String> = ds
        .base
        .fonts
        .tokens()
        .map(|(k, v)| (k, v.clone()))
        .collect();
    let mut base_rem = ds.base.type_scale.base.unwrap_or(1.0);
    let mut ratio = ds.base.type_scale.ratio.unwrap_or(1.2);
    for t in &layers {
        other.extend(t.tokens.clone());
        fonts.extend(t.fonts.tokens().map(|(k, v)| (k, v.clone())));
        base_rem = t.type_scale.base.unwrap_or(base_rem);
        ratio = t.type_scale.ratio.unwrap_or(ratio);
    }
    if combo.accessible {
        let a = &ds.modes.accessible;
        other.extend(a.tokens.clone());
        fonts.extend(a.fonts.tokens().map(|(k, v)| (k, v.clone())));
        base_rem = a.type_scale.base.unwrap_or(base_rem);
        ratio = a.type_scale.ratio.unwrap_or(ratio);
    }
    let mut tokens = Vec::new();
    for k in ["font-body", "font-heading", "font-mono"] {
        let v = fonts
            .get(k)
            .ok_or_else(|| format!("{} {combo}: {k} is not defined", theme.name))?;
        tokens.push((k.to_string(), v.clone()));
    }
    let text_rem: Vec<f64> = TEXT_STEPS
        .iter()
        .map(|(_, step)| base_rem * ratio.powi(*step))
        .collect();
    for ((k, _), v) in TEXT_STEPS.iter().zip(&text_rem) {
        tokens.push((k.to_string(), rem(*v)));
    }
    for k in OTHER_TOKENS {
        let v = other
            .get(*k)
            .ok_or_else(|| format!("{} {combo}: {k} is not defined", theme.name))?;
        tokens.push((k.to_string(), v.clone()));
    }

    Ok(Resolved {
        theme: theme.name.clone(),
        combo,
        colors,
        tokens,
        text_rem,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rem_trims_zeros() {
        assert_eq!(rem(1.0), "1rem");
        assert_eq!(rem(1.2), "1.2rem");
        assert_eq!(rem(1.0 / 1.2), "0.833rem");
    }
}
