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
mod settings;
mod splash;
mod state;
mod theme;

/// Builds the app and runs the game until the window closes.
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        .add_systems(Startup, setup_camera)
        // Loads the saved GameTheme and saves each change to it.
        .add_plugins(settings::SettingsPlugin::default())
        .add_plugins(theme::ThemePlugin)
        .init_state::<state::AppState>()
        .add_plugins(splash::SplashPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(card_render::CardRenderPlugin)
        .run();
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
    main();
}

/// Spawns the main camera at app startup so it's available for all states.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
