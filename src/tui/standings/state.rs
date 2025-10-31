use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
    pub scrollable: Scrollable,
}

impl State {
    pub fn new() -> Self {
        State {
            view: GroupBy::Division,
            subtab_focused: false,
            scrollable: Scrollable::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
