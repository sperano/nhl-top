use crate::tui::common::scrollable::Scrollable;

pub struct State {
    pub selected_index: usize, // 0 = left, 1 = middle, 2 = right
    pub subtab_focused: bool,
    pub box_selection_active: bool, // true when navigating game boxes
    pub selected_box: (usize, usize), // (row, col) of selected box
    pub grid_dimensions: (usize, usize), // (num_rows, num_cols) - updated during render
    pub boxscore_view_active: bool, // true when boxscore view is displayed
    pub boxscore_scrollable: Scrollable, // scrollable for boxscore view
    pub grid_scrollable: Scrollable, // scrollable for game grid
}

impl State {
    pub fn new() -> Self {
        State {
            selected_index: 2, // Start with middle date selected (index 2 of 5 dates)
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
