//! Basic strategy implementation for blackjack.
//!
//! Provides decision-making logic for demo mode and testing.

use crate::card::Rank;
use crate::game::{Action, GameState};
use crate::hand::Hand;

/// Strategy types for automated play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Strategy {
    /// Basic strategy - statistically optimal play without card counting
    #[default]
    Basic,
    /// Random actions for chaos testing
    Random,
    /// Never bust - stand on 12 or higher
    NeverBust,
}

impl std::str::FromStr for Strategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Strategy::Basic),
            "random" => Ok(Strategy::Random),
            "never_bust" | "neverbust" => Ok(Strategy::NeverBust),
            _ => Err(format!("Unknown strategy: {}", s)),
        }
    }
}

/// Decides the next action based on the strategy.
pub fn decide_action(state: &GameState, strategy: Strategy) -> Option<Action> {
    let actions = state.available_actions();
    if actions.is_empty() {
        return None;
    }

    match strategy {
        Strategy::Basic => basic_strategy_action(state, &actions),
        Strategy::Random => random_action(&actions),
        Strategy::NeverBust => never_bust_action(state, &actions),
    }
}

/// Implements basic strategy.
fn basic_strategy_action(state: &GameState, actions: &[Action]) -> Option<Action> {
    // Handle insurance - always decline (basic strategy)
    if actions.contains(&Action::Insurance(false)) {
        return Some(Action::Insurance(false));
    }

    // Handle deal
    if actions.contains(&Action::Deal) {
        return Some(Action::Deal);
    }

    let hand_index = state.current_hand_index()?;
    let hand = &state.player_hands[hand_index];
    let dealer_upcard = state.dealer_upcard();
    let dealer_value = dealer_upcard.value();

    // Check for pair splitting
    if hand.is_pair() && actions.contains(&Action::Split) {
        if should_split(hand, dealer_value) {
            return Some(Action::Split);
        }
    }

    let player_value = hand.value();
    let is_soft = hand.is_soft();

    // Soft hand strategy
    if is_soft {
        return soft_hand_strategy(player_value, dealer_value, hand.len() == 2, actions);
    }

    // Hard hand strategy
    hard_hand_strategy(player_value, dealer_value, hand.len() == 2, actions)
}

/// Determines if a pair should be split.
fn should_split(hand: &Hand, dealer_value: u8) -> bool {
    let card_rank = hand.cards()[0].rank;

    match card_rank {
        // Always split aces and eights
        Rank::Ace | Rank::Eight => true,

        // Never split tens or fives
        Rank::Ten | Rank::Jack | Rank::Queen | Rank::King | Rank::Five => false,

        // Split twos and threes vs 2-7
        Rank::Two | Rank::Three => dealer_value >= 2 && dealer_value <= 7,

        // Split fours vs 5-6
        Rank::Four => dealer_value >= 5 && dealer_value <= 6,

        // Split sixes vs 2-6
        Rank::Six => dealer_value >= 2 && dealer_value <= 6,

        // Split sevens vs 2-7
        Rank::Seven => dealer_value >= 2 && dealer_value <= 7,

        // Split nines vs 2-9 except 7
        Rank::Nine => dealer_value >= 2 && dealer_value <= 9 && dealer_value != 7,
    }
}

/// Strategy for soft hands.
fn soft_hand_strategy(
    player_value: u8,
    dealer_value: u8,
    can_double: bool,
    actions: &[Action],
) -> Option<Action> {
    // Soft 19-21: Always stand
    if player_value >= 19 {
        return Some(Action::Stand);
    }

    // Soft 18
    if player_value == 18 {
        // Double vs 3-6
        if can_double && dealer_value >= 3 && dealer_value <= 6 && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        // Stand vs 2, 7, 8
        if dealer_value == 2 || dealer_value == 7 || dealer_value == 8 {
            return Some(Action::Stand);
        }
        // Hit vs 9, 10, A
        return Some(Action::Hit);
    }

    // Soft 17
    if player_value == 17 {
        // Double vs 3-6
        if can_double && dealer_value >= 3 && dealer_value <= 6 && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    // Soft 15-16: Double vs 4-6, else hit
    if player_value >= 15 && player_value <= 16 {
        if can_double && dealer_value >= 4 && dealer_value <= 6 && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    // Soft 13-14: Double vs 5-6, else hit
    if player_value >= 13 && player_value <= 14 {
        if can_double && dealer_value >= 5 && dealer_value <= 6 && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    Some(Action::Hit)
}

/// Strategy for hard hands.
fn hard_hand_strategy(
    player_value: u8,
    dealer_value: u8,
    can_double: bool,
    actions: &[Action],
) -> Option<Action> {
    // 17+: Always stand
    if player_value >= 17 {
        return Some(Action::Stand);
    }

    // 13-16: Stand vs 2-6, hit vs 7+
    if player_value >= 13 && player_value <= 16 {
        if dealer_value >= 2 && dealer_value <= 6 {
            return Some(Action::Stand);
        }
        return Some(Action::Hit);
    }

    // 12: Stand vs 4-6, hit otherwise
    if player_value == 12 {
        if dealer_value >= 4 && dealer_value <= 6 {
            return Some(Action::Stand);
        }
        return Some(Action::Hit);
    }

    // 11: Always double
    if player_value == 11 {
        if can_double && actions.contains(&Action::Double) {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    // 10: Double vs 2-9
    if player_value == 10 {
        if can_double
            && dealer_value >= 2
            && dealer_value <= 9
            && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    // 9: Double vs 3-6
    if player_value == 9 {
        if can_double
            && dealer_value >= 3
            && dealer_value <= 6
            && actions.contains(&Action::Double)
        {
            return Some(Action::Double);
        }
        return Some(Action::Hit);
    }

    // 8 or less: Always hit
    Some(Action::Hit)
}

/// Random strategy for testing.
fn random_action(actions: &[Action]) -> Option<Action> {
    use rand::seq::SliceRandom;

    // Filter out insurance - randomly decide
    let filtered: Vec<_> = actions
        .iter()
        .filter(|a| !matches!(a, Action::Insurance(_)))
        .cloned()
        .collect();

    if filtered.is_empty() {
        // Must be insurance decision
        if rand::random() {
            Some(Action::Insurance(true))
        } else {
            Some(Action::Insurance(false))
        }
    } else {
        let mut rng = rand::thread_rng();
        filtered.choose(&mut rng).cloned()
    }
}

/// Never bust strategy - very conservative.
fn never_bust_action(state: &GameState, actions: &[Action]) -> Option<Action> {
    // Handle insurance - always decline
    if actions.contains(&Action::Insurance(false)) {
        return Some(Action::Insurance(false));
    }

    // Handle deal
    if actions.contains(&Action::Deal) {
        return Some(Action::Deal);
    }

    let hand_index = state.current_hand_index()?;
    let hand = &state.player_hands[hand_index];

    // Stand on 12 or higher
    if hand.value() >= 12 {
        Some(Action::Stand)
    } else {
        Some(Action::Hit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn test_strategy_parse() {
        assert_eq!("basic".parse::<Strategy>().unwrap(), Strategy::Basic);
        assert_eq!("random".parse::<Strategy>().unwrap(), Strategy::Random);
        assert!("invalid".parse::<Strategy>().is_err());
    }

    #[test]
    fn test_basic_strategy_deals() {
        let state = GameState::new(GameConfig::default());
        let action = decide_action(&state, Strategy::Basic);
        assert_eq!(action, Some(Action::Deal));
    }

    #[test]
    fn test_basic_strategy_declines_insurance() {
        let mut state = GameState::with_seed(GameConfig::default(), 99999);
        state.apply(Action::Deal).unwrap();

        if state.available_actions().contains(&Action::Insurance(false)) {
            let action = decide_action(&state, Strategy::Basic);
            assert_eq!(action, Some(Action::Insurance(false)));
        }
    }
}
