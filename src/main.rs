mod tui;
mod commands;
mod config;

use nhl_api::{Client, ClientConfig, Standing, DailySchedule};
use clap::{Parser, Subcommand, ValueEnum};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use std::time::{Duration, SystemTime};
use futures::future::join_all;

#[derive(Clone)]
pub struct SharedData {
    pub standings: Vec<Standing>,
    pub schedule: Option<DailySchedule>,
    pub period_scores: HashMap<i64, commands::scores_format::PeriodScores>,
    pub game_info: HashMap<i64, nhl_api::GameMatchup>,
    pub config: config::Config,
    pub last_refresh: Option<SystemTime>,
    pub game_date: nhl_api::GameDate,
    pub error_message: Option<String>,
}

impl Default for SharedData {
    fn default() -> Self {
        SharedData {
            standings: Vec::new(),
            schedule: None,
            period_scores: HashMap::new(),
            game_info: HashMap::new(),
            config: config::Config::default(),
            last_refresh: None,
            game_date: nhl_api::GameDate::today(),
            error_message: None,
        }
    }
}

pub type SharedDataHandle = Arc<RwLock<SharedData>>;

#[derive(Parser)]
#[command(name = "nhl")]
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
    /// Display scores for games with period-by-period breakdown
    Scores {
        /// Date in YYYY-MM-DD format (optional, defaults to today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Display current configuration
    Config,
}

/// Create an NHL API client with optional debug mode
fn create_client(debug: bool) -> Client {
    if debug {
        let config = ClientConfig::default().with_debug();
        Client::with_config(config).unwrap()
    } else {
        Client::new().unwrap()
    }
}

async fn fetch_data_loop(client: Client, shared_data: SharedDataHandle, interval: u64, mut refresh_rx: mpsc::Receiver<()>) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
    interval_timer.tick().await; // First tick completes immediately

    loop {
        // Wait for either the interval timer or a manual refresh signal
        tokio::select! {
            _ = interval_timer.tick() => {
                // Regular interval refresh
            }
            _ = refresh_rx.recv() => {
                // Manual refresh triggered
            }
        }
        // Fetch standings
        match client.current_league_standings().await {
            Ok(data) => {
                let mut shared = shared_data.write().await;
                shared.standings = data;
                shared.last_refresh = Some(SystemTime::now());
                shared.error_message = None; // Clear any previous errors
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.error_message = Some(format!("Failed to fetch standings: {}", e));
            }
        }

        // Fetch schedule for the current game_date
        let date = {
            let shared = shared_data.read().await;
            shared.game_date.clone()
        };
        match client.daily_schedule(Some(&date)).await {
            Ok(schedule) => {
                // Fetch period scores and game info for LIVE and FINAL games
                let mut period_scores = HashMap::new();
                let mut game_info = HashMap::new();

                // Collect all games that need fetching
                let games_to_fetch: Vec<_> = schedule.games.iter()
                    .filter(|game| game.game_state.has_started())
                    .collect();

                // Create futures for all landing requests
                let fetch_futures = games_to_fetch.iter().map(|game| {
                    let game_id = nhl_api::GameId::new(game.id);
                    let game_clone = (*game).clone();
                    let client_ref = &client;
                    async move {
                        let result = client_ref.landing(&game_id).await;
                        (game_clone, result)
                    }
                });

                // Execute all requests in parallel
                let results = join_all(fetch_futures).await;

                // Process results
                for (game, result) in results {
                    match result {
                        Ok(landing) => {
                            if let Some(summary) = &landing.summary {
                                let scores = commands::scores_format::extract_period_scores(
                                    summary,
                                    game.away_team.id,
                                    game.home_team.id,
                                );
                                period_scores.insert(game.id, scores);
                            }
                            // Store the full game info for clock/period display
                            game_info.insert(game.id, landing);
                        }
                        Err(e) => {
                            // Store error for individual game fetch failures
                            let mut shared = shared_data.write().await;
                            shared.error_message = Some(format!("Failed to fetch game {} data: {}", game.id, e));
                        }
                    }
                }

                let mut shared = shared_data.write().await;
                shared.schedule = Some(schedule);
                shared.period_scores = period_scores;
                shared.game_info = game_info;
                // Note: errors from individual game fetches are preserved
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.error_message = Some(format!("Failed to fetch schedule: {}", e));
            }
        }

    }
}

#[tokio::main]
async fn main() {
    let mut config = config::read();

    let cli = Cli::parse();
    config.debug = config.debug || cli.debug;

    // If no subcommand, run TUI
    if cli.command.is_none() {
        // Create shared data structure with config
        let shared_data: SharedDataHandle = Arc::new(RwLock::new(SharedData {
            standings: Vec::new(),
            schedule: None,
            period_scores: HashMap::new(),
            game_info: HashMap::new(),
            config: config.clone(),
            last_refresh: None,
            game_date: nhl_api::GameDate::today(),
            error_message: None,
        }));

        // Create channel for manual refresh triggers
        let (refresh_tx, refresh_rx) = mpsc::channel::<()>(10);

        // Create client for background task
        let bg_client = create_client(config.debug);

        // Spawn background task to fetch data
        let shared_data_clone = Arc::clone(&shared_data);
        let refresh_interval = config.refresh_interval as u64;
        tokio::spawn(async move {
            fetch_data_loop(bg_client, shared_data_clone, refresh_interval, refresh_rx).await;
        });

        if let Err(e) = tui::run(shared_data, refresh_tx).await {
            eprintln!("Error running TUI: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let command = cli.command.unwrap();

    // Handle Config command separately (doesn't need a client)
    if let Commands::Config = command {
        let (path_str, exists) = match config::get_config_path() {
            Some(path) => {
                let exists = path.exists();
                (path.display().to_string(), exists)
            }
            None => ("Unable to determine config path".to_string(), false),
        };

        println!("Configuration File: {} (Exists: {})", path_str, if exists { "yes" } else { "no" });
        println!();
        println!("Current Configuration:");
        println!("=====================");
        println!("debug: {}", config.debug);
        println!("refresh_interval: {} seconds", config.refresh_interval);
        println!("display_standings_western_first: {}", config.display_standings_western_first);
        println!("time_format: {}", config.time_format);
        return;
    }

    // Create client once for all other commands
    let client = create_client(config.debug);

    match command {
        Commands::Config => unreachable!(), // Already handled above
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
        Commands::Scores { date } => {
            commands::scores::run(&client, date).await;
        }
    }
}

// fn init_logging() -> Result<()> {
//     if let Err(e) = color_eyre::install() {
//         return Err(anyhow!("Failed to install color_eyre: {}", e));
//     }
//     // only enable logging in debug builds
//     #[cfg(debug_assertions)]
//     {
//         use simplelog::*;
//         use std::fs::File;
//
//         let pgm = env!("CARGO_PKG_NAME");
//         let xdg_dirs = BaseDirectories::with_prefix(pgm);
//         let log_path = xdg_dirs.place_cache_file(format!("{}.log", pgm))?;
//         let config = ConfigBuilder::new()
//             .set_thread_level(LevelFilter::Error)
//             .set_thread_mode(ThreadLogMode::Names)
//             .set_thread_padding(ThreadPadding::Left(9))
//             .set_level_padding(LevelPadding::Right)
//             .build();
//         CombinedLogger::init(vec![WriteLogger::new(
//             LevelFilter::Debug,
//             config,
//             File::create(log_path)?,
//         )])?;
//     }
//     Ok(())
// }

