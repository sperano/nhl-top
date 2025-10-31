use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;
use super::State;
use crate::{SharedData, SharedDataHandle};

/// Clears schedule-related data to show "Loading..." state while fetching new data
fn clear_schedule_data(data: &mut SharedData) {
    data.schedule = None;
    data.period_scores.clear();
    data.game_info.clear();
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
            // Navigate scores dates - move selection left
            if state.selected_index > 0 {
                // Move selection within visible window
                state.selected_index -= 1;
                // Update game_date to the newly selected date
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(-1);
                    // Clear schedule data to show "Loading..." while fetching
                    clear_schedule_data(&mut data);
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
            } else {
                // Already at leftmost position, shift window left
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(-1);
                    // Clear schedule data to show "Loading..." while fetching
                    clear_schedule_data(&mut data);
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
                // Keep selection at index 0 (leftmost)
            }
            true
        }
        KeyCode::Right => {
            // Navigate scores dates - move selection right
            if state.selected_index < 4 {
                // Move selection within visible window
                state.selected_index += 1;
                // Update game_date to the newly selected date
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(1);
                    // Clear schedule data to show "Loading..." while fetching
                    clear_schedule_data(&mut data);
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
            } else {
                // Already at rightmost position, shift window right
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(1);
                    // Clear schedule data to show "Loading..." while fetching
                    clear_schedule_data(&mut data);
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
                // Keep selection at index 4 (rightmost)
            }
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
            state.boxscore_scrollable.reset(); // Reset scroll position
            let mut data = shared_data.write().await;
            data.selected_game_id = None;
            data.boxscore = None;
            data.boxscore_loading = false;
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
    if let Some(ref schedule) = data.schedule {
        if game_index < schedule.games.len() {
            let game = &schedule.games[game_index];

            // Check if game has started
            if !game.game_state.has_started() {
                // Game hasn't started yet, show error message
                data.error_message = Some("Game hasn't started yet".to_string());
                return;
            }

            let game_id = game.id;

            // Set the selected game ID to trigger fetch in background
            data.selected_game_id = Some(game_id);
            data.boxscore_loading = true;
            data.boxscore = None;

            // Open the boxscore view and reset scroll position
            state.boxscore_view_active = true;
            state.boxscore_scrollable.reset();

            // Drop the write lock before triggering refresh
            drop(data);

            // Trigger immediate refresh to fetch boxscore
            let _ = refresh_tx.send(()).await;
        }
    }
}
