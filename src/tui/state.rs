use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

use nhl_api::{Boxscore, ClubStats, DailySchedule, GameDate, GameMatchup, PlayerLanding, Standing};

use crate::commands::scores_format::PeriodScores;
use crate::config::Config;

use super::document::{FocusableId, RowPosition};
use super::types::{SettingsCategory, StackedDocument, Tab};

/// Shared state for document-based navigation
///
/// This struct contains the common fields needed for document navigation,
/// scrolling, and focus management. It's embedded in any UI state that
/// uses the document system (Demo tab, Standings League/Conference views).
#[derive(Debug, Clone, Default)]
pub struct DocumentState {
    /// Current focus index within the document (None = no focus)
    pub focus_index: Option<usize>,
    /// Current scroll offset (lines from top)
    pub scroll_offset: u16,
    /// Viewport height (updated during render)
    pub viewport_height: u16,
    /// Y-positions of focusable elements (populated during render/data load)
    pub focusable_positions: Vec<u16>,
    /// IDs of focusable elements (for meaningful display when activating)
    pub focusable_ids: Vec<FocusableId>,
    /// Row positions for left/right navigation within Row elements
    pub focusable_row_positions: Vec<Option<RowPosition>>,
}

impl DocumentState {
    /// Get the number of focusable elements
    pub fn focusable_count(&self) -> usize {
        self.focusable_positions.len()
    }
}

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
    pub document_stack: Vec<DocumentStackEntry>,
    /// Whether focus is on content (true) or tab bar (false)
    pub content_focused: bool,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Scores,
            document_stack: Vec::new(),
            content_focused: false, // Start with tab bar focused
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentStackEntry {
    pub document: StackedDocument,
    /// Selected item index within the document (None = no selection)
    /// Used for navigating lists (players, games, etc.) within documents
    pub selected_index: Option<usize>,
    /// Scroll offset (lines from top) for document viewport
    pub scroll_offset: u16,
    /// Y-positions of focusable elements (for autoscroll)
    pub focusable_positions: Vec<u16>,
    /// Heights of focusable elements (for autoscroll)
    pub focusable_heights: Vec<u16>,
    /// Viewport height (for autoscroll calculations)
    pub viewport_height: u16,
}

impl DocumentStackEntry {
    /// Create a new document stack entry with default values
    pub fn new(document: StackedDocument) -> Self {
        Self {
            document,
            selected_index: Some(0),
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        }
    }

    /// Create a new document stack entry with a specific selected index
    pub fn with_selection(document: StackedDocument, selected_index: Option<usize>) -> Self {
        Self {
            document,
            selected_index,
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        }
    }
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
    pub settings: SettingsUiState,
}

/// UI state for Scores tab
///
/// PHASE 7 COMPLETE: Most state migrated to component-local state (ScoresTabState).
///
/// Remaining field needed for effects system:
/// - `game_date`: Maintained by RefreshSchedule action, read by RefreshData for timer-based refreshes
///
/// Component-local state (ScoresTabState) manages all UI state:
/// - Date navigation within the 5-date window (selected_date_index, game_date)
/// - Game selection UI (selected_game_index)
/// - Browse mode (whether user is navigating game boxes vs dates)
///
/// Note: game_date is duplicated between global and component state by design:
/// - Global: What schedule data is loaded (for effects system)
/// - Component: What date UI is viewing (for rendering)
/// These are kept in sync via RefreshSchedule action.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoresUiState {
    pub game_date: GameDate,
}

impl Default for ScoresUiState {
    fn default() -> Self {
        Self {
            game_date: GameDate::today(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SettingsUiState {
    pub selected_category: SettingsCategory,
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
    /// Cached terminal width for calculating game grid layout
    /// Updated during render, used by key handlers
    pub terminal_width: u16,
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
