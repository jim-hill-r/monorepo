use bevy::prelude::*;

mod card;
mod card_library;
mod card_render;
mod deck;
mod game;
mod rules;

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
        .add_plugins(card_render::CardRenderPlugin)
        .run();
}
