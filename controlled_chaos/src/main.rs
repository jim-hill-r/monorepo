use bevy::prelude::*;

mod card;
mod card_library;
mod card_render;
mod deck;
mod game;
mod rules;
mod splash;
mod state;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Controlled Chaos"),
                ..Default::default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera)
        .init_state::<state::AppState>()
        .add_plugins(splash::SplashPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(card_render::CardRenderPlugin)
        .run();
}

/// Spawns the main camera at app startup so it's available for all states.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
