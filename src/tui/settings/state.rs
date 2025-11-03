pub struct State {
    /// Index of currently selected color in the 4x4 grid (0-15)
    pub selected_color_index: usize,
    /// Whether subtab mode is focused (for main TUI navigation)
    pub subtab_focused: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            selected_color_index: 0,
            subtab_focused: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
