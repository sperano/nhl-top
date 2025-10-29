use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;
use super::State;
use crate::SharedDataHandle;

pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // Handle box selection mode separately
    if state.box_selection_active {
        return handle_box_navigation(key, state);
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
                    data.schedule = None;
                    data.period_scores.clear();
                    data.game_info.clear();
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
            } else {
                // Already at leftmost position, shift window left
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(-1);
                    // Clear schedule data to show "Loading..." while fetching
                    data.schedule = None;
                    data.period_scores.clear();
                    data.game_info.clear();
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
                // Keep selection at index 0 (leftmost)
            }
            true
        }
        KeyCode::Right => {
            // Navigate scores dates - move selection right
            if state.selected_index < 2 {
                // Move selection within visible window
                state.selected_index += 1;
                // Update game_date to the newly selected date
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(1);
                    // Clear schedule data to show "Loading..." while fetching
                    data.schedule = None;
                    data.period_scores.clear();
                    data.game_info.clear();
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
            } else {
                // Already at rightmost position, shift window right
                {
                    let mut data = shared_data.write().await;
                    data.game_date = data.game_date.add_days(1);
                    // Clear schedule data to show "Loading..." while fetching
                    data.schedule = None;
                    data.period_scores.clear();
                    data.game_info.clear();
                }
                // Trigger immediate refresh
                let _ = refresh_tx.send(()).await;
                // Keep selection at index 2 (rightmost)
            }
            true
        }
        _ => false, // Key not handled
    }
}

fn handle_box_navigation(key: KeyEvent, state: &mut State) -> bool {
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
        _ => false,
    }
}
