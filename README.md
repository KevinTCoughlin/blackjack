# Blackjack

A Unix-philosophy blackjack game with a stateless CLI.

[![Crates.io](https://img.shields.io/crates/v/blackjack.svg)](https://crates.io/crates/blackjack)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/Q5Q11SKIOF)

## Features

- **Stateless design** - Game state is fully serializable JSON, pipeable between commands
- **Unix composable** - Chain commands: `bj new | bj hit | bj stand`
- **Configurable rules** - Vegas strip defaults, customizable via TOML
- **Colored output** - Red/black suits, colored outcomes (disable with `--no-color`)
- **Demo mode** - Automated play with basic strategy for testing/demos
- **Library + CLI** - Use as a Rust library or standalone CLI

## Installation

```bash
cargo install blackjack
```

Or build from source:

```bash
git clone https://github.com/KevinTCoughlin/blackjack.git
cd blackjack
cargo install --path .
```

## Quick Start

### Interactive Play

```bash
bj play
```

Controls: `h`=hit, `s`=stand, `d`=double, `p`=split, `u`=surrender, `q`=quit

### Unix-Style (Stateless)

```bash
# Start a new game
bj new

# Pipe state through actions
bj new | bj hit | bj stand

# Save/restore game state without truncating the input before it is read
bj new > game.json
bj hit < game.json > game.next.json && mv game.next.json game.json
bj stand < game.json
```

### Demo Mode

```bash
# Run 100 games with basic strategy
bj demo -n 100

# Visual demo with delay between actions
bj demo --delay=500 --verbose

# Reproducible results with seed
bj demo --seed=12345 -n 100
```

### Pretty Output

```bash
bj new -f pretty
```

```
┌─────────────────────────────────────────┐
│              BLACKJACK                  │
├─────────────────────────────────────────┤
│  Dealer: [??] [K♥]          Value: ?   │
│                                         │
│  You:    [A♠] [10♦]        Value: 21   │
│                           BLACKJACK!    │
├─────────────────────────────────────────┤
│  [H]it [S]tand [D]ouble [U]surrender    │
└─────────────────────────────────────────┘
```

## Commands

| Command | Description |
|---------|-------------|
| `bj new` | Start a new game (deal initial cards) |
| `bj hit` | Take another card |
| `bj stand` | Stop taking cards |
| `bj double` | Double down |
| `bj split` | Split a pair |
| `bj surrender` | Surrender (forfeit half bet) |
| `bj insurance` | Accept/decline insurance |
| `bj play` | Interactive play mode |
| `bj demo` | Automated play with basic strategy |
| `bj rules` | Display current rules |

### Common Options

| Option | Description |
|--------|-------------|
| `--no-color` | Disable colored output (global; place before the command) |
| `-f, --format` | Output format for stateless commands: `json` (default) or `pretty` |
| `-c, --config` | TOML config for `new`, `play`, `demo`, and `rules` |
| `--seed` | Reproducible shuffle for `new`, `play`, and `demo` |

## Configuration

Create a TOML file to customize rules:

```toml
# vegas.toml - Vegas Strip rules
num_decks = 6
dealer_stands_soft_17 = true
blackjack_pays = 1.5
allow_double = true
allow_split = true
allow_surrender = true
surrender_type = "late"
```

```bash
bj new --config=vegas.toml
bj play --config=vegas.toml
```

### All Options

| Option | Default | Description |
|--------|---------|-------------|
| `num_decks` | 6 | Number of decks (1-8) |
| `reshuffle_threshold` | 0.25 | Reshuffle when this % remains |
| `dealer_stands_soft_17` | true | Dealer stands on soft 17 |
| `dealer_peeks` | true | Dealer peeks for blackjack |
| `allow_double` | true | Allow double down |
| `double_on` | "any" | "any", "hard_9_to_11", "hard_10_or_11", "hard_11_only" |
| `allow_double_after_split` | true | Allow double after split |
| `allow_split` | true | Allow splitting pairs |
| `max_splits` | 4 | Maximum hands from splits |
| `allow_resplit_aces` | false | Allow resplitting aces |
| `allow_hit_split_aces` | false | Allow hitting split aces |
| `allow_surrender` | true | Allow surrender |
| `surrender_type` | "late" | "none", "late", or "early" |
| `allow_insurance` | true | Allow insurance bet |
| `blackjack_pays` | 1.5 | Blackjack payout (3:2 = 1.5, 6:5 = 1.2) |
| `insurance_pays` | 2.0 | Insurance payout |
| `five_card_charlie` | false | Five cards auto-wins |

## Library Usage

```rust
use blackjack::{GameState, GameConfig, Action};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create game with default Vegas strip rules
    let mut game = GameState::new(GameConfig::default());

    // Deal initial cards
    game.apply(Action::Deal)?;

    // Play the game
    while !game.is_finished() {
        let actions = game.available_actions();
        if actions.is_empty() {
            break;
        }

        // Your logic to choose an action
        let action = actions[0].clone();
        game.apply(action)?;
    }

    // Check outcomes
    for outcome in &game.outcomes {
        println!("Hand {}: {:?} (payout: {:.1}x)",
            outcome.hand_index,
            outcome.outcome,
            outcome.payout
        );
    }

    Ok(())
}
```

## WezTerm Plugin

For a rich terminal UI experience, check out [wezterm-blackjack](https://github.com/KevinTCoughlin/wezterm-blackjack).

## Support

If you find this useful, consider supporting development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/Q5Q11SKIOF)

## License

MIT
