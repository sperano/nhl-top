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
    EnterContentFocus,  // Down key: move focus from tab bar to content
    ExitContentFocus,   // Up key: move focus from content back to tab bar
    EnterSubtabMode,    // Deprecated alias for EnterContentFocus
    ExitSubtabMode,     // Deprecated alias for ExitContentFocus
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

    // System actions
    Quit,
    Error(String),
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
    ModalMoveUp,    // Move selection up in modal
    ModalMoveDown,  // Move selection down in modal
    ModalConfirm,   // Confirm modal selection
    ModalCancel,    // Cancel modal without selecting
    CommitEdit(String), // Setting key to commit edit
    UpdateConfig(Box<crate::config::Config>),
}


impl Action {
    /// Returns true if this action should trigger a re-render
    pub fn should_render(&self) -> bool {
        !matches!(self, Self::Error(_))
    }
}
