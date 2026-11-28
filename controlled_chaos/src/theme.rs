//! The game's colours and text sizes, from the design system's gamey theme.
//!
//! Every colour on screen comes from `design_system_palette`, so the game
//! matches the other apps and gets the design system's contrast checks. The
//! player picks light or dark and turns accessible mode on or off on the
//! splash screen; [`GameTheme`] holds that choice, and `settings` saves it
//! between launches.

use bevy::prelude::*;
use design_system_palette::{GAMEY, Rgba, Scheme, Tokens};

/// Holds the player's [`GameTheme`] and keeps the window's clear colour on
/// the theme's `bg`.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTheme>().add_systems(
            Update,
            sync_clear_color.run_if(resource_changed::<GameTheme>),
        );
    }
}

/// Sets the window's clear colour, seen behind any UI, to the theme's `bg`.
pub fn sync_clear_color(theme: Res<GameTheme>, mut clear: ResMut<ClearColor>) {
    clear.0 = color(theme.tokens().bg);
}

/// The player's display choice: light or dark, and accessible mode.
///
/// The game always uses the gamey theme. Dark is the default, as the game
/// was dark before it used the design system.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameTheme {
    pub scheme: Scheme,
    pub accessible: bool,
}

impl Default for GameTheme {
    fn default() -> Self {
        Self {
            scheme: Scheme::Dark,
            accessible: false,
        }
    }
}

impl GameTheme {
    /// The resolved token values for this choice.
    pub fn tokens(&self) -> &'static Tokens {
        GAMEY.tokens(self.scheme, self.accessible)
    }

    /// Switches between light and dark.
    pub fn toggle_scheme(&mut self) {
        self.scheme = match self.scheme {
            Scheme::Light => Scheme::Dark,
            Scheme::Dark => Scheme::Light,
        };
    }

    /// Turns accessible mode on or off.
    pub fn toggle_accessible(&mut self) {
        self.accessible = !self.accessible;
    }

    /// Scales a font size designed for the standard 16 px body text to this
    /// mode, so accessible mode's larger base size applies everywhere.
    pub fn font(&self, size: f32) -> f32 {
        size * self.tokens().text_px[BODY_STEP] / STANDARD_BODY_PX
    }

    /// The minimum height of a touch target: 48 px, or the design system's
    /// `target-min` when that is larger.
    pub fn touch_target(&self) -> f32 {
        self.tokens().target_min_px.max(MIN_TOUCH_TARGET_PX)
    }

    /// The colours of a button in each of its states.
    pub fn button(&self) -> ButtonColors {
        let t = self.tokens();
        ButtonColors {
            normal: t.accent,
            hovered: t.accent_hover,
            pressed: t.border_strong,
            disabled: t.surface_raised,
            label: t.on_accent,
            pressed_label: readable_on(t, t.border_strong),
            disabled_label: t.text_muted,
        }
    }
}

/// Where `text_md`, the body size, sits in `Tokens::text_px`.
const BODY_STEP: usize = 2;

/// The body size the game's font sizes were designed for.
const STANDARD_BODY_PX: f32 = 16.0;

/// The game's own minimum touch target, for phones.
const MIN_TOUCH_TARGET_PX: f32 = 48.0;

/// A button's background and label colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonColors {
    pub normal: Rgba,
    pub hovered: Rgba,
    pub pressed: Rgba,
    pub disabled: Rgba,
    pub label: Rgba,
    pub pressed_label: Rgba,
    pub disabled_label: Rgba,
}

impl ButtonColors {
    /// Background and label for a button that is `live` (does something)
    /// and in the given interaction state.
    pub fn state(&self, live: bool, interaction: Interaction) -> (Rgba, Rgba) {
        match (live, interaction) {
            (false, _) => (self.disabled, self.disabled_label),
            (true, Interaction::Pressed) => (self.pressed, self.pressed_label),
            (true, Interaction::Hovered) => (self.hovered, self.label),
            (true, Interaction::None) => (self.normal, self.label),
        }
    }
}

/// A design system colour as a Bevy colour.
pub fn color(c: Rgba) -> Color {
    Color::srgba_u8(c.0, c.1, c.2, c.3)
}

/// Whichever of the theme's `text` and `bg` colours reads better on `bg`:
/// used for labels on coloured fills such as card frames.
pub fn readable_on(t: &Tokens, bg: Rgba) -> Rgba {
    if contrast(t.text, bg) >= contrast(t.bg, bg) {
        t.text
    } else {
        t.bg
    }
}

/// The WCAG 2.2 contrast ratio between two opaque colours, from 1 to 21.
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// WCAG relative luminance.
fn luminance(c: Rgba) -> f32 {
    let [r, g, b, _] = c.to_f32().map(|v| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All four combinations the player can choose.
    fn every_choice() -> Vec<GameTheme> {
        let mut all = Vec::new();
        for scheme in [Scheme::Light, Scheme::Dark] {
            for accessible in [false, true] {
                all.push(GameTheme { scheme, accessible });
            }
        }
        all
    }

    /// WCAG minimum for normal text: AA, or AAA in accessible mode.
    fn min_text(theme: &GameTheme) -> f32 {
        if theme.accessible { 7.0 } else { 4.5 }
    }

    #[test]
    fn contrast_matches_wcag_examples() {
        let black = Rgba(0, 0, 0, 255);
        let white = Rgba(255, 255, 255, 255);
        assert!((contrast(black, white) - 21.0).abs() < 0.01);
        assert!((contrast(white, white) - 1.0).abs() < 0.01);
        // #767676 on white is the classic 4.54:1.
        let grey = Rgba(0x76, 0x76, 0x76, 255);
        assert!((contrast(grey, white) - 4.54).abs() < 0.01);
    }

    #[test]
    fn default_is_gamey_dark_standard() {
        let theme = GameTheme::default();
        assert_eq!(theme.scheme, Scheme::Dark);
        assert!(!theme.accessible);
        assert_eq!(theme.tokens(), &design_system_palette::GAMEY_DARK);
    }

    #[test]
    fn toggles_flip_one_setting_each() {
        let mut theme = GameTheme::default();
        theme.toggle_scheme();
        assert_eq!(theme.scheme, Scheme::Light);
        assert!(!theme.accessible);
        theme.toggle_accessible();
        assert!(theme.accessible);
        assert_eq!(
            theme.tokens(),
            &design_system_palette::GAMEY_ACCESSIBLE_LIGHT
        );
        theme.toggle_scheme();
        theme.toggle_accessible();
        assert_eq!(theme, GameTheme::default());
    }

    #[test]
    fn accessible_mode_scales_text_up_and_standard_keeps_it() {
        for theme in every_choice() {
            let size = theme.font(16.0);
            if theme.accessible {
                assert!(size > 16.0, "{theme:?}");
            } else {
                assert_eq!(size, 16.0, "{theme:?}");
            }
        }
    }

    #[test]
    fn touch_targets_are_at_least_48px() {
        for theme in every_choice() {
            assert!(theme.touch_target() >= 48.0, "{theme:?}");
        }
    }

    #[test]
    fn button_labels_are_readable_in_every_live_state() {
        for theme in every_choice() {
            let b = theme.button();
            for interaction in [
                Interaction::None,
                Interaction::Hovered,
                Interaction::Pressed,
            ] {
                let (bg, label) = b.state(true, interaction);
                let ratio = contrast(label, bg);
                assert!(
                    ratio >= min_text(&theme),
                    "{theme:?} {interaction:?}: {ratio:.2}"
                );
            }
        }
    }

    #[test]
    fn disabled_buttons_look_different_from_live_ones() {
        for theme in every_choice() {
            let b = theme.button();
            let (disabled, _) = b.state(false, Interaction::None);
            assert_ne!(disabled, b.normal, "{theme:?}");
            assert_ne!(disabled, b.hovered, "{theme:?}");
        }
    }

    #[test]
    fn screen_text_is_readable_on_its_background() {
        for theme in every_choice() {
            let t = theme.tokens();
            for bg in [t.bg, t.surface] {
                for fg in [t.text, t.text_muted, t.accent] {
                    let ratio = contrast(fg, bg);
                    assert!(ratio >= min_text(&theme), "{theme:?}: {ratio:.2}");
                }
            }
        }
    }

    #[test]
    fn category_colours_stand_out_from_the_card_body() {
        // Cards draw their category colour as a frame and artwork block on a
        // `surface` body, with `text` labels, so the colour is never behind
        // text: the data palette is only checked to 3:1 (non-text).
        for theme in every_choice() {
            let t = theme.tokens();
            for (i, &frame) in t.data.iter().enumerate() {
                let ratio = contrast(frame, t.surface);
                assert!(ratio >= 3.0, "{theme:?} data-{}: {ratio:.2}", i + 1);
            }
        }
    }

    #[test]
    fn plugin_sets_the_clear_colour_from_the_theme() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ClearColor>()
            .add_plugins(ThemePlugin);
        app.update();
        let bg = GameTheme::default().tokens().bg;
        assert_eq!(app.world().resource::<ClearColor>().0, color(bg));

        app.world_mut().resource_mut::<GameTheme>().toggle_scheme();
        app.update();
        let light = app.world().resource::<GameTheme>().tokens().bg;
        assert_eq!(app.world().resource::<ClearColor>().0, color(light));
    }

    #[test]
    fn color_converts_channels() {
        let c = color(Rgba(255, 0, 0, 255)).to_srgba();
        assert_eq!([c.red, c.green, c.blue, c.alpha], [1.0, 0.0, 0.0, 1.0]);
    }
}
