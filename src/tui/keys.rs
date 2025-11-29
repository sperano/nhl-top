/// Keyboard event to action mapping
///
/// This module handles converting crossterm KeyEvents into framework Actions.
/// It contains all the keyboard navigation logic for the TUI.
use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, trace};

use crossterm::event::KeyModifiers;

use super::action::{Action, ScoresAction, SettingsAction, StandingsAction};
use super::component_store::ComponentStateStore;
use super::components::scores_tab::ScoresTabState;
use super::components::standings_tab::StandingsTabState;
use super::state::AppState;
use super::types::Tab;

/// Helper to check if scores tab is in browse mode (box selection)
fn is_scores_browse_mode_active(component_states: &ComponentStateStore) -> bool {
    component_states
        .get::<ScoresTabState>("app/scores_tab")
        .map(|s| s.is_browse_mode())
        .unwrap_or(false)
}

/// Helper to check if standings tab is in browse mode
fn is_standings_browse_mode_active(component_states: &ComponentStateStore) -> bool {
    component_states
        .get::<StandingsTabState>("app/standings_tab")
        .map(|s| s.is_browse_mode())
        .unwrap_or(false)
}

/// Helper to check if settings tab has modal open
fn is_settings_modal_open(component_states: &ComponentStateStore) -> bool {
    use super::components::settings_tab::SettingsTabState;
    component_states
        .get::<SettingsTabState>("app/settings_tab")
        .map(|s| s.modal.is_some())
        .unwrap_or(false)
}

/// Handle global keys that work regardless of tab or focus state
fn handle_global_keys(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),
        KeyCode::Char('/') => Some(Action::ToggleCommandPalette),
        _ => None,
    }
}

/// Handle ESC key with priority-based navigation up through focus hierarchy
fn handle_esc_key(state: &AppState, component_states: &ComponentStateStore) -> Option<Action> {
    use crate::tui::components::settings_tab::{ModalMsg, SettingsTabMsg};

    // Priority 1: If there's a panel open, close it
    if !state.navigation.panel_stack.is_empty() {
        debug!("KEY: ESC pressed with panel open - popping panel");
        return Some(Action::PopPanel);
    }

    // Priority 2: If settings modal is open, close it
    if is_settings_modal_open(component_states) {
        debug!("KEY: ESC pressed with settings modal open - closing modal");
        return Some(Action::ComponentMessage {
            path: "app/settings_tab".to_string(),
            message: Box::new(SettingsTabMsg::Modal(ModalMsg::Cancel)),
        });
    }

    // Priority 3: If in box selection mode on Scores tab, exit to date subtabs
    if is_scores_browse_mode_active(component_states) {
        debug!("KEY: ESC pressed in box selection - exiting to date subtabs");
        return Some(Action::ScoresAction(ScoresAction::ExitBoxSelection));
    }

    // Priority 4: If in browse mode on Standings tab, exit to view subtabs
    if is_standings_browse_mode_active(component_states) {
        debug!("KEY: ESC pressed in browse mode - exiting to view subtabs");
        return Some(Action::StandingsAction(StandingsAction::ExitBrowseMode));
    }

    // Priority 5: If content is focused, return to tab bar
    if state.navigation.content_focused {
        debug!("KEY: ESC pressed in content - returning to tab bar");
        return Some(Action::ExitContentFocus);
    }

    // Priority 6: At top level (tab bar), do nothing - use 'q' to quit
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

/// Handle direct tab switching via number keys (1-4)
fn handle_number_keys(key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Char('1') => Some(Action::NavigateTab(Tab::Scores)),
        KeyCode::Char('2') => Some(Action::NavigateTab(Tab::Standings)),
        KeyCode::Char('3') => Some(Action::NavigateTab(Tab::Settings)),
        KeyCode::Char('4') => Some(Action::NavigateTab(Tab::Demo)),
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
fn handle_scores_tab_keys(
    state: &AppState,
    key_code: KeyCode,
    component_states: &ComponentStateStore,
) -> Option<Action> {
    if is_scores_browse_mode_active(component_states) {
        // Box selection mode - arrows navigate within game grid
        // Calculate boxes_per_row from cached terminal width
        use crate::layout_constants::GAME_BOX_WITH_MARGIN;
        let boxes_per_row = (state.system.terminal_width / GAME_BOX_WITH_MARGIN).max(1);

        match key_code {
            KeyCode::Down => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionDown(boxes_per_row))),
            KeyCode::Left => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionLeft)),
            KeyCode::Right => Some(Action::ScoresAction(ScoresAction::MoveGameSelectionRight)),
            KeyCode::Enter => {
                // Look up game_id from component state and schedule
                use crate::tui::components::scores_tab::ScoresTabState;

                if let Some(scores_state) = component_states.get::<ScoresTabState>("app/scores_tab") {
                    if let Some(selected_index) = scores_state.doc_nav.focus_index {
                        if let Some(schedule) = state.data.schedule.as_ref().as_ref() {
                            if let Some(game) = schedule.games.get(selected_index) {
                                return Some(Action::ScoresAction(ScoresAction::SelectGame(game.id)));
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    } else {
        // Date navigation mode - arrows navigate dates
        match key_code {
            KeyCode::Left => Some(Action::ScoresAction(ScoresAction::DateLeft)),
            KeyCode::Right => Some(Action::ScoresAction(ScoresAction::DateRight)),
            KeyCode::Down => Some(Action::ScoresAction(ScoresAction::EnterBoxSelection)),
            KeyCode::Enter => {
                // Look up game_id from component state and schedule (first game)
                use crate::tui::components::scores_tab::ScoresTabState;

                if let Some(_scores_state) = component_states.get::<ScoresTabState>("app/scores_tab") {
                    if let Some(schedule) = state.data.schedule.as_ref().as_ref() {
                        if let Some(game) = schedule.games.first() {
                            return Some(Action::ScoresAction(ScoresAction::SelectGame(game.id)));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

/// Handle League standings navigation with document system (Phase 7: Routes to component)
fn handle_standings_league_keys(key: KeyEvent, _state: &AppState) -> Option<Action> {
    use crate::tui::components::standings_tab::StandingsTabMsg;
    use crate::tui::document_nav::DocumentNavMsg;

    let nav_msg = match key.code {
        // Tab key for focus navigation
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                DocumentNavMsg::FocusPrev
            } else {
                DocumentNavMsg::FocusNext
            }
        }
        KeyCode::BackTab => DocumentNavMsg::FocusPrev,
        // Enter to activate focused element - TODO: handle activation
        KeyCode::Enter => return None,
        // Up/Down arrows for focus navigation
        KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::FocusPrev,
        KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::FocusNext,
        // Left/Right arrows for Row navigation (Conference view)
        KeyCode::Left if !key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::FocusLeft,
        KeyCode::Right if !key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::FocusRight,
        // Shift+Arrow keys for scrolling
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollDown(1),
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollUp(1),
        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollUp(1),
        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollDown(1),
        // Page navigation
        KeyCode::PageUp => DocumentNavMsg::PageUp,
        KeyCode::PageDown => DocumentNavMsg::PageDown,
        KeyCode::Home => DocumentNavMsg::ScrollToTop,
        KeyCode::End => DocumentNavMsg::ScrollToBottom,
        _ => return None,
    };

    Some(Action::ComponentMessage {
        path: "app/standings_tab".to_string(),
        message: Box::new(StandingsTabMsg::DocNav(nav_msg)),
    })
}

/// Handle Standings tab view selection mode (cycling between Division/Conference/League/Wildcard)
/// Note: Browse mode is handled by handle_standings_league_keys via document navigation
fn handle_standings_tab_keys(key_code: KeyCode, _state: &AppState) -> Option<Action> {
    // View selection mode - arrows navigate views
    match key_code {
        KeyCode::Left => Some(Action::StandingsAction(StandingsAction::CycleViewLeft)),
        KeyCode::Right => Some(Action::StandingsAction(StandingsAction::CycleViewRight)),
        KeyCode::Down => Some(Action::StandingsAction(StandingsAction::EnterBrowseMode)),
        _ => None,
    }
}

/// Handle Settings tab navigation
fn handle_settings_tab_keys(key: KeyEvent, state: &AppState, component_states: &ComponentStateStore) -> Option<Action> {
    use crate::tui::components::settings_tab::{ModalMsg, SettingsTabMsg};
    use crate::tui::document_nav::DocumentNavMsg;

    // Check if modal is open - if so, handle modal navigation first
    if is_settings_modal_open(component_states) {
        return match key.code {
            KeyCode::Up => Some(Action::ComponentMessage {
                path: "app/settings_tab".to_string(),
                message: Box::new(SettingsTabMsg::Modal(ModalMsg::Up)),
            }),
            KeyCode::Down => Some(Action::ComponentMessage {
                path: "app/settings_tab".to_string(),
                message: Box::new(SettingsTabMsg::Modal(ModalMsg::Down)),
            }),
            KeyCode::Enter => Some(Action::ComponentMessage {
                path: "app/settings_tab".to_string(),
                message: Box::new(SettingsTabMsg::Modal(ModalMsg::Confirm)),
            }),
            KeyCode::Esc => Some(Action::ComponentMessage {
                path: "app/settings_tab".to_string(),
                message: Box::new(SettingsTabMsg::Modal(ModalMsg::Cancel)),
            }),
            _ => None,
        };
    }

    // No modal open - handle normal navigation
    // Left/Right always navigate categories
    match key.code {
        KeyCode::Left => return Some(Action::SettingsAction(SettingsAction::NavigateCategoryLeft)),
        KeyCode::Right => return Some(Action::SettingsAction(SettingsAction::NavigateCategoryRight)),
        _ => {}
    }

    // Handle document navigation within the current category
    let nav_msg = match key.code {
        // Tab key for focus navigation
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                debug!("KEY: Shift-Tab in Settings tab - focus previous");
                DocumentNavMsg::FocusPrev
            } else {
                debug!("KEY: Tab in Settings tab - focus next");
                DocumentNavMsg::FocusNext
            }
        }
        // Arrow keys for focus navigation
        KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            debug!("KEY: Up in Settings tab - focus previous");
            DocumentNavMsg::FocusPrev
        }
        KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            debug!("KEY: Down in Settings tab - focus next");
            DocumentNavMsg::FocusNext
        }
        // Shift+Arrow keys for scrolling
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollDown(1),
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollUp(1),
        // Page navigation
        KeyCode::PageUp => DocumentNavMsg::PageUp,
        KeyCode::PageDown => DocumentNavMsg::PageDown,
        KeyCode::Home => DocumentNavMsg::ScrollToTop,
        KeyCode::End => DocumentNavMsg::ScrollToBottom,
        // Enter key activates the focused setting
        KeyCode::Enter => {
            debug!("KEY: Enter in Settings tab - activate focused setting");
            return Some(Action::ComponentMessage {
                path: "app/settings_tab".to_string(),
                message: Box::new(SettingsTabMsg::ActivateSetting(state.system.config.clone())),
            });
        }
        _ => return None,
    };

    Some(Action::ComponentMessage {
        path: "app/settings_tab".to_string(),
        message: Box::new(SettingsTabMsg::DocNav(nav_msg)),
    })
}

/// Handle Demo tab navigation (Phase 7: Routes to component)
fn handle_demo_tab_keys(key: KeyEvent, _state: &AppState) -> Option<Action> {
    use crate::tui::components::demo_tab::DemoTabMessage;
    use crate::tui::document_nav::DocumentNavMsg;

    let nav_msg = match key.code {
        // Tab key for focus navigation
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                debug!("KEY: Shift-Tab in Demo tab - focus previous");
                DocumentNavMsg::FocusPrev
            } else {
                debug!("KEY: Tab in Demo tab - focus next");
                DocumentNavMsg::FocusNext
            }
        }
        KeyCode::BackTab => {
            debug!("KEY: BackTab in Demo tab - focus previous");
            DocumentNavMsg::FocusPrev
        }
        // Enter to activate focused element - TODO: handle activation
        KeyCode::Enter => {
            debug!("KEY: Enter in Demo tab - activate focused");
            return None;
        }
        // Left/Right arrows for row navigation (jump between side-by-side elements)
        KeyCode::Left => {
            debug!("KEY: Left in Demo tab - focus left in row");
            DocumentNavMsg::FocusLeft
        }
        KeyCode::Right => {
            debug!("KEY: Right in Demo tab - focus right in row");
            DocumentNavMsg::FocusRight
        }
        // Up/Down arrows for focus navigation
        KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            debug!("KEY: Up in Demo tab - focus previous");
            DocumentNavMsg::FocusPrev
        }
        KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            debug!("KEY: Down in Demo tab - focus next");
            DocumentNavMsg::FocusNext
        }
        // Shift+Arrow keys for scrolling
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollDown(1),
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => DocumentNavMsg::ScrollUp(1),
        // Page navigation
        KeyCode::PageUp => DocumentNavMsg::PageUp,
        KeyCode::PageDown => DocumentNavMsg::PageDown,
        KeyCode::Home => DocumentNavMsg::ScrollToTop,
        KeyCode::End => DocumentNavMsg::ScrollToBottom,
        _ => return None,
    };

    Some(Action::ComponentMessage {
        path: "app/demo_tab".to_string(),
        message: Box::new(DemoTabMessage::DocNav(nav_msg)),
    })
}

/// Convert a KeyEvent into an Action based on current application state
///
/// This function implements all keyboard navigation:
/// - Global keys (q, /, ESC)
/// - Tab bar focus: Left/Right navigate tabs, Down enters content
/// - Content focus: Context-sensitive navigation, Up returns to tab bar
/// - Panel navigation (ESC to close)
///
/// Phase 7: Now reads from component state instead of global state for component-specific checks
pub fn key_to_action(
    key: KeyEvent,
    state: &AppState,
    component_states: &crate::tui::component_store::ComponentStateStore,
) -> Option<Action> {
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
        return handle_esc_key(state, component_states);
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
        let action = handle_tab_bar_navigation(key.code);
        if action.is_some() {
            debug!("KEY: Tab bar navigation: {:?}", action);
        }
        return action;
    }

    // CONTENT FOCUSED: context-sensitive navigation

    // 6. Handle Up key with special logic (returns to tab bar unless in nested mode)
    if key.code == KeyCode::Up {
        // Check if we're in a nested mode first
        if is_scores_browse_mode_active(component_states) {
            // In box selection - Up navigates within grid
            use crate::layout_constants::GAME_BOX_WITH_MARGIN;
            let boxes_per_row = (state.system.terminal_width / GAME_BOX_WITH_MARGIN).max(1);
            return Some(Action::ScoresAction(ScoresAction::MoveGameSelectionUp(boxes_per_row)));
        } else if current_tab == Tab::Demo {
            // Demo tab - Up handled by handle_demo_tab_keys (both plain and Shift)
        } else if current_tab == Tab::Settings {
            // Settings tab - Up handled by handle_settings_tab_keys (both plain and Shift)
        } else if current_tab == Tab::Standings && is_standings_browse_mode_active(component_states) {
            // Standings browse mode - Up handled by handle_standings_league_keys (both plain and Shift)
        } else {
            // Not in nested mode - Up returns to tab bar
            debug!("KEY: Up pressed in content - returning to tab bar");
            return Some(Action::ExitContentFocus);
        }
    }

    // 6b. Handle Down key for Demo tab - delegated to handle_demo_tab_keys
    // (Both plain Down for focus navigation and Shift+Down for scrolling)

    // 6c. Handle Down key for standings browse mode - delegated to handle_standings_league_keys
    // (Both plain Down for focus navigation and Shift+Down for scrolling)

    // 7. Delegate to tab-specific handlers
    match current_tab {
        Tab::Scores => handle_scores_tab_keys(state, key.code, component_states),
        Tab::Standings => {
            // All standings views use document navigation in browse mode
            if is_standings_browse_mode_active(component_states) {
                handle_standings_league_keys(key, state)
            } else {
                handle_standings_tab_keys(key.code, state)
            }
        }
        Tab::Settings => handle_settings_tab_keys(key, state, component_states),
        Tab::Demo => handle_demo_tab_keys(key, state),
    }
}

// /// Get the setting key for a given category and index (for boolean toggling)
// fn get_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
//     match category {
//         SettingsCategory::Logging => None, // No boolean settings in Logging
//         SettingsCategory::Display => {
//             // Display category: index 1 = use_unicode (Theme is at 0)
//             match index {
//                 1 => Some("use_unicode".to_string()),
//                 _ => None,
//             }
//         }
//         SettingsCategory::Data => {
//             // Data category: index 1 = western_teams_first
//             match index {
//                 1 => Some("western_teams_first".to_string()),
//                 _ => None,
//             }
//         }
//     }
// }

// /// Get the editable setting key for a given category and index (for string/int editing)
// fn get_editable_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
//     match category {
//         SettingsCategory::Logging => {
//             // Logging category: 0 = log_level, 1 = log_file
//             match index {
//                 0 => Some("log_level".to_string()),
//                 1 => Some("log_file".to_string()),
//                 _ => None,
//             }
//         }
//         SettingsCategory::Display => {
//             // Display category: 0 = theme (list-based editable)
//             match index {
//                 0 => Some("theme".to_string()),
//                 _ => None,
//             }
//         }
//         SettingsCategory::Data => {
//             // Data category: 0 = refresh_interval, 2 = time_format
//             match index {
//                 0 => Some("refresh_interval".to_string()),
//                 2 => Some("time_format".to_string()),
//                 _ => None,
//             }
//         }
//     }
// }

// TODO: Tests disabled - key handling refactored to use document system and component messages
// Most of these tests are obsolete and need to be rewritten for the new architecture
#[cfg(all(test, feature = "disabled_tests"))]
#[allow(dead_code)]
mod tests_disabled {
    use super::*;
    use crate::commands::standings::GroupBy;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_component_states() -> ComponentStateStore {
        ComponentStateStore::new()
    }

    fn make_component_states_with_box_selection() -> ComponentStateStore {
        let mut store = ComponentStateStore::new();
        let mut scores_state = ScoresTabState::default();
        scores_state.doc_nav.focus_index = Some(0); // Select first game (activates browse mode)
        store.insert("app/scores_tab".to_string(), scores_state);
        store
    }

    fn make_component_states_with_browse_mode() -> ComponentStateStore {
        let mut store = ComponentStateStore::new();
        let mut standings_state = StandingsTabState::default();
        standings_state.doc_nav.focus_index = Some(0); // Activate browse mode
        store.insert("app/standings_tab".to_string(), standings_state);
        store
    }

    #[test]
    fn test_quit_key() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('q')), &state, &component_states);
        assert!(matches!(action, Some(Action::Quit)));
    }

    #[test]
    fn test_number_keys_navigate_tabs() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('1')), &state, &component_states);
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

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
        assert!(matches!(action, Some(Action::PopPanel)));
    }

    #[test]
    fn test_esc_exits_content_focus_when_in_subtabs() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
        assert!(matches!(action, Some(Action::ExitContentFocus)));
    }

    #[test]
    fn test_esc_at_tab_bar_does_nothing() {
        let state = AppState::default(); // content_focused = false by default

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
        assert!(
            action.is_none(),
            "ESC at tab bar should do nothing, not quit"
        );
    }

    #[test]
    fn test_q_quits_application() {
        let state = AppState::default();
        let component_states = make_component_states();

        let action = key_to_action(make_key(KeyCode::Char('q')), &state, &component_states);
        assert!(matches!(action, Some(Action::Quit)));
    }

    #[test]
    fn test_esc_exits_box_selection_mode() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Esc), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::ExitBoxSelection))
        ));
    }

    #[test]
    fn test_esc_exits_browse_mode() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Esc), &state, &component_states);
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
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Down), &state, &component_states);
        // All standings views use document navigation
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_browse_mode_up_arrow() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Up), &state, &component_states);
        // All standings views use document navigation
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_browse_mode_left_arrow_focuses_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Left), &state, &component_states);
        // All standings views use document navigation
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_browse_mode_right_arrow_focuses_right() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Right), &state, &component_states);
        // All standings views use document navigation
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_division_browse_mode_left_arrow_focuses_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Left), &state, &component_states);
        // Left arrow should focus left in Division view (document navigation)
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_division_browse_mode_right_arrow_focuses_right() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Right), &state, &component_states);
        // Right arrow should focus right in Division view (document navigation)
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_standings_view_mode_left_cycles_view() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states(); // browse_mode = false (default)

        let action = key_to_action(make_key(KeyCode::Left), &state, &component_states);
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
        let component_states = make_component_states(); // browse_mode = false (default)

        let action = key_to_action(make_key(KeyCode::Down), &state, &component_states);
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

        let action = key_to_action(make_key(KeyCode::Up), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
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
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('/')), &state, &component_states);
        assert!(matches!(action, Some(Action::ToggleCommandPalette)));
    }

    #[test]
    fn test_uppercase_q_quits() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('Q')), &state, &component_states);
        assert!(matches!(action, Some(Action::Quit)));
    }

    // ESC priority tests
    #[test]
    fn test_esc_exits_settings_mode() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Settings;
        state.navigation.content_focused = true;
        state.ui.settings.settings_mode = true;

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Up), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
        assert!(matches!(action, Some(Action::PanelSelectItem)));
    }

    // Number keys for all tabs
    #[test]
    fn test_number_key_2_navigates_to_standings() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('2')), &state, &component_states);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Standings))));
    }

    #[test]
    fn test_number_key_3_navigates_to_stats() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('3')), &state, &component_states);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Stats))));
    }

    #[test]
    fn test_number_key_4_navigates_to_players() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('4')), &state, &component_states);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Players))));
    }

    #[test]
    fn test_number_key_5_navigates_to_settings() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('5')), &state, &component_states);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Settings))));
    }

    #[test]
    fn test_number_key_6_navigates_to_browser() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::Char('6')), &state, &component_states);
        assert!(matches!(action, Some(Action::NavigateTab(Tab::Demo))));
    }

    // Tab bar navigation tests
    #[test]
    fn test_tab_bar_left_arrow_navigates_left() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Left), &state, &make_component_states());
        assert!(matches!(action, Some(Action::NavigateTabLeft)));
    }

    #[test]
    fn test_tab_bar_right_arrow_navigates_right() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Right), &state, &make_component_states());
        assert!(matches!(action, Some(Action::NavigateTabRight)));
    }

    #[test]
    fn test_tab_bar_down_arrow_enters_content_focus() {
        let mut state = AppState::default();
        state.navigation.content_focused = false;

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
        assert!(matches!(action, Some(Action::EnterContentFocus)));
    }

    // Content focus Up key returning to tab bar
    #[test]
    fn test_content_up_key_returns_to_tab_bar() {
        let mut state = AppState::default();
        state.navigation.content_focused = true;
        state.navigation.current_tab = Tab::Scores;
        // Not in box selection mode (default)
        let component_states = make_component_states();

        let action = key_to_action(make_key(KeyCode::Up), &state, &component_states);
        assert!(matches!(action, Some(Action::ExitContentFocus)));
    }

    // Scores tab - box selection mode
    #[test]
    fn test_scores_box_selection_up() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Up), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionUp(_)))
        ));
    }

    #[test]
    fn test_scores_box_selection_down() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Down), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionDown(_)))
        ));
    }

    #[test]
    fn test_scores_box_selection_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Left), &state, &component_states);
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
        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Right), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::MoveGameSelectionRight))
        ));
    }

    #[test]
    fn test_scores_box_selection_enter_with_no_schedule() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_box_selection();

        // With no schedule data, Enter should return None
        let action = key_to_action(make_key(KeyCode::Enter), &state, &component_states);
        assert!(action.is_none());
    }

    #[test]
    fn test_scores_box_selection_enter_with_schedule() {
        use nhl_api::{DailySchedule, GameState, GameType, ScheduleGame, ScheduleTeam};
        use std::sync::Arc;

        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;

        // Add schedule data with a test game
        let game = ScheduleGame {
            id: 12345,
            game_type: GameType::RegularSeason,
            game_date: None,
            start_time_utc: "2024-11-26T00:00:00Z".to_string(),
            away_team: ScheduleTeam {
                id: 10,
                abbrev: "TOR".to_string(),
                place_name: None,
                logo: String::new(),
                score: None,
            },
            home_team: ScheduleTeam {
                id: 8,
                abbrev: "MTL".to_string(),
                place_name: None,
                logo: String::new(),
                score: None,
            },
            game_state: GameState::Live,
        };
        let schedule = DailySchedule {
            next_start_date: None,
            previous_start_date: None,
            date: "2024-11-26".to_string(),
            games: vec![game],
            number_of_games: 1,
        };
        state.data.schedule = Arc::new(Some(schedule));

        let component_states = make_component_states_with_box_selection();

        let action = key_to_action(make_key(KeyCode::Enter), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::SelectGame(12345)))
        ));
    }

    // Scores tab - date mode
    #[test]
    fn test_scores_date_mode_left() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states(); // box_selection_active = false (default)

        let action = key_to_action(make_key(KeyCode::Left), &state, &component_states);
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
        let component_states = make_component_states(); // box_selection_active = false (default)

        let action = key_to_action(make_key(KeyCode::Right), &state, &component_states);
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
        let component_states = make_component_states(); // box_selection_active = false (default)

        let action = key_to_action(make_key(KeyCode::Down), &state, &component_states);
        assert!(matches!(
            action,
            Some(Action::ScoresAction(ScoresAction::EnterBoxSelection))
        ));
    }

    #[test]
    fn test_scores_date_mode_enter_no_schedule() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;
        state.navigation.content_focused = true;
        let component_states = make_component_states(); // box_selection_active = false (default)

        // With no schedule, Enter should return None
        let action = key_to_action(make_key(KeyCode::Enter), &state, &component_states);
        assert!(action.is_none());
    }

    // Standings tab
    #[test]
    fn test_standings_browse_mode_enter() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states_with_browse_mode();

        let action = key_to_action(make_key(KeyCode::Enter), &state, &component_states);
        // TODO: Enter activation not yet implemented
        assert!(action.is_none());
    }

    #[test]
    fn test_standings_view_mode_right_cycles_view() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Standings;
        state.navigation.content_focused = true;
        let component_states = make_component_states(); // browse_mode = false (default)

        let action = key_to_action(make_key(KeyCode::Right), &state, &component_states);
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
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
        let component_states = make_component_states();

        let action = key_to_action(make_key(KeyCode::Char('a')), &state, &component_states);
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

        let action = key_to_action(make_key(KeyCode::Backspace), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Esc), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Up), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Up), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Left), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Right), &state, &make_component_states());
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

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
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
        assert!(key_to_action(make_key(KeyCode::Left), &state, &make_component_states()).is_none());
        assert!(key_to_action(make_key(KeyCode::Right), &state, &make_component_states()).is_none());
        assert!(key_to_action(make_key(KeyCode::Down), &state, &make_component_states()).is_none());
    }

    #[test]
    fn test_unknown_key_returns_none() {
        let state = AppState::default();
        let component_states = make_component_states();
        let action = key_to_action(make_key(KeyCode::F(1)), &state, &component_states);
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

        let action = key_to_action(make_key(KeyCode::Enter), &state, &make_component_states());
        // Should return None since color settings aren't editable
        assert!(action.is_none());
    }

    // Demo tab tests
    #[test]
    fn test_demo_tab_up_focuses_previous() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Up), &state, &make_component_states());
        // Up should focus previous element, NOT exit to tab bar
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_demo_tab_down_focuses_next() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(make_key(KeyCode::Down), &state, &make_component_states());
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_demo_tab_shift_down_scrolls() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT), &state, &make_component_states());
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_demo_tab_tab_focuses_next() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), &state, &make_component_states());
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }

    #[test]
    fn test_demo_tab_shift_tab_focuses_prev() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.navigation.content_focused = true;

        let action = key_to_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT), &state, &make_component_states());
        assert!(matches!(
            action,
            Some(Action::ComponentMessage { .. })
        ));
    }
}
