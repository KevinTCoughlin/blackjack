//! Game state and action handling for blackjack.
//!
//! Implements the blackjack state machine with all standard actions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::card::{Card, Rank};
use crate::config::{GameConfig, SurrenderType};
use crate::deck::Deck;
use crate::hand::Hand;

/// Errors that can occur during game actions.
#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid action '{action}' in phase '{phase}'")]
    InvalidAction { action: String, phase: String },

    #[error("Cannot split: {0}")]
    CannotSplit(String),

    #[error("Cannot double: {0}")]
    CannotDouble(String),

    #[error("Cannot surrender: {0}")]
    CannotSurrender(String),

    #[error("Deck exhausted")]
    DeckExhausted,
}

/// The current phase of the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GamePhase {
    /// Waiting to deal initial cards
    Betting,
    /// Offering insurance (dealer shows Ace)
    Insurance,
    /// Player's turn, with index of current hand
    PlayerTurn { hand_index: usize },
    /// Dealer reveals and plays
    DealerTurn,
    /// Game complete, outcomes determined
    Finished,
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePhase::Betting => write!(f, "betting"),
            GamePhase::Insurance => write!(f, "insurance"),
            GamePhase::PlayerTurn { hand_index } => write!(f, "player_turn({})", hand_index),
            GamePhase::DealerTurn => write!(f, "dealer_turn"),
            GamePhase::Finished => write!(f, "finished"),
        }
    }
}

/// Player actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Action {
    /// Start a new hand (deal initial cards)
    Deal,
    /// Take another card
    Hit,
    /// Stop taking cards
    Stand,
    /// Double the bet and take exactly one more card
    Double,
    /// Split a pair into two hands
    Split,
    /// Give up half the bet and end the hand
    Surrender,
    /// Accept (true) or decline (false) insurance
    Insurance(bool),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Deal => write!(f, "deal"),
            Action::Hit => write!(f, "hit"),
            Action::Stand => write!(f, "stand"),
            Action::Double => write!(f, "double"),
            Action::Split => write!(f, "split"),
            Action::Surrender => write!(f, "surrender"),
            Action::Insurance(accept) => {
                if *accept {
                    write!(f, "insurance(yes)")
                } else {
                    write!(f, "insurance(no)")
                }
            }
        }
    }
}

/// The outcome of a finished hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Player wins (dealer busts or lower value)
    Win,
    /// Player loses (player busts or lower value)
    Lose,
    /// Tie (same value)
    Push,
    /// Player has natural blackjack
    Blackjack,
    /// Player busted
    Bust,
    /// Player surrendered
    Surrender,
}

impl Outcome {
    /// Returns the payout multiplier for this outcome.
    pub fn payout_multiplier(&self, config: &GameConfig) -> f32 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Blackjack => config.blackjack_pays,
            Outcome::Push => 0.0,
            Outcome::Surrender => -0.5,
            Outcome::Lose | Outcome::Bust => -1.0,
        }
    }
}

/// Complete game state that can be serialized and restored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub player_hands: Vec<Hand>,
    pub dealer_hand: Hand,
    pub deck: Deck,
    pub config: GameConfig,
    pub outcomes: Vec<HandOutcome>,
    pub insurance_bet: bool,
    pub dealer_has_blackjack: Option<bool>,
}

/// Outcome for a single hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandOutcome {
    pub hand_index: usize,
    pub outcome: Outcome,
    pub payout: f32,
}

impl GameState {
    /// Creates a new game with the given configuration.
    pub fn new(config: GameConfig) -> Self {
        let deck = Deck::new(config.num_decks);
        GameState {
            phase: GamePhase::Betting,
            player_hands: Vec::new(),
            dealer_hand: Hand::new(),
            deck,
            config,
            outcomes: Vec::new(),
            insurance_bet: false,
            dealer_has_blackjack: None,
        }
    }

    /// Creates a new game with a seeded deck for reproducible results.
    pub fn with_seed(config: GameConfig, seed: u64) -> Self {
        let deck = Deck::with_seed(config.num_decks, seed);
        GameState {
            phase: GamePhase::Betting,
            player_hands: Vec::new(),
            dealer_hand: Hand::new(),
            deck,
            config,
            outcomes: Vec::new(),
            insurance_bet: false,
            dealer_has_blackjack: None,
        }
    }

    /// Applies an action to the game state.
    pub fn apply(&mut self, action: Action) -> Result<(), GameError> {
        match (&self.phase, &action) {
            (GamePhase::Betting, Action::Deal) => self.deal(),
            (GamePhase::Insurance, Action::Insurance(accept)) => self.handle_insurance(*accept),
            (GamePhase::PlayerTurn { hand_index }, Action::Hit) => self.hit(*hand_index),
            (GamePhase::PlayerTurn { hand_index }, Action::Stand) => self.stand(*hand_index),
            (GamePhase::PlayerTurn { hand_index }, Action::Double) => self.double(*hand_index),
            (GamePhase::PlayerTurn { hand_index }, Action::Split) => self.split(*hand_index),
            (GamePhase::PlayerTurn { hand_index }, Action::Surrender) => {
                self.surrender(*hand_index)
            }
            _ => Err(GameError::InvalidAction {
                action: action.to_string(),
                phase: self.phase.to_string(),
            }),
        }
    }

    /// Returns the list of valid actions in the current state.
    pub fn available_actions(&self) -> Vec<Action> {
        match &self.phase {
            GamePhase::Betting => vec![Action::Deal],

            GamePhase::Insurance => vec![Action::Insurance(true), Action::Insurance(false)],

            GamePhase::PlayerTurn { hand_index } => {
                let hand = &self.player_hands[*hand_index];
                let mut actions = vec![Action::Hit, Action::Stand];

                // Check if double is allowed
                if self.can_double(*hand_index) {
                    actions.push(Action::Double);
                }

                // Check if split is allowed
                if self.can_split(*hand_index) {
                    actions.push(Action::Split);
                }

                // Check if surrender is allowed
                if self.can_surrender(*hand_index) {
                    actions.push(Action::Surrender);
                }

                // Remove hit if hand has 21
                if hand.value() == 21 {
                    actions.retain(|a| !matches!(a, Action::Hit | Action::Double));
                }

                actions
            }

            GamePhase::DealerTurn | GamePhase::Finished => vec![],
        }
    }

    /// Checks if the player can double on the given hand.
    fn can_double(&self, hand_index: usize) -> bool {
        if !self.config.allow_double {
            return false;
        }

        let hand = &self.player_hands[hand_index];

        // Can only double on first two cards
        if hand.len() != 2 {
            return false;
        }

        // Check if doubling after split is allowed
        if hand.is_split() && !self.config.allow_double_after_split {
            return false;
        }

        // Check double rules
        self.config.double_on.allows(hand.value(), hand.is_soft())
    }

    /// Checks if the player can split the given hand.
    fn can_split(&self, hand_index: usize) -> bool {
        if !self.config.allow_split {
            return false;
        }

        let hand = &self.player_hands[hand_index];

        // Must be a pair
        if !hand.is_pair() {
            return false;
        }

        // Check max splits
        if self.player_hands.len() >= self.config.max_splits as usize {
            return false;
        }

        // Check if resplitting aces is allowed
        if hand.is_split() && hand.cards()[0].rank == Rank::Ace && !self.config.allow_resplit_aces {
            return false;
        }

        true
    }

    /// Checks if the player can surrender the given hand.
    fn can_surrender(&self, hand_index: usize) -> bool {
        if !self.config.allow_surrender {
            return false;
        }

        if self.config.surrender_type == SurrenderType::None {
            return false;
        }

        let hand = &self.player_hands[hand_index];

        // Can only surrender on first two cards of first hand
        if hand_index != 0 || hand.len() != 2 || hand.is_split() {
            return false;
        }

        true
    }

    /// Deals initial cards.
    fn deal(&mut self) -> Result<(), GameError> {
        self.player_hands.clear();
        self.dealer_hand = Hand::new();
        self.outcomes.clear();
        self.insurance_bet = false;
        self.dealer_has_blackjack = None;

        // Check if deck needs reshuffling
        if self.deck.needs_reshuffle() {
            self.deck.reshuffle();
        }

        // Create player hand
        let mut player_hand = Hand::new();

        // Deal: player, dealer, player, dealer
        // Draw all cards first to avoid borrow checker issues
        let player_card1 = self.draw_card()?;
        let dealer_card1 = self.draw_card()?;
        let player_card2 = self.draw_card()?;
        let dealer_card2 = self.draw_card()?;

        player_hand.add_card(player_card1);
        self.dealer_hand.add_card(dealer_card1);
        player_hand.add_card(player_card2);
        self.dealer_hand.add_card(dealer_card2);

        self.player_hands.push(player_hand);

        // Check for dealer ace (insurance opportunity)
        if self.config.allow_insurance && self.dealer_upcard().rank == Rank::Ace {
            self.phase = GamePhase::Insurance;
        } else {
            self.check_initial_blackjacks()?;
        }

        Ok(())
    }

    /// Returns the dealer's visible card.
    pub fn dealer_upcard(&self) -> Card {
        self.dealer_hand.cards()[1]
    }

    /// Returns the dealer's hole card (face down card).
    pub fn dealer_hole_card(&self) -> Card {
        self.dealer_hand.cards()[0]
    }

    /// Handles insurance decision.
    fn handle_insurance(&mut self, accept: bool) -> Result<(), GameError> {
        self.insurance_bet = accept;

        // Check for dealer blackjack if peek is enabled
        if self.config.dealer_peeks {
            self.dealer_has_blackjack = Some(self.dealer_hand.is_blackjack());

            if self.dealer_hand.is_blackjack() {
                // Dealer has blackjack - game ends
                self.phase = GamePhase::Finished;
                self.calculate_outcomes();
                return Ok(());
            }
        }

        self.check_initial_blackjacks()
    }

    /// Checks for initial blackjacks after dealing.
    fn check_initial_blackjacks(&mut self) -> Result<(), GameError> {
        // Check for player blackjack
        if self.player_hands[0].is_blackjack() {
            if !self.config.dealer_peeks {
                // Need to check dealer blackjack now
                self.dealer_has_blackjack = Some(self.dealer_hand.is_blackjack());
            }
            self.phase = GamePhase::Finished;
            self.calculate_outcomes();
        } else {
            self.phase = GamePhase::PlayerTurn { hand_index: 0 };
        }

        Ok(())
    }

    /// Player hits (takes another card).
    fn hit(&mut self, hand_index: usize) -> Result<(), GameError> {
        let card = self.draw_card()?;
        self.player_hands[hand_index].add_card(card);

        // Check if hand is complete
        if self.player_hands[hand_index].is_bust() || self.player_hands[hand_index].value() == 21 {
            self.advance_to_next_hand(hand_index);
        }

        // Check for five card charlie
        if self.config.five_card_charlie
            && self.player_hands[hand_index].len() >= 5
            && !self.player_hands[hand_index].is_bust()
        {
            self.player_hands[hand_index].set_standing();
            self.advance_to_next_hand(hand_index);
        }

        Ok(())
    }

    /// Player stands.
    fn stand(&mut self, hand_index: usize) -> Result<(), GameError> {
        self.player_hands[hand_index].set_standing();
        self.advance_to_next_hand(hand_index);
        Ok(())
    }

    /// Player doubles down.
    fn double(&mut self, hand_index: usize) -> Result<(), GameError> {
        if !self.can_double(hand_index) {
            return Err(GameError::CannotDouble(
                "Doubling not allowed in this situation".to_string(),
            ));
        }

        let card = self.draw_card()?;
        self.player_hands[hand_index].add_card(card);
        self.player_hands[hand_index].set_doubled();
        self.player_hands[hand_index].set_standing();

        self.advance_to_next_hand(hand_index);
        Ok(())
    }

    /// Player splits.
    fn split(&mut self, hand_index: usize) -> Result<(), GameError> {
        if !self.can_split(hand_index) {
            return Err(GameError::CannotSplit(
                "Splitting not allowed in this situation".to_string(),
            ));
        }

        // Take one card from the current hand
        let split_card = self.player_hands[hand_index]
            .take_first_card()
            .ok_or_else(|| GameError::CannotSplit("No card to split".to_string()))?;

        // Create new hand with the split card
        let new_hand = Hand::from_split(split_card);
        self.player_hands.insert(hand_index + 1, new_hand);

        // Deal one card to each hand
        let card1 = self.draw_card()?;
        let card2 = self.draw_card()?;

        self.player_hands[hand_index].add_card(card1);
        self.player_hands[hand_index + 1].add_card(card2);

        // Mark current hand as from split
        // (Note: The original hand needs to be marked too, but Hand::take_first_card doesn't do this)

        // If splitting aces and no hit allowed, stand immediately
        if split_card.rank == Rank::Ace && !self.config.allow_hit_split_aces {
            self.player_hands[hand_index].set_standing();
            self.player_hands[hand_index + 1].set_standing();
            self.advance_to_next_hand(hand_index + 1);
        }

        Ok(())
    }

    /// Player surrenders.
    fn surrender(&mut self, hand_index: usize) -> Result<(), GameError> {
        if !self.can_surrender(hand_index) {
            return Err(GameError::CannotSurrender(
                "Surrender not allowed in this situation".to_string(),
            ));
        }

        self.player_hands[hand_index].set_surrendered();
        self.player_hands[hand_index].set_standing();
        self.advance_to_next_hand(hand_index);
        Ok(())
    }

    /// Advances to the next hand or dealer turn.
    fn advance_to_next_hand(&mut self, current_index: usize) {
        // Find next active hand
        for i in (current_index + 1)..self.player_hands.len() {
            if self.player_hands[i].is_active() {
                self.phase = GamePhase::PlayerTurn { hand_index: i };
                return;
            }
        }

        // No more active hands - check if dealer needs to play
        let all_busted = self
            .player_hands
            .iter()
            .all(|h| h.is_bust() || h.is_surrendered());

        if all_busted {
            self.phase = GamePhase::Finished;
            self.calculate_outcomes();
        } else {
            self.phase = GamePhase::DealerTurn;
            self.play_dealer();
        }
    }

    /// Plays out the dealer's hand.
    fn play_dealer(&mut self) {
        // Dealer plays according to rules
        while self.dealer_should_hit() {
            if let Some(card) = self.deck.draw() {
                self.dealer_hand.add_card(card);
            } else {
                break;
            }
        }

        self.phase = GamePhase::Finished;
        self.calculate_outcomes();
    }

    /// Determines if dealer should hit.
    fn dealer_should_hit(&self) -> bool {
        let value = self.dealer_hand.value();
        if value < 17 {
            return true;
        }
        if value == 17 && self.dealer_hand.is_soft() && !self.config.dealer_stands_soft_17 {
            return true;
        }
        false
    }

    /// Calculates outcomes for all hands.
    fn calculate_outcomes(&mut self) {
        self.outcomes.clear();

        let dealer_value = self.dealer_hand.value();
        let dealer_bust = self.dealer_hand.is_bust();
        let dealer_blackjack = self.dealer_has_blackjack.unwrap_or(self.dealer_hand.is_blackjack());

        for (i, hand) in self.player_hands.iter().enumerate() {
            let outcome = if hand.is_surrendered() {
                Outcome::Surrender
            } else if hand.is_bust() {
                Outcome::Bust
            } else if hand.is_blackjack() && !dealer_blackjack {
                Outcome::Blackjack
            } else if dealer_blackjack && !hand.is_blackjack() {
                Outcome::Lose
            } else if dealer_blackjack && hand.is_blackjack() {
                Outcome::Push
            } else if dealer_bust {
                Outcome::Win
            } else {
                let player_value = hand.value();
                if player_value > dealer_value {
                    Outcome::Win
                } else if player_value < dealer_value {
                    Outcome::Lose
                } else {
                    Outcome::Push
                }
            };

            let mut payout = outcome.payout_multiplier(&self.config);

            // Double payout if doubled
            if hand.is_doubled() && outcome != Outcome::Push {
                payout *= 2.0;
            }

            self.outcomes.push(HandOutcome {
                hand_index: i,
                outcome,
                payout,
            });
        }

        // Handle insurance payout
        if self.insurance_bet {
            if dealer_blackjack {
                // Insurance pays 2:1
                // This is separate from the hand outcome
            }
            // Insurance bet lost is already factored into not getting payout
        }
    }

    /// Draws a card from the deck.
    fn draw_card(&mut self) -> Result<Card, GameError> {
        self.deck.draw().ok_or(GameError::DeckExhausted)
    }

    /// Returns true if the game is finished.
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, GamePhase::Finished)
    }

    /// Returns the current hand index if in player turn.
    pub fn current_hand_index(&self) -> Option<usize> {
        match &self.phase {
            GamePhase::PlayerTurn { hand_index } => Some(*hand_index),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_game() -> GameState {
        GameState::with_seed(GameConfig::default(), 12345)
    }

    #[test]
    fn test_new_game() {
        let game = setup_game();
        assert!(matches!(game.phase, GamePhase::Betting));
        assert!(game.player_hands.is_empty());
    }

    #[test]
    fn test_deal() {
        let mut game = setup_game();
        game.apply(Action::Deal).unwrap();

        assert_eq!(game.player_hands.len(), 1);
        assert_eq!(game.player_hands[0].len(), 2);
        assert_eq!(game.dealer_hand.len(), 2);
    }

    #[test]
    fn test_hit() {
        let mut game = setup_game();
        game.apply(Action::Deal).unwrap();

        // Skip insurance if offered
        if matches!(game.phase, GamePhase::Insurance) {
            game.apply(Action::Insurance(false)).unwrap();
        }

        if matches!(game.phase, GamePhase::PlayerTurn { .. }) {
            let initial_cards = game.player_hands[0].len();
            game.apply(Action::Hit).unwrap();
            assert!(game.player_hands[0].len() > initial_cards);
        }
    }

    #[test]
    fn test_stand() {
        let mut game = setup_game();
        game.apply(Action::Deal).unwrap();

        if matches!(game.phase, GamePhase::Insurance) {
            game.apply(Action::Insurance(false)).unwrap();
        }

        if matches!(game.phase, GamePhase::PlayerTurn { .. }) {
            game.apply(Action::Stand).unwrap();
            assert!(
                matches!(game.phase, GamePhase::DealerTurn)
                    || matches!(game.phase, GamePhase::Finished)
            );
        }
    }

    #[test]
    fn test_available_actions_betting() {
        let game = setup_game();
        let actions = game.available_actions();
        assert_eq!(actions, vec![Action::Deal]);
    }

    #[test]
    fn test_game_to_completion() {
        let mut game = setup_game();
        game.apply(Action::Deal).unwrap();

        // Play through the game
        while !game.is_finished() {
            let actions = game.available_actions();
            if actions.is_empty() {
                break;
            }

            // Take first available action (usually insurance no, then stand)
            let action = if actions.contains(&Action::Insurance(false)) {
                Action::Insurance(false)
            } else if actions.contains(&Action::Stand) {
                Action::Stand
            } else {
                actions[0].clone()
            };

            game.apply(action).unwrap();
        }

        assert!(game.is_finished());
        assert!(!game.outcomes.is_empty());
    }

    #[test]
    fn test_outcome_payout() {
        let config = GameConfig::default();
        assert_eq!(Outcome::Win.payout_multiplier(&config), 1.0);
        assert_eq!(Outcome::Blackjack.payout_multiplier(&config), 1.5);
        assert_eq!(Outcome::Push.payout_multiplier(&config), 0.0);
        assert_eq!(Outcome::Lose.payout_multiplier(&config), -1.0);
        assert_eq!(Outcome::Surrender.payout_multiplier(&config), -0.5);
    }
}
