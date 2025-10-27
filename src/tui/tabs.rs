use crate::commands::standings::GroupBy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Scores,
    Standings,
    Settings,
}

impl Tab {
    pub fn name(&self) -> &str {
        match self {
            Tab::Scores => "Scores",
            Tab::Standings => "Standings",
            Tab::Settings => "Settings",
        }
    }

    pub fn all() -> [Tab; 3] {
        [Tab::Scores, Tab::Standings, Tab::Settings]
    }
}

pub struct AppState {
    pub current_tab: Tab,
    pub standings_view: GroupBy,
    pub subtab_focused: bool,
    pub scores_selected_index: usize, // 0 = left, 1 = middle, 2 = right
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            current_tab: Tab::Scores,
            standings_view: GroupBy::Division,
            subtab_focused: false,
            scores_selected_index: 1, // Start with middle date selected
        }
    }
}
