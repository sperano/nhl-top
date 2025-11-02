use crossterm::event::{KeyCode, KeyEvent};
use super::State;

/// Navigate to a different column and clamp team index to new column's bounds
fn navigate_to_column(state: &mut State, new_column: usize, new_column_team_count: usize) {
    state.selected_column = new_column;
    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
        state.selected_team_index = new_column_team_count - 1;
    }
}

pub fn handle_key(key: KeyEvent, state: &mut State) -> bool {
    // Get layout information
    let layout = match &state.layout_cache {
        Some(layout) => layout,
        None => return false, // No data, can't handle navigation
    };

    let max_columns = layout.column_count();
    let team_count_in_current_column = layout.columns
        .get(state.selected_column)
        .map(|col| col.team_count)
        .unwrap_or(0);

    // If team selection is active, handle team navigation
    if state.team_selection_active {
        match key.code {
            KeyCode::Up => {
                if state.selected_team_index == 0 {
                    // At first team in column, exit team selection mode
                    state.team_selection_active = false;
                    true
                } else {
                    // Navigate up within current column
                    state.selected_team_index = state.selected_team_index.saturating_sub(1);
                    true
                }
            }
            KeyCode::Down => {
                // Navigate down within current column
                if state.selected_team_index + 1 < team_count_in_current_column {
                    state.selected_team_index += 1;
                }
                true
            }
            KeyCode::Left => {
                if state.selected_column > 0 {
                    let new_column = state.selected_column - 1;
                    let new_column_team_count = layout.columns.get(new_column).map(|col| col.team_count).unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::Right => {
                if state.selected_column + 1 < max_columns {
                    let new_column = state.selected_column + 1;
                    let new_column_team_count = layout.columns.get(new_column).map(|col| col.team_count).unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Handle scrolling even in team selection mode
                state.scrollable.handle_key(key)
            }
            KeyCode::Enter => {
                // Log the selected team for debugging
                if let Some(team) = layout.get_team(state.selected_column, state.selected_team_index) {
                    tracing::info!(
                        "Selected team: {} | View: {:?} | Column: {} | Index: {} | Division: {} | Conference: {}",
                        team.team_common_name.default,
                        state.view,
                        state.selected_column,
                        state.selected_team_index,
                        team.division_name,
                        team.conference_name.as_deref().unwrap_or("N/A")
                    );
                }
                true
            }
            KeyCode::Esc => {
                // Exit team selection mode
                state.team_selection_active = false;
                true
            }
            _ => false, // Key not handled in team selection mode
        }
    } else {
        // Subtab focused but team selection not active
        match key.code {
            KeyCode::Left => {
                state.view = state.view.prev();
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                state.layout_cache = None;
                true
            }
            KeyCode::Right => {
                state.view = state.view.next();
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                state.layout_cache = None;
                true
            }
            KeyCode::Down => {
                // Enter team selection mode if there are teams
                if team_count_in_current_column > 0 {
                    state.team_selection_active = true;
                }
                true
            }
            KeyCode::Up => {
                // If at top of scroll, don't handle (let main handler exit subtab mode)
                // Otherwise handle scrolling
                if state.scrollable.scroll_offset == 0 {
                    false // Not handled - will exit subtab mode
                } else {
                    state.scrollable.handle_key(key)
                }
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Handle scrolling
                state.scrollable.handle_key(key)
            }
            _ => false, // Key not handled
        }
    }
}
