//! Saves the player's display settings ([`GameTheme`]) between launches.
//!
//! The design system keeps such choices in files, not on a server, so the
//! settings are a few lines of TOML:
//!
//! ```toml
//! scheme = "dark"
//! accessible = false
//! ```
//!
//! Where they live depends on the platform ([`SettingsStore::for_platform`]):
//!
//! - **Web:** the browser's `localStorage`, under `controlled_chaos.settings`.
//! - **Desktop:** `settings.toml` in a `controlled_chaos` folder in the
//!   platform's config directory (`~/Library/Application Support` on macOS,
//!   `$XDG_CONFIG_HOME` or `~/.config` on Linux, `%APPDATA%` on Windows).
//! - **Android:** `settings.toml` in the app's internal data directory.
//!
//! With nothing saved yet, the game starts in dark mode, and in accessible
//! mode when the platform asks for more contrast. Only the web build can
//! tell (the `prefers-contrast: more` media query); Bevy doesn't expose the
//! desktop or Android setting.

// The web build keeps the settings in localStorage, so it never uses the
// file store or its names.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use std::path::PathBuf;

use bevy::prelude::*;
use design_system_palette::Scheme;

use crate::theme::GameTheme;

/// The settings file's name on desktop and Android.
pub const FILE_NAME: &str = "settings.toml";

/// The folder in the desktop config directory that holds [`FILE_NAME`].
pub const APP_DIR: &str = "controlled_chaos";

/// The `localStorage` key the web build saves the settings under.
#[cfg(target_arch = "wasm32")]
pub const STORAGE_KEY: &str = "controlled_chaos.settings";

/// Loads the saved [`GameTheme`] at startup and saves it whenever the player
/// changes it.
pub struct SettingsPlugin {
    pub store: SettingsStore,
}

impl Default for SettingsPlugin {
    fn default() -> Self {
        Self {
            store: SettingsStore::for_platform(),
        }
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let base = GameTheme {
            accessible: prefers_more_contrast(),
            ..GameTheme::default()
        };
        let theme = self.store.read().map_or(base, |text| parse(&text, base));
        app.insert_resource(theme)
            .insert_resource(self.store.clone())
            .add_systems(Update, save_settings);
    }
}

/// Writes the theme to the store when the player changes it. Inserting the
/// theme at startup doesn't count, so a launch that changes nothing writes
/// nothing.
pub fn save_settings(theme: Res<GameTheme>, store: Res<SettingsStore>) {
    if !theme.is_changed() || theme.is_added() {
        return;
    }
    if let Err(err) = store.write(&to_toml(&theme)) {
        warn!("couldn't save the display settings: {err}");
    }
}

/// Where the settings are kept.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub enum SettingsStore {
    /// A TOML file at this path (desktop and Android).
    File(PathBuf),
    /// The browser's `localStorage` (web).
    #[cfg(target_arch = "wasm32")]
    Browser,
    /// Nowhere: the platform has no place for them, so they last until the
    /// game closes.
    None,
}

impl SettingsStore {
    /// The store for the platform the game is running on.
    pub fn for_platform() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            SettingsStore::Browser
        }
        #[cfg(target_os = "android")]
        {
            bevy::winit::ANDROID_APP
                .get()
                .and_then(|app| app.internal_data_path())
                .map_or(SettingsStore::None, |dir| {
                    SettingsStore::File(dir.join(FILE_NAME))
                })
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            desktop_settings_path(|name| std::env::var_os(name))
                .map_or(SettingsStore::None, SettingsStore::File)
        }
    }

    /// The saved settings text, or `None` when nothing is saved or it can't
    /// be read.
    pub fn read(&self) -> Option<String> {
        match self {
            SettingsStore::File(path) => std::fs::read_to_string(path).ok(),
            #[cfg(target_arch = "wasm32")]
            SettingsStore::Browser => web_sys::window()?
                .local_storage()
                .ok()??
                .get_item(STORAGE_KEY)
                .ok()?,
            SettingsStore::None => None,
        }
    }

    /// Saves the settings text, creating the file's folder if needed.
    pub fn write(&self, text: &str) -> Result<(), String> {
        match self {
            SettingsStore::File(path) => {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                }
                std::fs::write(path, text).map_err(|e| e.to_string())
            }
            #[cfg(target_arch = "wasm32")]
            SettingsStore::Browser => web_sys::window()
                .ok_or("no browser window")?
                .local_storage()
                .ok()
                .flatten()
                .ok_or("localStorage is not available")?
                .set_item(STORAGE_KEY, text)
                .map_err(|e| format!("{e:?}")),
            SettingsStore::None => Ok(()),
        }
    }
}

/// The settings file's path on desktop, from the environment variable lookup
/// `var` (normally `std::env::var_os`).
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn desktop_settings_path(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    let absolute = |name: &str| {
        var(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let base = if cfg!(target_os = "macos") {
        absolute("HOME")?.join("Library/Application Support")
    } else if cfg!(windows) {
        absolute("APPDATA")?
    } else {
        absolute("XDG_CONFIG_HOME").or_else(|| absolute("HOME").map(|h| h.join(".config")))?
    };
    Some(base.join(APP_DIR).join(FILE_NAME))
}

/// Whether the platform asks for more contrast. Only the browser says.
fn prefers_more_contrast() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.match_media("(prefers-contrast: more)").ok().flatten())
            .is_some_and(|query| query.matches())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// The settings as TOML.
pub fn to_toml(theme: &GameTheme) -> String {
    let scheme = match theme.scheme {
        Scheme::Light => "light",
        Scheme::Dark => "dark",
    };
    format!(
        "# Controlled Chaos display settings, saved by the game.\n\
         scheme = \"{scheme}\"\n\
         accessible = {}\n",
        theme.accessible
    )
}

/// Reads settings saved by [`to_toml`]. A setting that is missing or has a
/// value the game doesn't know keeps its value from `base`, so a damaged or
/// older file never stops the game from starting.
pub fn parse(text: &str, base: GameTheme) -> GameTheme {
    let mut theme = base;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.split('#').next().unwrap_or("").trim();
        match (key.trim(), value) {
            ("scheme", "\"light\"") => theme.scheme = Scheme::Light,
            ("scheme", "\"dark\"") => theme.scheme = Scheme::Dark,
            ("accessible", "true") => theme.accessible = true,
            ("accessible", "false") => theme.accessible = false,
            _ => {}
        }
    }
    theme
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

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

    /// An empty folder of its own for one test.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "controlled_chaos-settings-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// An app with the settings plugin, keeping its settings at `path`.
    fn app_with_file(path: &Path) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SettingsPlugin {
            store: SettingsStore::File(path.to_path_buf()),
        });
        app
    }

    #[test]
    fn every_choice_survives_a_round_trip() {
        for theme in every_choice() {
            let text = to_toml(&theme);
            assert_eq!(parse(&text, GameTheme::default()), theme, "{text}");
        }
    }

    #[test]
    fn saved_text_is_plain_toml() {
        let theme = GameTheme {
            scheme: Scheme::Light,
            accessible: true,
        };
        let text = to_toml(&theme);
        assert!(text.contains("scheme = \"light\"\n"), "{text}");
        assert!(text.contains("accessible = true\n"), "{text}");
    }

    #[test]
    fn missing_and_unknown_settings_keep_the_base() {
        let base = GameTheme {
            scheme: Scheme::Dark,
            accessible: true,
        };
        assert_eq!(parse("", base), base);
        assert_eq!(parse("scheme = \"sepia\"\naccessible = 3\n", base), base);
        assert_eq!(parse("volume = 11\nnot toml at all", base), base);
        let light = parse("scheme = \"light\"", base);
        assert_eq!(light.scheme, Scheme::Light);
        assert!(light.accessible);
    }

    #[test]
    fn parse_allows_spacing_and_comments() {
        let text = "  # a comment\n  scheme=\"light\"   # trailing\naccessible   =   true\n";
        let theme = parse(text, GameTheme::default());
        assert_eq!(theme.scheme, Scheme::Light);
        assert!(theme.accessible);
    }

    #[test]
    fn starts_with_the_default_when_nothing_is_saved() {
        let path = temp_dir("nothing-saved").join(FILE_NAME);
        let mut app = app_with_file(&path);
        app.update();
        assert_eq!(*app.world().resource::<GameTheme>(), GameTheme::default());
        // Starting doesn't write a file: only a change does.
        assert!(!path.exists());
    }

    #[test]
    fn starts_with_the_saved_settings() {
        let dir = temp_dir("saved");
        let path = dir.join(FILE_NAME);
        let saved = GameTheme {
            scheme: Scheme::Light,
            accessible: true,
        };
        SettingsStore::File(path.clone())
            .write(&to_toml(&saved))
            .expect("write settings");
        let mut app = app_with_file(&path);
        app.update();
        assert_eq!(*app.world().resource::<GameTheme>(), saved);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn saves_a_change_for_the_next_launch() {
        let dir = temp_dir("change");
        let path = dir.join(APP_DIR).join(FILE_NAME);
        let mut app = app_with_file(&path);
        app.update();
        app.world_mut().resource_mut::<GameTheme>().toggle_scheme();
        app.world_mut()
            .resource_mut::<GameTheme>()
            .toggle_accessible();
        app.update();

        let mut next = app_with_file(&path);
        next.update();
        let theme = *next.world().resource::<GameTheme>();
        assert_eq!(theme.scheme, Scheme::Light);
        assert!(theme.accessible);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_settings_plugin_overrides_the_theme_plugin_default() {
        let dir = temp_dir("with-theme-plugin");
        let path = dir.join(FILE_NAME);
        let saved = GameTheme {
            scheme: Scheme::Light,
            accessible: false,
        };
        SettingsStore::File(path.clone())
            .write(&to_toml(&saved))
            .expect("write settings");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ClearColor>()
            .add_plugins(SettingsPlugin {
                store: SettingsStore::File(path),
            })
            .add_plugins(crate::theme::ThemePlugin);
        app.update();
        assert_eq!(*app.world().resource::<GameTheme>(), saved);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_unwritable_store_doesnt_stop_the_game() {
        // A file where the settings folder should be makes the write fail.
        let dir = temp_dir("unwritable");
        std::fs::create_dir_all(&dir).expect("create dir");
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, "").expect("write blocker");
        let mut app = app_with_file(&blocker.join(FILE_NAME));
        app.update();
        app.world_mut().resource_mut::<GameTheme>().toggle_scheme();
        app.update();
        assert_eq!(app.world().resource::<GameTheme>().scheme, Scheme::Light);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn store_none_keeps_nothing() {
        assert_eq!(SettingsStore::None.read(), None);
        assert_eq!(SettingsStore::None.write("scheme = \"light\""), Ok(()));
    }

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let pairs: Vec<(String, OsString)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), OsString::from(v)))
            .collect();
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_uses_xdg_config_home_then_dot_config() {
        let xdg = desktop_settings_path(env(&[("XDG_CONFIG_HOME", "/cfg"), ("HOME", "/home/p")]));
        assert_eq!(
            xdg,
            Some(PathBuf::from("/cfg/controlled_chaos/settings.toml"))
        );
        let home = desktop_settings_path(env(&[("HOME", "/home/p")]));
        assert_eq!(
            home,
            Some(PathBuf::from(
                "/home/p/.config/controlled_chaos/settings.toml"
            ))
        );
        // The XDG spec ignores a relative XDG_CONFIG_HOME.
        let relative =
            desktop_settings_path(env(&[("XDG_CONFIG_HOME", "cfg"), ("HOME", "/home/p")]));
        assert_eq!(relative, home);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_uses_application_support() {
        let path = desktop_settings_path(env(&[("HOME", "/Users/p")]));
        assert_eq!(
            path,
            Some(PathBuf::from(
                "/Users/p/Library/Application Support/controlled_chaos/settings.toml"
            ))
        );
    }

    #[test]
    fn no_home_means_no_settings_file() {
        assert_eq!(desktop_settings_path(env(&[])), None);
    }
}
