pub struct State {
    /// Index of currently selected setting in the settings list
    pub selected_setting_index: usize,
    /// Whether subtab mode is focused (for main TUI navigation)
    pub subtab_focused: bool,
    /// Editing state: Some((setting_name, edit_buffer)) when editing a string/int
    pub editing: Option<(String, String)>,
    /// List modal state: Some((setting_name, options, selected_index)) when showing dropdown
    pub list_modal: Option<(String, Vec<String>, usize)>,
    /// Color picker modal state: Some(setting_name) when showing color picker
    pub color_modal: Option<String>,
    /// Selected color index in the color picker (0-23 for 4x6 grid)
    pub selected_color_index: usize,
}

impl State {
    pub fn new() -> Self {
        State {
            selected_setting_index: 0,
            subtab_focused: false,
            editing: None,
            list_modal: None,
            color_modal: None,
            selected_color_index: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
