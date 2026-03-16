use bevy::prelude::*;

use crate::card::Card;

/// Marks the game as single-player mode where Player 2 is controlled by the AI.
///
/// When this resource is present, Player 2 (index 1) acts automatically each
/// turn instead of waiting for keyboard input.
#[derive(Resource, Debug, Default)]
pub struct AiOpponent;

/// Returns the hand index of the card the AI should play when attacking.
///
/// Strategy: play the highest-value card to maximise the chance of winning the
/// battle.
pub fn choose_attack_card(hand: &[Card]) -> usize {
    hand.iter()
        .enumerate()
        .max_by_key(|(_, c)| c.value)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Returns the hand index of the card the AI should play when defending.
///
/// Strategy: play the lowest-value card that strictly beats the attacker's
/// value.  If no such card exists, play the lowest-value card to minimise the
/// loss.
pub fn choose_defense_card(hand: &[Card], attacker_value: u32) -> usize {
    // Find the lowest card that beats the attacker.
    if let Some((idx, _)) = hand
        .iter()
        .enumerate()
        .filter(|(_, c)| c.value > attacker_value)
        .min_by_key(|(_, c)| c.value)
    {
        return idx;
    }

    // No winning card — play the lowest-value card to minimise damage.
    hand.iter()
        .enumerate()
        .min_by_key(|(_, c)| c.value)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hand(values: &[u32]) -> Vec<Card> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| Card::new(format!("Card{i}"), v))
            .collect()
    }

    // ── choose_attack_card ────────────────────────────────────────────────────

    #[test]
    fn choose_attack_card_returns_highest_value_index() {
        let hand = make_hand(&[5, 12, 7]);
        assert_eq!(choose_attack_card(&hand), 1);
    }

    #[test]
    fn choose_attack_card_single_card_returns_zero() {
        let hand = make_hand(&[10]);
        assert_eq!(choose_attack_card(&hand), 0);
    }

    #[test]
    fn choose_attack_card_highest_is_last() {
        let hand = make_hand(&[3, 7, 14]);
        assert_eq!(choose_attack_card(&hand), 2);
    }

    // ── choose_defense_card ───────────────────────────────────────────────────

    #[test]
    fn choose_defense_card_picks_lowest_winning_card() {
        // Attacker has value 7; hand has 5, 10, 8, 3.
        // Winning cards: 10 (index 1), 8 (index 2).  Lowest winning: 8 (index 2).
        let hand = make_hand(&[5, 10, 8, 3]);
        assert_eq!(choose_defense_card(&hand, 7), 2);
    }

    #[test]
    fn choose_defense_card_no_winning_card_picks_lowest() {
        // Attacker has value 14; hand has 5, 10, 8.
        // No winning cards.  Lowest: 5 (index 0).
        let hand = make_hand(&[5, 10, 8]);
        assert_eq!(choose_defense_card(&hand, 14), 0);
    }

    #[test]
    fn choose_defense_card_exact_tie_is_not_winning() {
        // A tie is not a win (defender value must strictly exceed attacker value).
        // Attacker has value 7; hand has 7, 5, 6.  No winner; lowest is 5 (index 1).
        let hand = make_hand(&[7, 5, 6]);
        assert_eq!(choose_defense_card(&hand, 7), 1);
    }

    #[test]
    fn choose_defense_card_single_card_returns_zero() {
        let hand = make_hand(&[3]);
        assert_eq!(choose_defense_card(&hand, 10), 0);
    }

    #[test]
    fn choose_defense_card_picks_exact_beat_not_overkill() {
        // Attacker has value 5; hand has 14, 6, 3.
        // Winning cards: 14 (index 0), 6 (index 1).  Lowest winning: 6 (index 1).
        let hand = make_hand(&[14, 6, 3]);
        assert_eq!(choose_defense_card(&hand, 5), 1);
    }
}
