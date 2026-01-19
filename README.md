# blackjack

A Unix-philosophy blackjack game with a stateless CLI. The game state is fully serializable, allowing it to be piped between commands or stored/restored from files.

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

## Usage

### Interactive Play

```bash
bj play
```

### Stateless Commands (Unix-style)

```bash
# Start a new game
bj new

# Pipe state through actions
bj new | bj hit | bj stand

# Or use files
bj new > game.json
bj hit < game.json > game.json
bj stand < game.json
```

### Demo Mode

Run automated games using basic strategy:

```bash
# Single demo game
bj demo

# Run 100 games and show statistics
bj demo --count=100

# Visual demo with delay
bj demo --delay=500 --verbose

# Reproducible run
bj demo --seed=12345 --count=100
```

### Show Rules

```bash
bj rules
bj rules --config=vegas.toml
```

## Commands

| Command | Description |
|---------|-------------|
| `new` | Start a new game (deal initial cards) |
| `hit` | Take another card |
| `stand` | Stop taking cards |
| `double` | Double down |
| `split` | Split a pair |
| `surrender` | Surrender (give up half bet) |
| `insurance` | Accept or decline insurance |
| `play` | Interactive play mode |
| `demo` | Automated play with basic strategy |
| `rules` | Show current rules |

## Configuration

Create a TOML config file to customize rules:

```toml
# vegas.toml
num_decks = 6
dealer_stands_soft_17 = true
blackjack_pays = 1.5
allow_surrender = true
surrender_type = "late"
```

Use with: `bj new --config=vegas.toml`

### Available Options

| Option | Default | Description |
|--------|---------|-------------|
| `num_decks` | 6 | Number of decks (1-8) |
| `reshuffle_threshold` | 0.25 | Reshuffle when this % remains |
| `dealer_stands_soft_17` | true | Dealer stands on soft 17 |
| `dealer_peeks` | true | Dealer peeks for blackjack |
| `allow_double` | true | Allow double down |
| `double_on` | "any" | When doubling is allowed |
| `allow_double_after_split` | true | Double after split |
| `allow_split` | true | Allow splitting pairs |
| `max_splits` | 4 | Maximum hands from splits |
| `allow_resplit_aces` | false | Resplit aces |
| `allow_hit_split_aces` | false | Hit split aces |
| `allow_surrender` | true | Allow surrender |
| `surrender_type` | "late" | "none", "late", or "early" |
| `allow_insurance` | true | Allow insurance bet |
| `blackjack_pays` | 1.5 | Blackjack payout (3:2 = 1.5) |
| `insurance_pays` | 2.0 | Insurance payout (2:1) |
| `five_card_charlie` | false | Five cards wins automatically |

## Library Usage

```rust
use blackjack::{GameState, GameConfig, Action};

let mut game = GameState::new(GameConfig::default());
game.apply(Action::Deal)?;

while !game.is_finished() {
    let actions = game.available_actions();
    // Choose an action...
    game.apply(Action::Stand)?;
}

for outcome in &game.outcomes {
    println!("{:?}: payout = {}", outcome.outcome, outcome.payout);
}
```

## License

MIT
