use crate::tui::common::scrollable::Scrollable;
use crate::tui::scores::game_details::GameDetailsState;
use crate::tui::scores::panel::ScoresPanel;
use crate::tui::navigation::NavigationContext;

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
    pub game_details: GameDetailsState,
    pub navigation: Option<NavigationContext<ScoresPanel, String, ()>>,
    pub panel_scrollable: Scrollable,
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
            game_details: GameDetailsState::new(),
            navigation: None,
            panel_scrollable: Scrollable::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::tui::context::NavigationContextProvider for State {
    fn get_available_actions(&self) -> Vec<crate::tui::widgets::Action> {
        let mut actions = vec![];

        if self.subtab_focused {
            actions.push(crate::tui::widgets::Action {
                key: "←→".to_string(),
                label: "Change Date".to_string(),
                enabled: true,
            });
        }

        actions
    }

    fn get_keyboard_hints(&self) -> Vec<crate::tui::widgets::KeyHint> {
        use crate::tui::widgets::{KeyHint, KeyHintStyle};
        let mut hints = vec![];

        if self.subtab_focused {
            hints.push(KeyHint {
                key: "←→".to_string(),
                action: "Change Date".to_string(),
                style: KeyHintStyle::Important,
            });
            hints.push(KeyHint {
                key: "↑".to_string(),
                action: "Back".to_string(),
                style: KeyHintStyle::Normal,
            });
        } else {
            hints.push(KeyHint {
                key: "↓".to_string(),
                action: "Select Date".to_string(),
                style: KeyHintStyle::Important,
            });
        }

        hints.push(KeyHint {
            key: "ESC".to_string(),
            action: "Exit".to_string(),
            style: KeyHintStyle::Subtle,
        });

        hints
    }
}
