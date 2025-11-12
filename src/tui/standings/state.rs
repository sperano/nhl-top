use crate::tui::widgets::Container;
use crate::commands::standings::GroupBy;
use crate::tui::common::scrollable::Scrollable;
use crate::tui::navigation::NavigationContext;
use super::panel::StandingsPanel;
use std::collections::HashMap;

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

/// Selection state for a specific view (column + team index)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewSelection {
    pub column: usize,
    pub team_index: usize,
}

impl ViewSelection {
    pub fn new(column: usize, team_index: usize) -> Self {
        Self { column, team_index }
    }

    pub fn default() -> Self {
        Self { column: 0, team_index: 0 }
    }
}

pub struct State {
    pub container: Option<Container>,
    pub subtab_focused: bool,

    // Active fields for current implementation
    pub view: GroupBy,
    pub team_selection_active: bool,
    pub selected_team_index: usize,
    pub selected_column: usize,
    pub scrollable: Scrollable,

    // Per-view selection memory
    // Remembers which team was selected in each view when you switch between them
    pub view_selections: HashMap<GroupBy, ViewSelection>,

    // Navigation context for panel drill-down (team details, player details)
    // Uses PanelState to store panel-specific state (selection, scrolling, etc.)
    pub navigation: NavigationContext<StandingsPanel, String, PanelState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            container: None,
            subtab_focused: false,
            view: GroupBy::Wildcard,
            team_selection_active: false,
            selected_team_index: 0,
            selected_column: 0,
            scrollable: Scrollable::new(),
            view_selections: HashMap::new(),
            navigation: NavigationContext::new(),
        }
    }

    /// Save current selection for the current view
    pub fn save_current_selection(&mut self) {
        let selection = ViewSelection::new(self.selected_column, self.selected_team_index);
        self.view_selections.insert(self.view, selection);
    }

    /// Restore selection for the current view, or use default if not saved
    /// Returns true if a saved selection was restored
    pub fn restore_selection_for_view(&mut self) -> bool {
        if let Some(selection) = self.view_selections.get(&self.view).copied() {
            self.selected_column = selection.column;
            self.selected_team_index = selection.team_index;
            true
        } else {
            // No saved selection, use default
            self.selected_column = 0;
            self.selected_team_index = 0;
            false
        }
    }

    /// Validate and clamp selection to ensure it's within bounds for the given layout
    /// Call this after restoring selection to ensure it's still valid
    pub fn validate_selection(&mut self, max_columns: usize, team_counts: &[usize]) {
        // Clamp column to valid range
        if self.selected_column >= max_columns {
            self.selected_column = if max_columns > 0 { max_columns - 1 } else { 0 };
        }

        // Clamp team index to valid range for current column
        if let Some(&team_count) = team_counts.get(self.selected_column) {
            if self.selected_team_index >= team_count && team_count > 0 {
                self.selected_team_index = team_count - 1;
            } else if team_count == 0 {
                self.selected_team_index = 0;
            }
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
