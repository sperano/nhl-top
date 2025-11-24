/// Keyboard event to action mapping
///
/// This module handles converting crossterm KeyEvents into framework Actions.
/// It contains all the keyboard navigation logic for the TUI.
use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, trace};

use crossterm::event::KeyModifiers;

use super::action::{Action, DocumentAction, ScoresAction, SettingsAction, StandingsAction};
use super::state::AppState;
use super::types::{SettingsCategory, Tab};

/// Handle global keys that work regardless of tab or focus state
fn handle_global_keys(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Char('/') => Some(Action::ToggleCommandPalette),
        _ => None,
    }
}

/// Handle ESC key with priority-based navigation up through focus hierarchy
fn handle_esc_key(state: &AppState) -> Option<Action> {
    // Priority 1: If there's a panel open, close it
    if !state.navigation.panel_stack.is_empty() {
        debug!("KEY: ESC pressed with panel open - popping panel");
        return Some(Action::PopPanel);
    }

    // Priority 2: If in modal on Settings tab, cancel modal
    if state.ui.settings.modal_open {
        debug!("KEY: ESC pressed in modal - canceling modal");
        return Some(Action::SettingsAction(SettingsAction::ModalCancel));
    }

    // Priority 2.5: If editing on Settings tab, cancel editing
    if state.ui.settings.editing {
        debug!("KEY: ESC pressed while editing - canceling edit");
        return Some(Action::SettingsAction(SettingsAction::CancelEditing));
    }

    // Priority 3: If in box selection mode on Scores tab, exit to date subtabs
    if state.ui.scores.box_selection_active {
        debug!("KEY: ESC pressed in box selection - exiting to date subtabs");
        return Some(Action::ScoresAction(ScoresAction::ExitBoxSelection));
    }

    // Priority 4: If in browse mode on Standings tab, exit to view subtabs
    if state.ui.standings.browse_mode {
        debug!("KEY: ESC pressed in browse mode - exiting to view subtabs");
        return Some(Action::StandingsAction(StandingsAction::ExitBrowseMode));
    }

    // Priority 5: If in settings mode on Settings tab, exit to category subtabs
    if state.ui.settings.settings_mode {
        debug!("KEY: ESC pressed in settings mode - exiting to category subtabs");
        return Some(Action::SettingsAction(SettingsAction::ExitSettingsMode));
    }

    // Priority 6: If content is focused, return to tab bar
    if state.navigation.content_focused {
        debug!("KEY: ESC pressed in content - returning to tab bar");
        return Some(Action::ExitContentFocus);
    }

    // Priority 7: At top level (tab bar), do nothing - use 'q' to quit
    debug!("KEY: ESC pressed at tab bar - ignoring (use 'q' to quit)");
    None
}

/// Handle navigation when a panel is open
fn handle_panel_navigation(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Up => {
            debug!("KEY: Up pressed in panel - moving selection up");
            Some(Action::PanelSelectPrevious)
        }
        KeyCode::Down => {
            debug!("KEY: Down pressed in panel - moving selection down");
            Some(Action::PanelSelectNext)
        }
        KeyCode::Enter => {
            debug!("KEY: Enter pressed in panel - selecting item");
            Some(Action::PanelSelectItem)
        }
        _ => None,
    }
}

/// Handle direct tab switching via number keys (1-6)
fn handle_number_keys(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Char('1') => Some(Action::NavigateTab(Tab::Scores)),
        KeyCode::Char('2') => Some(Action::NavigateTab(Tab::Standings)),
        KeyCode::Char('3') => Some(Action::NavigateTab(Tab::Stats)),
        KeyCode::Char('4') => Some(Action::NavigateTab(Tab::Players)),
        KeyCode::Char('5') => Some(Action::NavigateTab(Tab::Settings)),
        KeyCode::Char('6') => Some(Action::NavigateTab(Tab::Demo)),
        _ => None,
    }
}

/// Handle navigation when tab bar is focused (Left/Right/Down)
fn handle_tab_bar_navigation(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Left => Some(Action::NavigateTabLeft),
        KeyCode::Right => Some(Action::NavigateTabRight),
        KeyCode::Down => {
            debug!("KEY: Down pressed on tab bar - entering content focus");
            Some(Action::EnterContentFocus)
        }
        _ => None,
    }
}

/// Handle Scores tab navigation (box selection mode vs date mode)
fn handle_scores_tab_keys(key_code: KeyCode, state: &AppState) -> Option<Action> {
    if state.ui.scores.box_selection_active {
        // Box selection mode - arrows navigate within game grid
        match key_code {
            KeyCode::Down => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionDown)),
            KeyCode::Left => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionLeft)),
            KeyCode::Right => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionRight)),
            KeyCode::Enter => Some(Action::ScoresAction(ScoresAction::SelectGame)),
            _ => None,
        }
    } else {
        // Date navigation mode - arrows navigate dates
        match key_code {
            KeyCode::Left => Some(Action::ScoresAction(ScoresAction::DateLeft)),
            KeyCode::Right => Some(Action::ScoresAction(ScoresAction::DateRight)),
            KeyCode::Down => Some(Action::ScoresAction(ScoresAction::EnterBoxSelection)),
            KeyCode::Enter => Some(Action::ScoresAction(ScoresAction::SelectGame)),
            _ => None,
        }
    }
}

/// Handle Standings tab navigation (browse mode vs view selection mode)
fn handle_standings_tab_keys(key_code: KeyCode, state: &AppState) -> Option<Action> {
    if state.ui.standings.browse_mode {
        // Browse mode - navigate teams and columns
        match key_code {
            KeyCode::Down => Some(Action::StandingsAction(StandingsAction::MoveSelectionDown)),
            KeyCode::Left => Some(Action::StandingsAction(StandingsAction::MoveSelectionLeft)),
            KeyCode::Right => Some(Action::StandingsAction(StandingsAction::MoveSelectionRight)),
            KeyCode::Enter => Some(Action::StandingsAction(StandingsAction::SelectTeam)),
            KeyCode::PageDown => Some(Action::StandingsAction(StandingsAction::PageDown)),
            KeyCode::PageUp => Some(Action::StandingsAction(StandingsAction::PageUp)),
            KeyCode::Home => Some(Action::StandingsAction(StandingsAction::GoToTop)),
            KeyCode::End => Some(Action::StandingsAction(StandingsAction::GoToBottom)),
            _ => None,
        }
    } else {
        // View selection mode - arrows navigate views (Division/Conference/League)
        match key_code {
            KeyCode::Left => Some(Action::StandingsAction(StandingsAction::CycleViewLeft)),
            KeyCode::Right => Some(Action::StandingsAction(StandingsAction::CycleViewRight)),
            KeyCode::Down => Some(Action::StandingsAction(StandingsAction::EnterBrowseMode)),
            _ => None,
        }
    }
}

/// Handle Settings tab navigation (modal, editing, settings mode, or category mode)
fn handle_settings_tab_keys(key_code: KeyCode, state: &AppState) -> Option<Action> {
    // Check if modal is open
    if state.ui.settings.modal_open {
        // In modal mode - handle modal navigation
        return match key_code {
            KeyCode::Up => Some(Action::SettingsAction(SettingsAction::ModalMoveUp)),
            KeyCode::Down => Some(Action::SettingsAction(SettingsAction::ModalMoveDown)),
            KeyCode::Enter => Some(Action::SettingsAction(SettingsAction::ModalConfirm)),
            _ => None,
        };
    }

    // Check if editing
    if state.ui.settings.editing {
        // In editing mode - handle text input
        return match key_code {
            KeyCode::Char(ch) => Some(Action::SettingsAction(SettingsAction::AppendChar(ch))),
            KeyCode::Backspace => Some(Action::SettingsAction(SettingsAction::DeleteChar)),
            KeyCode::Enter => {
                // Commit the edit
                let setting_key = get_editable_setting_key_for_index(
                    state.ui.settings.selected_category,
                    state.ui.settings.selected_setting_index,
                );
                setting_key.map(|key| Action::SettingsAction(SettingsAction::CommitEdit(key)))
            }
            _ => None,
        };
    }

    // Check if in settings navigation mode
    if state.ui.settings.settings_mode {
        // In settings navigation mode - arrows navigate settings, Enter activates
        return match key_code {
            KeyCode::Up => Some(Action::SettingsAction(SettingsAction::MoveSelectionUp)),
            KeyCode::Down => Some(Action::SettingsAction(SettingsAction::MoveSelectionDown)),
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
                editable_key.map(|key| Action::SettingsAction(SettingsAction::StartEditing(key)))
            }
            _ => None,
        };
    }

    // In category navigation mode - arrows navigate categories, Down enters settings
    match key_code {
        KeyCode::Left => Some(Action::SettingsAction(SettingsAction::NavigateCategoryLeft)),
        KeyCode::Right => Some(Action::SettingsAction(
            SettingsAction::NavigateCategoryRight,
        )),
        KeyCode::Down => Some(Action::SettingsAction(SettingsAction::EnterSettingsMode)),
        _ => None,
    }
}

/// Handle Demo tab navigation (document system with Tab/Shift-Tab focus navigation)
fn handle_demo_tab_keys(key: KeyEvent, _state: &AppState) -> Option<Action> {
    match key.code {
        // Tab key for focus navigation
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                debug!("KEY: Shift-Tab in Demo tab - focus previous");
                Some(Action::DocumentAction(DocumentAction::FocusPrev))
            } else {
                debug!("KEY: Tab in Demo tab - focus next");
                Some(Action::DocumentAction(DocumentAction::FocusNext))
            }
        }
        KeyCode::BackTab => {
            debug!("KEY: BackTab in Demo tab - focus previous");
            Some(Action::DocumentAction(DocumentAction::FocusPrev))
        }
        // Enter to activate focused element
        KeyCode::Enter => {
            debug!("KEY: Enter in Demo tab - activate focused");
            Some(Action::DocumentAction(DocumentAction::ActivateFocused))
        }
        // Arrow keys for scrolling
        KeyCode::Down => Some(Action::DocumentAction(DocumentAction::ScrollDown(1))),
        KeyCode::Left => Some(Action::DocumentAction(DocumentAction::ScrollUp(1))),
        KeyCode::Right => Some(Action::DocumentAction(DocumentAction::ScrollDown(1))),
        // Page navigation
        KeyCode::PageUp => Some(Action::DocumentAction(DocumentAction::PageUp)),
        KeyCode::PageDown => Some(Action::DocumentAction(DocumentAction::PageDown)),
        KeyCode::Home => Some(Action::DocumentAction(DocumentAction::ScrollToTop)),
        KeyCode::End => Some(Action::DocumentAction(DocumentAction::ScrollToBottom)),
        _ => None,
    }
}

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

    // 1. Check global keys (q/Q, /)
    if let Some(action) = handle_global_keys(key.code) {
        return Some(action);
    }

    // 2. Check ESC key (7-priority hierarchy)
    if key.code == KeyCode::Esc {
        return handle_esc_key(state);
    }

    // 3. Check panel navigation (when panel is open)
    if !state.navigation.panel_stack.is_empty() {
        if let Some(action) = handle_panel_navigation(key.code) {
            return Some(action);
        }
    }

    // 4. Check number keys for direct tab switching
    if let Some(action) = handle_number_keys(key.code) {
        return Some(action);
    }

    // 5. Handle navigation based on focus level
    if !content_focused {
        // TAB BAR FOCUSED: delegate to tab bar handler
        return handle_tab_bar_navigation(key.code);
    }

    // CONTENT FOCUSED: context-sensitive navigation

    // 6. Handle Up key with special logic (returns to tab bar unless in nested mode)
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
        } else if current_tab == Tab::Demo {
            // Demo tab - Up focuses previous element (wraps around, doesn't exit)
            debug!("KEY: Up pressed in Demo tab - focus previous");
            return Some(Action::DocumentAction(DocumentAction::FocusPrev));
        } else {
            // Not in nested mode - Up returns to tab bar
            debug!("KEY: Up pressed in content - returning to tab bar");
            return Some(Action::ExitContentFocus);
        }
    }

    // 7. Delegate to tab-specific handlers
    match current_tab {
        Tab::Scores => handle_scores_tab_keys(key.code, state),
        Tab::Standings => handle_standings_tab_keys(key.code, state),
        Tab::Settings => handle_settings_tab_keys(key.code, state),
        Tab::Demo => handle_demo_tab_keys(key, state),
        _ => None, // Other tabs: no special content navigation yet
    }
}

/// Get the setting key for a given category and index (for boolean toggling)
fn get_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => None, // No boolean settings in Logging
        SettingsCategory::Display => {
            // Display category: index 1 = use_unicode (Theme is at 0)
            match index {
                1 => Some("use_unicode".to_string()),
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
            // Display category: 0 = theme (list-based editable)
            match index {
                0 => Some("theme".to_string()),
                _ => None,
            }
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
    use crate::commands::standings::GroupBy;
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
        state
            .navigation
            .panel_stack
            .push(super::super::state::PanelState {
                panel: super::super::types::Panel::Boxscore { game_id: 123 },
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
        assert!(
            action.is_none(),
            "ESC at tab bar should do nothing, not quit"
        );
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

    // Global keys tests
    #[test]
    fn test_slash_toggles_command_palette() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('/')), &state);
        assert!(matches!(action, Some(Action::ToggleCommandPalette)));
    }

    #[test]
    fn test_uppercase_q_quits() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('Q')), &state);
        assert!(matches!(action, Some(Action::Quit)));
    }

    // ESC priority tests
    #[test]
    fn test_esc_exits_settings_mode() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ExitSettingsMode))
        ));
    }

    // Panel navigation tests
    #[test]
    fn test_panel_up_key_selects_previous() {
        let mut state = AppState::default();
        state
            .navigation
            .panel_stack
            .push(super::super::state::PanelState {
                panel: super::super::types::Panel::Boxscore { game_id: 123 },
                selected_index: Some(1),
            });

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(action, Some(Action::PanelSelectPrevious)));
    }

    #[test]
    fn test_panel_down_key_selects_next() {
        let mut state = AppState::default();
        state
            .navigation
            .panel_stack
            .push(super::super::state::PanelState {
                panel: super::super::types::Panel::Boxscore { game_id: 123 },
                selected_index: Some(0),
            });

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(action, Some(Action::PanelSelectNext)));
    }

    #[test]
    fn test_panel_enter_key_selects_item() {
        let mut state = AppState::default();
        state
            .navigation
            .panel_stack
            .push(super::super::state::PanelState {
                panel: super::super::types::Panel::Boxscore { game_id: 123 },
                selected_index: Some(0),
            });

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(action, Some(Action::PanelSelectItem)));
    }

    // Number keys for all tabs
    #[test]
    fn test_number_key_2_navigates_to_standings() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('2')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Standings))));
    }

    #[test]
    fn test_number_key_3_navigates_to_stats() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('3')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Stats))));
    }

    #[test]
    fn test_number_key_4_navigates_to_players() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('4')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Players))));
    }

    #[test]
    fn test_number_key_5_navigates_to_settings() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('5')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Settings))));
    }

    #[test]
    fn test_number_key_6_navigates_to_browser() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::Char('6')), &state);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Demo))));
    }

    // Tab bar navigation tests
    #[test]
    fn test_tab_bar_left_arrow_navigates_left() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Left), &state);
        assert!(matches!(action, Some(Action::NavigateTabLeft)));
    }

    #[test]
    fn test_tab_bar_right_arrow_navigates_right() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Right), &state);
        assert!(matches!(action, Some(Action::NavigateTabRight)));
    }

    #[test]
    fn test_tab_bar_down_arrow_enters_content_focus() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(action, Some(Action::EnterContentFocus)));
    }

    // Content focus Up key returning to tab bar
    #[test]
    fn test_content_up_key_returns_to_tab_bar() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;
        state.navigation.current_tab = Tab::Scores;
        // Not in box selection mode
        state.ui.scores.box_selection_active = false;

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(action, Some(Action::ExitContentFocus)));
    }

    // Scores tab - box selection mode
    #[test]
    fn test_scores_box_selection_up() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = true;

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionUp))
        ));
    }

    #[test]
    fn test_scores_box_selection_down() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = true;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionDown))
        ));
    }

    #[test]
    fn test_scores_box_selection_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = true;

        let action = key_to_action(make_key(KeyCode::Left), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionLeft))
        ));
    }

    #[test]
    fn test_scores_box_selection_right() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = true;

        let action = key_to_action(make_key(KeyCode::Right), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionRight))
        ));
    }

    #[test]
    fn test_scores_box_selection_enter() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = true;

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::SelectGame))
        ));
    }

    // Scores tab - date mode
    #[test]
    fn test_scores_date_mode_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = false;

        let action = key_to_action(make_key(KeyCode::Left), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::DateLeft))
        ));
    }

    #[test]
    fn test_scores_date_mode_right() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = false;

        let action = key_to_action(make_key(KeyCode::Right), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::DateRight))
        ));
    }

    #[test]
    fn test_scores_date_mode_down_enters_box_selection() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = false;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::EnterBoxSelection))
        ));
    }

    #[test]
    fn test_scores_date_mode_enter() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        state.ui.scores.box_selection_active = false;

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::SelectGame))
        ));
    }

    // Standings tab
    #[test]
    fn test_standings_browse_mode_enter() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = true;

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::SelectTeam))
        ));
    }

    #[test]
    fn test_standings_view_mode_right_cycles_view() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        state.ui.standings.browse_mode = false;

        let action = key_to_action(make_key(KeyCode::Right), &state);
        assert!(matches!(
            action,
            Some(Action::StandingsAction(StandingsAction::CycleViewRight))
        ));
    }

    // Settings tab - modal
    #[test]
    fn test_settings_modal_enter_confirms() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.modal_open = true;

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ModalConfirm))
        ));
    }

    #[test]
    fn test_settings_modal_esc_cancels() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.modal_open = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ModalCancel))
        ));
    }

    // Settings tab - editing mode
    #[test]
    fn test_settings_editing_char_input() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.editing = true;
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 1; // log_file

        let action = key_to_action(make_key(KeyCode::Char('a')), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::AppendChar('a')))
        ));
    }

    #[test]
    fn test_settings_editing_backspace() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.editing = true;

        let action = key_to_action(make_key(KeyCode::Backspace), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::DeleteChar))
        ));
    }

    #[test]
    fn test_settings_editing_enter_commits() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.editing = true;
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 1; // log_file

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::CommitEdit(_)))
        ));
    }

    #[test]
    fn test_settings_editing_esc_cancels() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.editing = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::CancelEditing))
        ));
    }

    // Settings tab - settings navigation mode
    #[test]
    fn test_settings_mode_up_at_top_exits_to_categories() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;
        state.ui.settings.selected_setting_index = 0; // At top

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ExitSettingsMode))
        ));
    }

    #[test]
    fn test_settings_mode_up_navigates_settings() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;
        state.ui.settings.selected_setting_index = 1; // Not at top

        let action = key_to_action(make_key(KeyCode::Up), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::MoveSelectionUp))
        ));
    }

    #[test]
    fn test_settings_mode_down_navigates_settings() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::MoveSelectionDown))
        ));
    }

    #[test]
    fn test_settings_mode_enter_toggles_boolean() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 1; // use_unicode (index 0 is theme)

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::ToggleBoolean(_)))
        ));
    }

    #[test]
    fn test_settings_mode_enter_starts_editing() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 0; // log_level

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::StartEditing(_)))
        ));
    }

    // Settings tab - category navigation
    #[test]
    fn test_settings_category_mode_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = false;

        let action = key_to_action(make_key(KeyCode::Left), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::NavigateCategoryLeft))
        ));
    }

    #[test]
    fn test_settings_category_mode_right() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = false;

        let action = key_to_action(make_key(KeyCode::Right), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(
                SettingsAction::NavigateCategoryRight
            ))
        ));
    }

    #[test]
    fn test_settings_category_mode_down_enters_settings() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = false;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::SettingsAction(SettingsAction::EnterSettingsMode))
        ));
    }

    // Helper function tests - get_setting_key_for_index
    #[test]
    fn test_get_setting_key_logging_returns_none() {
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Logging, 0),
            None
        );
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Logging, 1),
            None
        );
    }

    #[test]
    fn test_get_setting_key_display_use_unicode() {
        // Display index 1 is Use Unicode (index 0 is Theme)
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Display, 1),
            Some("use_unicode".to_string())
        );
    }

    #[test]
    fn test_get_setting_key_display_other_indices() {
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Display, 0),
            None
        ); // Theme is editable, not boolean
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Display, 2),
            None
        );
    }

    #[test]
    fn test_get_setting_key_data_western_teams_first() {
        assert_eq!(
            get_setting_key_for_index(SettingsCategory::Data, 1),
            Some("western_teams_first".to_string())
        );
    }

    #[test]
    fn test_get_setting_key_data_other_indices() {
        assert_eq!(get_setting_key_for_index(SettingsCategory::Data, 0), None);
        assert_eq!(get_setting_key_for_index(SettingsCategory::Data, 2), None);
    }

    // Helper function tests - get_editable_setting_key_for_index
    #[test]
    fn test_get_editable_setting_key_logging_log_level() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Logging, 0),
            Some("log_level".to_string())
        );
    }

    #[test]
    fn test_get_editable_setting_key_logging_log_file() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Logging, 1),
            Some("log_file".to_string())
        );
    }

    #[test]
    fn test_get_editable_setting_key_logging_out_of_bounds() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Logging, 2),
            None
        );
    }

    #[test]
    fn test_get_editable_setting_key_display_theme() {
        // Display index 0 is Theme (list-based editable)
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Display, 0),
            Some("theme".to_string())
        );
    }

    #[test]
    fn test_get_editable_setting_key_display_other_indices() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Display, 1),
            None
        ); // Use Unicode is boolean
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Display, 2),
            None
        ); // Colors not editable
    }

    #[test]
    fn test_get_editable_setting_key_data_refresh_interval() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Data, 0),
            Some("refresh_interval".to_string())
        );
    }

    #[test]
    fn test_get_editable_setting_key_data_time_format() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Data, 2),
            Some("time_format".to_string())
        );
    }

    #[test]
    fn test_get_editable_setting_key_data_other_indices() {
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Data, 1),
            None
        );
        assert_eq!(
            get_editable_setting_key_for_index(SettingsCategory::Data, 3),
            None
        );
    }

    // Edge cases
    #[test]
    fn test_other_tabs_with_no_content_navigation() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Stats;
        state.navigation.content_focused = true;

        // Left/Right/Down should return None for tabs with no navigation
        assert!(key_to_action(make_key(KeyCode::Left), &state).is_none());
        assert!(key_to_action(make_key(KeyCode::Right), &state).is_none());
        assert!(key_to_action(make_key(KeyCode::Down), &state).is_none());
    }

    #[test]
    fn test_unknown_key_returns_none() {
        let state = AppState::default();
        let action = key_to_action(make_key(KeyCode::F(1)), &state);
        assert!(action.is_none());
    }

    #[test]
    fn test_settings_editing_enter_with_no_editable_key() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.editing = true;
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 2; // Selection Color - not editable

        let action = key_to_action(make_key(KeyCode::Enter), &state);
        // Should return None since color settings aren't editable
        assert!(action.is_none());
    }

    // Demo tab tests
    #[test]
    fn test_demo_tab_up_focuses_previous() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Up), &state);
        // Up should focus previous element, NOT exit to tab bar
        assert!(matches!(
            action,
            Some(Action::DocumentAction(DocumentAction::FocusPrev))
        ));
    }

    #[test]
    fn test_demo_tab_down_scrolls() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Down), &state);
        assert!(matches!(
            action,
            Some(Action::DocumentAction(DocumentAction::ScrollDown(1)))
        ));
    }

    #[test]
    fn test_demo_tab_tab_focuses_next() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), &state);
        assert!(matches!(
            action,
            Some(Action::DocumentAction(DocumentAction::FocusNext))
        ));
    }

    #[test]
    fn test_demo_tab_shift_tab_focuses_prev() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT), &state);
        assert!(matches!(
            action,
            Some(Action::DocumentAction(DocumentAction::FocusPrev))
        ));
    }
}
