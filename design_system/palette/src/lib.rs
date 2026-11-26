//! The design system's token values as plain Rust constants, for Bevy games
//! and anything else that can't read CSS. No dependencies, so it works with
//! any Bevy version: turn a colour into a `bevy::Color` with
//! `Color::srgba_u8(c.0, c.1, c.2, c.3)`.
//!
//! ```
//! use design_system_palette::{GAMEY, Scheme};
//! let t = GAMEY.tokens(Scheme::Dark, false);
//! assert_eq!(t.bg.hex(), "#000000");
//! ```
//!
//! The values are generated from `design_system/tokens/` by
//! `design_system_cli render` into `src/generated.rs`.

// Generated code is laid out by the generator, not rustfmt.
#[rustfmt::skip]
mod generated;

pub use generated::*;

/// An sRGB colour: red, green, blue and alpha, 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    /// The channels as fractions of 1, still gamma-encoded (sRGB).
    pub fn to_f32(self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3].map(|c| f32::from(c) / 255.0)
    }

    /// `#rrggbb`, or `#rrggbbaa` when not opaque.
    pub fn hex(self) -> String {
        if self.3 == 255 {
            format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.0, self.1, self.2, self.3)
        }
    }
}

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scheme {
    Light,
    Dark,
}

/// One theme's tokens in every scheme and mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub label: &'static str,
    pub light: &'static Tokens,
    pub dark: &'static Tokens,
    pub accessible_light: &'static Tokens,
    pub accessible_dark: &'static Tokens,
}

impl Theme {
    pub fn tokens(&self, scheme: Scheme, accessible: bool) -> &'static Tokens {
        match (scheme, accessible) {
            (Scheme::Light, false) => self.light,
            (Scheme::Dark, false) => self.dark,
            (Scheme::Light, true) => self.accessible_light,
            (Scheme::Dark, true) => self.accessible_dark,
        }
    }
}

/// A standard theme by name.
pub fn theme(name: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_every_standard_theme() {
        for name in ["earthy", "airy", "professional", "colorful", "gamey"] {
            assert_eq!(theme(name).map(|t| t.name), Some(name));
        }
        assert!(theme("bears").is_none());
    }

    #[test]
    fn accessible_mode_is_bigger() {
        for t in THEMES {
            let (std, acc) = (
                t.tokens(Scheme::Light, false),
                t.tokens(Scheme::Light, true),
            );
            assert!(acc.text_px[2] > std.text_px[2], "{}", t.name);
            assert_eq!(acc.target_min_px, 44.0);
        }
    }

    #[test]
    fn hex_round_trips() {
        assert_eq!(Rgba(0x8a, 0x4b, 0x1f, 0xff).hex(), "#8a4b1f");
        assert_eq!(Rgba(0, 0, 0, 0x80).hex(), "#00000080");
        assert_eq!(Rgba(255, 0, 0, 255).to_f32(), [1.0, 0.0, 0.0, 1.0]);
    }
}
