use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use super::State;
use super::state::{DATE_WINDOW_MIN_INDEX, DATE_WINDOW_MAX_INDEX, DATE_WINDOW_CENTER};
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
                // Move left within the window and recenter on the new date
                state.selected_index -= 1;
                // Moving left by 1 position means game_date should shift left by 1 day
                // So the new selected date becomes the center
                navigate_to_date(-1, shared_data, refresh_tx).await;
                // Reset to center after the window updates
                state.selected_index = DATE_WINDOW_CENTER;
            } else {
                // At left edge, shift window left by 1 day and stay at left edge
                navigate_to_date(-1, shared_data, refresh_tx).await;
            }
            true
        }
        KeyCode::Right => {
            if state.selected_index < DATE_WINDOW_MAX_INDEX {
                // Move right within the window and recenter on the new date
                state.selected_index += 1;
                // Moving right by 1 position means game_date should shift right by 1 day
                // So the new selected date becomes the center
                navigate_to_date(1, shared_data, refresh_tx).await;
                // Reset to center after the window updates
                state.selected_index = DATE_WINDOW_CENTER;
            } else {
                // At right edge, shift window right by 1 day and stay at right edge
                navigate_to_date(1, shared_data, refresh_tx).await;
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
    use crate::tui::scores::state::{DATE_WINDOW_MIN_INDEX, DATE_WINDOW_MAX_INDEX, DATE_WINDOW_CENTER, DATE_WINDOW_SIZE};
    use nhl_api::GameDate;
    use chrono::NaiveDate;

    /// Calculate the full window (game_date is always at center)
    fn calculate_window(game_date: &GameDate) -> [NaiveDate; DATE_WINDOW_SIZE] {
        let base_date = match game_date {
            GameDate::Date(d) => d.clone(),
            GameDate::Now => chrono::Local::now().date_naive(),
        };
        [
            base_date + chrono::Duration::days(-2),
            base_date + chrono::Duration::days(-1),
            base_date + chrono::Duration::days(0),
            base_date + chrono::Duration::days(1),
            base_date + chrono::Duration::days(2),
        ]
    }

    #[test]
    fn test_window_always_centered_on_game_date() {
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Window is always [game_date-2, game_date-1, game_date, game_date+1, game_date+2]
        let window = calculate_window(&game_date);
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());
        assert_eq!(window[1], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        assert_eq!(window[2], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()); // CENTER
        assert_eq!(window[3], NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        assert_eq!(window[4], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
    }

    #[test]
    fn test_navigation_within_window() {
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let window = calculate_window(&game_date);

        // Start at center (index 2), viewing Jan 15
        assert_eq!(window[2], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Press Left to index 1, viewing Jan 14 (same window, same game_date)
        assert_eq!(window[1], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());

        // Press Left to index 0, viewing Jan 13 (same window, same game_date)
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());

        // Press Right from center to index 3, viewing Jan 16
        assert_eq!(window[3], NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());

        // Press Right to index 4, viewing Jan 17
        assert_eq!(window[4], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
    }

    #[test]
    fn test_navigation_at_left_edge_shifts_window() {
        // Start: game_date=Jan 15, selected_index=0
        // Window: [Jan 13, Jan 14, Jan 15, Jan 16, Jan 17], viewing Jan 13
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let window = calculate_window(&game_date);
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());

        // Press Left: game_date becomes Jan 14, selected_index stays at 0
        // Window: [Jan 12, Jan 13, Jan 14, Jan 15, Jan 16], viewing Jan 12
        let new_game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        let new_window = calculate_window(&new_game_date);
        assert_eq!(new_window[0], NaiveDate::from_ymd_opt(2024, 1, 12).unwrap());
        assert_eq!(new_window[2], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
    }

    #[test]
    fn test_navigation_at_right_edge_shifts_window() {
        // Start: game_date=Jan 15, selected_index=4
        // Window: [Jan 13, Jan 14, Jan 15, Jan 16, Jan 17], viewing Jan 17
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let window = calculate_window(&game_date);
        assert_eq!(window[4], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());

        // Press Right: game_date becomes Jan 16, selected_index stays at 4
        // Window: [Jan 14, Jan 15, Jan 16, Jan 17, Jan 18], viewing Jan 18
        let new_game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        let new_window = calculate_window(&new_game_date);
        assert_eq!(new_window[4], NaiveDate::from_ymd_opt(2024, 1, 18).unwrap());
        assert_eq!(new_window[2], NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
    }

    #[test]
    fn test_complete_navigation_sequence() {
        // Start: game_date=Jan 15 (today), selected_index=2 (center)
        // Window: [Jan 13, Jan 14, **Jan 15**, Jan 16, Jan 17]
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let window = calculate_window(&game_date);
        assert_eq!(window[2], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Press Left: selected_index=1, window unchanged
        // Viewing: Jan 14
        assert_eq!(window[1], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());

        // Press Left: selected_index=0, window unchanged
        // Viewing: Jan 13
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());

        // Press Left at edge: game_date=Jan 14, selected_index=0
        // Window: [Jan 12, **Jan 13**, Jan 14, Jan 15, Jan 16]
        let new_game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        let new_window = calculate_window(&new_game_date);
        assert_eq!(new_window[0], NaiveDate::from_ymd_opt(2024, 1, 12).unwrap());
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
