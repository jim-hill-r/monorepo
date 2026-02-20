use bevy::prelude::*;

mod card;
mod deck;
mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Controlled Chaos"),
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugins(game::GamePlugin)
        .run();
}
