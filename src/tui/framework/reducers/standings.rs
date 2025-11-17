use tracing::debug;

use crate::commands::standings::GroupBy;
use crate::tui::framework::action::{Panel, StandingsAction};
use crate::tui::framework::component::Effect;
use crate::tui::framework::state::{AppState, PanelState};

use super::standings_layout::{
    build_standings_layout,
    count_teams_in_conference_column,
    count_teams_in_division_column,
    count_teams_in_wildcard_column,
};

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
    }
}

fn handle_cycle_view(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = match new_state.ui.standings.view {
        GroupBy::Wildcard => GroupBy::Division,
        GroupBy::Division => GroupBy::Conference,
        GroupBy::Conference => GroupBy::League,
        GroupBy::League => GroupBy::Wildcard,
    };

    // Reset selection when changing views
    reset_standings_selection(&mut new_state);
    (new_state, Effect::None)
}

fn handle_cycle_view_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = match new_state.ui.standings.view {
        GroupBy::Wildcard => GroupBy::League,
        GroupBy::Division => GroupBy::Wildcard,
        GroupBy::Conference => GroupBy::Division,
        GroupBy::League => GroupBy::Conference,
    };
    (new_state, Effect::None)
}

fn handle_cycle_view_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = match new_state.ui.standings.view {
        GroupBy::Wildcard => GroupBy::Division,
        GroupBy::Division => GroupBy::Conference,
        GroupBy::Conference => GroupBy::League,
        GroupBy::League => GroupBy::Wildcard,
    };
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
    // Build layout map from standings data (same logic as renderer)
    if let Some(ref standings) = state.data.standings {
        let layout = build_standings_layout(
            standings,
            state.ui.standings.view,
            state.system.config.display_standings_western_first,
        );

        // Lookup team at selected position
        if let Some(col) = layout.get(state.ui.standings.selected_column) {
            if let Some(team_abbrev) = col.get(state.ui.standings.selected_row) {
                debug!(
                    "STANDINGS: Selected team: {} (row={}, col={})",
                    team_abbrev,
                    state.ui.standings.selected_row,
                    state.ui.standings.selected_column
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
                    state.ui.standings.selected_row,
                    state.ui.standings.selected_column
                );
            }
        }
    }

    (state, Effect::None)
}

fn handle_select_team_by_position(state: AppState, column: usize, row: usize) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.selected_column = column;
    new_state.ui.standings.selected_row = row;
    (new_state, Effect::None)
}

fn handle_move_selection_up(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Get team count (respects column in Conference/Division/Wildcard views)
    if let Some(ref standings) = new_state.data.standings {
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
            } else {
                new_state.ui.standings.selected_row -= 1;
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_move_selection_down(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Get team count (respects column in Conference/Division/Wildcard views)
    if let Some(ref standings) = new_state.data.standings {
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
            } else {
                new_state.ui.standings.selected_row += 1;
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_move_selection_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Conference, Division, and Wildcard views have 2 columns for navigation
    if matches!(new_state.ui.standings.view, GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard) {
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
    if matches!(new_state.ui.standings.view, GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard) {
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

// Helper functions

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
    if let Some(ref standings) = state.data.standings {
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
