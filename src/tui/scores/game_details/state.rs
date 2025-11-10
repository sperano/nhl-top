use crate::tui::common::scrollable::Scrollable;

/// Represents which section of the game details is currently selected
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayerSection {
    ScoringSummary(usize), // Index in scoring plays
    AwayForwards,
    AwayDefense,
    AwayGoalies,
    HomeForwards,
    HomeDefense,
    HomeGoalies,
}

impl PlayerSection {
    /// Get the next section in the navigation order
    pub fn next(&self) -> Self {
        match self {
            Self::ScoringSummary(_) => Self::AwayForwards,
            Self::AwayForwards => Self::AwayDefense,
            Self::AwayDefense => Self::AwayGoalies,
            Self::AwayGoalies => Self::HomeForwards,
            Self::HomeForwards => Self::HomeDefense,
            Self::HomeDefense => Self::HomeGoalies,
            Self::HomeGoalies => Self::ScoringSummary(0),
        }
    }

    /// Get the previous section in the navigation order
    pub fn prev(&self) -> Self {
        match self {
            Self::ScoringSummary(_) => Self::HomeGoalies,
            Self::AwayForwards => Self::ScoringSummary(0),
            Self::AwayDefense => Self::AwayForwards,
            Self::AwayGoalies => Self::AwayDefense,
            Self::HomeForwards => Self::AwayGoalies,
            Self::HomeDefense => Self::HomeForwards,
            Self::HomeGoalies => Self::HomeDefense,
        }
    }
}

/// State for game details navigation and player selection
pub struct GameDetailsState {
    /// Whether player selection mode is active
    pub player_selection_active: bool,
    /// Currently selected section
    pub selected_section: PlayerSection,
    /// Index within the current section
    pub selected_index: usize,
    /// Scrollable state for game details view
    pub scrollable: Scrollable,
}

impl GameDetailsState {
    pub fn new() -> Self {
        Self {
            player_selection_active: false,
            selected_section: PlayerSection::ScoringSummary(0),
            selected_index: 0,
            scrollable: Scrollable::new(),
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.player_selection_active = false;
        self.selected_section = PlayerSection::ScoringSummary(0);
        self.selected_index = 0;
        self.scrollable = Scrollable::new();
    }
}

impl Default for GameDetailsState {
    fn default() -> Self {
        Self::new()
    }
}
