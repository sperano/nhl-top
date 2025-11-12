use crate::tui::widgets::Container;
use crate::tui::widgets::focus::Focusable;
use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;
use crate::tui::navigation::NavigationContext;
use crate::tui::common::CommonPanel;

/// Panel-specific state for TeamDetail view
#[derive(Debug, Clone)]
pub struct TeamDetailState {
    pub selected_player_index: usize,
    pub selection_active: bool,
    pub scrollable: Scrollable,
}

impl TeamDetailState {
    pub fn new() -> Self {
        Self {
            selected_player_index: 0,
            selection_active: false,
            scrollable: Scrollable::new(),
        }
    }
}

/// Panel-specific state for PlayerDetail view
#[derive(Debug, Clone)]
pub struct PlayerDetailState {
    pub selected_season_index: usize,
    pub selection_active: bool,
    pub scrollable: Scrollable,
}

impl PlayerDetailState {
    pub fn new() -> Self {
        Self {
            selected_season_index: 0,
            selection_active: false,
            scrollable: Scrollable::new(),
        }
    }
}

/// Enum to hold panel-specific state
#[derive(Debug, Clone)]
pub enum PanelState {
    TeamDetail(TeamDetailState),
    PlayerDetail(PlayerDetailState),
}


pub struct State {
    pub container: Option<Container>,
    pub subtab_focused: bool,

    // Current standings view (Division, Conference, League, Wildcard)
    pub view: GroupBy,

    // Scrollable for panel views (TeamDetail, PlayerDetail)
    pub scrollable: Scrollable,

    // Navigation context for panel drill-down (team details, player details)
    // Uses PanelState to store panel-specific state (selection, scrolling, etc.)
    pub navigation: NavigationContext<CommonPanel, String, PanelState>,

    // FocusableTable widgets for each column (1 for League, 2 for others)
    pub team_tables: Vec<Box<dyn Focusable>>,
    // Index of the currently focused table (None if in view selection mode)
    pub focused_table_index: Option<usize>,
}

impl State {
    pub fn new() -> Self {
        Self {
            container: None,
            subtab_focused: false,
            view: GroupBy::Wildcard,
            scrollable: Scrollable::new(),
            navigation: NavigationContext::new(),
            team_tables: Vec::new(),
            focused_table_index: None,
        }
    }

    // === OLD IMPLEMENTATION - COMMENTED FOR REFERENCE ===
    // pub fn new() -> Self {
    //     State {
    //         view: GroupBy::Wildcard,
    //         subtab_focused: false,
    //         scrollable: Scrollable::new(),
    //         team_selection_active: false,
    //         selected_team_index: 0,
    //         selected_column: 0,
    //         layout_cache: None,
    //         container: None,
    //         navigation: None,
    //         panel_scrollable: Scrollable::new(),
    //         panel_selection_active: false,
    //         panel_selected_index: 0,
    //     }
    // }
    //
    // /// Update the layout cache with new standings data
    // /// Should be called whenever standings data changes or view changes
    // pub fn update_layout(&mut self, standings: &[nhl_api::Standing], western_first: bool) {
    //     self.layout_cache = Some(StandingsLayout::build(standings, self.view, western_first));
    // }
    //
    // /// Invalidate the layout cache (will be rebuilt on next render)
    // pub fn invalidate_layout(&mut self) {
    //     self.layout_cache = None;
    // }
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

        // Add view name
        items.push(match self.view {
            GroupBy::Division => "Division".to_string(),
            GroupBy::Conference => "Conference".to_string(),
            GroupBy::League => "League".to_string(),
            GroupBy::Wildcard => "Wild Card".to_string(),
        });

        // Add navigation trail if present
        if !self.navigation.is_at_root() {
            items.extend(self.navigation.stack.breadcrumb_trail());
        }

        items
    }
}
