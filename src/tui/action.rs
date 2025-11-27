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
    // SelectTeam(String),
    // SelectPlayer(i64),
    RefreshData,
    RefreshSchedule(GameDate),  // Refresh schedule for specific date

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
    UpdateTerminalWidth(u16),
}

/// Tab-specific actions for Scores
#[derive(Debug, Clone)]
pub enum ScoresAction {
    DateLeft,
    DateRight,
    EnterBoxSelection,
    ExitBoxSelection,
    SelectGame(i64),             // game_id
    MoveGameSelectionUp(u16),    // boxes_per_row
    MoveGameSelectionDown(u16),  // boxes_per_row
    MoveGameSelectionLeft,
    MoveGameSelectionRight,
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

// DocumentAction removed in Phase 10 - now handled by component messages
// (StandingsTabMsg::DocNav, DemoTabMessage::DocNav)

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
            Self::RefreshData => Self::RefreshData,
            Self::RefreshSchedule(date) => Self::RefreshSchedule(date.clone()),
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
            Self::UpdateTerminalWidth(width) => Self::UpdateTerminalWidth(*width),
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
