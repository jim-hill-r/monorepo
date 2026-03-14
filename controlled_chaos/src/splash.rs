use bevy::prelude::*;

use crate::state::AppState;

/// Background color of the splash screen.
pub const SPLASH_BG: Color = Color::srgb(0.1, 0.1, 0.2);

/// Normal button color.
pub const BUTTON_NORMAL: Color = Color::srgb(0.3, 0.3, 0.6);

/// Hovered button color.
pub const BUTTON_HOVERED: Color = Color::srgb(0.4, 0.4, 0.8);

/// Pressed button color.
pub const BUTTON_PRESSED: Color = Color::srgb(0.5, 0.5, 1.0);

/// Marks the root entity of the splash screen so it can be cleaned up on exit.
#[derive(Component, Debug)]
pub struct SplashRoot;

/// Marks the "Start Game" button on the splash screen.
#[derive(Component, Debug)]
pub struct StartGameButton;

/// Plugin that manages the splash screen state.
///
/// When the app is in [`AppState::Splash`] this plugin renders a full-screen
/// overlay with the game title and a "Start Game" button.  Pressing the button
/// transitions to [`AppState::InGame`] and the overlay is torn down.
pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), spawn_splash)
            .add_systems(
                Update,
                (handle_start_button, highlight_start_button).run_if(in_state(AppState::Splash)),
            )
            .add_systems(OnExit(AppState::Splash), despawn_splash);
    }
}

/// Spawns the full-screen splash UI hierarchy.
///
/// The layout is:
/// ```text
/// SplashRoot (full-screen column, centered)
///   ├── Title text  "Controlled Chaos"
///   └── StartGameButton
///         └── "Start Game" label
/// ```
pub fn spawn_splash(mut commands: Commands) {
    commands
        .spawn((
            SplashRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(32.0),
                    ..default()
                },
                background_color: BackgroundColor(SPLASH_BG),
                ..default()
            },
        ))
        .with_children(|root| {
            // Game title
            root.spawn(TextBundle::from_section(
                "Controlled Chaos",
                TextStyle {
                    font_size: 64.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // Start Game button
            root.spawn((
                StartGameButton,
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(BUTTON_NORMAL),
                    ..default()
                },
            ))
            .with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "Start Game",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });
}

/// Transitions to [`AppState::InGame`] when the Start Game button is pressed.
pub fn handle_start_button(
    mut next_state: ResMut<NextState<AppState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::InGame);
        }
    }
}

/// Updates the Start Game button background to reflect hover and press states.
pub fn highlight_start_button(
    mut query: Query<(&Interaction, &mut BackgroundColor), With<StartGameButton>>,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BUTTON_PRESSED),
            Interaction::Hovered => BackgroundColor(BUTTON_HOVERED),
            Interaction::None => BackgroundColor(BUTTON_NORMAL),
        };
    }
}

/// Despawns all entities tagged with [`SplashRoot`] (and their children).
pub fn despawn_splash(mut commands: Commands, query: Query<Entity, With<SplashRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn make_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn spawn_splash_creates_exactly_one_root() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);

        let count = app
            .world_mut()
            .query::<&SplashRoot>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should spawn exactly one splash root");
    }

    #[test]
    fn spawn_splash_creates_start_button() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);

        let count = app
            .world_mut()
            .query::<&StartGameButton>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should spawn exactly one start game button");
    }

    #[test]
    fn spawn_splash_root_has_children() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);

        let mut q = app.world_mut().query::<(&SplashRoot, &Children)>();
        let (_, children) = q.single(app.world());
        assert!(!children.is_empty(), "splash root should have child nodes");
    }

    #[test]
    fn despawn_splash_removes_root() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);
        app.world_mut().run_system_once(despawn_splash);

        let count = app
            .world_mut()
            .query::<&SplashRoot>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "despawn_splash should remove the root entity");
    }

    #[test]
    fn despawn_splash_removes_start_button_as_child() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);
        app.world_mut().run_system_once(despawn_splash);

        let count = app
            .world_mut()
            .query::<&StartGameButton>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "despawn_splash should recursively remove button children"
        );
    }
}
