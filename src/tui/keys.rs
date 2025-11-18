/// Keyboard event to action mapping
///
/// This module handles converting crossterm KeyEvents into framework Actions.
/// It contains all the keyboard navigation logic for the TUI.

use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, trace};

use super::action::{Action, ScoresAction, SettingsAction, StandingsAction};
use super::state::AppState;
use super::types::{SettingsCategory, Tab};

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

        // Priority 3: If in browse mode on Standings tab, exit to view subtabs
        if state.ui.standings.browse_mode {
            debug!("KEY: ESC pressed in browse mode - exiting to view subtabs");
            return Some(Action::StandingsAction(StandingsAction::ExitBrowseMode));
        }

        // Priority 3.5: If in settings mode on Settings tab, exit to category subtabs
        if state.ui.settings.settings_mode {
            debug!("KEY: ESC pressed in settings mode - exiting to category subtabs");
            return Some(Action::SettingsAction(SettingsAction::ExitSettingsMode));
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

    // Panel navigation - handle Up/Down/Enter when a panel is open
    if !state.navigation.panel_stack.is_empty() {
        match key.code {
            KeyCode::Up => {
                debug!("KEY: Up pressed in panel - moving selection up");
                return Some(Action::PanelSelectPrevious);
            }
            KeyCode::Down => {
                debug!("KEY: Down pressed in panel - moving selection down");
                return Some(Action::PanelSelectNext);
            }
            KeyCode::Enter => {
                debug!("KEY: Enter pressed in panel - selecting item");
                return Some(Action::PanelSelectItem);
            }
            _ => {}
        }
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
            } else if state.ui.standings.browse_mode {
                // In browse mode - Up navigates teams
                return Some(Action::StandingsAction(StandingsAction::MoveSelectionUp));
            } else if state.ui.settings.modal_open || state.ui.settings.editing {
                // Modal or editing active - let tab-specific handler deal with it
                // Fall through to tab-specific handling below
            } else if state.ui.settings.settings_mode {
                // In settings navigation - Up navigates settings (unless at top)
                if state.ui.settings.selected_setting_index == 0 {
                    // At top - return to category tabs
                    debug!("KEY: Up pressed at top of settings - returning to category tabs");
                    return Some(Action::SettingsAction(SettingsAction::ExitSettingsMode));
                } else {
                    // Navigate settings
                    return Some(Action::SettingsAction(SettingsAction::MoveSelectionUp));
                }
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
                // On Standings tab: check if in browse mode
                if state.ui.standings.browse_mode {
                    // Browse mode - navigate teams and columns
                    match key.code {
                        KeyCode::Down => {
                            return Some(Action::StandingsAction(StandingsAction::MoveSelectionDown))
                        }
                        KeyCode::Left => {
                            // In Conference view, left/right switch between conferences
                            return Some(Action::StandingsAction(StandingsAction::MoveSelectionLeft))
                        }
                        KeyCode::Right => {
                            return Some(Action::StandingsAction(StandingsAction::MoveSelectionRight))
                        }
                        KeyCode::Enter => {
                            return Some(Action::StandingsAction(StandingsAction::SelectTeam))
                        }
                        _ => {}
                    }
                } else {
                    // View selection mode - arrows navigate views (Division/Conference/League)
                    match key.code {
                        KeyCode::Left => {
                            return Some(Action::StandingsAction(StandingsAction::CycleViewLeft))
                        }
                        KeyCode::Right => {
                            return Some(Action::StandingsAction(StandingsAction::CycleViewRight))
                        }
                        KeyCode::Down => {
                            return Some(Action::StandingsAction(StandingsAction::EnterBrowseMode))
                        }
                        _ => {}
                    }
                }
            }
            Tab::Settings => {
                // Check if modal is open
                if state.ui.settings.modal_open {
                    // In modal mode - handle modal navigation
                    match key.code {
                        KeyCode::Up => {
                            return Some(Action::SettingsAction(SettingsAction::ModalMoveUp))
                        }
                        KeyCode::Down => {
                            return Some(Action::SettingsAction(SettingsAction::ModalMoveDown))
                        }
                        KeyCode::Enter => {
                            return Some(Action::SettingsAction(SettingsAction::ModalConfirm))
                        }
                        KeyCode::Esc => {
                            return Some(Action::SettingsAction(SettingsAction::ModalCancel))
                        }
                        _ => {}
                    }
                } else if state.ui.settings.editing {
                    // In editing mode - handle text input (for non-list settings)
                    match key.code {
                        KeyCode::Char(ch) => {
                            return Some(Action::SettingsAction(SettingsAction::AppendChar(ch)))
                        }
                        KeyCode::Backspace => {
                            return Some(Action::SettingsAction(SettingsAction::DeleteChar))
                        }
                        KeyCode::Enter => {
                            // Commit the edit
                            let setting_key = get_editable_setting_key_for_index(
                                state.ui.settings.selected_category,
                                state.ui.settings.selected_setting_index,
                            );
                            if let Some(key) = setting_key {
                                return Some(Action::SettingsAction(SettingsAction::CommitEdit(key)));
                            }
                        }
                        KeyCode::Esc => {
                            return Some(Action::SettingsAction(SettingsAction::CancelEditing))
                        }
                        _ => {}
                    }
                } else if state.ui.settings.settings_mode {
                    // In settings navigation mode - arrows navigate settings, Enter activates
                    match key.code {
                        KeyCode::Up => {
                            return Some(Action::SettingsAction(SettingsAction::MoveSelectionUp))
                        }
                        KeyCode::Down => {
                            return Some(Action::SettingsAction(SettingsAction::MoveSelectionDown))
                        }
                        KeyCode::Enter => {
                            // Check if it's a boolean setting (toggle) or editable setting (start edit)
                            let boolean_key = get_setting_key_for_index(
                                state.ui.settings.selected_category,
                                state.ui.settings.selected_setting_index,
                            );
                            if let Some(key) = boolean_key {
                                return Some(Action::SettingsAction(SettingsAction::ToggleBoolean(key)));
                            }

                            let editable_key = get_editable_setting_key_for_index(
                                state.ui.settings.selected_category,
                                state.ui.settings.selected_setting_index,
                            );
                            if let Some(key) = editable_key {
                                return Some(Action::SettingsAction(SettingsAction::StartEditing(key)));
                            }
                        }
                        _ => {}
                    }
                } else {
                    // In category navigation mode - arrows navigate categories, Down enters settings
                    match key.code {
                        KeyCode::Left => {
                            return Some(Action::SettingsAction(SettingsAction::NavigateCategoryLeft))
                        }
                        KeyCode::Right => {
                            return Some(Action::SettingsAction(SettingsAction::NavigateCategoryRight))
                        }
                        KeyCode::Down => {
                            return Some(Action::SettingsAction(SettingsAction::EnterSettingsMode))
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // Other tabs: no special content navigation yet
            }
        }
    }

    None
}

/// Get the setting key for a given category and index (for boolean toggling)
fn get_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => None, // No boolean settings in Logging
        SettingsCategory::Display => {
            // Display category: index 0 = use_unicode
            match index {
                0 => Some("use_unicode".to_string()),
                _ => None,
            }
        }
        SettingsCategory::Data => {
            // Data category: index 1 = western_teams_first
            match index {
                1 => Some("western_teams_first".to_string()),
                _ => None,
            }
        }
    }
}

/// Get the editable setting key for a given category and index (for string/int editing)
fn get_editable_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => {
            // Logging category: 0 = log_level, 1 = log_file
            match index {
                0 => Some("log_level".to_string()),
                1 => Some("log_file".to_string()),
                _ => None,
            }
        }
        SettingsCategory::Display => {
            // Display category has boolean and color settings, not editable strings
            None
        }
        SettingsCategory::Data => {
            // Data category: 0 = refresh_interval, 2 = time_format
            match index {
                0 => Some("refresh_interval".to_string()),
                2 => Some("time_format".to_string()),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::commands::standings::GroupBy;

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
            panel: super::super::types::Panel::Boxscore { game_id: 123 },
            scroll_offset: 0,
            selected_index: None,
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
    fn test_esc_exits_browse_mode() {
        let mut state = AppState::default();
        state.ui.standings.browse_mode = true;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::ExitBrowseMode))
        ));
    }

    #[test]
    fn test_standings_browse_mode_down_arrow() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = true;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::MoveSelectionDown))
        ));
    }

    #[test]
    fn test_standings_browse_mode_up_arrow() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = true;

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::MoveSelectionUp))
        ));
    }

    #[test]
    fn test_standings_browse_mode_left_arrow_moves_selection() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = true;
        state.ui.standings.view = GroupBy::Conference; // Conference view has multiple columns

        let action = key_to_action(make_key(KeyCode::Left), &state);
        // Left arrow should move selection in Conference view
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::MoveSelectionLeft))
        ));
    }

    #[test]
    fn test_standings_browse_mode_right_arrow_moves_selection() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = true;
        state.ui.standings.view = GroupBy::Conference; // Conference view has multiple columns

        let action = key_to_action(make_key(KeyCode::Right), &state);
        // Right arrow should move selection in Conference view
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::MoveSelectionRight))
        ));
    }

    #[test]
    fn test_standings_view_mode_left_cycles_view() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = false;

        let action = key_to_action(make_key(KeyCode::Left), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::CycleViewLeft))
        ));
    }

    #[test]
    fn test_standings_view_mode_down_enters_browse_mode() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = false;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::EnterBrowseMode))
        ));
    }

    #[test]
    fn test_settings_modal_up_key_navigates_modal() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.modal_open = true;
        state.ui.settings.modal_selected_index = 1;

        let action = key_to_action(make_key(KeyCode::Up), &state);
        // Should navigate modal, not exit to tab bar or navigate settings
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ModalMoveUp))
        ));
    }

    #[test]
    fn test_settings_modal_down_key_navigates_modal() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.modal_open = true;
        state.ui.settings.modal_selected_index = 0;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        // Should navigate modal
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ModalMoveDown))
        ));
    }
}
