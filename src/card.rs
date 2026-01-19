//! Card types for blackjack game.
//!
//! This module defines the core card types: `Suit`, `Rank`, and `Card`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The four suits in a standard deck of cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    /// Returns the Unicode symbol for this suit.
    pub fn symbol(&self) -> &'static str {
        match self {
            Suit::Spades => "♠",
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
        }
    }

    /// Returns all suits in standard order.
    pub fn all() -> [Suit; 4] {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Card ranks from Ace through King.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    /// Returns the blackjack value of this rank.
    /// Ace returns 11 (soft value), caller handles ace reduction.
    pub fn value(&self) -> u8 {
        match self {
            Rank::Ace => 11,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
        }
    }

    /// Returns the short symbol for this rank.
    pub fn symbol(&self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        }
    }

    /// Returns all ranks in standard order.
    pub fn all() -> [Rank; 13] {
        [
            Rank::Ace,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
        ]
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// A playing card with a rank and suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// Creates a new card.
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card { rank, suit }
    }

    /// Returns the blackjack value of this card.
    pub fn value(&self) -> u8 {
        self.rank.value()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_values() {
        assert_eq!(Card::new(Rank::Ace, Suit::Spades).value(), 11);
        assert_eq!(Card::new(Rank::Two, Suit::Hearts).value(), 2);
        assert_eq!(Card::new(Rank::Ten, Suit::Diamonds).value(), 10);
        assert_eq!(Card::new(Rank::Jack, Suit::Clubs).value(), 10);
        assert_eq!(Card::new(Rank::Queen, Suit::Spades).value(), 10);
        assert_eq!(Card::new(Rank::King, Suit::Hearts).value(), 10);
    }

    #[test]
    fn test_card_display() {
        let card = Card::new(Rank::Ace, Suit::Spades);
        assert_eq!(format!("{}", card), "A♠");

        let card = Card::new(Rank::King, Suit::Hearts);
        assert_eq!(format!("{}", card), "K♥");

        let card = Card::new(Rank::Ten, Suit::Diamonds);
        assert_eq!(format!("{}", card), "10♦");
    }

    #[test]
    fn test_suit_symbols() {
        assert_eq!(Suit::Spades.symbol(), "♠");
        assert_eq!(Suit::Hearts.symbol(), "♥");
        assert_eq!(Suit::Diamonds.symbol(), "♦");
        assert_eq!(Suit::Clubs.symbol(), "♣");
    }

    #[test]
    fn test_all_suits() {
        let suits = Suit::all();
        assert_eq!(suits.len(), 4);
    }

    #[test]
    fn test_all_ranks() {
        let ranks = Rank::all();
        assert_eq!(ranks.len(), 13);
    }
}
