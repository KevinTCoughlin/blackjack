//! Output formatting for blackjack.
//!
//! Supports JSON for machine-readable output and pretty text with card art.

use crate::card::Card;
use crate::game::{GamePhase, GameState, Outcome};
use crate::hand::Hand;

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Machine-readable JSON
    #[default]
    Json,
    /// Human-readable with card art
    Pretty,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "pretty" | "text" => Ok(OutputFormat::Pretty),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Formats the game state according to the specified format.
pub fn format_game_state(state: &GameState, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(state).unwrap_or_default(),
        OutputFormat::Pretty => format_pretty(state),
    }
}

/// Creates a pretty-printed representation of the game state.
fn format_pretty(state: &GameState) -> String {
    let mut output = String::new();

    // Header
    output.push_str("┌─────────────────────────────────────────┐\n");
    output.push_str("│              BLACKJACK                  │\n");
    output.push_str("├─────────────────────────────────────────┤\n");

    // Dealer hand
    let dealer_str = format_dealer_hand(state);
    output.push_str(&format!("│  Dealer: {:<31}│\n", dealer_str));

    // Dealer value
    let dealer_value = if state.is_finished() || matches!(state.phase, GamePhase::DealerTurn) {
        format!("Value: {}", state.dealer_hand.value())
    } else {
        "Value: ?".to_string()
    };
    output.push_str(&format!("│{:>41}│\n", dealer_value));

    output.push_str("│                                         │\n");

    // Player hands
    for (i, hand) in state.player_hands.iter().enumerate() {
        let hand_label = if state.player_hands.len() > 1 {
            format!("Hand {}: ", i + 1)
        } else {
            "You:    ".to_string()
        };

        let cards_str = format_hand_cards(hand);
        output.push_str(&format!("│  {}{:<28}│\n", hand_label, cards_str));

        // Hand status
        let status = format_hand_status(state, i, hand);
        output.push_str(&format!("│{:>41}│\n", status));
    }

    output.push_str("├─────────────────────────────────────────┤\n");

    // Available actions or outcomes
    if state.is_finished() {
        output.push_str(&format_outcomes(state));
    } else {
        output.push_str(&format_actions(state));
    }

    output.push_str("└─────────────────────────────────────────┘\n");

    output
}

/// Formats the dealer's hand, hiding the hole card if needed.
fn format_dealer_hand(state: &GameState) -> String {
    if state.dealer_hand.is_empty() {
        return "".to_string();
    }

    let cards = state.dealer_hand.cards();

    if state.is_finished() || matches!(state.phase, GamePhase::DealerTurn) {
        // Show all cards
        format_cards(cards)
    } else {
        // Hide hole card
        let mut result = format_card_box("??");
        result.push(' ');
        if cards.len() > 1 {
            result.push_str(&format_card_box(&cards[1].to_string()));
        }
        result
    }
}

/// Formats a hand's cards.
fn format_hand_cards(hand: &Hand) -> String {
    format_cards(hand.cards())
}

/// Formats a slice of cards.
fn format_cards(cards: &[Card]) -> String {
    cards
        .iter()
        .map(|c| format_card_box(&c.to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Formats a single card in a box.
fn format_card_box(card_str: &str) -> String {
    format!("[{}]", card_str)
}

/// Formats the status line for a hand.
fn format_hand_status(state: &GameState, hand_index: usize, hand: &Hand) -> String {
    if hand.is_blackjack() {
        return "BLACKJACK!".to_string();
    }

    if hand.is_bust() {
        return "BUST!".to_string();
    }

    if hand.is_surrendered() {
        return "SURRENDERED".to_string();
    }

    let value_str = format!("Value: {}", hand.value());

    if hand.is_soft() {
        format!("{} (soft)", value_str)
    } else if state.current_hand_index() == Some(hand_index) {
        format!("{} ◄", value_str)
    } else {
        value_str
    }
}

/// Formats the available actions.
fn format_actions(state: &GameState) -> String {
    let actions = state.available_actions();
    if actions.is_empty() {
        return "│                                         │\n".to_string();
    }

    let action_strs: Vec<&str> = actions
        .iter()
        .filter_map(|a| match a {
            crate::game::Action::Hit => Some("[H]it"),
            crate::game::Action::Stand => Some("[S]tand"),
            crate::game::Action::Double => Some("[D]ouble"),
            crate::game::Action::Split => Some("[P]split"),
            crate::game::Action::Surrender => Some("[U]surrender"),
            crate::game::Action::Insurance(true) => Some("[Y]es ins"),
            crate::game::Action::Insurance(false) => Some("[N]o ins"),
            crate::game::Action::Deal => Some("[Enter] Deal"),
        })
        .collect();

    format!("│  {:<39}│\n", action_strs.join(" "))
}

/// Formats the game outcomes.
fn format_outcomes(state: &GameState) -> String {
    let mut lines = String::new();

    for outcome in &state.outcomes {
        let hand_label = if state.player_hands.len() > 1 {
            format!("Hand {}: ", outcome.hand_index + 1)
        } else {
            "".to_string()
        };

        let outcome_str = match outcome.outcome {
            Outcome::Win => "WIN!",
            Outcome::Lose => "LOSE",
            Outcome::Push => "PUSH",
            Outcome::Blackjack => "BLACKJACK!",
            Outcome::Bust => "BUST",
            Outcome::Surrender => "SURRENDER",
        };

        let payout_str = if outcome.payout > 0.0 {
            format!("+{:.1}x", outcome.payout)
        } else if outcome.payout < 0.0 {
            format!("{:.1}x", outcome.payout)
        } else {
            "0x".to_string()
        };

        lines.push_str(&format!(
            "│  {}{:<20} {:<13}│\n",
            hand_label, outcome_str, payout_str
        ));
    }

    if lines.is_empty() {
        "│                                         │\n".to_string()
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn test_output_format_parse() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "pretty".parse::<OutputFormat>().unwrap(),
            OutputFormat::Pretty
        );
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_json_output() {
        let state = GameState::new(GameConfig::default());
        let output = format_game_state(&state, OutputFormat::Json);
        assert!(output.contains("\"phase\""));
        assert!(output.contains("Betting"));
    }

    #[test]
    fn test_pretty_output() {
        let state = GameState::new(GameConfig::default());
        let output = format_game_state(&state, OutputFormat::Pretty);
        assert!(output.contains("BLACKJACK"));
        assert!(output.contains("Dealer"));
    }
}
