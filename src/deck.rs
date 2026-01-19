//! Deck management for blackjack.
//!
//! Supports multi-deck shoes with configurable reshuffle thresholds.

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::card::{Card, Rank, Suit};

/// A deck (or shoe) of cards for blackjack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
    dealt: usize,
    num_decks: u8,
    reshuffle_threshold: f32,
    #[serde(skip)]
    rng_seed: Option<u64>,
}

impl Deck {
    /// Creates a new shuffled deck with the specified number of standard 52-card decks.
    pub fn new(num_decks: u8) -> Self {
        let mut deck = Deck {
            cards: Vec::with_capacity(52 * num_decks as usize),
            dealt: 0,
            num_decks,
            reshuffle_threshold: 0.25,
            rng_seed: None,
        };
        deck.reset_and_shuffle();
        deck
    }

    /// Creates a new deck with a specific seed for reproducible shuffling.
    pub fn with_seed(num_decks: u8, seed: u64) -> Self {
        let mut deck = Deck {
            cards: Vec::with_capacity(52 * num_decks as usize),
            dealt: 0,
            num_decks,
            reshuffle_threshold: 0.25,
            rng_seed: Some(seed),
        };
        deck.reset_and_shuffle();
        deck
    }

    /// Sets the reshuffle threshold (percentage of deck that triggers reshuffle).
    pub fn with_reshuffle_threshold(mut self, threshold: f32) -> Self {
        self.reshuffle_threshold = threshold.clamp(0.1, 0.5);
        self
    }

    /// Resets and shuffles the deck.
    fn reset_and_shuffle(&mut self) {
        self.cards.clear();
        for _ in 0..self.num_decks {
            for suit in Suit::all() {
                for rank in Rank::all() {
                    self.cards.push(Card::new(rank, suit));
                }
            }
        }
        self.dealt = 0;
        self.shuffle();
    }

    /// Shuffles the remaining undealt cards using Fisher-Yates algorithm.
    pub fn shuffle(&mut self) {
        let remaining = &mut self.cards[self.dealt..];
        if let Some(seed) = self.rng_seed {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            remaining.shuffle(&mut rng);
        } else {
            let mut rng = rand::thread_rng();
            remaining.shuffle(&mut rng);
        }
    }

    /// Draws a card from the deck.
    /// Returns None if the deck is exhausted.
    pub fn draw(&mut self) -> Option<Card> {
        if self.dealt < self.cards.len() {
            let card = self.cards[self.dealt];
            self.dealt += 1;
            Some(card)
        } else {
            None
        }
    }

    /// Checks if the deck needs reshuffling based on the threshold.
    pub fn needs_reshuffle(&self) -> bool {
        let remaining = self.cards.len() - self.dealt;
        let total = self.cards.len();
        (remaining as f32 / total as f32) <= self.reshuffle_threshold
    }

    /// Reshuffles all cards back into the deck.
    pub fn reshuffle(&mut self) {
        self.reset_and_shuffle();
    }

    /// Returns the number of cards remaining in the deck.
    pub fn remaining(&self) -> usize {
        self.cards.len() - self.dealt
    }

    /// Returns the total number of cards in the deck.
    pub fn total(&self) -> usize {
        self.cards.len()
    }

    /// Returns the number of decks in this shoe.
    pub fn num_decks(&self) -> u8 {
        self.num_decks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_creation() {
        let deck = Deck::new(1);
        assert_eq!(deck.total(), 52);
        assert_eq!(deck.remaining(), 52);
    }

    #[test]
    fn test_multi_deck() {
        let deck = Deck::new(6);
        assert_eq!(deck.total(), 312);
        assert_eq!(deck.remaining(), 312);
    }

    #[test]
    fn test_draw_card() {
        let mut deck = Deck::new(1);
        let card = deck.draw();
        assert!(card.is_some());
        assert_eq!(deck.remaining(), 51);
    }

    #[test]
    fn test_draw_all_cards() {
        let mut deck = Deck::new(1);
        for _ in 0..52 {
            assert!(deck.draw().is_some());
        }
        assert!(deck.draw().is_none());
        assert_eq!(deck.remaining(), 0);
    }

    #[test]
    fn test_needs_reshuffle() {
        let mut deck = Deck::new(1).with_reshuffle_threshold(0.25);
        // Draw 39 cards (75% of deck), leaving 13 (25%)
        for _ in 0..39 {
            deck.draw();
        }
        assert!(deck.needs_reshuffle());
    }

    #[test]
    fn test_reshuffle() {
        let mut deck = Deck::new(1);
        for _ in 0..26 {
            deck.draw();
        }
        assert_eq!(deck.remaining(), 26);
        deck.reshuffle();
        assert_eq!(deck.remaining(), 52);
    }

    #[test]
    fn test_seeded_deck_reproducibility() {
        let mut deck1 = Deck::with_seed(1, 12345);
        let mut deck2 = Deck::with_seed(1, 12345);

        for _ in 0..10 {
            assert_eq!(deck1.draw(), deck2.draw());
        }
    }

    #[test]
    fn test_seeded_deck_different_seeds() {
        let mut deck1 = Deck::with_seed(1, 12345);
        let mut deck2 = Deck::with_seed(1, 54321);

        // Different seeds should produce different orders (with high probability)
        let cards1: Vec<_> = (0..10).filter_map(|_| deck1.draw()).collect();
        let cards2: Vec<_> = (0..10).filter_map(|_| deck2.draw()).collect();
        assert_ne!(cards1, cards2);
    }
}
