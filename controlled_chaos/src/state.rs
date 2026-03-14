use bevy::prelude::*;

/// Top-level application state.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// The initial splash screen shown before the game starts.
    #[default]
    Splash,
    /// The main game loop is running.
    InGame,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_default_is_splash() {
        assert_eq!(AppState::default(), AppState::Splash);
    }

    #[test]
    fn app_state_variants_are_not_equal() {
        assert_ne!(AppState::Splash, AppState::InGame);
    }
}
