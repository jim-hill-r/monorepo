use bevy::prelude::*;

use crate::state::AppState;
use crate::theme::{self, GameTheme};

/// Marks the root entity of the splash screen so it can be cleaned up on exit.
#[derive(Component, Debug)]
pub struct SplashRoot;

/// Marks the "Start Game" button on the splash screen.
#[derive(Component, Debug)]
pub struct StartGameButton;

/// A display setting the splash screen offers as a button.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingButton {
    /// Switches between the light and dark colour scheme.
    Scheme,
    /// Turns accessible mode (higher contrast, larger text) on or off.
    Accessible,
}

impl SettingButton {
    /// The button's label, showing the current value, such as
    /// `"Colours: Dark"`.
    pub fn label(self, theme: &GameTheme) -> &'static str {
        match self {
            SettingButton::Scheme => match theme.scheme {
                design_system_palette::Scheme::Light => "Colours: Light",
                design_system_palette::Scheme::Dark => "Colours: Dark",
            },
            SettingButton::Accessible => {
                if theme.accessible {
                    "Accessible: On"
                } else {
                    "Accessible: Off"
                }
            }
        }
    }

    /// Changes the setting this button controls.
    pub fn apply(self, theme: &mut GameTheme) {
        match self {
            SettingButton::Scheme => theme.toggle_scheme(),
            SettingButton::Accessible => theme.toggle_accessible(),
        }
    }
}

/// Marks every button on the splash screen, so they share hover colours.
#[derive(Component, Debug)]
pub struct SplashButton;

/// Plugin that manages the splash screen state.
///
/// When the app is in [`AppState::Splash`] this plugin renders a full-screen
/// overlay with the game title, a "Start Game" button and the display
/// settings (light or dark, and accessible mode). Pressing Start Game
/// transitions to [`AppState::InGame`] and the overlay is torn down; changing
/// a setting redraws the splash screen in the new colours.
pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), spawn_splash)
            .add_systems(
                Update,
                (
                    handle_start_button,
                    handle_setting_buttons,
                    respawn_splash.run_if(resource_changed::<GameTheme>),
                    highlight_splash_buttons,
                )
                    .chain()
                    .run_if(in_state(AppState::Splash)),
            )
            .add_systems(OnExit(AppState::Splash), despawn_splash);
    }
}

/// Width of each splash screen button, in pixels.
const SPLASH_BUTTON_WIDTH: f32 = 260.0;

/// Spawns the full-screen splash UI hierarchy.
///
/// The layout is:
/// ```text
/// SplashRoot (full-screen column, centered)
///   ├── Title text  "Controlled Chaos"
///   ├── StartGameButton
///   │     └── "Start Game" label
///   └── Settings row
///         ├── SettingButton::Scheme      "Colours: Dark"
///         └── SettingButton::Accessible  "Accessible: Off"
/// ```
pub fn spawn_splash(mut commands: Commands, theme: Res<GameTheme>) {
    let tokens = theme.tokens();
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
                    row_gap: Val::Px(24.0),
                    ..default()
                },
                background_color: BackgroundColor(theme::color(tokens.bg)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Game title
            root.spawn(TextBundle::from_section(
                "Controlled Chaos",
                TextStyle {
                    font_size: theme.font(56.0),
                    color: theme::color(tokens.text),
                    ..default()
                },
            ));

            spawn_splash_button(root, &theme, StartGameButton, "Start Game");

            // Display settings, side by side.
            root.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(16.0),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                for setting in [SettingButton::Scheme, SettingButton::Accessible] {
                    spawn_splash_button(row, &theme, setting, setting.label(&theme));
                }
            });
        });
}

/// Spawns one splash screen button with a text label.
fn spawn_splash_button(
    parent: &mut ChildBuilder,
    theme: &GameTheme,
    marker: impl Component,
    label: &str,
) {
    let colors = theme.button();
    parent
        .spawn((
            marker,
            SplashButton,
            ButtonBundle {
                style: Style {
                    width: Val::Px(SPLASH_BUTTON_WIDTH),
                    min_height: Val::Px(theme.touch_target().max(56.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(theme::color(colors.normal)),
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: theme.font(22.0),
                    color: theme::color(colors.label),
                    ..default()
                },
            ));
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

/// Changes a display setting when its button is pressed.
pub fn handle_setting_buttons(
    mut theme: ResMut<GameTheme>,
    interaction_query: Query<(&Interaction, &SettingButton), Changed<Interaction>>,
) {
    for (interaction, setting) in &interaction_query {
        if *interaction == Interaction::Pressed {
            setting.apply(&mut theme);
        }
    }
}

/// Redraws the splash screen after a display setting changed, so every
/// colour and size follows the new choice.
pub fn respawn_splash(
    mut commands: Commands,
    theme: Res<GameTheme>,
    roots: Query<Entity, With<SplashRoot>>,
) {
    if roots.is_empty() {
        return;
    }
    for entity in &roots {
        commands.entity(entity).despawn_recursive();
    }
    spawn_splash(commands, theme);
}

/// Colours the splash screen buttons and their labels for their hover and
/// press states.
pub fn highlight_splash_buttons(
    theme: Res<GameTheme>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor, &Children), With<SplashButton>>,
    mut labels: Query<&mut Text>,
) {
    let colors = theme.button();
    for (interaction, mut bg, children) in &mut buttons {
        let (fill, label) = colors.state(true, *interaction);
        let fill = theme::color(fill);
        if bg.0 != fill {
            bg.0 = fill;
        }
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                let label = theme::color(label);
                if text.sections[0].style.color != label {
                    text.sections[0].style.color = label;
                }
            }
        }
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
        app.init_resource::<GameTheme>();
        app
    }

    fn count<C: Component>(app: &mut App) -> usize {
        app.world_mut().query::<&C>().iter(app.world()).count()
    }

    /// The splash root's background colour.
    fn splash_bg(app: &mut App) -> Color {
        let mut q = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<SplashRoot>>();
        q.single(app.world()).0
    }

    /// The entity of the given setting's button.
    fn setting_button(app: &mut App, setting: SettingButton) -> Entity {
        let mut q = app.world_mut().query::<(Entity, &SettingButton)>();
        q.iter(app.world())
            .find(|(_, s)| **s == setting)
            .map(|(e, _)| e)
            .expect("setting button should exist")
    }

    /// An app running the splash systems the plugin adds, minus the state
    /// machinery.
    fn make_settings_app() -> App {
        let mut app = make_test_app();
        app.add_systems(
            Update,
            (
                handle_setting_buttons,
                respawn_splash.run_if(resource_changed::<GameTheme>),
            )
                .chain(),
        );
        app.world_mut().run_system_once(spawn_splash);
        app.update();
        app
    }

    #[test]
    fn spawn_splash_creates_both_setting_buttons() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);
        assert_eq!(count::<SettingButton>(&mut app), 2);
        assert_eq!(
            count::<SplashButton>(&mut app),
            3,
            "start plus two settings"
        );
    }

    #[test]
    fn setting_labels_show_the_current_value() {
        let mut theme = GameTheme::default();
        assert_eq!(SettingButton::Scheme.label(&theme), "Colours: Dark");
        assert_eq!(SettingButton::Accessible.label(&theme), "Accessible: Off");
        SettingButton::Scheme.apply(&mut theme);
        SettingButton::Accessible.apply(&mut theme);
        assert_eq!(SettingButton::Scheme.label(&theme), "Colours: Light");
        assert_eq!(SettingButton::Accessible.label(&theme), "Accessible: On");
    }

    #[test]
    fn splash_uses_the_theme_background() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);
        let expected = theme::color(GameTheme::default().tokens().bg);
        assert_eq!(splash_bg(&mut app), expected);
    }

    #[test]
    fn pressing_the_scheme_button_switches_to_light_and_redraws() {
        let mut app = make_settings_app();
        let button = setting_button(&mut app, SettingButton::Scheme);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        let theme = *app.world().resource::<GameTheme>();
        assert_eq!(theme.scheme, design_system_palette::Scheme::Light);
        assert_eq!(count::<SplashRoot>(&mut app), 1, "redrawn, not duplicated");
        assert_eq!(splash_bg(&mut app), theme::color(theme.tokens().bg));
    }

    #[test]
    fn pressing_the_accessible_button_turns_accessible_mode_on() {
        let mut app = make_settings_app();
        let button = setting_button(&mut app, SettingButton::Accessible);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        let theme = app.world().resource::<GameTheme>();
        assert!(theme.accessible);
        assert_eq!(theme.scheme, design_system_palette::Scheme::Dark);
        assert_eq!(count::<SettingButton>(&mut app), 2);
    }

    #[test]
    fn highlight_colours_hovered_buttons() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_splash);
        let button = setting_button(&mut app, SettingButton::Scheme);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Hovered);
        app.world_mut().run_system_once(highlight_splash_buttons);

        let hovered = GameTheme::default().button().hovered;
        let bg = app.world().get::<BackgroundColor>(button).map(|b| b.0);
        assert_eq!(bg, Some(theme::color(hovered)));
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
