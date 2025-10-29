use crate::commands::standings::GroupBy;

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            view: GroupBy::Division,
            subtab_focused: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
