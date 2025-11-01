use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
    pub scrollable: Scrollable,
    pub team_selection_active: bool,
    pub selected_team_index: usize,
    pub selected_column: usize, // 0 = left, 1 = right (for Division/Conference views)
}

impl State {
    pub fn new() -> Self {
        State {
            view: GroupBy::Division,
            subtab_focused: false,
            scrollable: Scrollable::new(),
            team_selection_active: false,
            selected_team_index: 0,
            selected_column: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
