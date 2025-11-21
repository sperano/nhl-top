use tracing::debug;

use crate::commands::standings::GroupBy;
use crate::tui::action::StandingsAction;
use crate::tui::component::Effect;
use crate::tui::state::{AppState, PanelState};
use crate::tui::types::Panel;

use super::standings_layout::{
    build_standings_layout, count_teams_in_conference_column, count_teams_in_division_column,
    count_teams_in_wildcard_column,
};

/// Page size for PageUp/PageDown navigation
const PAGE_SIZE: usize = 10;

/// Sub-reducer for standings tab
pub fn reduce_standings(state: AppState, action: StandingsAction) -> (AppState, Effect) {
    match action {
        StandingsAction::CycleView => handle_cycle_view(state),
        StandingsAction::CycleViewLeft => handle_cycle_view_left(state),
        StandingsAction::CycleViewRight => handle_cycle_view_right(state),
        StandingsAction::EnterBrowseMode => handle_enter_browse_mode(state),
        StandingsAction::ExitBrowseMode => handle_exit_browse_mode(state),
        StandingsAction::SelectTeam => handle_select_team(state),
        StandingsAction::SelectTeamByPosition(column, row) => {
            handle_select_team_by_position(state, column, row)
        }
        StandingsAction::MoveSelectionUp => handle_move_selection_up(state),
        StandingsAction::MoveSelectionDown => handle_move_selection_down(state),
        StandingsAction::MoveSelectionLeft => handle_move_selection_left(state),
        StandingsAction::MoveSelectionRight => handle_move_selection_right(state),
        StandingsAction::PageDown => handle_page_down(state),
        StandingsAction::PageUp => handle_page_up(state),
        StandingsAction::GoToTop => handle_go_to_top(state),
        StandingsAction::GoToBottom => handle_go_to_bottom(state),
        StandingsAction::UpdateViewportHeight(height) => {
            let mut new_state = state;
            new_state.ui.standings.viewport_height = height;
            (new_state, Effect::None)
        }
    }
}

fn handle_cycle_view(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = new_state.ui.standings.view.next();

    // Rebuild layout cache when view changes
    rebuild_standings_layout_cache(&mut new_state);

    // Reset selection when changing views
    reset_standings_selection(&mut new_state);
    (new_state, Effect::None)
}

fn handle_cycle_view_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = new_state.ui.standings.view.prev();

    // Rebuild layout cache when view changes
    rebuild_standings_layout_cache(&mut new_state);

    // Reset selection when changing views
    reset_standings_selection(&mut new_state);
    (new_state, Effect::None)
}

fn handle_cycle_view_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = new_state.ui.standings.view.next();

    // Rebuild layout cache when view changes
    rebuild_standings_layout_cache(&mut new_state);

    // Reset selection when changing views
    reset_standings_selection(&mut new_state);
    (new_state, Effect::None)
}

fn handle_enter_browse_mode(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.browse_mode = true;
    (new_state, Effect::None)
}

fn handle_exit_browse_mode(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.browse_mode = false;
    (new_state, Effect::None)
}

fn handle_select_team(state: AppState) -> (AppState, Effect) {
    // Use cached layout instead of rebuilding
    // Extract team_abbrev from cached layout before moving state
    let team_abbrev_opt = state
        .ui
        .standings
        .layout
        .get(state.ui.standings.selected_column)
        .and_then(|col| col.get(state.ui.standings.selected_row))
        .cloned();

    if let Some(team_abbrev) = team_abbrev_opt {
        debug!(
            "STANDINGS: Selected team: {} (row={}, col={})",
            team_abbrev, state.ui.standings.selected_row, state.ui.standings.selected_column
        );

        // Push TeamDetail panel onto navigation stack
        let panel = Panel::TeamDetail {
            abbrev: team_abbrev.clone(),
        };

        let mut new_state = state;
        new_state.navigation.panel_stack.push(PanelState {
            panel,
            scroll_offset: 0,
            selected_index: Some(0), // Start with first player selected
        });

        debug!("STANDINGS: Pushed TeamDetail panel for {}", team_abbrev);

        return (new_state, Effect::None);
    } else {
        debug!(
            "STANDINGS: No team at position (row={}, col={})",
            state.ui.standings.selected_row, state.ui.standings.selected_column
        );
    }

    (state, Effect::None)
}

fn handle_select_team_by_position(
    state: AppState,
    column: usize,
    row: usize,
) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.selected_column = column;
    new_state.ui.standings.selected_row = row;
    (new_state, Effect::None)
}

fn handle_move_selection_up(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Get team count (respects column in Conference/Division/Wildcard views)
    if let Some(standings) = new_state.data.standings.as_ref().as_ref() {
        let team_count = get_team_count_for_column(
            standings,
            new_state.ui.standings.view,
            new_state.ui.standings.selected_column,
            new_state.system.config.display_standings_western_first,
        );

        if team_count > 0 {
            let max_row = team_count - 1;
            if new_state.ui.standings.selected_row == 0 {
                // At first team - wrap to last team
                new_state.ui.standings.selected_row = max_row;
                // Ensure selection stays visible (will auto-scroll to show last team)
                ensure_selection_visible(&mut new_state);
                debug!("STANDINGS: Wrapped to bottom");
            } else {
                new_state.ui.standings.selected_row -= 1;
                // Ensure selection stays visible
                ensure_selection_visible(&mut new_state);
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_move_selection_down(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Get team count (respects column in Conference/Division/Wildcard views)
    if let Some(standings) = new_state.data.standings.as_ref().as_ref() {
        let team_count = get_team_count_for_column(
            standings,
            new_state.ui.standings.view,
            new_state.ui.standings.selected_column,
            new_state.system.config.display_standings_western_first,
        );

        if team_count > 0 {
            let max_row = team_count - 1;
            if new_state.ui.standings.selected_row >= max_row {
                // At last team - wrap to first team
                new_state.ui.standings.selected_row = 0;
                new_state.ui.standings.scroll_offset = 0;
                debug!("STANDINGS: Wrapped to top, reset scroll_offset to 0");
            } else {
                new_state.ui.standings.selected_row += 1;
                // Ensure selection stays visible
                ensure_selection_visible(&mut new_state);
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_move_selection_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Conference, Division, and Wildcard views have 2 columns for navigation
    if matches!(
        new_state.ui.standings.view,
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard
    ) {
        // Wrap around: 0 -> 1
        new_state.ui.standings.selected_column = if new_state.ui.standings.selected_column == 0 {
            1
        } else {
            0
        };

        // Clamp row to max teams in new column if needed
        clamp_row_to_column_bounds(&mut new_state);
    }

    (new_state, Effect::None)
}

fn handle_move_selection_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Conference, Division, and Wildcard views have 2 columns for navigation
    if matches!(
        new_state.ui.standings.view,
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard
    ) {
        // Wrap around: 1 -> 0
        new_state.ui.standings.selected_column = if new_state.ui.standings.selected_column == 1 {
            0
        } else {
            1
        };

        // Clamp row to max teams in new column if needed
        clamp_row_to_column_bounds(&mut new_state);
    }

    (new_state, Effect::None)
}

fn handle_page_down(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    let team_count = get_team_count(&new_state);

    if team_count > 0 {
        let max_row = team_count - 1;
        let new_row = (new_state.ui.standings.selected_row + PAGE_SIZE).min(max_row);
        new_state.ui.standings.selected_row = new_row;
        ensure_selection_visible(&mut new_state);
        debug!("STANDINGS: PageDown - moved to row {}", new_row);
    }

    (new_state, Effect::None)
}

fn handle_page_up(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    let team_count = get_team_count(&new_state);

    if team_count > 0 {
        let new_row = new_state
            .ui
            .standings
            .selected_row
            .saturating_sub(PAGE_SIZE);
        new_state.ui.standings.selected_row = new_row;
        ensure_selection_visible(&mut new_state);
        debug!("STANDINGS: PageUp - moved to row {}", new_row);
    }

    (new_state, Effect::None)
}

fn handle_go_to_top(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.selected_row = 0;
    new_state.ui.standings.scroll_offset = 0;
    debug!("STANDINGS: GoToTop - row 0, scroll 0");

    (new_state, Effect::None)
}

fn handle_go_to_bottom(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    let team_count = get_team_count(&new_state);

    if team_count > 0 {
        let last_row = team_count - 1;
        new_state.ui.standings.selected_row = last_row;
        new_state.ui.standings.scroll_offset = 0;
        debug!("STANDINGS: GoToBottom - row {}, scroll 0", last_row);
    }

    (new_state, Effect::None)
}

// Helper functions

/// Ensure the selected team is visible by adjusting scroll_offset
///
/// This function implements auto-scroll logic:
/// - If selection is above the visible window, scroll up
/// - If selection is below the visible window, scroll down
/// - Uses actual viewport_height from rendering for pixel-perfect scrolling
fn ensure_selection_visible(state: &mut AppState) {
    let selected = state.ui.standings.selected_row;
    let scroll = state.ui.standings.scroll_offset;
    let viewport_height = state.ui.standings.viewport_height;

    // If selection is above visible window, scroll up
    if selected < scroll {
        state.ui.standings.scroll_offset = selected;
        debug!(
            "STANDINGS: Auto-scroll UP to keep row {} visible (scroll_offset: {} -> {})",
            selected, scroll, state.ui.standings.scroll_offset
        );
        return;
    }

    // If selection is below visible window, scroll down
    let visible_end = scroll + viewport_height;
    if selected >= visible_end {
        let new_scroll = selected.saturating_sub(viewport_height - 1);
        debug!(
            "STANDINGS: Auto-scroll DOWN to keep row {} visible (scroll_offset: {} -> {}, viewport_height: {})",
            selected, scroll, new_scroll, viewport_height
        );
        state.ui.standings.scroll_offset = new_scroll;
    }
}

/// Helper to get total team count for current view
fn get_team_count(state: &AppState) -> usize {
    if let Some(standings) = state.data.standings.as_ref().as_ref() {
        get_team_count_for_column(
            standings,
            state.ui.standings.view,
            state.ui.standings.selected_column,
            state.system.config.display_standings_western_first,
        )
    } else {
        0
    }
}

/// Rebuild the standings layout cache from current standings data
fn rebuild_standings_layout_cache(state: &mut AppState) {
    if let Some(standings) = state.data.standings.as_ref().as_ref() {
        state.ui.standings.layout = build_standings_layout(
            standings,
            state.ui.standings.view,
            state.system.config.display_standings_western_first,
        );
    }
}

fn reset_standings_selection(state: &mut AppState) {
    state.ui.standings.selected_column = 0;
    state.ui.standings.selected_row = 0;
    state.ui.standings.scroll_offset = 0;
}

fn get_team_count_for_column(
    standings: &[nhl_api::Standing],
    view: GroupBy,
    column: usize,
    western_first: bool,
) -> usize {
    match view {
        GroupBy::Conference => count_teams_in_conference_column(standings, column),
        GroupBy::Division => count_teams_in_division_column(standings, column, western_first),
        GroupBy::Wildcard => count_teams_in_wildcard_column(standings, column, western_first),
        _ => standings.len(),
    }
}

fn clamp_row_to_column_bounds(state: &mut AppState) {
    if let Some(standings) = state.data.standings.as_ref().as_ref() {
        let team_count = get_team_count_for_column(
            standings,
            state.ui.standings.view,
            state.ui.standings.selected_column,
            state.system.config.display_standings_western_first,
        );

        if state.ui.standings.selected_row >= team_count && team_count > 0 {
            state.ui.standings.selected_row = team_count - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_cycle_view_wildcard_to_division() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;

        let (new_state, effect) = reduce_standings(state, StandingsAction::CycleView);

        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_division_to_conference() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleView);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_cycle_view_conference_to_league() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleView);

        assert_eq!(new_state.ui.standings.view, GroupBy::League);
    }

    #[test]
    fn test_cycle_view_league_to_wildcard() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleView);

        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_cycle_view_left_wildcard_to_league() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;

        let (new_state, effect) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::League);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_left_division_to_wildcard() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_cycle_view_left_conference_to_division() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
    }

    #[test]
    fn test_cycle_view_left_league_to_conference() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_cycle_view_right_wildcard_to_division() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;

        let (new_state, effect) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_right_division_to_conference() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_cycle_view_right_conference_to_league() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::League);
    }

    #[test]
    fn test_cycle_view_right_league_to_wildcard() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_enter_browse_mode() {
        let mut state = AppState::default();
        state.ui.standings.browse_mode = false;

        let (new_state, effect) = reduce_standings(state, StandingsAction::EnterBrowseMode);

        assert!(new_state.ui.standings.browse_mode);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_exit_browse_mode() {
        let mut state = AppState::default();
        state.ui.standings.browse_mode = true;

        let (new_state, effect) = reduce_standings(state, StandingsAction::ExitBrowseMode);

        assert!(!new_state.ui.standings.browse_mode);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_select_team_with_no_standings() {
        let state = AppState::default();

        let (new_state, effect) = reduce_standings(state, StandingsAction::SelectTeam);

        assert_eq!(new_state.navigation.panel_stack.len(), 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_select_team_by_position() {
        let state = AppState::default();

        let (new_state, effect) =
            reduce_standings(state, StandingsAction::SelectTeamByPosition(1, 5));

        assert_eq!(new_state.ui.standings.selected_column, 1);
        assert_eq!(new_state.ui.standings.selected_row, 5);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_up_with_no_standings() {
        let state = AppState::default();
        let initial_row = state.ui.standings.selected_row;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should not change without standings data
        assert_eq!(new_state.ui.standings.selected_row, initial_row);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_down_with_no_standings() {
        let state = AppState::default();
        let initial_row = state.ui.standings.selected_row;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should not change without standings data
        assert_eq!(new_state.ui.standings.selected_row, initial_row);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_left_in_conference_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 0;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        assert_eq!(new_state.ui.standings.selected_column, 1); // Wrapped to column 1
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_left_in_division_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 1;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        assert_eq!(new_state.ui.standings.selected_column, 0); // Moved to column 0
    }

    #[test]
    fn test_move_selection_left_in_wildcard_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;
        state.ui.standings.selected_column = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        assert_eq!(new_state.ui.standings.selected_column, 1); // Wrapped
    }

    #[test]
    fn test_move_selection_left_in_league_view_does_nothing() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_column = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        assert_eq!(new_state.ui.standings.selected_column, 0); // No change in League view
    }

    #[test]
    fn test_move_selection_right_in_conference_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 1;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        assert_eq!(new_state.ui.standings.selected_column, 0); // Wrapped to column 0
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_right_in_division_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        assert_eq!(new_state.ui.standings.selected_column, 1); // Moved to column 1
    }

    #[test]
    fn test_move_selection_right_in_wildcard_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;
        state.ui.standings.selected_column = 1;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        assert_eq!(new_state.ui.standings.selected_column, 0); // Wrapped
    }

    #[test]
    fn test_move_selection_right_in_league_view_does_nothing() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_column = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        assert_eq!(new_state.ui.standings.selected_column, 0); // No change in League view
    }

    #[test]
    fn test_cycle_view_resets_selection() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 1;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleView);

        // Selection should be reset
        assert_eq!(new_state.ui.standings.selected_column, 0);
        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert_eq!(new_state.ui.standings.scroll_offset, 0);
    }

    #[test]
    fn test_cycle_view_left_resets_selection() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 1;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        // Selection should be reset
        assert_eq!(new_state.ui.standings.selected_column, 0);
        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert_eq!(new_state.ui.standings.scroll_offset, 0);
        // View should change
        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_cycle_view_right_resets_selection() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 1;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        // Selection should be reset
        assert_eq!(new_state.ui.standings.selected_column, 0);
        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert_eq!(new_state.ui.standings.scroll_offset, 0);
        // View should change
        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    // Tests with actual standings data
    #[test]
    fn test_select_team_with_standings() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        let standings = create_test_standings();
        state.data.standings = Arc::new(Some(standings.clone()));
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 0;
        state.ui.standings.selected_row = 0;

        // Build layout cache (normally done when standings are loaded or view changes)
        state.ui.standings.layout = build_standings_layout(
            &standings,
            state.ui.standings.view,
            false, // western_first = false
        );

        let (new_state, effect) = reduce_standings(state, StandingsAction::SelectTeam);

        // Should push TeamDetail panel for first team (FLA in Atlantic)
        assert_eq!(new_state.navigation.panel_stack.len(), 1);
        match &new_state.navigation.panel_stack[0].panel {
            Panel::TeamDetail { abbrev } => {
                assert_eq!(abbrev, "FLA");
            }
            _ => panic!("Expected TeamDetail panel"),
        }
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_select_team_by_position_with_standings() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, effect) =
            reduce_standings(state, StandingsAction::SelectTeamByPosition(1, 3));

        // Should set the selected position (no panel push)
        assert_eq!(new_state.ui.standings.selected_column, 1);
        assert_eq!(new_state.ui.standings.selected_row, 3);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_up_with_standings_from_middle() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        assert_eq!(new_state.ui.standings.selected_row, 4);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_up_with_standings_from_top() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 0;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should wrap to last team (32 teams total, so max_row = 31)
        assert_eq!(new_state.ui.standings.selected_row, 31);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_down_with_standings_from_middle() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 6);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_down_with_standings_from_bottom() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 31; // Last team

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should wrap to first team
        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_left_with_standings_clamps_row() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 1; // Western
        state.ui.standings.selected_row = 10; // Row that exists in Western but maybe not in Eastern

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        // Should switch to Eastern column and clamp row if needed
        assert_eq!(new_state.ui.standings.selected_column, 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_right_with_standings_clamps_row() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 0; // Eastern
        state.ui.standings.selected_row = 10;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        // Should switch to Western column and clamp row if needed
        assert_eq!(new_state.ui.standings.selected_column, 1);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_up_in_division_view() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 0; // Eastern divisions
        state.ui.standings.selected_row = 2;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        assert_eq!(new_state.ui.standings.selected_row, 1);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_down_in_conference_view() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 0; // Eastern
        state.ui.standings.selected_row = 5;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 6);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_in_wildcard_view() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Wildcard;
        state.ui.standings.selected_column = 0; // Eastern
        state.ui.standings.selected_row = 5;

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 6);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_move_selection_left_clamps_row_when_new_column_shorter() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings.selected_column = 1; // Western divisions
        state.ui.standings.selected_row = 20; // High row that might not exist in Eastern

        let (new_state, effect) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        // Should switch to Eastern column and clamp row to max available
        assert_eq!(new_state.ui.standings.selected_column, 0);
        // Row should be clamped to the max row available in Eastern column
        assert!(new_state.ui.standings.selected_row <= 15); // At most 16 Eastern teams
        assert!(matches!(effect, Effect::None));
    }

    // Auto-scroll tests
    #[test]
    fn test_auto_scroll_down_when_selection_moves_below_visible() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.viewport_height = 20;
        state.ui.standings.selected_row = 19; // Just at edge of visible (0-19)
        state.ui.standings.scroll_offset = 0;

        // Move down - should trigger scroll
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 20);
        assert_eq!(
            new_state.ui.standings.scroll_offset, 1,
            "Should scroll down by 1 to keep row 20 visible (20 - 20 + 1 = 1)"
        );

        // Selection should be within visible window
        let viewport_height = new_state.ui.standings.viewport_height;
        assert!(new_state.ui.standings.selected_row >= new_state.ui.standings.scroll_offset);
        assert!(
            new_state.ui.standings.selected_row
                < new_state.ui.standings.scroll_offset + viewport_height
        );
    }

    #[test]
    fn test_auto_scroll_down_multiple_times() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.viewport_height = 20;
        state.ui.standings.selected_row = 0;
        state.ui.standings.scroll_offset = 0;

        // Move down 25 times
        for i in 1..=25 {
            let (new_state, _) =
                reduce_standings(state.clone(), StandingsAction::MoveSelectionDown);
            state = new_state;

            assert_eq!(state.ui.standings.selected_row, i);
            // Scroll should track selection to keep it visible
            let viewport_height = state.ui.standings.viewport_height;
            if i >= viewport_height {
                assert!(
                    state.ui.standings.scroll_offset > 0,
                    "After moving to row {}, scroll should have started",
                    i
                );
            }
        }
    }

    #[test]
    fn test_auto_scroll_up_when_selection_moves_above_visible() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 15;
        state.ui.standings.scroll_offset = 10;

        // Move up - should scroll up to keep selection visible
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        assert_eq!(new_state.ui.standings.selected_row, 14);
        assert_eq!(
            new_state.ui.standings.scroll_offset, 10,
            "Should not scroll yet - row 14 is still visible (10-29)"
        );

        // Move up more
        let mut state = new_state;
        for _ in 0..5 {
            let (new_state, _) = reduce_standings(state.clone(), StandingsAction::MoveSelectionUp);
            state = new_state;
        }

        assert_eq!(state.ui.standings.selected_row, 9);
        assert_eq!(
            state.ui.standings.scroll_offset, 9,
            "Should scroll up to keep row 9 visible"
        );
    }

    #[test]
    fn test_wrap_to_top_resets_scroll() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 31;
        state.ui.standings.scroll_offset = 15;

        // Wrap from bottom to top
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert_eq!(
            new_state.ui.standings.scroll_offset, 0,
            "Wrapping to top should reset scroll_offset"
        );
    }

    #[test]
    fn test_wrap_to_bottom_resets_scroll() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 0;
        state.ui.standings.scroll_offset = 0;
        state.ui.standings.viewport_height = 20;

        // Wrap from top to bottom (32 teams, selected_row = 31)
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        assert_eq!(new_state.ui.standings.selected_row, 31);
        // With viewport_height=20, last visible row is scroll_offset + 19
        // To show row 31, scroll_offset should be 31 - 19 = 12
        assert_eq!(
            new_state.ui.standings.scroll_offset, 12,
            "Wrapping to bottom should auto-scroll to show last team"
        );
    }

    #[test]
    fn test_no_scroll_when_selection_already_visible() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 10;
        state.ui.standings.scroll_offset = 5;

        let initial_scroll = state.ui.standings.scroll_offset;

        // Move down within visible window (5-24)
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 11);
        assert_eq!(
            new_state.ui.standings.scroll_offset, initial_scroll,
            "Scroll should not change when selection moves within visible window"
        );
    }

    #[test]
    fn test_no_scroll_with_empty_standings() {
        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(vec![]));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 0;
        state.ui.standings.scroll_offset = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        assert_eq!(new_state.ui.standings.selected_row, 0);
        assert_eq!(new_state.ui.standings.scroll_offset, 0);
    }

    #[test]
    fn test_ensure_selection_visible_scroll_down() {
        let mut state = AppState::default();
        state.ui.standings.selected_row = 25;
        state.ui.standings.scroll_offset = 0;

        ensure_selection_visible(&mut state);

        // Row 25 with scroll 0 means row is at position 25
        // Visible window is 0-19, so row 25 is outside
        // New scroll should be: 25 - 20 + 1 = 6
        assert_eq!(state.ui.standings.scroll_offset, 6);
    }

    #[test]
    fn test_ensure_selection_visible_scroll_up() {
        let mut state = AppState::default();
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 10;

        ensure_selection_visible(&mut state);

        assert_eq!(
            state.ui.standings.scroll_offset, 5,
            "Should scroll up to make row 5 visible"
        );
    }

    #[test]
    fn test_ensure_selection_visible_no_change() {
        let mut state = AppState::default();
        state.ui.standings.selected_row = 10;
        state.ui.standings.scroll_offset = 5;

        ensure_selection_visible(&mut state);

        assert_eq!(
            state.ui.standings.scroll_offset, 5,
            "Should not change scroll when selection is visible"
        );
    }

    // Page navigation tests
    #[test]
    fn test_page_down_moves_selection_by_page_size() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::PageDown);

        assert_eq!(
            new_state.ui.standings.selected_row, 15,
            "PageDown should move 10 rows (PAGE_SIZE)"
        );
        // Should trigger auto-scroll since row 15 is outside initial viewport (0-19)
    }

    #[test]
    fn test_page_down_at_end_clamps_to_last_team() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 28; // Near end (32 teams total)
        state.ui.standings.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::PageDown);

        assert_eq!(
            new_state.ui.standings.selected_row, 31,
            "PageDown should clamp to last team (31)"
        );
    }

    #[test]
    fn test_page_up_moves_selection_by_page_size() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 20;
        state.ui.standings.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::PageUp);

        assert_eq!(
            new_state.ui.standings.selected_row, 10,
            "PageUp should move up 10 rows (PAGE_SIZE)"
        );
    }

    #[test]
    fn test_page_up_at_top_clamps_to_zero() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::PageUp);

        assert_eq!(
            new_state.ui.standings.selected_row, 0,
            "PageUp should clamp to first team when near top"
        );
    }

    #[test]
    fn test_go_to_top_resets_to_first_team() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 25;
        state.ui.standings.scroll_offset = 15;

        let (new_state, _) = reduce_standings(state, StandingsAction::GoToTop);

        assert_eq!(
            new_state.ui.standings.selected_row, 0,
            "GoToTop should move to first team"
        );
        assert_eq!(
            new_state.ui.standings.scroll_offset, 0,
            "GoToTop should reset scroll to 0"
        );
    }

    #[test]
    fn test_go_to_bottom_moves_to_last_team() {
        use crate::tui::testing::create_test_standings;

        let mut state = AppState::default();
        state.data.standings = Arc::new(Some(create_test_standings()));
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5;
        state.ui.standings.scroll_offset = 2;

        let (new_state, _) = reduce_standings(state, StandingsAction::GoToBottom);

        assert_eq!(
            new_state.ui.standings.selected_row, 31,
            "GoToBottom should move to last team (31)"
        );
        assert_eq!(
            new_state.ui.standings.scroll_offset, 0,
            "GoToBottom should reset scroll to 0"
        );
    }
}
