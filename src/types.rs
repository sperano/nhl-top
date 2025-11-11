/// Shared types used across the application
///
/// This module contains type definitions that are shared between
/// the library (commands, tui) and the binary (main.rs).

use nhl_api::{Standing, DailySchedule};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Global constants
pub const NHL_LEAGUE_ABBREV: &str = "NHL";

/// Shared data structure for TUI mode
///
/// Contains all application state that is shared between the background
/// data fetching loop and the TUI rendering loop via Arc<RwLock<>>.
#[derive(Clone)]
pub struct SharedData {
    pub standings: Arc<Vec<Standing>>,
    pub schedule: Arc<Option<DailySchedule>>,
    pub period_scores: Arc<HashMap<i64, crate::commands::scores_format::PeriodScores>>,
    pub game_info: Arc<HashMap<i64, nhl_api::GameMatchup>>,
    pub boxscore: Arc<Option<nhl_api::Boxscore>>,
    pub club_stats: Arc<HashMap<String, nhl_api::ClubStats>>,
    pub player_info: Arc<HashMap<i64, nhl_api::PlayerLanding>>,
    pub config: crate::config::Config,
    pub last_refresh: Option<SystemTime>,
    pub game_date: nhl_api::GameDate,
    pub status_message: Option<String>,
    pub status_is_error: bool,
    pub selected_game_id: Option<i64>,
    pub boxscore_loading: bool,
    pub selected_team_abbrev: Option<String>,
    pub club_stats_loading: bool,
    pub selected_player_id: Option<i64>,
    pub player_info_loading: bool,
}

impl Default for SharedData {
    fn default() -> Self {
        SharedData {
            standings: Arc::new(Vec::new()),
            schedule: Arc::new(None),
            period_scores: Arc::new(HashMap::new()),
            game_info: Arc::new(HashMap::new()),
            boxscore: Arc::new(None),
            club_stats: Arc::new(HashMap::new()),
            player_info: Arc::new(HashMap::new()),
            config: crate::config::Config::default(),
            last_refresh: None,
            game_date: nhl_api::GameDate::today(),
            status_message: None,
            status_is_error: false,
            selected_game_id: None,
            boxscore_loading: false,
            selected_team_abbrev: None,
            club_stats_loading: false,
            selected_player_id: None,
            player_info_loading: false,
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

    /// Set an error status message
    pub fn set_error(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_is_error = true;
    }

    /// Set a non-error status message
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_is_error = false;
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_is_error = false;
    }
}

/// Type alias for thread-safe shared data
pub type SharedDataHandle = Arc<RwLock<SharedData>>;
