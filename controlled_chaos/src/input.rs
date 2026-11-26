//! Player input: the keyboard and on-screen buttons both turn into
//! [`PlayerAction`] events, so the game rules see one kind of input whether
//! the player pressed a key, clicked or tapped.

use bevy::prelude::*;

/// Number of hand slots that have a key (**1** to **8**) and a button.
pub const HAND_SLOTS: usize = 8;

/// Something the human player asks the game to do.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    /// Draw a card in the Draw phase, or end the turn in the End phase.
    /// **Space**, or the "Next" button.
    Advance,
    /// Pick the card at this hand index, to attack in the Play phase or to
    /// defend in the Battle phase. **1** to **8**, or tapping the card.
    PickCard(usize),
}

/// An on-screen button that sends `self.0` when pressed. Bevy's UI turns
/// mouse clicks and touches into [`Interaction::Pressed`], so the same button
/// works with a mouse, on a touch screen and on Android.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionButton(pub PlayerAction);

/// The digit keys for hand slots 1 to 8, in order.
const HAND_KEYS: [KeyCode; HAND_SLOTS] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
];

/// Returns the hand index of the first digit key (**1** to **8**) pressed
/// this frame, if any.
pub fn pressed_hand_index(keys: &ButtonInput<KeyCode>) -> Option<usize> {
    HAND_KEYS.iter().position(|key| keys.just_pressed(*key))
}

/// Returns the actions the keyboard asked for this frame.
pub fn keyboard_actions(keys: &ButtonInput<KeyCode>) -> Vec<PlayerAction> {
    let mut actions = Vec::new();
    if keys.just_pressed(KeyCode::Space) {
        actions.push(PlayerAction::Advance);
    }
    if let Some(index) = pressed_hand_index(keys) {
        actions.push(PlayerAction::PickCard(index));
    }
    actions
}

/// Sends a [`PlayerAction`] for each key pressed this frame.
pub fn send_keyboard_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mut actions: EventWriter<PlayerAction>,
) {
    actions.send_batch(keyboard_actions(&keys));
}

/// Sends the [`PlayerAction`] of each visible [`ActionButton`] pressed this
/// frame.
pub fn send_button_actions(
    buttons: Query<(&Interaction, &ActionButton, &Style), Changed<Interaction>>,
    mut actions: EventWriter<PlayerAction>,
) {
    for (interaction, button, style) in &buttons {
        if *interaction == Interaction::Pressed && style.display != Display::None {
            actions.send(button.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn keys_pressed(pressed: &[KeyCode]) -> ButtonInput<KeyCode> {
        let mut keys = ButtonInput::default();
        for key in pressed {
            keys.press(*key);
        }
        keys
    }

    #[test]
    fn pressed_hand_index_maps_digits_to_indices() {
        for (index, key) in HAND_KEYS.iter().enumerate() {
            assert_eq!(pressed_hand_index(&keys_pressed(&[*key])), Some(index));
        }
    }

    #[test]
    fn pressed_hand_index_ignores_other_keys() {
        let keys = keys_pressed(&[KeyCode::Space, KeyCode::Digit9, KeyCode::Digit0]);
        assert_eq!(pressed_hand_index(&keys), None);
    }

    #[test]
    fn pressed_hand_index_picks_the_lowest_digit() {
        let keys = keys_pressed(&[KeyCode::Digit5, KeyCode::Digit2]);
        assert_eq!(pressed_hand_index(&keys), Some(1));
    }

    #[test]
    fn keyboard_actions_space_advances() {
        assert_eq!(
            keyboard_actions(&keys_pressed(&[KeyCode::Space])),
            [PlayerAction::Advance]
        );
    }

    #[test]
    fn keyboard_actions_digit_picks_card() {
        assert_eq!(
            keyboard_actions(&keys_pressed(&[KeyCode::Digit3])),
            [PlayerAction::PickCard(2)]
        );
    }

    #[test]
    fn keyboard_actions_nothing_pressed() {
        assert!(keyboard_actions(&ButtonInput::default()).is_empty());
    }

    /// Spawns one button with `interaction` and `display`, runs
    /// [`send_button_actions`] and returns the actions it sent.
    fn button_actions(interaction: Interaction, display: Display) -> Vec<PlayerAction> {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_event::<PlayerAction>();
        app.world_mut().spawn((
            ActionButton(PlayerAction::PickCard(4)),
            interaction,
            Style {
                display,
                ..default()
            },
        ));
        app.world_mut().run_system_once(send_button_actions);
        let events = app.world().resource::<Events<PlayerAction>>();
        events.get_reader().read(events).copied().collect()
    }

    #[test]
    fn pressed_button_sends_its_action() {
        assert_eq!(
            button_actions(Interaction::Pressed, Display::Flex),
            [PlayerAction::PickCard(4)]
        );
    }

    #[test]
    fn hovered_button_sends_nothing() {
        assert!(button_actions(Interaction::Hovered, Display::Flex).is_empty());
    }

    #[test]
    fn hidden_button_sends_nothing() {
        assert!(button_actions(Interaction::Pressed, Display::None).is_empty());
    }
}
