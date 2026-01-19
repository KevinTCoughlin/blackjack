//! Blackjack CLI - Unix-philosophy blackjack game.
//!
//! A stateless, composable blackjack game that reads state from stdin,
//! processes an action, and outputs new state to stdout.

use std::io::{self, BufRead, Write};
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};

use blackjack::{
    decide_action, format_game_state, set_color_enabled, Action, GameConfig, GameState,
    OutputFormat, Strategy,
};

#[derive(Parser)]
#[command(name = "bj")]
#[command(author = "Kevin Coughlin")]
#[command(version)]
#[command(about = "A Unix-philosophy blackjack game", long_about = None)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new game (outputs initial state after dealing)
    New {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Path to custom configuration file (TOML)
        #[arg(short, long)]
        config: Option<String>,

        /// Seed for reproducible shuffling
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Take a hit (draw another card)
    Hit {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Stand (stop drawing cards)
    Stand {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Double down (double bet, take one card, then stand)
    Double {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Split a pair into two hands
    Split {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Surrender (give up half the bet)
    Surrender {
        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Accept or decline insurance
    Insurance {
        /// Accept insurance bet
        #[arg(short, long)]
        accept: bool,

        /// Output format: json or pretty
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Interactive play mode
    Play {
        /// Path to custom configuration file (TOML)
        #[arg(short, long)]
        config: Option<String>,

        /// Seed for reproducible shuffling
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Demo mode - automated play with basic strategy
    Demo {
        /// Number of games to play
        #[arg(short = 'n', long, default_value = "1")]
        count: u32,

        /// Delay between actions in milliseconds (for visual demos)
        #[arg(short, long, default_value = "0")]
        delay: u64,

        /// Strategy to use: basic, random, never_bust
        #[arg(short, long, default_value = "basic")]
        strategy: String,

        /// Seed for reproducible results
        #[arg(long)]
        seed: Option<u64>,

        /// Show verbose output for each action
        #[arg(short, long)]
        verbose: bool,

        /// Path to custom configuration file (TOML)
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Show the current rules from configuration
    Rules {
        /// Path to custom configuration file (TOML)
        #[arg(short, long)]
        config: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Handle --no-color flag
    if cli.no_color {
        set_color_enabled(false);
    }

    let result = match cli.command {
        Commands::New {
            format,
            config,
            seed,
        } => cmd_new(&format, config.as_deref(), seed),
        Commands::Hit { format } => cmd_action(Action::Hit, &format),
        Commands::Stand { format } => cmd_action(Action::Stand, &format),
        Commands::Double { format } => cmd_action(Action::Double, &format),
        Commands::Split { format } => cmd_action(Action::Split, &format),
        Commands::Surrender { format } => cmd_action(Action::Surrender, &format),
        Commands::Insurance { accept, format } => cmd_action(Action::Insurance(accept), &format),
        Commands::Play { config, seed } => cmd_play(config.as_deref(), seed),
        Commands::Demo {
            count,
            delay,
            strategy,
            seed,
            verbose,
            config,
        } => cmd_demo(count, delay, &strategy, seed, verbose, config.as_deref()),
        Commands::Rules { config } => cmd_rules(config.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn load_config(path: Option<&str>) -> Result<GameConfig, String> {
    match path {
        Some(p) => GameConfig::from_file(p).map_err(|e| e.to_string()),
        None => Ok(GameConfig::default()),
    }
}

fn parse_format(format: &str) -> Result<OutputFormat, String> {
    format.parse()
}

fn cmd_new(format: &str, config_path: Option<&str>, seed: Option<u64>) -> Result<(), String> {
    let config = load_config(config_path)?;
    let format = parse_format(format)?;

    let mut state = match seed {
        Some(s) => GameState::with_seed(config, s),
        None => GameState::new(config),
    };

    // Deal initial cards
    state.apply(Action::Deal).map_err(|e| e.to_string())?;

    println!("{}", format_game_state(&state, format));
    Ok(())
}

fn cmd_action(action: Action, format: &str) -> Result<(), String> {
    let format = parse_format(format)?;

    // Read state from stdin
    let mut input = String::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        input.push_str(&line.map_err(|e| e.to_string())?);
        input.push('\n');
    }

    let mut state: GameState = serde_json::from_str(&input)
        .map_err(|e| format!("Failed to parse game state from stdin: {}", e))?;

    // Apply the action
    state.apply(action).map_err(|e| e.to_string())?;

    println!("{}", format_game_state(&state, format));
    Ok(())
}

fn cmd_play(config_path: Option<&str>, seed: Option<u64>) -> Result<(), String> {
    let config = load_config(config_path)?;

    let mut state = match seed {
        Some(s) => GameState::with_seed(config, s),
        None => GameState::new(config),
    };

    println!("Welcome to Blackjack!");
    println!("Commands: h=hit, s=stand, d=double, p=split, u=surrender, n=new, q=quit");
    println!();

    // Initial deal
    state.apply(Action::Deal).map_err(|e| e.to_string())?;

    loop {
        // Show current state
        println!("{}", format_game_state(&state, OutputFormat::Pretty));

        if state.is_finished() {
            print!("Play again? (Y/n): ");
            io::stdout().flush().ok();

            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();

            let response = input.trim().to_lowercase();
            // Default to yes - only quit on explicit 'n'
            if response == "n" || response == "no" || response == "q" {
                break;
            } else {
                state = GameState::new(state.config.clone());
                state.apply(Action::Deal).map_err(|e| e.to_string())?;
                continue;
            }
        }

        let actions = state.available_actions();
        if actions.is_empty() {
            continue;
        }

        print!("> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let action = match input.trim().to_lowercase().as_str() {
            "h" | "hit" => Some(Action::Hit),
            "s" | "stand" => Some(Action::Stand),
            "d" | "double" => Some(Action::Double),
            "p" | "split" => Some(Action::Split),
            "u" | "surrender" => Some(Action::Surrender),
            "y" | "yes" if actions.contains(&Action::Insurance(true)) => {
                Some(Action::Insurance(true))
            }
            "n" | "no" if actions.contains(&Action::Insurance(false)) => {
                Some(Action::Insurance(false))
            }
            "q" | "quit" => break,
            "" if actions.contains(&Action::Deal) => Some(Action::Deal),
            _ => {
                println!("Invalid command. Try: h, s, d, p, u, y, n, q");
                continue;
            }
        };

        if let Some(action) = action {
            if actions.contains(&action)
                || matches!(action, Action::Insurance(_))
                    && (actions.contains(&Action::Insurance(true))
                        || actions.contains(&Action::Insurance(false)))
            {
                if let Err(e) = state.apply(action) {
                    println!("Error: {}", e);
                }
            } else {
                println!("Action not available");
            }
        }
    }

    println!("Thanks for playing!");
    Ok(())
}

fn cmd_demo(
    count: u32,
    delay: u64,
    strategy_str: &str,
    seed: Option<u64>,
    verbose: bool,
    config_path: Option<&str>,
) -> Result<(), String> {
    let config = load_config(config_path)?;
    let strategy: Strategy = strategy_str.parse()?;

    let mut wins = 0u32;
    let mut losses = 0u32;
    let mut pushes = 0u32;
    let mut blackjacks = 0u32;
    let mut total_payout = 0.0f32;

    for game_num in 0..count {
        let game_seed = seed.map(|s| s.wrapping_add(game_num as u64));

        let mut state = match game_seed {
            Some(s) => GameState::with_seed(config.clone(), s),
            None => GameState::new(config.clone()),
        };

        if verbose {
            println!("=== Game {} ===", game_num + 1);
        }

        // Play the game
        while !state.is_finished() {
            if let Some(action) = decide_action(&state, strategy) {
                if verbose {
                    println!("{}", format_game_state(&state, OutputFormat::Pretty));
                    println!("Action: {:?}", action);

                    if delay > 0 {
                        thread::sleep(Duration::from_millis(delay));
                    }
                }

                if let Err(e) = state.apply(action) {
                    eprintln!("Error: {}", e);
                    break;
                }
            } else {
                break;
            }
        }

        if verbose {
            println!("{}", format_game_state(&state, OutputFormat::Pretty));
            println!();
        }

        // Collect statistics
        for outcome in &state.outcomes {
            total_payout += outcome.payout;
            match outcome.outcome {
                blackjack::Outcome::Win => wins += 1,
                blackjack::Outcome::Lose | blackjack::Outcome::Bust => losses += 1,
                blackjack::Outcome::Push => pushes += 1,
                blackjack::Outcome::Blackjack => {
                    wins += 1;
                    blackjacks += 1;
                }
                blackjack::Outcome::Surrender => losses += 1,
            }
        }
    }

    // Print statistics
    let total_hands = wins + losses + pushes;
    println!("=== Statistics ({} games, {} hands) ===", count, total_hands);
    println!(
        "Wins:       {} ({:.1}%)",
        wins,
        (wins as f32 / total_hands as f32) * 100.0
    );
    println!(
        "Losses:     {} ({:.1}%)",
        losses,
        (losses as f32 / total_hands as f32) * 100.0
    );
    println!(
        "Pushes:     {} ({:.1}%)",
        pushes,
        (pushes as f32 / total_hands as f32) * 100.0
    );
    println!("Blackjacks: {}", blackjacks);
    println!(
        "Net payout: {:.2} units ({:.2}%)",
        total_payout,
        (total_payout / total_hands as f32) * 100.0
    );

    Ok(())
}

fn cmd_rules(config_path: Option<&str>) -> Result<(), String> {
    let config = load_config(config_path)?;

    println!("=== Blackjack Rules ===");
    println!();
    println!("Deck Configuration:");
    println!("  Number of decks: {}", config.num_decks);
    println!(
        "  Reshuffle at: {:.0}% remaining",
        config.reshuffle_threshold * 100.0
    );
    println!();
    println!("Dealer Rules:");
    println!(
        "  Dealer stands on soft 17: {}",
        if config.dealer_stands_soft_17 {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "  Dealer peeks for blackjack: {}",
        if config.dealer_peeks { "Yes" } else { "No" }
    );
    println!();
    println!("Player Options:");
    println!(
        "  Double down: {}",
        if config.allow_double { "Yes" } else { "No" }
    );
    println!("  Double on: {:?}", config.double_on);
    println!(
        "  Double after split: {}",
        if config.allow_double_after_split {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "  Split: {}",
        if config.allow_split { "Yes" } else { "No" }
    );
    println!("  Max splits: {} hands", config.max_splits);
    println!(
        "  Resplit aces: {}",
        if config.allow_resplit_aces {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "  Hit split aces: {}",
        if config.allow_hit_split_aces {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "  Surrender: {}",
        if config.allow_surrender { "Yes" } else { "No" }
    );
    println!("  Surrender type: {:?}", config.surrender_type);
    println!(
        "  Insurance: {}",
        if config.allow_insurance { "Yes" } else { "No" }
    );
    println!();
    println!("Payouts:");
    println!(
        "  Blackjack pays: {}:1 ({:.1}x)",
        if config.blackjack_pays == 1.5 {
            "3:2"
        } else {
            "6:5"
        },
        config.blackjack_pays
    );
    println!("  Insurance pays: 2:1 ({:.1}x)", config.insurance_pays);
    println!();
    println!("Special Rules:");
    println!(
        "  Five card charlie: {}",
        if config.five_card_charlie {
            "Yes"
        } else {
            "No"
        }
    );

    Ok(())
}
