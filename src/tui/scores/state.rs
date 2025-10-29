pub struct State {
    pub selected_index: usize, // 0 = left, 1 = middle, 2 = right
    pub subtab_focused: bool,
    pub box_selection_active: bool, // true when navigating game boxes
    pub selected_box: (usize, usize), // (row, col) of selected box
    pub grid_dimensions: (usize, usize), // (num_rows, num_cols) - updated during render
}

impl State {
    pub fn new() -> Self {
        State {
            selected_index: 1, // Start with middle date selected
            subtab_focused: false,
            box_selection_active: false,
            selected_box: (0, 0),
            grid_dimensions: (0, 0),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
