//! Hand management for blackjack.
//!
//! Handles card collections, value calculations with ace logic, and hand state tracking.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::card::{Card, Rank};

/// A player or dealer hand in blackjack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    cards: Vec<Card>,
    is_split: bool,
    is_doubled: bool,
    is_surrendered: bool,
    is_standing: bool,
}

impl Hand {
    /// Creates a new empty hand.
    pub fn new() -> Self {
        Hand {
            cards: Vec::new(),
            is_split: false,
            is_doubled: false,
            is_surrendered: false,
            is_standing: false,
        }
    }

    /// Creates a hand marked as coming from a split.
    pub fn from_split(card: Card) -> Self {
        Hand {
            cards: vec![card],
            is_split: true,
            is_doubled: false,
            is_surrendered: false,
            is_standing: false,
        }
    }

    /// Adds a card to the hand.
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Calculates the best value of the hand.
    /// Aces are counted as 11 unless that would bust, then as 1.
    pub fn value(&self) -> u8 {
        let mut total: u8 = self.cards.iter().map(|c| c.value()).sum();
        let mut aces = self.cards.iter().filter(|c| c.rank == Rank::Ace).count();

        // Reduce aces from 11 to 1 as needed to avoid busting
        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        total
    }

    /// Returns true if the hand is "soft" (has an ace counted as 11).
    pub fn is_soft(&self) -> bool {
        let hard_total: u8 = self
            .cards
            .iter()
            .map(|c| if c.rank == Rank::Ace { 1 } else { c.value() })
            .sum();
        let aces = self.cards.iter().filter(|c| c.rank == Rank::Ace).count();

        // If adding 10 to hard total (for one ace as 11) doesn't bust, it's soft
        aces > 0 && hard_total + 10 <= 21
    }

    /// Returns true if the hand has busted (value > 21).
    pub fn is_bust(&self) -> bool {
        self.value() > 21
    }

    /// Returns true if the hand is a natural blackjack (two cards totaling 21, not from split).
    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value() == 21 && !self.is_split
    }

    /// Returns true if the hand is a pair (two cards of same rank).
    pub fn is_pair(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].rank == self.cards[1].rank
    }

    /// Returns the cards in the hand.
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Returns the number of cards in the hand.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns true if the hand has no cards.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Returns whether this hand came from a split.
    pub fn is_split(&self) -> bool {
        self.is_split
    }

    /// Returns whether this hand has been doubled down.
    pub fn is_doubled(&self) -> bool {
        self.is_doubled
    }

    /// Marks the hand as doubled down.
    pub fn set_doubled(&mut self) {
        self.is_doubled = true;
    }

    /// Returns whether this hand has been surrendered.
    pub fn is_surrendered(&self) -> bool {
        self.is_surrendered
    }

    /// Marks the hand as surrendered.
    pub fn set_surrendered(&mut self) {
        self.is_surrendered = true;
    }

    /// Returns whether the player is standing on this hand.
    pub fn is_standing(&self) -> bool {
        self.is_standing
    }

    /// Marks the hand as standing.
    pub fn set_standing(&mut self) {
        self.is_standing = true;
    }

    /// Returns true if the player can take more actions on this hand.
    pub fn is_active(&self) -> bool {
        !self.is_standing && !self.is_bust() && !self.is_surrendered
    }

    /// Takes the first card from the hand (for splitting).
    pub fn take_first_card(&mut self) -> Option<Card> {
        if self.cards.len() >= 2 {
            Some(self.cards.remove(0))
        } else {
            None
        }
    }
}

impl Default for Hand {
    fn default() -> Self {
        Hand::new()
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cards: Vec<String> = self.cards.iter().map(|c| c.to_string()).collect();
        write!(f, "[{}]", cards.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;

    fn card(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn test_empty_hand() {
        let hand = Hand::new();
        assert!(hand.is_empty());
        assert_eq!(hand.value(), 0);
    }

    #[test]
    fn test_simple_value() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Five, Suit::Spades));
        hand.add_card(card(Rank::Seven, Suit::Hearts));
        assert_eq!(hand.value(), 12);
    }

    #[test]
    fn test_ace_as_eleven() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::Six, Suit::Hearts));
        assert_eq!(hand.value(), 17);
        assert!(hand.is_soft());
    }

    #[test]
    fn test_ace_as_one() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::Six, Suit::Hearts));
        hand.add_card(card(Rank::Eight, Suit::Diamonds));
        assert_eq!(hand.value(), 15); // Ace counts as 1
        assert!(!hand.is_soft());
    }

    #[test]
    fn test_multiple_aces() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::Ace, Suit::Hearts));
        assert_eq!(hand.value(), 12); // One 11, one 1
        assert!(hand.is_soft());
    }

    #[test]
    fn test_blackjack() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::King, Suit::Hearts));
        assert!(hand.is_blackjack());
        assert_eq!(hand.value(), 21);
    }

    #[test]
    fn test_not_blackjack_after_split() {
        let mut hand = Hand::from_split(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::King, Suit::Hearts));
        assert!(!hand.is_blackjack()); // 21 from split is not blackjack
        assert_eq!(hand.value(), 21);
    }

    #[test]
    fn test_not_blackjack_three_cards() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Seven, Suit::Spades));
        hand.add_card(card(Rank::Seven, Suit::Hearts));
        hand.add_card(card(Rank::Seven, Suit::Diamonds));
        assert!(!hand.is_blackjack());
        assert_eq!(hand.value(), 21);
    }

    #[test]
    fn test_pair() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Eight, Suit::Spades));
        hand.add_card(card(Rank::Eight, Suit::Hearts));
        assert!(hand.is_pair());
    }

    #[test]
    fn test_not_pair_different_ranks() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Eight, Suit::Spades));
        hand.add_card(card(Rank::Nine, Suit::Hearts));
        assert!(!hand.is_pair());
    }

    #[test]
    fn test_bust() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ten, Suit::Spades));
        hand.add_card(card(Rank::Seven, Suit::Hearts));
        hand.add_card(card(Rank::Eight, Suit::Diamonds));
        assert!(hand.is_bust());
        assert_eq!(hand.value(), 25);
    }

    #[test]
    fn test_display() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Ace, Suit::Spades));
        hand.add_card(card(Rank::King, Suit::Hearts));
        assert_eq!(format!("{}", hand), "[A♠ K♥]");
    }

    #[test]
    fn test_five_card_charlie_value() {
        let mut hand = Hand::new();
        hand.add_card(card(Rank::Two, Suit::Spades));
        hand.add_card(card(Rank::Three, Suit::Hearts));
        hand.add_card(card(Rank::Four, Suit::Diamonds));
        hand.add_card(card(Rank::Five, Suit::Clubs));
        hand.add_card(card(Rank::Two, Suit::Hearts));
        assert_eq!(hand.len(), 5);
        assert_eq!(hand.value(), 16);
        assert!(!hand.is_bust());
    }
}
