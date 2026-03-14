use bevy::prelude::*;

use crate::card::Card;
use crate::state::AppState;

/// Width of a rendered card in pixels.
pub const CARD_WIDTH: f32 = 120.0;

/// Height of a rendered card in pixels.
pub const CARD_HEIGHT: f32 = 180.0;

/// Height of the artwork placeholder area inside a card.
pub const CARD_ARTWORK_HEIGHT: f32 = 80.0;

/// Background color of the card frame.
pub const CARD_FRAME_COLOR: Color = Color::srgb(0.2, 0.2, 0.4);

/// Background color of the artwork placeholder.
pub const CARD_ARTWORK_COLOR: Color = Color::srgb(0.4, 0.4, 0.6);

/// Marks the root UI node of a rendered card visual.
///
/// The `source` field holds the entity that owns the [`Card`] data this
/// visual was spawned for, allowing systems to match visuals to their logical
/// card entities.
#[derive(Component, Debug)]
pub struct CardVisualRoot {
    pub source: Entity,
}

impl CardVisualRoot {
    /// Returns the logical card entity this visual was created for.
    pub fn source(&self) -> Entity {
        self.source
    }
}

/// Marks the artwork placeholder node within a card visual.
#[derive(Component, Debug)]
pub struct CardArtwork;

/// Marks the name text node within a card visual.
#[derive(Component, Debug)]
pub struct CardNameText;

/// Marks the stats text node within a card visual.
#[derive(Component, Debug)]
pub struct CardStatsText;

/// Plugin that registers card-rendering systems.
pub struct CardRenderPlugin;

impl Plugin for CardRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_card_visuals_from_cards)
            .add_systems(
                Update,
                cleanup_orphaned_visuals.run_if(in_state(AppState::InGame)),
            );
    }
}

/// Startup system: spawns a UI card visual for every [`Card`] entity present
/// at launch.
pub fn spawn_card_visuals_from_cards(mut commands: Commands, cards: Query<(Entity, &Card)>) {
    for (entity, card) in &cards {
        let visual = spawn_card_visual(&mut commands, entity, card);
        debug!(
            "Spawned card visual {:?} for source entity {:?} ({})",
            visual, entity, card.name
        );
    }
}

/// Update system: removes card visuals whose source card entity no longer
/// exists (e.g. after a card has been played or discarded).
pub fn cleanup_orphaned_visuals(
    mut commands: Commands,
    visuals: Query<(Entity, &CardVisualRoot)>,
    cards: Query<&Card>,
) {
    for (visual_entity, root) in &visuals {
        if cards.get(root.source()).is_err() {
            debug!("Removing orphaned card visual {:?}", visual_entity);
            commands.entity(visual_entity).despawn_recursive();
        }
    }
}

/// Spawns the full UI hierarchy representing a single card and returns the
/// root entity.
///
/// The hierarchy is:
/// ```text
/// CardVisualRoot (colored frame, CARD_WIDTH × CARD_HEIGHT)
///   ├── CardNameText   (card name, top of frame)
///   ├── CardArtwork    (artwork placeholder, colored rectangle)
///   └── CardStatsText  (card stats / power value, bottom of frame)
/// ```
pub fn spawn_card_visual(commands: &mut Commands, source: Entity, card: &Card) -> Entity {
    commands
        .spawn((
            CardVisualRoot { source },
            NodeBundle {
                style: Style {
                    width: Val::Px(CARD_WIDTH),
                    height: Val::Px(CARD_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                background_color: BackgroundColor(CARD_FRAME_COLOR),
                ..default()
            },
        ))
        .with_children(|frame| {
            // Card name
            frame.spawn((
                CardNameText,
                TextBundle::from_section(
                    card.name.clone(),
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));

            // Artwork placeholder
            frame.spawn((
                CardArtwork,
                NodeBundle {
                    style: Style {
                        width: Val::Px(CARD_WIDTH - 16.0),
                        height: Val::Px(CARD_ARTWORK_HEIGHT),
                        ..default()
                    },
                    background_color: BackgroundColor(CARD_ARTWORK_COLOR),
                    ..default()
                },
            ));

            // Stats (power value)
            frame.spawn((
                CardStatsText,
                TextBundle::from_section(
                    format!("Power: {}", card.value),
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));
        })
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Creates a minimal Bevy app suitable for unit testing without a window or
    /// renderer.
    fn make_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn spawn_card_visual_creates_root_with_correct_source() {
        let mut app = make_test_app();

        let card = Card::new("Fireball", 10);
        let source = app.world_mut().spawn(card.clone()).id();

        // Run spawn as a one-shot system so commands are fully flushed.
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, source, &card);
            });

        let mut q = app.world_mut().query::<&CardVisualRoot>();
        let root = q.single(app.world());
        assert_eq!(root.source(), source);
    }

    #[test]
    fn spawn_card_visual_attaches_name_text_child() {
        let mut app = make_test_app();

        let card = Card::new("Ace", 14);
        let source = app.world_mut().spawn(card.clone()).id();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, source, &card);
            });

        let mut q = app.world_mut().query::<(&CardVisualRoot, &Children)>();
        let (_, children) = q.single(app.world());
        let has_name_text = children
            .iter()
            .any(|child| app.world().get::<CardNameText>(*child).is_some());
        assert!(has_name_text, "one child should be marked CardNameText");
    }

    #[test]
    fn spawn_card_visual_attaches_artwork_child() {
        let mut app = make_test_app();

        let card = Card::new("King", 13);
        let source = app.world_mut().spawn(card.clone()).id();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, source, &card);
            });

        let mut q = app.world_mut().query::<(&CardVisualRoot, &Children)>();
        let (_, children) = q.single(app.world());
        let has_artwork = children
            .iter()
            .any(|child| app.world().get::<CardArtwork>(*child).is_some());
        assert!(has_artwork, "one child should be marked CardArtwork");
    }

    #[test]
    fn spawn_card_visual_attaches_stats_text_child() {
        let mut app = make_test_app();

        let card = Card::new("Queen", 12);
        let source = app.world_mut().spawn(card.clone()).id();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, source, &card);
            });

        let mut q = app.world_mut().query::<(&CardVisualRoot, &Children)>();
        let (_, children) = q.single(app.world());
        let has_stats = children
            .iter()
            .any(|child| app.world().get::<CardStatsText>(*child).is_some());
        assert!(has_stats, "one child should be marked CardStatsText");
    }

    #[test]
    fn spawn_card_visuals_from_cards_system_creates_one_visual_per_card() {
        let mut app = make_test_app();

        app.world_mut().spawn(Card::new("Ace", 14));
        app.world_mut().spawn(Card::new("King", 13));

        app.add_systems(Update, spawn_card_visuals_from_cards);
        app.update();

        let visual_count = app
            .world_mut()
            .query::<&CardVisualRoot>()
            .iter(app.world())
            .count();
        assert_eq!(visual_count, 2, "one visual per card entity");
    }

    #[test]
    fn spawn_card_visuals_from_cards_system_links_source_entity() {
        let mut app = make_test_app();

        let source = app.world_mut().spawn(Card::new("Jack", 11)).id();

        app.add_systems(Update, spawn_card_visuals_from_cards);
        app.update();

        let mut q = app.world_mut().query::<&CardVisualRoot>();
        let root = q.iter(app.world()).next().expect("should have one visual");
        assert_eq!(root.source(), source);
    }

    #[test]
    fn cleanup_orphaned_visuals_removes_visual_when_source_despawned() {
        let mut app = make_test_app();

        // Spawn a card entity and a matching visual.
        let card = Card::new("Ace", 14);
        let card_entity = app.world_mut().spawn(card.clone()).id();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, card_entity, &card);
            });

        // Despawn the card entity to orphan the visual.
        app.world_mut().despawn(card_entity);

        // Run the cleanup system.
        app.add_systems(Update, cleanup_orphaned_visuals);
        app.update();

        let visual_count = app
            .world_mut()
            .query::<&CardVisualRoot>()
            .iter(app.world())
            .count();
        assert_eq!(visual_count, 0, "orphaned visual should be removed");
    }

    #[test]
    fn cleanup_orphaned_visuals_keeps_visual_when_source_alive() {
        let mut app = make_test_app();

        let card = Card::new("King", 13);
        let card_entity = app.world_mut().spawn(card.clone()).id();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_card_visual(&mut commands, card_entity, &card);
            });

        app.add_systems(Update, cleanup_orphaned_visuals);
        app.update();

        let visual_count = app
            .world_mut()
            .query::<&CardVisualRoot>()
            .iter(app.world())
            .count();
        assert_eq!(visual_count, 1, "visual for live card should remain");
    }
}
