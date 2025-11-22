use nhl_api::{Boxscore, ClubStats, DailySchedule, GameDate, GameMatchup, PlayerLanding, Standing};

use super::types::{Panel, Tab};

/// Global actions - like Redux actions
///
/// All state changes in the application happen through actions.
/// Actions are dispatched from:
/// - User input (key events)
/// - Effects (async data loading)
/// - Middleware (logging, side effects)
#[derive(Debug, Clone)]
pub enum Action {
    // Navigation actions
    NavigateTab(Tab),
    NavigateTabLeft,
    NavigateTabRight,
    EnterContentFocus, // Down key: move focus from tab bar to content
    ExitContentFocus,  // Up key: move focus from content back to tab bar
    PushPanel(Panel),
    PopPanel,
    ToggleCommandPalette,

    // Data actions
    SetGameDate(GameDate),
    SelectTeam(String),
    SelectPlayer(i64),
    RefreshData,

    // Data loaded (from effects)
    StandingsLoaded(Result<Vec<Standing>, String>),
    ScheduleLoaded(Result<DailySchedule, String>),
    GameDetailsLoaded(i64, Result<GameMatchup, String>),
    BoxscoreLoaded(i64, Result<Boxscore, String>),
    TeamRosterStatsLoaded(String, Result<ClubStats, String>),
    PlayerStatsLoaded(i64, Result<PlayerLanding, String>),

    // UI actions
    ScrollUp(usize),
    ScrollDown(usize),
    FocusNext,
    FocusPrevious,

    // Panel-specific actions
    PanelSelectNext,     // Move selection down in current panel
    PanelSelectPrevious, // Move selection up in current panel
    PanelSelectItem,     // Activate/enter selected item in panel

    // Component-specific actions (nested)
    ScoresAction(ScoresAction),
    StandingsAction(StandingsAction),
    SettingsAction(SettingsAction),
    DocumentAction(DocumentAction),

    // System actions
    Quit,
    Error(String),
    SetStatusMessage { message: String, is_error: bool },
}

/// Tab-specific actions for Scores
#[derive(Debug, Clone)]
pub enum ScoresAction {
    DateLeft,
    DateRight,
    EnterBoxSelection,
    ExitBoxSelection,
    SelectGame,
    SelectGameById(i64),
    MoveGameSelectionUp,
    MoveGameSelectionDown,
    MoveGameSelectionLeft,
    MoveGameSelectionRight,
    UpdateBoxesPerRow(u16), // Update grid layout for navigation
}

/// Tab-specific actions for Standings
#[derive(Debug, Clone)]
pub enum StandingsAction {
    CycleView,
    CycleViewLeft,
    CycleViewRight,
    SelectTeam,
    SelectTeamByPosition(usize, usize), // column, row
    EnterBrowseMode,
    ExitBrowseMode,
    MoveSelectionUp,
    MoveSelectionDown,
    MoveSelectionLeft,
    MoveSelectionRight,
    PageDown,
    PageUp,
    GoToTop,
    GoToBottom,
    UpdateViewportHeight(usize), // Update actual visible height from renderer
}

/// Tab-specific actions for Settings
#[derive(Debug, Clone)]
pub enum SettingsAction {
    NavigateCategoryLeft,
    NavigateCategoryRight,
    EnterSettingsMode,
    ExitSettingsMode,
    MoveSelectionUp,
    MoveSelectionDown,
    ToggleBoolean(String), // Setting key to toggle
    StartEditing(String),  // Setting key to start editing (opens modal for list settings)
    CancelEditing,
    AppendChar(char),
    DeleteChar,
    // Modal navigation actions
    ModalMoveUp,        // Move selection up in modal
    ModalMoveDown,      // Move selection down in modal
    ModalConfirm,       // Confirm modal selection
    ModalCancel,        // Cancel modal without selecting
    CommitEdit(String), // Setting key to commit edit
    UpdateConfig(Box<crate::config::Config>),
}

/// Document-related actions for viewport scrolling and focus navigation
#[derive(Debug, Clone)]
pub enum DocumentAction {
    /// Navigate to next focusable element (Tab key)
    FocusNext,
    /// Navigate to previous focusable element (Shift-Tab key)
    FocusPrev,
    /// Activate the currently focused element (Enter key)
    ActivateFocused,
    /// Scroll viewport up by N lines
    ScrollUp(u16),
    /// Scroll viewport down by N lines
    ScrollDown(u16),
    /// Scroll to top of document
    ScrollToTop,
    /// Scroll to bottom of document
    ScrollToBottom,
    /// Page up (scroll by viewport height)
    PageUp,
    /// Page down (scroll by viewport height)
    PageDown,
    /// Sync focusable element positions from component (for accurate autoscrolling)
    /// Parameters: (positions, viewport_height)
    SyncFocusablePositions(Vec<u16>, u16),
}

impl Action {
    /// Returns true if this action should trigger a re-render
    pub fn should_render(&self) -> bool {
        !matches!(self, Self::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_render_returns_true_for_most_actions() {
        assert!(Action::NavigateTabLeft.should_render());
        assert!(Action::NavigateTabRight.should_render());
        assert!(Action::EnterContentFocus.should_render());
        assert!(Action::ExitContentFocus.should_render());
        assert!(Action::RefreshData.should_render());
        assert!(Action::Quit.should_render());
        assert!(Action::ToggleCommandPalette.should_render());
        assert!(Action::PopPanel.should_render());
        assert!(Action::ScrollUp(5).should_render());
        assert!(Action::ScrollDown(10).should_render());
        assert!(Action::FocusNext.should_render());
        assert!(Action::FocusPrevious.should_render());
    }

    #[test]
    fn test_should_render_returns_false_for_error_actions() {
        assert!(!Action::Error("test error".to_string()).should_render());
        assert!(!Action::Error("another error".to_string()).should_render());
        assert!(!Action::Error(String::new()).should_render());
    }
}
