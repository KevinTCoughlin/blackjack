//! Game configuration for blackjack.
//!
//! Supports loading from TOML files with sensible defaults (Vegas strip rules).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when loading configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

/// Rules for when doubling down is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DoubleRule {
    /// Can double on any two cards
    #[default]
    Any,
    /// Can only double on hard 9, 10, or 11
    Hard9To11,
    /// Can only double on hard 10 or 11
    Hard10Or11,
    /// Can only double on hard 11
    Hard11Only,
}

impl DoubleRule {
    /// Checks if doubling is allowed for a given hand value and soft status.
    pub fn allows(&self, value: u8, is_soft: bool) -> bool {
        match self {
            DoubleRule::Any => true,
            DoubleRule::Hard9To11 => !is_soft && (9..=11).contains(&value),
            DoubleRule::Hard10Or11 => !is_soft && (10..=11).contains(&value),
            DoubleRule::Hard11Only => !is_soft && value == 11,
        }
    }
}

/// When surrender is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SurrenderType {
    /// No surrender allowed
    #[default]
    None,
    /// Late surrender (after dealer checks for blackjack)
    Late,
    /// Early surrender (before dealer checks for blackjack)
    Early,
}

/// Game configuration with all blackjack rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    // Deck configuration
    /// Number of decks in the shoe (1-8)
    pub num_decks: u8,
    /// Percentage of deck remaining that triggers reshuffle (0.1-0.5)
    pub reshuffle_threshold: f32,

    // Dealer rules
    /// Dealer stands on soft 17 (true) or hits soft 17 (false)
    pub dealer_stands_soft_17: bool,
    /// Dealer peeks for blackjack when showing ace or ten
    pub dealer_peeks: bool,

    // Player options
    /// Allow doubling down
    pub allow_double: bool,
    /// Rules for when doubling is allowed
    pub double_on: DoubleRule,
    /// Allow doubling after splitting
    pub allow_double_after_split: bool,
    /// Allow splitting pairs
    pub allow_split: bool,
    /// Maximum number of times player can split (2-4 total hands)
    pub max_splits: u8,
    /// Allow re-splitting aces
    pub allow_resplit_aces: bool,
    /// Allow hitting split aces
    pub allow_hit_split_aces: bool,
    /// Allow surrender
    pub allow_surrender: bool,
    /// Type of surrender allowed
    pub surrender_type: SurrenderType,
    /// Allow insurance bet
    pub allow_insurance: bool,

    // Payouts
    /// Blackjack payout multiplier (1.5 for 3:2, 1.2 for 6:5)
    pub blackjack_pays: f32,
    /// Insurance payout multiplier (typically 2.0 for 2:1)
    pub insurance_pays: f32,

    // Special rules
    /// Five card charlie wins automatically
    pub five_card_charlie: bool,
}

impl Default for GameConfig {
    /// Returns Vegas strip rules as the default configuration.
    fn default() -> Self {
        GameConfig {
            // 6-deck shoe is standard
            num_decks: 6,
            reshuffle_threshold: 0.25,

            // Standard dealer rules
            dealer_stands_soft_17: true,
            dealer_peeks: true,

            // Standard player options
            allow_double: true,
            double_on: DoubleRule::Any,
            allow_double_after_split: true,
            allow_split: true,
            max_splits: 4,
            allow_resplit_aces: false,
            allow_hit_split_aces: false,
            allow_surrender: true,
            surrender_type: SurrenderType::Late,
            allow_insurance: true,

            // Standard payouts
            blackjack_pays: 1.5,
            insurance_pays: 2.0,

            // No five card charlie by default
            five_card_charlie: false,
        }
    }
}

impl GameConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a TOML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: GameConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, ConfigError> {
        let config: GameConfig = toml::from_str(toml_str)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration values.
    fn validate(&self) -> Result<(), ConfigError> {
        if !(1..=8).contains(&self.num_decks) {
            return Err(ConfigError::ValidationError(
                "num_decks must be between 1 and 8".to_string(),
            ));
        }

        if !(0.1..=0.5).contains(&self.reshuffle_threshold) {
            return Err(ConfigError::ValidationError(
                "reshuffle_threshold must be between 0.1 and 0.5".to_string(),
            ));
        }

        if !(2..=4).contains(&self.max_splits) {
            return Err(ConfigError::ValidationError(
                "max_splits must be between 2 and 4".to_string(),
            ));
        }

        if self.blackjack_pays <= 0.0 {
            return Err(ConfigError::ValidationError(
                "blackjack_pays must be positive".to_string(),
            ));
        }

        if self.insurance_pays <= 0.0 {
            return Err(ConfigError::ValidationError(
                "insurance_pays must be positive".to_string(),
            ));
        }

        Ok(())
    }

    /// Serializes the configuration to TOML.
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.num_decks, 6);
        assert!(config.dealer_stands_soft_17);
        assert_eq!(config.blackjack_pays, 1.5);
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
            num_decks = 1
            dealer_stands_soft_17 = false
            blackjack_pays = 1.2
        "#;
        let config = GameConfig::from_toml(toml).unwrap();
        assert_eq!(config.num_decks, 1);
        assert!(!config.dealer_stands_soft_17);
        assert_eq!(config.blackjack_pays, 1.2);
    }

    #[test]
    fn test_invalid_num_decks() {
        let toml = "num_decks = 10";
        let result = GameConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_double_rule_any() {
        let rule = DoubleRule::Any;
        assert!(rule.allows(9, false));
        assert!(rule.allows(12, true));
        assert!(rule.allows(5, false));
    }

    #[test]
    fn test_double_rule_hard_9_to_11() {
        let rule = DoubleRule::Hard9To11;
        assert!(rule.allows(9, false));
        assert!(rule.allows(10, false));
        assert!(rule.allows(11, false));
        assert!(!rule.allows(8, false));
        assert!(!rule.allows(12, false));
        assert!(!rule.allows(11, true)); // soft 11
    }

    #[test]
    fn test_double_rule_hard_10_or_11() {
        let rule = DoubleRule::Hard10Or11;
        assert!(!rule.allows(9, false));
        assert!(rule.allows(10, false));
        assert!(rule.allows(11, false));
    }

    #[test]
    fn test_double_rule_hard_11_only() {
        let rule = DoubleRule::Hard11Only;
        assert!(!rule.allows(10, false));
        assert!(rule.allows(11, false));
        assert!(!rule.allows(11, true));
    }

    #[test]
    fn test_config_roundtrip() {
        let config = GameConfig::default();
        let toml = config.to_toml();
        let parsed = GameConfig::from_toml(&toml).unwrap();
        assert_eq!(config.num_decks, parsed.num_decks);
        assert_eq!(config.blackjack_pays, parsed.blackjack_pays);
    }
}
