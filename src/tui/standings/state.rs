use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;
use super::layout::StandingsLayout;

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
    pub scrollable: Scrollable,
    pub team_selection_active: bool,
    pub selected_team_index: usize,
    pub selected_column: usize, // 0 = left, 1 = right (for Division/Conference views)
    pub layout_cache: Option<StandingsLayout>, // Cached layout for current view
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
            layout_cache: None,
        }
    }

    /// Update the layout cache with new standings data
    /// Should be called whenever standings data changes or view changes
    pub fn update_layout(&mut self, standings: &[nhl_api::Standing], western_first: bool) {
        self.layout_cache = Some(StandingsLayout::build(standings, self.view, western_first));
    }

    /// Invalidate the layout cache (will be rebuilt on next render)
    pub fn invalidate_layout(&mut self) {
        self.layout_cache = None;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
