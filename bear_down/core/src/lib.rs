//! The bear_down simulator: data loading, Elo ratings, standings and
//! tiebreakers, Monte Carlo simulation and rooting analysis.
//!
//! This crate does no I/O beyond parsing. The `bear_down_cli` binary downloads
//! the data and writes the results; see `docs/decisions/0001`.

pub mod elo;
pub mod games;
pub mod sim;
pub mod standings;

pub use elo::{EloParams, Ratings};
pub use games::{Game, GameType, Outcome, Schedule, Score, Season, Week};
pub use sim::{GameImpact, SimConfig, Simulation, Swing, TeamOdds, simulate};
pub use standings::{Conference, Division, Record, Region, Standings, Team};

/// The Bears' team abbreviation in nflverse data.
pub const BEARS: &str = "CHI";

/// The NFC North teams' abbreviations in nflverse data, Bears first.
pub const NFC_NORTH: [&str; 4] = [BEARS, "DET", "GB", "MIN"];

/// Whether `team` (an nflverse abbreviation) plays in the NFC North.
pub fn is_nfc_north(team: &str) -> bool {
    NFC_NORTH.contains(&team)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bears_are_in_the_nfc_north() {
        assert!(is_nfc_north(BEARS));
    }

    #[test]
    fn every_division_rival_is_in_the_nfc_north() {
        for team in ["DET", "GB", "MIN"] {
            assert!(is_nfc_north(team), "{team} should be in the NFC North");
        }
    }

    #[test]
    fn other_teams_are_not_in_the_nfc_north() {
        for team in ["KC", "PHI", "chi", "", "CHI "] {
            assert!(
                !is_nfc_north(team),
                "{team:?} should not be in the NFC North"
            );
        }
    }
}
