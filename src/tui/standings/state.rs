use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;
use crate::tui::navigation::NavigationContext;
use super::layout::StandingsLayout;
use super::panel::StandingsPanel;

pub struct State {
    pub view: GroupBy,
    pub subtab_focused: bool,
    pub scrollable: Scrollable,
    pub team_selection_active: bool,
    pub selected_team_index: usize,
    pub selected_column: usize,
    pub layout_cache: Option<StandingsLayout>,
    pub navigation: Option<NavigationContext<StandingsPanel, String, ()>>,
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

impl crate::tui::context::NavigationContextProvider for State {
    fn get_available_actions(&self) -> Vec<crate::tui::widgets::Action> {
        let mut actions = vec![];

        if self.subtab_focused {
            actions.push(crate::tui::widgets::Action {
                key: "←→".to_string(),
                label: "Change View".to_string(),
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
                action: "Change View".to_string(),
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
                action: "Select View".to_string(),
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

impl crate::tui::context::BreadcrumbProvider for State {
    fn get_breadcrumb_items(&self) -> Vec<String> {
        let mut items = vec!["Standings".to_string()];

        items.push(match self.view {
            GroupBy::Division => "Division".to_string(),
            GroupBy::Conference => "Conference".to_string(),
            GroupBy::League => "League".to_string(),
            GroupBy::Wildcard => "Wild Card".to_string(),
        });

        items
    }
}
