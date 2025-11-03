use crossterm::event::{KeyCode, KeyEvent};
use super::State;

/// Handle key events for settings tab
pub fn handle_key(key: KeyEvent, state: &mut State) -> bool {
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
            // Log the selected color
            let color_names = [
                "Coral Red", "Bright Orange", "Golden Yellow", "Mint Green",
                "Sky Blue", "Claude Purple", "Hot Pink", "Turquoise",
                "Light Pink", "Powder Blue", "Plum", "Khaki",
                "Teal", "Amethyst", "Pumpkin", "Ocean Blue",
                "Chocolate", "Rosy Brown", "Salmon", "Olive",
                "Crimson", "Cyan", "Indigo", "Lime Green",
            ];
            let selected_color_name = color_names[state.selected_color_index];
            tracing::info!("User selected color: {}", selected_color_name);
            true
        }
        _ => false,
    }
}
