use nhl_api::{Boxscore, DailySchedule, GameDate, GameMatchup, Standing};

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
    TeamRosterLoaded(String, Result<Roster, String>),
    PlayerStatsLoaded(i64, Result<PlayerStats, String>),

    // UI actions
    ScrollUp(usize),
    ScrollDown(usize),
    FocusNext,
    FocusPrevious,

    // Component-specific actions (nested)
    ScoresAction(ScoresAction),
    StandingsAction(StandingsAction),

    // System actions
    Quit,
    Error(String),
}

/// Tab enum for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Scores,
    Standings,
    Stats,
    Players,
    Settings,
    Browser,
}

/// Panel types for drill-down views
#[derive(Debug, Clone)]
pub enum Panel {
    Boxscore { game_id: i64 },
    TeamDetail { abbrev: String },
    PlayerDetail { player_id: i64 },
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
    EnterTeamMode,
    ExitTeamMode,
    MoveSelectionUp,
    MoveSelectionDown,
    MoveSelectionLeft,
    MoveSelectionRight,
}

// Placeholder types for future implementation
#[derive(Debug, Clone)]
pub struct Roster {
    pub team_abbrev: String,
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub player_id: i64,
}

impl Action {
    /// Returns true if this action should trigger a re-render
    pub fn should_render(&self) -> bool {
        !matches!(self, Self::Error(_))
    }
}
