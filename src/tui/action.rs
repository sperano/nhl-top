use nhl_api::{Boxscore, ClubStats, DailySchedule, GameDate, GameMatchup, PlayerLanding, Standing};
use std::any::Any;

use super::component::Effect;
use super::types::{Panel, Tab};

/// Trait for type-erased component messages
///
/// This allows messages to be dispatched to components without knowing
/// their concrete type at the call site. The message carries its own
/// logic for updating component state.
pub trait ComponentMessageTrait: Send + Sync + std::fmt::Debug {
    /// Apply this message to a component state, returning an effect
    fn apply(&self, state: &mut dyn Any) -> Effect;

    /// Clone this message into a Box (for cloning Action enum)
    fn clone_box(&self) -> Box<dyn ComponentMessageTrait>;
}

/// Global actions - like Redux actions
///
/// All state changes in the application happen through actions.
/// Actions are dispatched from:
/// - User input (key events)
/// - Effects (async data loading)
/// - Middleware (logging, side effects)
#[derive(Debug)]
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
    // SelectTeam(String),
    // SelectPlayer(i64),
    RefreshData,

    // Data loaded (from effects)
    StandingsLoaded(Result<Vec<Standing>, String>),
    ScheduleLoaded(Result<DailySchedule, String>),
    GameDetailsLoaded(i64, Result<GameMatchup, String>),
    BoxscoreLoaded(i64, Result<Boxscore, String>),
    TeamRosterStatsLoaded(String, Result<ClubStats, String>),
    PlayerStatsLoaded(i64, Result<PlayerLanding, String>),

    // UI actions
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

    /// Dispatch a message to a specific component
    ///
    /// This is part of the React-like component system refactor.
    /// Instead of global actions for every UI interaction, components
    /// can handle their own messages for local state updates.
    ComponentMessage {
        /// Component path (e.g., "app/scores_tab")
        path: String,
        /// Type-erased message to dispatch
        message: Box<dyn ComponentMessageTrait>,
    },

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
    CycleViewLeft,
    CycleViewRight,
    EnterBrowseMode,
    ExitBrowseMode,
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
    /// Navigate to next focusable element (Tab/Down key)
    FocusNext,
    /// Navigate to previous focusable element (Shift-Tab/Up key)
    FocusPrev,
    /// Navigate to corresponding element in left sibling within a Row (Left key)
    FocusLeft,
    /// Navigate to corresponding element in right sibling within a Row (Right key)
    FocusRight,
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
    /// Update viewport heights from terminal (dispatched on resize/render)
    /// This is critical for correct autoscroll calculations
    /// Parameters: (demo_viewport_height, standings_viewport_height)
    UpdateViewportHeight { demo: u16, standings: u16 },
    /// Sync focusable element positions from component (for accurate autoscrolling)
    /// Parameters: (positions, viewport_height)
    SyncFocusablePositions(Vec<u16>, u16),
}

impl Clone for Action {
    fn clone(&self) -> Self {
        match self {
            Self::NavigateTab(tab) => Self::NavigateTab(*tab),
            Self::NavigateTabLeft => Self::NavigateTabLeft,
            Self::NavigateTabRight => Self::NavigateTabRight,
            Self::EnterContentFocus => Self::EnterContentFocus,
            Self::ExitContentFocus => Self::ExitContentFocus,
            Self::PushPanel(panel) => Self::PushPanel(panel.clone()),
            Self::PopPanel => Self::PopPanel,
            Self::ToggleCommandPalette => Self::ToggleCommandPalette,
            Self::SetGameDate(date) => Self::SetGameDate(date.clone()),
            Self::RefreshData => Self::RefreshData,
            Self::StandingsLoaded(result) => Self::StandingsLoaded(result.clone()),
            Self::ScheduleLoaded(result) => Self::ScheduleLoaded(result.clone()),
            Self::GameDetailsLoaded(id, result) => Self::GameDetailsLoaded(*id, result.clone()),
            Self::BoxscoreLoaded(id, result) => Self::BoxscoreLoaded(*id, result.clone()),
            Self::TeamRosterStatsLoaded(abbrev, result) => {
                Self::TeamRosterStatsLoaded(abbrev.clone(), result.clone())
            }
            Self::PlayerStatsLoaded(id, result) => Self::PlayerStatsLoaded(*id, result.clone()),
            Self::FocusNext => Self::FocusNext,
            Self::FocusPrevious => Self::FocusPrevious,
            Self::PanelSelectNext => Self::PanelSelectNext,
            Self::PanelSelectPrevious => Self::PanelSelectPrevious,
            Self::PanelSelectItem => Self::PanelSelectItem,
            Self::ScoresAction(action) => Self::ScoresAction(action.clone()),
            Self::StandingsAction(action) => Self::StandingsAction(action.clone()),
            Self::SettingsAction(action) => Self::SettingsAction(action.clone()),
            Self::DocumentAction(action) => Self::DocumentAction(action.clone()),
            Self::ComponentMessage { path, message } => Self::ComponentMessage {
                path: path.clone(),
                message: message.clone_box(),
            },
            Self::Quit => Self::Quit,
            Self::Error(msg) => Self::Error(msg.clone()),
            Self::SetStatusMessage { message, is_error } => Self::SetStatusMessage {
                message: message.clone(),
                is_error: *is_error,
            },
        }
    }
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
