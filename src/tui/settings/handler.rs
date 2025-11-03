use crossterm::event::{KeyCode, KeyEvent};
use super::State;
use crate::SharedDataHandle;

// Import the COLORS array from view module
use super::view::COLORS;

/// Handle key events for settings tab
pub async fn handle_key(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle) -> bool {
    // Clear any existing status message when navigating
    if matches!(key.code, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) {
        state.status_message = None;
    }

    // Color picker is active when in subtab mode
    match key.code {
        KeyCode::Up => {
            // Move up one row (subtract 4), or signal to exit if at top row
            if state.selected_color_index >= 4 {
                state.selected_color_index -= 4;
                true
            } else {
                // At top row, let default handler exit subtab mode
                false
            }
        }
        KeyCode::Down => {
            // Move down one row (add 4)
            if state.selected_color_index + 4 < 24 {
                state.selected_color_index += 4;
            }
            true
        }
        KeyCode::Left => {
            // Move left one column
            if state.selected_color_index % 4 != 0 {
                state.selected_color_index -= 1;
            }
            true
        }
        KeyCode::Right => {
            // Move right one column
            if state.selected_color_index % 4 != 3 {
                state.selected_color_index += 1;
            }
            true
        }
        KeyCode::Enter => {
            // Get the selected color
            let (selected_color, selected_name) = COLORS[state.selected_color_index];

            // Update the theme in SharedData
            let mut data = shared_data.write().await;
            data.config.theme.selection_fg = selected_color;

            // Set status message
            state.status_message = Some(format!("✓ Theme color changed to {}", selected_name));

            tracing::info!("User selected color: {} - theme updated", selected_name);
            true
        }
        _ => false,
    }
}
