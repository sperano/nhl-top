use crossterm::event::{KeyCode, KeyEvent};
use super::State;

/// Navigate to a different column and clamp team index to new column's bounds
fn navigate_to_column(state: &mut State, new_column: usize, new_column_team_count: usize) {
    state.selected_column = new_column;
    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
        state.selected_team_index = new_column_team_count - 1;
    }
}

const TOTAL_PLAYERS: usize = 15;
const TOTAL_GOALIES: usize = 2;
const TOTAL_SELECTABLE_ITEMS: usize = TOTAL_PLAYERS + TOTAL_GOALIES;

fn handle_team_detail_view(key: KeyEvent, state: &mut State) -> bool {
    if state.team_detail_player_selection_active {
        match key.code {
            KeyCode::Up => {
                if state.team_detail_selected_player_index == 0 {
                    state.team_detail_player_selection_active = false;
                } else {
                    state.team_detail_selected_player_index = state.team_detail_selected_player_index.saturating_sub(1);
                }
                true
            }
            KeyCode::Down => {
                if state.team_detail_selected_player_index + 1 < TOTAL_SELECTABLE_ITEMS {
                    state.team_detail_selected_player_index += 1;
                }
                true
            }
            KeyCode::Enter => {
                let player_name = get_player_name_by_index(state.team_detail_selected_player_index);
                tracing::info!("Selected player: {}", player_name);
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.team_detail_scrollable.handle_key(key)
            }
            KeyCode::Esc => {
                state.team_detail_player_selection_active = false;
                true
            }
            _ => false,
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                state.team_detail_view_active = false;
                state.team_detail_scrollable.reset();
                state.selected_team_name = None;
                state.team_detail_player_selection_active = false;
                state.team_detail_selected_player_index = 0;
                true
            }
            KeyCode::Down => {
                state.team_detail_player_selection_active = true;
                state.team_detail_selected_player_index = 0;
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.team_detail_scrollable.handle_key(key)
            }
            _ => true,
        }
    }
}

fn get_player_name_by_index(index: usize) -> &'static str {
    let players = [
        "Auston Matthews",
        "Mitchell Marner",
        "William Nylander",
        "John Tavares",
        "Morgan Rielly",
        "Matthew Knies",
        "Tyler Bertuzzi",
        "Max Domi",
        "Jake McCabe",
        "T.J. Brodie",
        "Calle Jarnkrok",
        "Bobby McMann",
        "David Kampf",
        "Timothy Liljegren",
        "Noah Gregor",
        "Ilya Samsonov",
        "Joseph Woll",
    ];

    players.get(index).unwrap_or(&"Unknown Player")
}

pub fn handle_key(key: KeyEvent, state: &mut State) -> bool {
    // If team detail view is active, handle separately
    if state.team_detail_view_active {
        return handle_team_detail_view(key, state);
    }

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
                if let Some(team) = layout.get_team(state.selected_column, state.selected_team_index) {
                    state.team_detail_view_active = true;
                    state.team_detail_scrollable.reset();
                    state.selected_team_name = Some(team.team_common_name.default.clone());
                    state.team_detail_player_selection_active = false;
                    state.team_detail_selected_player_index = 0;
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
