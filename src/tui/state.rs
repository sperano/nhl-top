use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

use nhl_api::{Boxscore, ClubStats, DailySchedule, GameDate, GameMatchup, PlayerLanding, Standing};

use crate::commands::scores_format::PeriodScores;
use crate::commands::standings::GroupBy;
use crate::config::Config;

use super::document::{FocusableId, RowPosition};
use super::types::{Panel, SettingsCategory, Tab};

/// Root application state - single source of truth
///
/// This is the entire application state in one place.
/// All state changes happen through the reducer.
/// Components receive slices of this state as props.
#[derive(Debug, Clone, Default)]
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
    /// Selected item index within the panel (None = no selection)
    /// Used for navigating lists (players, games, etc.) within panels
    pub selected_index: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct DataState {
    // API data - wrapped in Arc to avoid deep clones on every reducer call
    pub standings: Arc<Option<Vec<Standing>>>,
    pub schedule: Arc<Option<DailySchedule>>,
    pub game_info: Arc<HashMap<i64, GameMatchup>>,
    pub period_scores: Arc<HashMap<i64, PeriodScores>>,
    pub boxscores: Arc<HashMap<i64, Boxscore>>,
    pub team_roster_stats: Arc<HashMap<String, ClubStats>>,
    pub player_data: Arc<HashMap<i64, PlayerLanding>>,

    // Loading states
    pub loading: HashSet<LoadingKey>,

    // Errors
    pub errors: HashMap<String, String>,
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

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub scores: ScoresUiState,
    pub standings: StandingsUiState,
    pub settings: SettingsUiState,
    pub demo: DemoUiState,
}

/// UI state for the Demo tab (document system showcase)
#[derive(Debug, Clone, Default)]
pub struct DemoUiState {
    /// Current focus index within the document (None = no focus)
    pub focus_index: Option<usize>,
    /// Current scroll offset
    pub scroll_offset: u16,
    /// Viewport height (updated during render)
    pub viewport_height: u16,
    /// Y-positions of focusable elements (populated by component during render)
    /// Used by reducer for accurate autoscrolling
    pub focusable_positions: Vec<u16>,
    /// IDs of focusable elements (populated by component during render)
    /// Used for meaningful display when activating elements
    pub focusable_ids: Vec<FocusableId>,
    /// Row positions for left/right navigation within Row elements
    pub focusable_row_positions: Vec<Option<RowPosition>>,
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
            layout: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SettingsUiState {
    pub selected_category: SettingsCategory,
    pub selected_setting_index: usize,
    pub settings_mode: bool, // true = navigating settings, false = navigating categories
    pub editing: bool,       // true = editing a setting value, false = not editing
    pub edit_buffer: String, // Buffer for editing string/int values
    pub modal_open: bool,    // true = list selection modal is open
    pub modal_selected_index: usize, // Selected index within the modal
}

/// Default help message shown in the status bar
pub const DEFAULT_STATUS_MESSAGE: &str =
    "Keys: ←→ navigate | ↓ enter | ↑/ESC back | q quit | 1-6 jump to tab | / command palette";

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub last_refresh: Option<SystemTime>,
    pub config: Config,
    pub status_message: Option<String>,
    pub status_is_error: bool,
}

impl SystemState {
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_is_error = false;
    }

    pub fn set_status_error_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_is_error = true;
    }

    pub fn reset_status_message(&mut self) {
        self.status_message = Some(DEFAULT_STATUS_MESSAGE.to_string());
        self.status_is_error = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_status_message() {
        let mut state = SystemState::default();

        state.set_status_message("Test message".to_string());

        assert_eq!(state.status_message, Some("Test message".to_string()));
        assert!(!state.status_is_error);
    }

    #[test]
    fn test_set_status_error_message() {
        let mut state = SystemState::default();

        state.set_status_error_message("Error message".to_string());

        assert_eq!(state.status_message, Some("Error message".to_string()));
        assert!(state.status_is_error);
    }

    #[test]
    fn test_set_status_message_overwrites_error_flag() {
        let mut state = SystemState::default();

        // First set an error
        state.set_status_error_message("Error".to_string());
        assert!(state.status_is_error);

        // Then set a normal message - should clear error flag
        state.set_status_message("Normal message".to_string());
        assert_eq!(state.status_message, Some("Normal message".to_string()));
        assert!(!state.status_is_error);
    }

    #[test]
    fn test_set_status_error_message_overwrites_normal_message() {
        let mut state = SystemState::default();

        // First set a normal message
        state.set_status_message("Normal".to_string());
        assert!(!state.status_is_error);

        // Then set an error - should set error flag
        state.set_status_error_message("Error message".to_string());
        assert_eq!(state.status_message, Some("Error message".to_string()));
        assert!(state.status_is_error);
    }

    #[test]
    fn test_reset_status_message() {
        let mut state = SystemState::default();

        // Set a custom message
        state.set_status_message("Custom message".to_string());
        assert_eq!(state.status_message, Some("Custom message".to_string()));

        // Reset should restore default
        state.reset_status_message();
        assert_eq!(
            state.status_message,
            Some(DEFAULT_STATUS_MESSAGE.to_string())
        );
        assert!(!state.status_is_error);
    }

    #[test]
    fn test_reset_status_message_clears_error_flag() {
        let mut state = SystemState::default();

        // Set an error message
        state.set_status_error_message("Error".to_string());
        assert!(state.status_is_error);

        // Reset should clear error flag
        state.reset_status_message();
        assert!(!state.status_is_error);
    }
}
