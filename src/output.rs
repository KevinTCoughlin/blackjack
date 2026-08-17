//! Output formatting for blackjack.
//!
//! Supports JSON for machine-readable output and pretty text with card art.

use colored::Colorize;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::card::{Card, Suit};
use crate::game::{GamePhase, GameState, Outcome};
use crate::hand::Hand;

/// Global flag to control color output
static USE_COLOR: AtomicBool = AtomicBool::new(true);

/// Enable or disable colored output
pub fn set_color_enabled(enabled: bool) {
    USE_COLOR.store(enabled, Ordering::SeqCst);
}

/// Check if color is enabled
pub fn is_color_enabled() -> bool {
    USE_COLOR.load(Ordering::SeqCst)
}

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
    output.push_str(&format!(
        "│  Dealer: {:<31}│\n",
        strip_ansi_for_padding(&dealer_str, 31)
    ));

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
        let padded = strip_ansi_for_padding(&cards_str, 28);
        output.push_str(&format!("│  {}{}│\n", hand_label, padded));

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

/// Pads a string containing ANSI codes to a visual width
fn strip_ansi_for_padding(s: &str, width: usize) -> String {
    // Count visible characters (excluding ANSI escape sequences)
    let visible_len = strip_ansi_codes(s).chars().count();
    let padding = width.saturating_sub(visible_len);
    format!("{}{}", s, " ".repeat(padding))
}

/// Strips ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Formats a card with colored suit
fn format_card_colored(card: &Card) -> String {
    let rank_str = card.rank.symbol();
    let suit_str = card.suit.symbol();

    if is_color_enabled() {
        match card.suit {
            Suit::Hearts | Suit::Diamonds => {
                format!("{}{}", rank_str, suit_str.red())
            }
            Suit::Spades | Suit::Clubs => {
                format!("{}{}", rank_str, suit_str)
            }
        }
    } else {
        format!("{}{}", rank_str, suit_str)
    }
}

/// Formats the dealer's hand, hiding the hole card if needed.
fn format_dealer_hand(state: &GameState) -> String {
    if state.dealer_hand.is_empty() {
        return "".to_string();
    }

    let cards = state.dealer_hand.cards();

    if state.is_finished() || matches!(state.phase, GamePhase::DealerTurn) {
        // Show all cards
        format_cards_colored(cards)
    } else {
        // Hide hole card
        let mut result = format_card_box("??");
        result.push(' ');
        if cards.len() > 1 {
            result.push_str(&format_card_box(&format_card_colored(&cards[1])));
        }
        result
    }
}

/// Formats a hand's cards.
fn format_hand_cards(hand: &Hand) -> String {
    format_cards_colored(hand.cards())
}

/// Formats a slice of cards with colors.
fn format_cards_colored(cards: &[Card]) -> String {
    cards
        .iter()
        .map(|c| format_card_box(&format_card_colored(c)))
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
        if is_color_enabled() {
            return "BLACKJACK!".green().bold().to_string();
        }
        return "BLACKJACK!".to_string();
    }

    if hand.is_bust() {
        if is_color_enabled() {
            return "BUST!".red().bold().to_string();
        }
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
        .map(|a| match a {
            crate::game::Action::Hit => "[H]it",
            crate::game::Action::Stand => "[S]tand",
            crate::game::Action::Double => "[D]ouble",
            crate::game::Action::Split => "[P]split",
            crate::game::Action::Surrender => "[U]surrender",
            crate::game::Action::Insurance(true) => "[Y]es ins",
            crate::game::Action::Insurance(false) => "[N]o ins",
            crate::game::Action::Deal => "[Enter] Deal",
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

        let outcome_str = if is_color_enabled() {
            match outcome.outcome {
                Outcome::Win => "WIN!".green().bold().to_string(),
                Outcome::Lose => "LOSE".red().to_string(),
                Outcome::Push => "PUSH".yellow().to_string(),
                Outcome::Blackjack => "BLACKJACK!".green().bold().to_string(),
                Outcome::Bust => "BUST".red().to_string(),
                Outcome::Surrender => "SURRENDER".yellow().to_string(),
            }
        } else {
            match outcome.outcome {
                Outcome::Win => "WIN!",
                Outcome::Lose => "LOSE",
                Outcome::Push => "PUSH",
                Outcome::Blackjack => "BLACKJACK!",
                Outcome::Bust => "BUST",
                Outcome::Surrender => "SURRENDER",
            }
            .to_string()
        };

        let payout_str = if outcome.payout > 0.0 {
            if is_color_enabled() {
                format!("+{:.1}x", outcome.payout).green().to_string()
            } else {
                format!("+{:.1}x", outcome.payout)
            }
        } else if outcome.payout < 0.0 {
            if is_color_enabled() {
                format!("{:.1}x", outcome.payout).red().to_string()
            } else {
                format!("{:.1}x", outcome.payout)
            }
        } else {
            "0x".to_string()
        };

        // Calculate visible widths for padding
        let outcome_visible = strip_ansi_codes(&outcome_str);
        let payout_visible = strip_ansi_codes(&payout_str);
        let hand_label_len = hand_label.len();

        let total_content = hand_label_len + outcome_visible.len() + payout_visible.len();
        let spacing = if 37 > total_content {
            37 - total_content
        } else {
            1
        };

        lines.push_str(&format!(
            "│  {}{}{}{}│\n",
            hand_label,
            outcome_str,
            " ".repeat(spacing),
            payout_str
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
        set_color_enabled(false);
        let state = GameState::new(GameConfig::default());
        let output = format_game_state(&state, OutputFormat::Pretty);
        assert!(output.contains("BLACKJACK"));
        assert!(output.contains("Dealer"));
    }

    #[test]
    fn test_color_toggle() {
        set_color_enabled(true);
        assert!(is_color_enabled());
        set_color_enabled(false);
        assert!(!is_color_enabled());
    }
}
