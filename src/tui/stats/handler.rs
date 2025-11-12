use super::State;
use crate::tui::widgets::focus::{Focusable, InputResult};
use crossterm::event::KeyEvent;

pub fn handle_key(
    key: KeyEvent,
    state: &mut State,
) -> InputResult {
    // Delegate to container if present
    if let Some(ref mut container) = state.container {
        container.handle_input(key)
    } else {
        InputResult::NotHandled
    }
}
