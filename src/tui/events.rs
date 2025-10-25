use crossterm::event::{KeyCode, KeyEvent};
use crate::commands::standings::GroupBy;
use super::tabs::{AppState, Tab};

pub enum AppAction {
    Continue,
    Exit,
}

pub fn handle_key_event(key: KeyEvent, state: &mut AppState) -> AppAction {
    match key.code {
        KeyCode::Esc => AppAction::Exit,

        // Arrow key navigation
        KeyCode::Left => {
            if state.subtab_focused {
                // Navigate sub-tabs (standings view)
                state.standings_view = match state.standings_view {
                    GroupBy::Division => GroupBy::League,
                    GroupBy::Conference => GroupBy::Division,
                    GroupBy::League => GroupBy::Conference,
                };
            } else {
                // Navigate main tabs
                state.current_tab = match state.current_tab {
                    Tab::Scores => Tab::Settings,
                    Tab::Standings => Tab::Scores,
                    Tab::Settings => Tab::Standings,
                };
                // Reset subtab focus when leaving Standings tab
                if state.current_tab != Tab::Standings {
                    state.subtab_focused = false;
                }
            }
            AppAction::Continue
        }
        KeyCode::Right => {
            if state.subtab_focused {
                // Navigate sub-tabs (standings view)
                state.standings_view = match state.standings_view {
                    GroupBy::Division => GroupBy::Conference,
                    GroupBy::Conference => GroupBy::League,
                    GroupBy::League => GroupBy::Division,
                };
            } else {
                // Navigate main tabs
                state.current_tab = match state.current_tab {
                    Tab::Scores => Tab::Standings,
                    Tab::Standings => Tab::Settings,
                    Tab::Settings => Tab::Scores,
                };
                // Reset subtab focus when leaving Standings tab
                if state.current_tab != Tab::Standings {
                    state.subtab_focused = false;
                }
            }
            AppAction::Continue
        }
        KeyCode::Down => {
            // Activate sub-tab navigation (only on Standings tab)
            if state.current_tab == Tab::Standings && !state.subtab_focused {
                state.subtab_focused = true;
            }
            AppAction::Continue
        }
        KeyCode::Up => {
            // Deactivate sub-tab navigation
            if state.subtab_focused {
                state.subtab_focused = false;
            }
            AppAction::Continue
        }

        _ => AppAction::Continue,
    }
}
