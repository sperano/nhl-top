mod tui;
mod commands;
mod background;
pub mod config;

use nhl_api::{Client, Standing, DailySchedule};
use clap::{Parser, Subcommand, ValueEnum};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use std::time::SystemTime;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

// Channel Constants
/// Buffer size for manual refresh trigger channel
const REFRESH_CHANNEL_BUFFER_SIZE: usize = 10;

// Default Configuration Constants
/// Default log level when not specified
const DEFAULT_LOG_LEVEL: &str = "info";

/// Default log file path (no logging to file)
const DEFAULT_LOG_FILE: &str = "/dev/null";

#[derive(Clone)]
pub struct SharedData {
    pub standings: Arc<Vec<Standing>>,
    pub schedule: Arc<Option<DailySchedule>>,
    pub period_scores: Arc<HashMap<i64, commands::scores_format::PeriodScores>>,
    pub game_info: Arc<HashMap<i64, nhl_api::GameMatchup>>,
    pub boxscore: Arc<Option<nhl_api::Boxscore>>,
    pub config: config::Config,
    pub last_refresh: Option<SystemTime>,
    pub game_date: nhl_api::GameDate,
    pub error_message: Option<String>,
    pub selected_game_id: Option<i64>,
    pub boxscore_loading: bool,
}

impl Default for SharedData {
    fn default() -> Self {
        SharedData {
            standings: Arc::new(Vec::new()),
            schedule: Arc::new(None),
            period_scores: Arc::new(HashMap::new()),
            game_info: Arc::new(HashMap::new()),
            boxscore: Arc::new(None),
            config: config::Config::default(),
            last_refresh: None,
            game_date: nhl_api::GameDate::today(),
            error_message: None,
            selected_game_id: None,
            boxscore_loading: false,
        }
    }
}

impl SharedData {
    /// Clear boxscore state (used when exiting boxscore view or switching tabs)
    pub fn clear_boxscore(&mut self) {
        self.selected_game_id = None;
        self.boxscore = Arc::new(None);
        self.boxscore_loading = false;
    }
}

pub type SharedDataHandle = Arc<RwLock<SharedData>>;

#[derive(Parser)]
#[command(name = "nhl")]
#[command(about = "NHL stats and standings CLI", long_about = "NHL stats and standings CLI\n\nIf no command is specified, the program starts in interactive mode.")]
struct Cli {
    /// Set log level (trace, debug, info, warn, error)
    #[arg(short = 'L', long, global = true, default_value = DEFAULT_LOG_LEVEL)]
    log_level: String,

    /// Log file path (default: /dev/null for no logging)
    #[arg(short = 'F', long, global = true, default_value = DEFAULT_LOG_FILE)]
    log_file: String,

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

impl GroupBy {
    /// Convert CLI GroupBy enum to commands::standings::GroupBy
    fn to_standings_groupby(&self) -> commands::standings::GroupBy {
        match self {
            GroupBy::Division => commands::standings::GroupBy::Division,
            GroupBy::Conference => commands::standings::GroupBy::Conference,
            GroupBy::League => commands::standings::GroupBy::League,
        }
    }
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

fn create_client() -> Client {
    match Client::new() {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to create NHL API client: {}", e);
            tracing::error!("{}", error_msg);
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }
}

fn init_logging(log_level: &str, log_file: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    let file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_file, e);
            return;
        }
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set tracing subscriber: {}", e);
    }
}

/// Handle the config command - display current configuration
fn handle_config_command() {
    let cfg = config::read();

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
    println!("log_level: {}", cfg.log_level);
    println!("log_file: {}", cfg.log_file);
    println!("refresh_interval: {} seconds", cfg.refresh_interval);
    println!("display_standings_western_first: {}", cfg.display_standings_western_first);
    println!("time_format: {}", cfg.time_format);
    println!();
    println!("[theme]");
    println!("selection_fg: {:?}", cfg.theme.selection_fg);
    println!("unfocused_selection_fg: {:?}{}",
        cfg.theme.unfocused_selection_fg(),
        if cfg.theme.unfocused_selection_fg.is_none() { " (auto: 50% darker)" } else { "" }
    );
}

/// Resolve log configuration from CLI args and config file
/// CLI arguments take precedence over config file
fn resolve_log_config<'a>(cli: &'a Cli, config: &'a config::Config) -> (&'a str, &'a str) {
    let log_level = if cli.log_level != DEFAULT_LOG_LEVEL {
        cli.log_level.as_str()
    } else {
        config.log_level.as_str()
    };

    let log_file = if cli.log_file != DEFAULT_LOG_FILE {
        cli.log_file.as_str()
    } else {
        config.log_file.as_str()
    };

    (log_level, log_file)
}

/// Run TUI mode with background data fetching
async fn run_tui_mode(config: config::Config) -> Result<(), std::io::Error> {
    let shared_data: SharedDataHandle = Arc::new(RwLock::new(SharedData {
        config: config.clone(),
        ..Default::default()
    }));

    // Create channel for manual refresh triggers
    let (refresh_tx, refresh_rx) = mpsc::channel::<()>(REFRESH_CHANNEL_BUFFER_SIZE);

    // Spawn background task to fetch data
    let bg_client = create_client();
    let shared_data_clone = Arc::clone(&shared_data);
    let refresh_interval = config.refresh_interval as u64;
    tokio::spawn(async move {
        background::fetch_data_loop(bg_client, shared_data_clone, refresh_interval, refresh_rx).await;
    });

    tui::run(shared_data, refresh_tx).await
}

/// Execute a CLI command by routing it to the appropriate command handler
async fn execute_command(client: &Client, command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Config => unreachable!("Config command should be handled before execute_command"),
        Commands::Standings { season, date, by } => {
            let group_by = by.to_standings_groupby();
            commands::standings::run(client, season, date, group_by).await
        }
        Commands::Boxscore { game_id } => {
            commands::boxscore::run(client, game_id).await
        }
        Commands::Schedule { date } => {
            commands::schedule::run(client, date).await
        }
        Commands::Scores { date } => {
            commands::scores::run(client, date).await
        }
    }
}

#[tokio::main]
async fn main() {
    let config = config::read();
    let cli = Cli::parse();

    // Resolve and initialize logging
    let (log_level, log_file) = resolve_log_config(&cli, &config);
    if log_file != DEFAULT_LOG_FILE {
        init_logging(log_level, log_file);
    }

    // If no subcommand, run TUI
    if cli.command.is_none() {
        if let Err(e) = run_tui_mode(config).await {
            eprintln!("Error running TUI: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let command = cli.command.unwrap();

    // Handle Config command separately (doesn't need a client)
    if let Commands::Config = command {
        handle_config_command();
        return;
    }

    // Create client and execute command
    let client = create_client();
    if let Err(e) = execute_command(&client, command).await {
        eprintln!("Error: {:#}", e);
        tracing::error!("Command failed: {:#}", e);
        std::process::exit(1);
    }
}
