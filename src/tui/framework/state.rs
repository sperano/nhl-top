use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use nhl_api::{Boxscore, ClubStats, DailySchedule, GameDate, GameMatchup, PlayerLanding, Standing};

use crate::commands::scores_format::PeriodScores;
use crate::commands::standings::GroupBy;
use crate::config::Config;

use super::action::{Panel, Tab};

/// Root application state - single source of truth
///
/// This is the entire application state in one place.
/// All state changes happen through the reducer.
/// Components receive slices of this state as props.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Navigation state (which tab, panel stack)
    pub navigation: NavigationState,

    /// Application data (from API)
    pub data: DataState,

    /// UI state per tab
    pub ui: UiState,

    /// System state
    pub system: SystemState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            navigation: NavigationState::default(),
            data: DataState::default(),
            ui: UiState::default(),
            system: SystemState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_tab: Tab,
    pub panel_stack: Vec<PanelState>,
    /// Whether focus is on content (true) or tab bar (false)
    pub content_focused: bool,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Scores,
            panel_stack: Vec::new(),
            content_focused: false, // Start with tab bar focused
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanelState {
    pub panel: Panel,
    pub scroll_offset: usize,
    /// Selected item index within the panel (None = no selection)
    /// Used for navigating lists (players, games, etc.) within panels
    pub selected_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DataState {
    // API data
    pub standings: Option<Vec<Standing>>,
    pub schedule: Option<DailySchedule>,
    pub game_info: HashMap<i64, GameMatchup>,
    pub period_scores: HashMap<i64, PeriodScores>,
    pub boxscores: HashMap<i64, Boxscore>,
    pub team_roster_stats: HashMap<String, ClubStats>,
    pub player_data: HashMap<i64, PlayerLanding>,

    // Loading states
    pub loading: HashSet<LoadingKey>,

    // Errors
    pub errors: HashMap<String, String>,
}

impl Default for DataState {
    fn default() -> Self {
        Self {
            standings: None,
            schedule: None,
            game_info: HashMap::new(),
            period_scores: HashMap::new(),
            boxscores: HashMap::new(),
            team_roster_stats: HashMap::new(),
            player_data: HashMap::new(),
            loading: HashSet::new(),
            errors: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingKey {
    Standings,
    Schedule(String), // GameDate formatted as string
    GameDetails(i64),
    Boxscore(i64),
    TeamRosterStats(String), // Team abbreviation
    PlayerStats(i64),
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub scores: ScoresUiState,
    pub standings: StandingsUiState,
    pub settings: SettingsUiState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            scores: ScoresUiState::default(),
            standings: StandingsUiState::default(),
            settings: SettingsUiState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoresUiState {
    pub selected_date_index: usize,
    pub game_date: GameDate,
    pub box_selection_active: bool,
    pub selected_game_index: Option<usize>,
    pub boxes_per_row: u16, // Calculated grid columns for game navigation
}

impl Default for ScoresUiState {
    fn default() -> Self {
        Self {
            selected_date_index: 2, // Middle of 5-date window
            game_date: GameDate::today(),
            box_selection_active: false,
            selected_game_index: None,
            boxes_per_row: 2, // Default to 2 columns
        }
    }
}

#[derive(Debug, Clone)]
pub struct StandingsUiState {
    pub view: GroupBy,
    pub browse_mode: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub scroll_offset: usize,
    /// Cached layout: layout[column][row] = team_abbrev
    /// Rebuilt when standings data changes or view changes
    pub layout: Vec<Vec<String>>,
}

impl Default for StandingsUiState {
    fn default() -> Self {
        Self {
            view: GroupBy::Wildcard,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            scroll_offset: 0,
            layout: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Logging,
    Display,
    Data,
}

impl Default for SettingsCategory {
    fn default() -> Self {
        Self::Logging
    }
}

#[derive(Debug, Clone)]
pub struct SettingsUiState {
    pub selected_category: SettingsCategory,
    pub selected_setting_index: usize,
    pub settings_mode: bool, // true = navigating settings, false = navigating categories
    pub editing: bool,        // true = editing a setting value, false = not editing
    pub edit_buffer: String, // Buffer for editing string/int values
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            selected_category: SettingsCategory::default(),
            selected_setting_index: 0,
            settings_mode: false,
            editing: false,
            edit_buffer: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub last_refresh: Option<SystemTime>,
    pub config: Config,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            last_refresh: None,
            config: Config::default(),
        }
    }
}
