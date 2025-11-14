/// Keyboard event to action mapping
///
/// This module handles converting crossterm KeyEvents into framework Actions.
/// It contains all the keyboard navigation logic for the TUI.

use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, trace};

use super::action::{Action, ScoresAction, StandingsAction, Tab};
use super::state::AppState;

/// Convert a KeyEvent into an Action based on current application state
///
/// This function implements all keyboard navigation:
/// - Global keys (q, /, ESC)
/// - Tab bar focus: Left/Right navigate tabs, Down enters content
/// - Content focus: Context-sensitive navigation, Up returns to tab bar
/// - Panel navigation (ESC to close)
pub fn key_to_action(key: KeyEvent, state: &AppState) -> Option<Action> {
    // Get current tab and focus state
    let current_tab = state.navigation.current_tab;
    let content_focused = state.navigation.content_focused;

    trace!(
        "KEY: {:?} (tab={:?}, content_focused={}, panel_stack_len={})",
        key.code,
        current_tab,
        content_focused,
        state.navigation.panel_stack.len()
    );

    // Global keys (work regardless of tab/focus)
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Some(Action::Quit),
        KeyCode::Char('/') => return Some(Action::ToggleCommandPalette),
        _ => {}
    }

    // ESC key handling - navigate up through focus hierarchy
    if key.code == KeyCode::Esc {
        // Priority 1: If there's a panel open, close it
        if !state.navigation.panel_stack.is_empty() {
            debug!("KEY: ESC pressed with panel open - popping panel");
            return Some(Action::PopPanel);
        }

        // Priority 2: If in box selection mode on Scores tab, exit to date subtabs
        if state.ui.scores.box_selection_active {
            debug!("KEY: ESC pressed in box selection - exiting to date subtabs");
            return Some(Action::ScoresAction(ScoresAction::ExitBoxSelection));
        }

        // Priority 3: If in team mode on Standings tab, exit to view subtabs
        if state.ui.standings.team_mode {
            debug!("KEY: ESC pressed in team mode - exiting to view subtabs");
            return Some(Action::StandingsAction(StandingsAction::ExitTeamMode));
        }

        // Priority 4: If content is focused, return to tab bar
        if content_focused {
            debug!("KEY: ESC pressed in content - returning to tab bar");
            return Some(Action::ExitContentFocus);
        }

        // Priority 5: At top level (tab bar), do nothing - use 'q' to quit
        debug!("KEY: ESC pressed at tab bar - ignoring (use 'q' to quit)");
        return None;
    }

    // Number keys for direct tab switching
    match key.code {
        KeyCode::Char('1') => return Some(Action::NavigateTab(Tab::Scores)),
        KeyCode::Char('2') => return Some(Action::NavigateTab(Tab::Standings)),
        KeyCode::Char('3') => return Some(Action::NavigateTab(Tab::Stats)),
        KeyCode::Char('4') => return Some(Action::NavigateTab(Tab::Players)),
        KeyCode::Char('5') => return Some(Action::NavigateTab(Tab::Settings)),
        KeyCode::Char('6') => return Some(Action::NavigateTab(Tab::Browser)),
        _ => {}
    }

    // Arrow key handling based on focus level
    if !content_focused {
        // TAB BAR FOCUSED: Left/Right navigate tabs, Down enters content
        match key.code {
            KeyCode::Left => return Some(Action::NavigateTabLeft),
            KeyCode::Right => return Some(Action::NavigateTabRight),
            KeyCode::Down => {
                debug!("KEY: Down pressed on tab bar - entering content focus");
                return Some(Action::EnterContentFocus);
            }
            _ => {}
        }
    } else {
        // CONTENT FOCUSED: Context-sensitive navigation based on current tab

        // Up key returns to tab bar (works on all tabs)
        if key.code == KeyCode::Up {
            // Check if we're in a nested mode first
            if state.ui.scores.box_selection_active {
                // In box selection - Up navigates within grid
                return Some(Action::ScoresAction(ScoresAction::MoveGameSelectionUp));
            } else if state.ui.standings.team_mode {
                // In team selection - Up navigates teams
                return Some(Action::StandingsAction(StandingsAction::MoveSelectionUp));
            } else {
                // Not in nested mode - Up returns to tab bar
                debug!("KEY: Up pressed in content - returning to tab bar");
                return Some(Action::ExitContentFocus);
            }
        }

        // Context-sensitive navigation based on current tab
        match current_tab {
            Tab::Scores => {
                // On Scores tab: check if in box selection mode
                if state.ui.scores.box_selection_active {
                    // Box selection mode - arrows navigate within game grid
                    match key.code {
                        KeyCode::Down => {
                            return Some(Action::ScoresAction(ScoresAction::MoveGameSelectionDown))
                        }
                        KeyCode::Left => {
                            return Some(Action::ScoresAction(ScoresAction::MoveGameSelectionLeft))
                        }
                        KeyCode::Right => {
                            return Some(Action::ScoresAction(ScoresAction::MoveGameSelectionRight))
                        }
                        KeyCode::Enter => {
                            return Some(Action::ScoresAction(ScoresAction::SelectGame))
                        }
                        _ => {}
                    }
                } else {
                    // Date navigation mode - arrows navigate dates
                    match key.code {
                        KeyCode::Left => {
                            return Some(Action::ScoresAction(ScoresAction::DateLeft))
                        }
                        KeyCode::Right => {
                            return Some(Action::ScoresAction(ScoresAction::DateRight))
                        }
                        KeyCode::Down => {
                            return Some(Action::ScoresAction(ScoresAction::EnterBoxSelection))
                        }
                        KeyCode::Enter => {
                            return Some(Action::ScoresAction(ScoresAction::SelectGame))
                        }
                        _ => {}
                    }
                }
            }
            Tab::Standings => {
                // On Standings tab: arrows navigate views (Division/Conference/League)
                match key.code {
                    KeyCode::Left => {
                        return Some(Action::StandingsAction(StandingsAction::CycleViewLeft))
                    }
                    KeyCode::Right => {
                        return Some(Action::StandingsAction(StandingsAction::CycleViewRight))
                    }
                    KeyCode::Down => {
                        return Some(Action::StandingsAction(StandingsAction::EnterTeamMode))
                    }
                    KeyCode::Enter => {
                        return Some(Action::StandingsAction(StandingsAction::SelectTeam))
                    }
                    _ => {}
                }
            }
            _ => {
                // Other tabs: no special content navigation yet
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_quit_key() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('q')), &state);
        assert!(matches!(action, Some(Action::Quit)));
    }

    #[test]
    fn test_number_keys_navigate_tabs() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('1')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Scores))));
    }

    #[test]
    fn test_esc_pops_panel_when_panel_open() {
        let mut state = AppState::default();
        state.navigation.panel_stack.push(super::super::state::PanelState {
            panel: super::super::action::Panel::Boxscore { game_id: 123 },
            scroll_offset: 0,
        });

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(action, Some(Action::PopPanel)));
    }

    #[test]
    fn test_esc_exits_content_focus_when_in_subtabs() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(action, Some(Action::ExitContentFocus)));
    }

    #[test]
    fn test_esc_at_tab_bar_does_nothing() {
        let state = AppState::default(); // content_focused = false by default

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(action.is_none(), "ESC at tab bar should do nothing, not quit");
    }

    #[test]
    fn test_q_quits_application() {
        let state = AppState::default();

        let action = key_to_action(make_key(KeyCode::Char('q')), &state);
        assert!(matches!(action, Some(Action::Quit)));
    }

    #[test]
    fn test_esc_exits_box_selection_mode() {
        let mut state = AppState::default();
        state.ui.scores.box_selection_active = true;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::ExitBoxSelection))
        ));
    }

    #[test]
    fn test_esc_exits_team_mode() {
        let mut state = AppState::default();
        state.ui.standings.team_mode = true;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::ExitTeamMode))
        ));
    }
}
