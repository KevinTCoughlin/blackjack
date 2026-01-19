//! Blackjack - A Unix-philosophy blackjack game.
//!
//! This library provides the core game logic for blackjack, designed to be
//! stateless and composable. The game state is fully serializable, allowing
//! it to be piped between commands or stored/restored from files.
//!
//! # Example
//!
//! ```rust
//! use blackjack::{GameState, GameConfig, Action};
//!
//! // Create a new game with default (Vegas strip) rules
//! let mut game = GameState::new(GameConfig::default());
//!
//! // Deal initial cards
//! game.apply(Action::Deal).unwrap();
//!
//! // Play through the game
//! while !game.is_finished() {
//!     let actions = game.available_actions();
//!     if actions.is_empty() {
//!         break;
//!     }
//!     // Take an action (e.g., hit, stand, etc.)
//!     game.apply(actions[0].clone()).unwrap();
//! }
//!
//! // Check outcomes
//! for outcome in &game.outcomes {
//!     println!("Hand {}: {:?}", outcome.hand_index, outcome.outcome);
//! }
//! ```

pub mod card;
pub mod config;
pub mod deck;
pub mod game;
pub mod hand;
pub mod output;
pub mod strategy;

// Re-export commonly used types
pub use card::{Card, Rank, Suit};
pub use config::{ConfigError, DoubleRule, GameConfig, SurrenderType};
pub use deck::Deck;
pub use game::{Action, GameError, GamePhase, GameState, HandOutcome, Outcome};
pub use hand::Hand;
pub use output::{format_game_state, OutputFormat};
pub use strategy::{decide_action, Strategy};
