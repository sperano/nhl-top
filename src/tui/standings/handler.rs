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
            true
        }
        KeyCode::Right => {
            // Navigate standings view
            state.view = match state.view {
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Division,
            };
            true
        }
        _ => false, // Key not handled
    }
}
