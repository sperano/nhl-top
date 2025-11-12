use crate::tui::widgets::Container;

pub struct State {
    pub container: Option<Container>,
}

impl State {
    pub fn new() -> Self {
        Self {
            container: None,
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
        vec![]
    }

    fn get_keyboard_hints(&self) -> Vec<crate::tui::widgets::KeyHint> {
        use crate::tui::widgets::{KeyHint, KeyHintStyle};
        vec![
            KeyHint {
                key: "ESC".to_string(),
                action: "Exit".to_string(),
                style: KeyHintStyle::Important,
            },
        ]
    }
}

impl crate::tui::context::BreadcrumbProvider for State {
    fn get_breadcrumb_items(&self) -> Vec<String> {
        vec!["Players".to_string()]
    }
}
