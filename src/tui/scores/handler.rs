use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use super::State;
use super::state::{DATE_WINDOW_MIN_INDEX, DATE_WINDOW_MAX_INDEX};
use crate::tui::error::TuiError;
use crate::{SharedData, SharedDataHandle};

/// Clears schedule-related data to show "Loading..." state while fetching new data
fn clear_schedule_data(data: &mut SharedData) {
    data.schedule = Arc::new(None);
    data.period_scores = Arc::new(HashMap::new());
    data.game_info = Arc::new(HashMap::new());
}

/// Navigate to a date by adjusting game_date and triggering refresh
///
/// This helper consolidates the common pattern of:
/// 1. Updating game_date by offset days
/// 2. Clearing schedule data to show "Loading..."
/// 3. Triggering an immediate refresh
async fn navigate_to_date(
    offset_days: i64,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) {
    {
        let mut data = shared_data.write().await;
        data.game_date = data.game_date.add_days(offset_days);
        clear_schedule_data(&mut data);
    }
    if let Err(e) = refresh_tx.send(()).await {
        tracing::error!("Failed to send refresh signal: {}", e);
    }
}

pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // Handle boxscore view mode
    if state.boxscore_view_active {
        return handle_boxscore_view(key, state, shared_data).await;
    }

    // Handle box selection mode separately
    if state.box_selection_active {
        return handle_box_navigation(key, state, shared_data, refresh_tx).await;
    }

    // Handle date selection mode
    match key.code {
        KeyCode::Down => {
            // Enter box selection mode
            state.box_selection_active = true;
            state.selected_box = (0, 0);
            true
        }
        KeyCode::Left => {
            if state.selected_index > DATE_WINDOW_MIN_INDEX {
                state.selected_index -= 1;
            }
            navigate_to_date(-1, shared_data, refresh_tx).await;
            true
        }
        KeyCode::Right => {
            if state.selected_index < DATE_WINDOW_MAX_INDEX {
                state.selected_index += 1;
            }
            navigate_to_date(1, shared_data, refresh_tx).await;
            true
        }
        _ => false, // Key not handled
    }
}

async fn handle_box_navigation(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    let (row, col) = state.selected_box;
    let (num_rows, num_cols) = state.grid_dimensions;

    // Handle no games case
    if num_rows == 0 || num_cols == 0 {
        return false;
    }

    match key.code {
        KeyCode::Up => {
            if row == 0 {
                // Exit box selection mode and return to date selection
                state.box_selection_active = false;
            } else {
                state.selected_box = (row - 1, col);
            }
            true
        }
        KeyCode::Down => {
            if row + 1 < num_rows {
                state.selected_box = (row + 1, col);
            }
            true
        }
        KeyCode::Left => {
            if col == 0 {
                // Wrap to previous row if not on first row
                if row > 0 {
                    state.selected_box = (row - 1, num_cols - 1);
                }
            } else {
                state.selected_box = (row, col - 1);
            }
            true
        }
        KeyCode::Right => {
            if col + 1 < num_cols {
                state.selected_box = (row, col + 1);
            } else if row + 1 < num_rows {
                // Wrap to next row
                state.selected_box = (row + 1, 0);
            }
            true
        }
        KeyCode::Enter => {
            // Select the game and trigger boxscore fetch
            select_game_for_boxscore(state, shared_data, refresh_tx).await;
            true
        }
        KeyCode::Esc => {
            // Exit box selection mode and return to date selection
            state.box_selection_active = false;
            true
        }
        _ => false,
    }
}

async fn handle_boxscore_view(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle) -> bool {
    match key.code {
        KeyCode::Esc => {
            // Close the boxscore view and return to game list
            state.boxscore_view_active = false;
            state.boxscore_scrollable.reset();
            let mut data = shared_data.write().await;
            data.clear_boxscore();
            true
        }
        KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
            // Handle scrolling
            state.boxscore_scrollable.handle_key(key)
        }
        _ => true, // Consume all keys while boxscore view is active
    }
}

async fn select_game_for_boxscore(state: &mut State, shared_data: &SharedDataHandle, refresh_tx: &mpsc::Sender<()>) {
    let (row, col) = state.selected_box;
    let (_, num_cols) = state.grid_dimensions;

    // Calculate the game index based on row and column
    let game_index = row * num_cols + col;

    // Get the game from the schedule and set selected_game_id
    let mut data = shared_data.write().await;
    if let Some(ref schedule) = data.schedule.as_ref() {
        if game_index < schedule.games.len() {
            let game = &schedule.games[game_index];

            // Check if game has started
            if !game.game_state.has_started() {
                data.error_message = Some(TuiError::GameNotStarted.to_string());
                return;
            }

            let game_id = game.id;

            // Set the selected game ID to trigger fetch in background
            data.selected_game_id = Some(game_id);
            data.boxscore_loading = true;
            data.boxscore = Arc::new(None);

            // Open the boxscore view and reset scroll position
            state.boxscore_view_active = true;
            state.boxscore_scrollable.reset();

            // Drop the write lock before triggering refresh
            drop(data);

            // Trigger immediate refresh to fetch boxscore
            if let Err(e) = refresh_tx.send(()).await {
                tracing::error!("Failed to send refresh signal for boxscore: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::scores::state::{DATE_WINDOW_MIN_INDEX, DATE_WINDOW_MAX_INDEX, DATE_WINDOW_CENTER};

    #[test]
    fn test_selected_index_navigation_left_within_window() {
        let mut state = State::new();
        state.selected_index = DATE_WINDOW_CENTER;

        if state.selected_index > DATE_WINDOW_MIN_INDEX {
            state.selected_index -= 1;
        }

        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_selected_index_navigation_left_at_edge() {
        let mut state = State::new();
        state.selected_index = DATE_WINDOW_MIN_INDEX;

        if state.selected_index > DATE_WINDOW_MIN_INDEX {
            state.selected_index -= 1;
        }

        assert_eq!(state.selected_index, DATE_WINDOW_MIN_INDEX);
    }

    #[test]
    fn test_selected_index_navigation_right_within_window() {
        let mut state = State::new();
        state.selected_index = DATE_WINDOW_CENTER;

        if state.selected_index < DATE_WINDOW_MAX_INDEX {
            state.selected_index += 1;
        }

        assert_eq!(state.selected_index, 3);
    }

    #[test]
    fn test_selected_index_navigation_right_at_edge() {
        let mut state = State::new();
        state.selected_index = DATE_WINDOW_MAX_INDEX;

        if state.selected_index < DATE_WINDOW_MAX_INDEX {
            state.selected_index += 1;
        }

        assert_eq!(state.selected_index, DATE_WINDOW_MAX_INDEX);
    }

    #[test]
    fn test_box_selection_initial_state() {
        let state = State::new();
        assert!(!state.box_selection_active);
        assert_eq!(state.selected_box, (0, 0));
    }

    #[test]
    fn test_box_selection_enter_mode() {
        let mut state = State::new();
        state.box_selection_active = true;
        state.selected_box = (0, 0);

        assert!(state.box_selection_active);
        assert_eq!(state.selected_box, (0, 0));
    }
}
