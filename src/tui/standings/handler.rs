use crossterm::event::{KeyCode, KeyEvent};
use crate::commands::standings::GroupBy;
use super::State;

pub fn handle_key(
    key: KeyEvent,
    state: &mut State,
) -> bool {
    match key.code {
        KeyCode::Left => {
            // Navigate standings view
            state.view = match state.view {
                GroupBy::Division => GroupBy::League,
                GroupBy::Conference => GroupBy::Division,
                GroupBy::League => GroupBy::Conference,
            };
            // Reset scroll when changing view
            state.scrollable.reset();
            true
        }
        KeyCode::Right => {
            // Navigate standings view
            state.view = match state.view {
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Division,
            };
            // Reset scroll when changing view
            state.scrollable.reset();
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
        KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
            // Handle scrolling
            state.scrollable.handle_key(key)
        }
        _ => false, // Key not handled
    }
}
