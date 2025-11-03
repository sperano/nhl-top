use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;
use crate::tui::navigation::NavigationContext;
use super::layout::StandingsLayout;
use super::panel::{StandingsPanel, PanelData};

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
    pub scrollable: Scrollable,
    pub team_selection_active: bool,
    pub selected_team_index: usize,
    pub selected_column: usize,
    pub layout_cache: Option<StandingsLayout>,
    pub navigation: Option<NavigationContext<StandingsPanel, String, PanelData>>,
    pub panel_scrollable: Scrollable,
    pub panel_selection_active: bool,
    pub panel_selected_index: usize,
}

impl State {
    pub fn new() -> Self {
        State {
            view: GroupBy::Wildcard,
            subtab_focused: false,
            scrollable: Scrollable::new(),
            team_selection_active: false,
            selected_team_index: 0,
            selected_column: 0,
            layout_cache: None,
            navigation: None,
            panel_scrollable: Scrollable::new(),
            panel_selection_active: false,
            panel_selected_index: 0,
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
