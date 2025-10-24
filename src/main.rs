mod tui;
mod commands;

use nhl_api::{Client, ClientConfig};
use clap::{Parser, Subcommand, ValueEnum};
use xdg::BaseDirectories;

#[derive(Parser)]
#[command(name = "nhl-top")]
#[command(about = "NHL stats and standings CLI", long_about = "NHL stats and standings CLI\n\nIf no command is specified, the program starts in interactive mode.")]
struct Cli {
    /// Enable debug mode with verbose output
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Copy, ValueEnum)]
enum GroupBy {
    /// Group by division
    #[value(name = "d")]
    Division,
    /// Group by conference
    #[value(name = "c")]
    Conference,
    /// Show league-wide standings
    #[value(name = "l")]
    League,
}

#[derive(Subcommand)]
enum Commands {
    /// Display NHL standings
    Standings {
        /// Season year (optional, defaults to current standings)
        #[arg(short, long)]
        season: Option<i64>,

        /// Date in YYYY-MM-DD format (optional)
        #[arg(short, long)]
        date: Option<String>,

        /// Group standings by: d=division, c=conference, l=league
        #[arg(short, long, default_value = "d")]
        by: GroupBy,
    },
    /// Display boxscore for a specific game
    Boxscore {
        /// Game ID (e.g., 2024020001)
        game_id: i64,
    },
    /// Display daily schedule of games
    Schedule {
        /// Date in YYYY-MM-DD format (optional, defaults to today)
        #[arg(short, long)]
        date: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let pgm = env!("CARGO_PKG_NAME");
    let xdg_dirs = BaseDirectories::with_prefix(pgm);

    let cli = Cli::parse();

    // If no subcommand, run TUI
    if cli.command.is_none() {
        if let Err(e) = tui::run() {
            eprintln!("Error running TUI: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Create client with debug mode if flag is set
    let client = if cli.debug {
        let config = ClientConfig::default().with_debug();
        Client::with_config(config).unwrap()
    } else {
        Client::new().unwrap()
    };

    match cli.command.unwrap() {
        Commands::Standings { season, date, by } => {
            let group_by = match by {
                GroupBy::Division => commands::standings::GroupBy::Division,
                GroupBy::Conference => commands::standings::GroupBy::Conference,
                GroupBy::League => commands::standings::GroupBy::League,
            };
            commands::standings::run(&client, season, date, group_by).await;
        }
        Commands::Boxscore { game_id } => {
            commands::boxscore::run(&client, game_id).await;
        }
        Commands::Schedule { date } => {
            commands::schedule::run(&client, date).await;
        }
    }
}
