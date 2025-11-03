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

/// Navigate within the window without shifting the window base
///
/// Calculates window_base from current game_date and selected_index,
/// then updates game_date to the new position in the same window
async fn navigate_within_window(
    old_index: usize,
    new_index: usize,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) {
    {
        let mut data = shared_data.write().await;
        // Calculate window base: leftmost date in current window
        let window_base = data.game_date.add_days(-(old_index as i64));
        // Update game_date to the new selected position in the same window
        data.game_date = window_base.add_days(new_index as i64);
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
                // Move left within the window - window base stays the same
                let old_index = state.selected_index;
                state.selected_index -= 1;
                navigate_within_window(old_index, state.selected_index, shared_data, refresh_tx).await;
            } else {
                // At left edge, shift window left by 1 day and stay at left edge
                navigate_to_date(-1, shared_data, refresh_tx).await;
            }
            true
        }
        KeyCode::Right => {
            if state.selected_index < DATE_WINDOW_MAX_INDEX {
                // Move right within the window - window base stays the same
                let old_index = state.selected_index;
                state.selected_index += 1;
                navigate_within_window(old_index, state.selected_index, shared_data, refresh_tx).await;
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
    use crate::tui::scores::state::DATE_WINDOW_SIZE;
    use nhl_api::GameDate;
    use chrono::NaiveDate;

    /// Calculate the full window from game_date and selected_index
    /// Window base = game_date - selected_index
    /// Window = [base, base+1, base+2, base+3, base+4]
    fn calculate_window(game_date: &GameDate, selected_index: usize) -> [NaiveDate; DATE_WINDOW_SIZE] {
        let viewing_date = match game_date {
            GameDate::Date(d) => d.clone(),
            GameDate::Now => chrono::Local::now().date_naive(),
        };

        // Calculate window base (leftmost date)
        let base_date = viewing_date - chrono::Duration::days(selected_index as i64);

        [
            base_date + chrono::Duration::days(0),
            base_date + chrono::Duration::days(1),
            base_date + chrono::Duration::days(2),
            base_date + chrono::Duration::days(3),
            base_date + chrono::Duration::days(4),
        ]
    }

    #[test]
    fn test_window_calculation_from_base() {
        // When game_date=1/15 and selected_index=2
        // Window base = 1/15 - 2 = 1/13
        // Window = [1/13, 1/14, 1/15, 1/16, 1/17]
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let selected_index = 2;

        let window = calculate_window(&game_date, selected_index);
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());
        assert_eq!(window[1], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        assert_eq!(window[2], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(window[3], NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        assert_eq!(window[4], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());

        // When game_date=1/14 and selected_index=1
        // Window base = 1/14 - 1 = 1/13
        // Window = [1/13, 1/14, 1/15, 1/16, 1/17] (SAME window!)
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        let selected_index = 1;

        let window = calculate_window(&game_date, selected_index);
        assert_eq!(window[0], NaiveDate::from_ymd_opt(2024, 1, 13).unwrap());
        assert_eq!(window[1], NaiveDate::from_ymd_opt(2024, 1, 14).unwrap());
        assert_eq!(window[2], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(window[3], NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        assert_eq!(window[4], NaiveDate::from_ymd_opt(2024, 1, 17).unwrap());
    }

    #[test]
    fn test_navigation_within_window() {
        // Start: game_date=11/02, selected_index=2
        // Window base = 11/02 - 2 = 10/31
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04]
        let mut game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 2).unwrap());
        let mut selected_index = 2;
        let mut window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);
        assert_eq!(window[selected_index], NaiveDate::from_ymd_opt(2024, 11, 2).unwrap());

        // Press Left: selected_index=1, game_date=11/01
        // Window base = 10/31 (unchanged)
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Same window!
        selected_index = 1;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 1).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);
        assert_eq!(window[selected_index], NaiveDate::from_ymd_opt(2024, 11, 1).unwrap());

        // Press Left: selected_index=0, game_date=10/31
        // Window base = 10/31 (unchanged)
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Still same window!
        selected_index = 0;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);
        assert_eq!(window[selected_index], NaiveDate::from_ymd_opt(2024, 10, 31).unwrap());
    }

    #[test]
    fn test_navigation_at_left_edge_shifts_window() {
        // At edge: game_date=10/31, selected_index=0
        // Window base = 10/31 - 0 = 10/31
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04]
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap());
        let selected_index = 0;
        let window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);

        // Press Left at edge: game_date=10/30, selected_index=0
        // Window base = 10/30 - 0 = 10/30 (shifted by -1!)
        // Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!
        let new_game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 30).unwrap());
        let selected_index = 0;
        let new_window = calculate_window(&new_game_date, selected_index);

        assert_eq!(new_window, [
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        ]);
    }

    #[test]
    fn test_navigation_at_right_edge_shifts_window() {
        // At edge: game_date=11/02, selected_index=4
        // Window base = 11/02 - 4 = 10/29
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02]
        let game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 2).unwrap());
        let selected_index = 4;
        let window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press Right at edge: game_date=11/03, selected_index=4
        // Window base = 11/03 - 4 = 10/30 (shifted by +1!)
        // Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!
        let new_game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 3).unwrap());
        let selected_index = 4;
        let new_window = calculate_window(&new_game_date, selected_index);

        assert_eq!(new_window, [
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        ]);
    }

    #[test]
    fn test_complete_navigation_sequence_from_spec() {
        // This test follows the EXACT spec provided by the user

        // Start: game_date=11/02, selected_index=2
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04]
        let mut game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 2).unwrap());
        let mut selected_index = 2;
        let mut window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);

        // Press Left: selected_index=1, game_date=11/01
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Same window!
        selected_index = 1;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 1).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);

        // Press Left: selected_index=0, game_date=10/31
        // Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Still same window!
        selected_index = 0;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 4).unwrap(),
        ]);

        // Press Left at edge: game_date=10/30, selected_index=0
        // Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 30).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        ]);

        // Press Left at edge: game_date=10/29, selected_index=0
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Window shifted!
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 29).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press right: game_date=10/30, selected_index=1
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02]
        selected_index = 1;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 30).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press right: game_date=10/31, selected_index=2
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02]
        selected_index = 2;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press right: game_date=11/1, selected_index=3
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02]
        selected_index = 3;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 1).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press right: game_date=11/2, selected_index=4
        // Window: [10/29, 10/30, 10/31, 11/01, 11/02]
        selected_index = 4;
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 2).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 29).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
        ]);

        // Press right at edge: game_date=11/3, selected_index=4
        // Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!
        game_date = GameDate::Date(NaiveDate::from_ymd_opt(2024, 11, 3).unwrap());
        window = calculate_window(&game_date, selected_index);

        assert_eq!(window, [
            NaiveDate::from_ymd_opt(2024, 10, 30).unwrap(),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        ]);
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
