//! Controlled Chaos, a digital tabletop card game.
//!
//! The game lives in this library so every platform starts it the same way:
//! the desktop binary (`src/main.rs`) and the web build call [`main`], and the
//! Android build enters through `android_main`.

use bevy::prelude::*;

mod ai;
mod card;
mod card_library;
mod card_render;
mod deck;
mod game;
mod input;
mod rules;
mod script;
mod settings;
mod splash;
mod state;
mod theme;

/// Builds the app and runs the game until the window closes, or until the
/// script ends when the desktop build is started with `--script <file>`
/// ([`script`]).
pub fn main() -> AppExit {
    let script = match load_script() {
        Ok(script) => script,
        Err(message) => {
            eprintln!("controlled_chaos: {message}");
            eprintln!("usage: controlled_chaos [--script <file> [--screenshots <dir>]]");
            return AppExit::from_code(2);
        }
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Controlled Chaos"),
            // The web build draws into <canvas id="bevy"> and fills the
            // page. Desktop and Android ignore both settings.
            canvas: Some(String::from("#bevy")),
            fit_canvas_to_parent: true,
            ..Default::default()
        }),
        ..default()
    }))
    .add_systems(Startup, setup_camera);
    match script {
        Some(plugin) => {
            script::exit_on_panic();
            // A scripted run always looks the same: default settings, and
            // nothing saved.
            app.add_plugins(settings::SettingsPlugin {
                store: settings::SettingsStore::None,
            })
            .add_plugins(plugin);
        }
        // Loads the saved GameTheme and saves each change to it.
        None => {
            app.add_plugins(settings::SettingsPlugin::default());
        }
    }
    app.add_plugins(theme::ThemePlugin)
        .init_state::<state::AppState>()
        .add_plugins(splash::SplashPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(card_render::CardRenderPlugin)
        .run()
}

/// The [`script::ScriptPlugin`] the command line asks for, if any. The web
/// and Android builds have no command line, so they always play normally.
fn load_script() -> Result<Option<script::ScriptPlugin>, String> {
    if cfg!(any(target_arch = "wasm32", target_os = "android")) {
        return Ok(None);
    }
    let Some(args) = script::parse_args(std::env::args().skip(1))? else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&args.script)
        .map_err(|err| format!("couldn't read {}: {err}", args.script.display()))?;
    let script = script::parse(&text).map_err(|err| format!("{}: {err}", args.script.display()))?;
    Ok(Some(script::ScriptPlugin {
        script,
        screenshots: args.screenshots,
    }))
}

/// Android entry point. The NativeActivity loader looks the game up by this
/// exported name, so it can't be mangled. This is what `#[bevy_main]`
/// generates, written out so the crate can keep `unsafe_code` denied
/// everywhere else.
#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(android_app: bevy::winit::android_activity::AndroidApp) {
    let _ = bevy::winit::ANDROID_APP.set(android_app);
    let _ = main();
}

/// Spawns the main camera at app startup so it's available for all states.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
