use crossterm::event::{KeyCode, KeyEvent};
use crate::commands::standings::GroupBy;
use super::State;

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
                // Switch to left column, maintaining same rank/index
                if state.selected_column > 0 {
                    state.selected_column -= 1;
                    // Keep same index, but clamp to new column's team count
                    let new_column_team_count = layout.columns
                        .get(state.selected_column)
                        .map(|col| col.team_count)
                        .unwrap_or(0);
                    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
                        state.selected_team_index = new_column_team_count - 1;
                    }
                }
                true
            }
            KeyCode::Right => {
                // Switch to right column, maintaining same rank/index
                if state.selected_column + 1 < max_columns {
                    state.selected_column += 1;
                    // Keep same index, but clamp to new column's team count
                    let new_column_team_count = layout.columns
                        .get(state.selected_column)
                        .map(|col| col.team_count)
                        .unwrap_or(0);
                    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
                        state.selected_team_index = new_column_team_count - 1;
                    }
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
                // Navigate standings view
                state.view = match state.view {
                    GroupBy::Division => GroupBy::League,
                    GroupBy::Conference => GroupBy::Division,
                    GroupBy::League => GroupBy::Conference,
                };
                // Reset scroll and team selection when changing view
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                // Invalidate layout cache (will be rebuilt on next render)
                state.layout_cache = None;
                true
            }
            KeyCode::Right => {
                // Navigate standings view
                state.view = match state.view {
                    GroupBy::Division => GroupBy::Conference,
                    GroupBy::Conference => GroupBy::League,
                    GroupBy::League => GroupBy::Division,
                };
                // Reset scroll and team selection when changing view
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                // Invalidate layout cache (will be rebuilt on next render)
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
