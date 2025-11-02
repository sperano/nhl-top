use crate::tui::common::scrollable::Scrollable;

/// Date window configuration
pub const DATE_WINDOW_SIZE: usize = 5;
pub const DATE_WINDOW_CENTER: usize = 2;
pub const DATE_WINDOW_MIN_INDEX: usize = 0;
pub const DATE_WINDOW_MAX_INDEX: usize = 4;

pub struct State {
    pub selected_index: usize,
    pub subtab_focused: bool,
    pub box_selection_active: bool,
    pub selected_box: (usize, usize),
    pub grid_dimensions: (usize, usize),
    pub boxscore_view_active: bool,
    pub boxscore_scrollable: Scrollable,
    pub grid_scrollable: Scrollable,
}

impl State {
    pub fn new() -> Self {
        State {
            selected_index: DATE_WINDOW_CENTER,
            subtab_focused: false,
            box_selection_active: false,
            selected_box: (0, 0),
            grid_dimensions: (0, 0),
            boxscore_view_active: false,
            boxscore_scrollable: Scrollable::new(),
            grid_scrollable: Scrollable::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
