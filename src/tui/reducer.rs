use tracing::debug;

use super::action::{Action, SettingsAction};
use super::component::Effect;
use super::settings_helpers::find_initial_modal_index;
use super::state::AppState;
use super::types::{Panel, SettingsCategory};
use crate::config::Config;

// Import sub-reducers from the parent framework module
use crate::tui::reducers::{
    reduce_navigation,
    reduce_panels,
    reduce_data_loading,
    reduce_scores,
    reduce_standings,
};

/// Create an effect to save config to disk asynchronously
fn save_config_effect(config: Config) -> Effect {
    Effect::Async(Box::pin(async move {
        match crate::config::write(&config) {
            Ok(_) => {
                debug!("CONFIG: Successfully saved to disk");
                Action::SetStatusMessage {
                    message: "Configuration saved".to_string(),
                    is_error: false,
                }
            }
            Err(e) => {
                debug!("CONFIG: Failed to save: {}", e);
                Action::SetStatusMessage {
                    message: format!("Failed to save config: {}", e),
                    is_error: true,
                }
            }
        }
    }))
}

/// Pure state reducer - like Redux reducer
///
/// Takes current state and an action, returns new state and optional effect.
/// This function is PURE - no side effects, no I/O, no async.
/// All side effects are returned as `Effect` to be executed separately.
pub fn reduce(state: AppState, action: Action) -> (AppState, Effect) {
    // Try each sub-reducer in order
    // Each returns Option<(AppState, Effect)> - None means it didn't handle the action

    // Navigation actions
    if let Some(result) = reduce_navigation(&state, &action) {
        return result;
    }

    // Panel management actions
    if let Some(result) = reduce_panels(&state, &action) {
        return result;
    }

    // Data loading actions
    if let Some(result) = reduce_data_loading(&state, &action) {
        return result;
    }

    // Tab-specific action delegation
    match action {
        Action::ScoresAction(scores_action) => reduce_scores(state, scores_action),
        Action::StandingsAction(standings_action) => reduce_standings(state, standings_action),
        Action::SettingsAction(settings_action) => reduce_settings(state, settings_action),

        // Special cases that don't fit cleanly into sub-modules
        Action::SelectPlayer(player_id) => {
            debug!("PLAYER: Opening player detail panel for player_id={}", player_id);
            let mut new_state = state;

            // Push PlayerDetail panel onto stack
            new_state.navigation.panel_stack.push(super::state::PanelState {
                panel: Panel::PlayerDetail { player_id },
                scroll_offset: 0,
                selected_index: Some(0), // Start with first season selected
            });

            (new_state, Effect::None)
        }

        Action::SelectTeam(team_abbrev) => {
            debug!("TEAM: Opening team detail panel for team={}", team_abbrev);
            let mut new_state = state;

            // Push TeamDetail panel onto stack
            new_state.navigation.panel_stack.push(super::state::PanelState {
                panel: Panel::TeamDetail { abbrev: team_abbrev },
                scroll_offset: 0,
                selected_index: Some(0), // Start with first player selected
            });

            (new_state, Effect::None)
        }

        Action::SetStatusMessage { message, is_error } => {
            let mut new_state = state;
            if is_error {
                new_state.system.set_status_error_message(message);
            } else {
                new_state.system.set_status_message(message);
            }
            (new_state, Effect::None)
        }

        Action::Quit | Action::Error(_) => (state, Effect::None),

        _ => (state, Effect::None),
    }
}

/// Sub-reducer for settings tab
/// TODO: Move this to its own module once refactoring is complete
fn reduce_settings(state: AppState, action: SettingsAction) -> (AppState, Effect) {
    match action {
        SettingsAction::NavigateCategoryLeft => {
            let mut new_state = state;
            new_state.ui.settings.selected_category = match new_state.ui.settings.selected_category {
                SettingsCategory::Logging => SettingsCategory::Data,
                SettingsCategory::Display => SettingsCategory::Logging,
                SettingsCategory::Data => SettingsCategory::Display,
            };
            new_state.ui.settings.selected_setting_index = 0; // Reset selection
            (new_state, Effect::None)
        }

        SettingsAction::NavigateCategoryRight => {
            let mut new_state = state;
            new_state.ui.settings.selected_category = match new_state.ui.settings.selected_category {
                SettingsCategory::Logging => SettingsCategory::Display,
                SettingsCategory::Display => SettingsCategory::Data,
                SettingsCategory::Data => SettingsCategory::Logging,
            };
            new_state.ui.settings.selected_setting_index = 0; // Reset selection
            (new_state, Effect::None)
        }

        SettingsAction::EnterSettingsMode => {
            debug!("SETTINGS: Entering settings mode");
            let mut new_state = state;
            new_state.ui.settings.settings_mode = true;
            (new_state, Effect::None)
        }

        SettingsAction::ExitSettingsMode => {
            debug!("SETTINGS: Exiting settings mode");
            let mut new_state = state;
            new_state.ui.settings.settings_mode = false;
            (new_state, Effect::None)
        }

        SettingsAction::MoveSelectionUp => {
            let mut new_state = state;
            if new_state.ui.settings.selected_setting_index > 0 {
                new_state.ui.settings.selected_setting_index -= 1;
            }
            (new_state, Effect::None)
        }

        SettingsAction::MoveSelectionDown => {
            let mut new_state = state;
            // We'll validate max in the UI layer
            new_state.ui.settings.selected_setting_index += 1;
            (new_state, Effect::None)
        }

        SettingsAction::ToggleBoolean(key) => {
            debug!("SETTINGS: Toggling boolean setting: {}", key);
            let mut new_state = state;
            match key.as_str() {
                "use_unicode" => {
                    new_state.system.config.display.use_unicode = !new_state.system.config.display.use_unicode;
                    // Update box_chars based on use_unicode
                    new_state.system.config.display.box_chars = crate::formatting::BoxChars::from_use_unicode(
                        new_state.system.config.display.use_unicode
                    );
                }
                "western_teams_first" => {
                    new_state.system.config.display_standings_western_first = !new_state.system.config.display_standings_western_first;
                }
                _ => {
                    debug!("SETTINGS: Unknown boolean setting: {}", key);
                }
            }
            let config = new_state.system.config.clone();
            let effect = save_config_effect(config);
            (new_state, effect)
        }

        SettingsAction::StartEditing(key) => {
            debug!("SETTINGS: Starting edit for setting: {}", key);
            let mut new_state = state;

            // Check if this is a list-based setting (opens modal)
            let values = crate::tui::settings_helpers::get_setting_values(&key);
            if !values.is_empty() {
                // Open modal for list-based settings (e.g., log_level, theme)
                new_state.ui.settings.modal_open = true;
                new_state.ui.settings.modal_selected_index =
                    find_initial_modal_index(&new_state.system.config, &key);
            } else {
                // Start inline editing for text/number settings (e.g., log_file, refresh_interval, time_format)
                new_state.ui.settings.editing = true;
                // Initialize edit buffer with current value
                new_state.ui.settings.edit_buffer = match key.as_str() {
                    "log_file" => new_state.system.config.log_file.clone(),
                    "refresh_interval" => new_state.system.config.refresh_interval.to_string(),
                    "time_format" => new_state.system.config.time_format.clone(),
                    _ => String::new(),
                };
            }
            (new_state, Effect::None)
        }

        SettingsAction::CancelEditing => {
            debug!("SETTINGS: Canceling edit");
            let mut new_state = state;
            new_state.ui.settings.editing = false;
            new_state.ui.settings.edit_buffer.clear();
            (new_state, Effect::None)
        }

        SettingsAction::AppendChar(ch) => {
            let mut new_state = state;
            new_state.ui.settings.edit_buffer.push(ch);
            (new_state, Effect::None)
        }

        SettingsAction::DeleteChar => {
            let mut new_state = state;
            new_state.ui.settings.edit_buffer.pop();
            (new_state, Effect::None)
        }

        SettingsAction::CommitEdit(key) => {
            debug!("SETTINGS: Committing edit for: {}", key);
            let mut new_state = state;
            let buffer = &new_state.ui.settings.edit_buffer;

            // Apply the edit to the config
            match key.as_str() {
                "log_file" => {
                    new_state.system.config.log_file = buffer.clone();
                }
                "refresh_interval" => {
                    if let Ok(value) = buffer.parse::<u32>() {
                        new_state.system.config.refresh_interval = value;
                    } else {
                        debug!("SETTINGS: Invalid refresh_interval value: {}", buffer);
                    }
                }
                "time_format" => {
                    new_state.system.config.time_format = buffer.clone();
                }
                _ => {
                    debug!("SETTINGS: Unknown setting key: {}", key);
                }
            }

            // Clear editing state
            new_state.ui.settings.editing = false;
            new_state.ui.settings.edit_buffer.clear();

            let config = new_state.system.config.clone();
            let effect = save_config_effect(config);
            (new_state, effect)
        }

        SettingsAction::ModalMoveUp => {
            let mut new_state = state;
            let setting_key = crate::tui::settings_helpers::get_editable_setting_key(
                new_state.ui.settings.selected_category,
                new_state.ui.settings.selected_setting_index,
            );

            if let Some(key) = setting_key {
                let values = crate::tui::settings_helpers::get_setting_values(&key);
                if !values.is_empty() {
                    if new_state.ui.settings.modal_selected_index == 0 {
                        // Wrap to bottom
                        new_state.ui.settings.modal_selected_index = values.len() - 1;
                    } else {
                        new_state.ui.settings.modal_selected_index -= 1;
                    }
                }
            }
            (new_state, Effect::None)
        }

        SettingsAction::ModalMoveDown => {
            let mut new_state = state;
            let setting_key = crate::tui::settings_helpers::get_editable_setting_key(
                new_state.ui.settings.selected_category,
                new_state.ui.settings.selected_setting_index,
            );

            if let Some(key) = setting_key {
                let values = crate::tui::settings_helpers::get_setting_values(&key);
                if !values.is_empty() {
                    if new_state.ui.settings.modal_selected_index >= values.len() - 1 {
                        // Wrap to top
                        new_state.ui.settings.modal_selected_index = 0;
                    } else {
                        new_state.ui.settings.modal_selected_index += 1;
                    }
                }
            }
            (new_state, Effect::None)
        }

        SettingsAction::ModalConfirm => {
            debug!("SETTINGS: Confirming modal selection");
            let mut new_state = state;

            // Get the setting key for the current selection
            let setting_key = crate::tui::settings_helpers::get_editable_setting_key(
                new_state.ui.settings.selected_category,
                new_state.ui.settings.selected_setting_index,
            );

            if let Some(key) = setting_key {
                let values = crate::tui::settings_helpers::get_setting_values(&key);
                let selected_index = new_state.ui.settings.modal_selected_index;

                if selected_index < values.len() {
                    let selected_value = values[selected_index];

                    // Apply the selection to the config
                    match key.as_str() {
                        "log_level" => {
                            new_state.system.config.log_level = selected_value.to_string();
                        }
                        "theme" => {
                            new_state.system.config.display.theme_name = Some(selected_value.to_string());
                            new_state.system.config.display.apply_theme();
                        }
                        _ => {
                            debug!("SETTINGS: Unknown list setting: {}", key);
                        }
                    }
                }
            }

            // Close modal
            new_state.ui.settings.modal_open = false;
            new_state.ui.settings.modal_selected_index = 0;

            let config = new_state.system.config.clone();
            let effect = save_config_effect(config);
            (new_state, effect)
        }

        SettingsAction::ModalCancel => {
            debug!("SETTINGS: Canceling modal");
            let mut new_state = state;
            new_state.ui.settings.modal_open = false;
            new_state.ui.settings.modal_selected_index = 0;
            (new_state, Effect::None)
        }

        SettingsAction::UpdateConfig(config) => {
            debug!("SETTINGS: Updating config");
            let mut new_state = state;
            new_state.system.config = *config;
            (new_state, Effect::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::action::{ScoresAction, StandingsAction};
    use crate::tui::types::Tab;

    #[test]
    fn test_navigation_actions_are_handled() {
        let state = AppState::default();
        let action = Action::NavigateTab(Tab::Settings);

        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.navigation.current_tab, Tab::Settings);
        assert!(new_state.navigation.panel_stack.is_empty());
        assert!(!new_state.navigation.content_focused);
    }

    #[test]
    fn test_panel_actions_are_handled() {
        let state = AppState::default();
        let panel = super::super::types::Panel::TeamDetail {
            abbrev: "BOS".to_string(),
        };
        let action = Action::PushPanel(panel.clone());

        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.navigation.panel_stack.len(), 1);
    }

    #[test]
    fn test_scores_actions_are_delegated() {
        use nhl_api::GameDate;

        let mut state = AppState::default();
        state.ui.scores.game_date = GameDate::from_ymd(2024, 11, 15).unwrap();
        state.ui.scores.selected_date_index = 2;

        let action = Action::ScoresAction(ScoresAction::DateLeft);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.scores.selected_date_index, 1);
    }

    #[test]
    fn test_standings_actions_are_delegated() {
        use crate::commands::standings::GroupBy;

        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let action = Action::StandingsAction(StandingsAction::CycleView);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_data_loading_actions_are_handled() {
        let state = AppState::default();
        let action = Action::RefreshData;

        let (new_state, _) = reduce(state, action);

        assert!(new_state.system.last_refresh.is_some());
    }

    #[test]
    fn test_select_player_opens_panel() {
        let state = AppState::default();
        let action = Action::SelectPlayer(8471214);

        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.navigation.panel_stack.len(), 1);
        match &new_state.navigation.panel_stack[0].panel {
            Panel::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8471214);
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
        assert_eq!(new_state.navigation.panel_stack[0].scroll_offset, 0);
        assert_eq!(new_state.navigation.panel_stack[0].selected_index, Some(0));
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_select_team_opens_panel() {
        let state = AppState::default();
        let action = Action::SelectTeam("BOS".to_string());

        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.navigation.panel_stack.len(), 1);
        match &new_state.navigation.panel_stack[0].panel {
            Panel::TeamDetail { abbrev } => {
                assert_eq!(abbrev, "BOS");
            }
            _ => panic!("Expected TeamDetail panel"),
        }
        assert_eq!(new_state.navigation.panel_stack[0].scroll_offset, 0);
        assert_eq!(new_state.navigation.panel_stack[0].selected_index, Some(0));
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_quit_action_does_nothing_to_state() {
        let state = AppState::default();
        let action = Action::Quit;

        let (new_state, effect) = reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(new_state.navigation.current_tab, state.navigation.current_tab);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_error_action_does_nothing_to_state() {
        let state = AppState::default();
        let action = Action::Error("test error".to_string());

        let (new_state, effect) = reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(new_state.navigation.current_tab, state.navigation.current_tab);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_unknown_action_does_nothing() {
        let state = AppState::default();
        let action = Action::ToggleCommandPalette;

        let (new_state, effect) = reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(new_state.navigation.current_tab, state.navigation.current_tab);
        assert!(matches!(effect, Effect::None));
    }

    // Settings reducer tests
    #[test]
    fn test_settings_navigate_category_left_from_logging() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 2;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Data);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0); // Reset to 0
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_navigate_category_left_from_display() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 1;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Logging);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
    }

    #[test]
    fn test_settings_navigate_category_left_from_data() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Data;
        state.ui.settings.selected_setting_index = 1;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Display);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
    }

    #[test]
    fn test_settings_navigate_category_right_from_logging() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 2;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Display);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_navigate_category_right_from_display() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Display;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Data);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
    }

    #[test]
    fn test_settings_navigate_category_right_from_data() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Data;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Logging);
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
    }

    #[test]
    fn test_settings_enter_settings_mode() {
        let mut state = AppState::default();
        state.ui.settings.settings_mode = false;

        let action = Action::SettingsAction(SettingsAction::EnterSettingsMode);
        let (new_state, effect) = reduce(state, action);

        assert!(new_state.ui.settings.settings_mode);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_exit_settings_mode() {
        let mut state = AppState::default();
        state.ui.settings.settings_mode = true;

        let action = Action::SettingsAction(SettingsAction::ExitSettingsMode);
        let (new_state, effect) = reduce(state, action);

        assert!(!new_state.ui.settings.settings_mode);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_move_selection_up_from_middle() {
        let mut state = AppState::default();
        state.ui.settings.selected_setting_index = 2;

        let action = Action::SettingsAction(SettingsAction::MoveSelectionUp);
        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_setting_index, 1);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_move_selection_up_from_top() {
        let mut state = AppState::default();
        state.ui.settings.selected_setting_index = 0;

        let action = Action::SettingsAction(SettingsAction::MoveSelectionUp);
        let (new_state, _) = reduce(state, action);

        // Should stay at 0
        assert_eq!(new_state.ui.settings.selected_setting_index, 0);
    }

    #[test]
    fn test_settings_move_selection_down() {
        let mut state = AppState::default();
        state.ui.settings.selected_setting_index = 1;

        let action = Action::SettingsAction(SettingsAction::MoveSelectionDown);
        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.ui.settings.selected_setting_index, 2);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_settings_unhandled_action_does_nothing() {
        let state = AppState::default();
        let action = Action::SettingsAction(SettingsAction::ModalMoveUp);

        let (new_state, effect) = reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(new_state.ui.settings.selected_category, state.ui.settings.selected_category);
        assert_eq!(new_state.ui.settings.selected_setting_index, state.ui.settings.selected_setting_index);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_set_status_message_with_error() {
        let state = AppState::default();
        let action = Action::SetStatusMessage {
            message: "Test error message".to_string(),
            is_error: true,
        };

        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.system.status_message, Some("Test error message".to_string()));
        assert!(new_state.system.status_is_error);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_set_status_message_without_error() {
        let state = AppState::default();
        let action = Action::SetStatusMessage {
            message: "Configuration saved".to_string(),
            is_error: false,
        };

        let (new_state, effect) = reduce(state, action);

        assert_eq!(new_state.system.status_message, Some("Configuration saved".to_string()));
        assert!(!new_state.system.status_is_error);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_toggle_boolean_returns_save_effect() {
        let state = AppState::default();
        let action = Action::SettingsAction(SettingsAction::ToggleBoolean("use_unicode".to_string()));

        let (new_state, effect) = reduce(state.clone(), action);

        // Config should be toggled
        assert_eq!(new_state.system.config.display.use_unicode, !state.system.config.display.use_unicode);

        // Should return an Async effect (save_config_effect)
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_toggle_boolean_use_unicode_updates_box_chars() {
        let mut state = AppState::default();
        state.system.config.display.use_unicode = true;

        let action = Action::SettingsAction(SettingsAction::ToggleBoolean("use_unicode".to_string()));

        let (new_state, _) = reduce(state, action);

        // Should toggle to false
        assert!(!new_state.system.config.display.use_unicode);
        // box_chars should be updated to ASCII
        assert_eq!(new_state.system.config.display.box_chars, crate::formatting::BoxChars::ascii());
    }

    #[test]
    fn test_toggle_boolean_western_teams_first() {
        let state = AppState::default();
        let action = Action::SettingsAction(SettingsAction::ToggleBoolean("western_teams_first".to_string()));

        let (new_state, _) = reduce(state.clone(), action);

        assert_eq!(
            new_state.system.config.display_standings_western_first,
            !state.system.config.display_standings_western_first
        );
    }

    #[test]
    fn test_toggle_boolean_unknown_setting() {
        let state = AppState::default();
        let action = Action::SettingsAction(SettingsAction::ToggleBoolean("unknown_setting".to_string()));

        let (new_state, _) = reduce(state.clone(), action);

        // State should not change for unknown settings
        assert_eq!(new_state.system.config.display.use_unicode, state.system.config.display.use_unicode);
        assert_eq!(
            new_state.system.config.display_standings_western_first,
            state.system.config.display_standings_western_first
        );
    }

    #[test]
    fn test_commit_edit_returns_save_effect() {
        let mut state = AppState::default();
        state.ui.settings.edit_buffer = "120".to_string();

        let action = Action::SettingsAction(SettingsAction::CommitEdit("refresh_interval".to_string()));

        let (new_state, effect) = reduce(state, action);

        // Config should be updated
        assert_eq!(new_state.system.config.refresh_interval, 120);
        // Editing state should be cleared
        assert!(!new_state.ui.settings.editing);
        assert!(new_state.ui.settings.edit_buffer.is_empty());

        // Should return an Async effect (save_config_effect)
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_commit_edit_log_file() {
        let mut state = AppState::default();
        state.ui.settings.edit_buffer = "/tmp/test.log".to_string();

        let action = Action::SettingsAction(SettingsAction::CommitEdit("log_file".to_string()));

        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.system.config.log_file, "/tmp/test.log");
    }

    #[test]
    fn test_commit_edit_time_format() {
        let mut state = AppState::default();
        state.ui.settings.edit_buffer = "%Y-%m-%d".to_string();

        let action = Action::SettingsAction(SettingsAction::CommitEdit("time_format".to_string()));

        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.system.config.time_format, "%Y-%m-%d");
    }

    #[test]
    fn test_commit_edit_invalid_refresh_interval() {
        let mut state = AppState::default();
        state.ui.settings.edit_buffer = "invalid".to_string();
        state.system.config.refresh_interval = 60;

        let action = Action::SettingsAction(SettingsAction::CommitEdit("refresh_interval".to_string()));

        let (new_state, _) = reduce(state, action);

        // Should not change on invalid value
        assert_eq!(new_state.system.config.refresh_interval, 60);
    }

    #[test]
    fn test_modal_confirm_returns_save_effect() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 0; // log_level
        state.ui.settings.modal_selected_index = 2; // "info"
        state.ui.settings.modal_open = true;

        let action = Action::SettingsAction(SettingsAction::ModalConfirm);

        let (new_state, effect) = reduce(state, action);

        // Config should be updated
        assert_eq!(new_state.system.config.log_level, "info");
        // Modal should be closed
        assert!(!new_state.ui.settings.modal_open);
        assert_eq!(new_state.ui.settings.modal_selected_index, 0);

        // Should return an Async effect (save_config_effect)
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_modal_confirm_theme_selection() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 0; // theme
        state.ui.settings.modal_selected_index = 1; // "orange"
        state.ui.settings.modal_open = true;

        let action = Action::SettingsAction(SettingsAction::ModalConfirm);

        let (new_state, _) = reduce(state, action);

        assert_eq!(new_state.system.config.display.theme_name, Some("orange".to_string()));
    }

    #[test]
    fn test_modal_initialized_with_current_theme() {
        let mut state = AppState::default();
        state.system.config.display.theme_name = Some("purple".to_string());
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 0; // theme

        let action = Action::SettingsAction(SettingsAction::StartEditing("theme".to_string()));

        let (new_state, _) = reduce(state, action);

        // Modal should be open
        assert!(new_state.ui.settings.modal_open);
        // Theme values: ["none", "orange", "green", "blue", "purple", "white"]
        // "purple" is at index 4
        assert_eq!(new_state.ui.settings.modal_selected_index, 4);
    }

    #[test]
    fn test_modal_initialized_with_current_log_level() {
        let mut state = AppState::default();
        state.system.config.log_level = "warn".to_string();
        state.ui.settings.selected_category = SettingsCategory::Logging;
        state.ui.settings.selected_setting_index = 0; // log_level

        let action = Action::SettingsAction(SettingsAction::StartEditing("log_level".to_string()));

        let (new_state, _) = reduce(state, action);

        // Modal should be open
        assert!(new_state.ui.settings.modal_open);
        // Log level values: ["trace", "debug", "info", "warn", "error"]
        // "warn" is at index 3
        assert_eq!(new_state.ui.settings.modal_selected_index, 3);
    }

    #[test]
    fn test_modal_initialized_at_zero_for_none_theme() {
        let mut state = AppState::default();
        state.system.config.display.theme_name = None;
        state.ui.settings.selected_category = SettingsCategory::Display;
        state.ui.settings.selected_setting_index = 0; // theme

        let action = Action::SettingsAction(SettingsAction::StartEditing("theme".to_string()));

        let (new_state, _) = reduce(state, action);

        // Modal should be open
        assert!(new_state.ui.settings.modal_open);
        // Theme values: ["none", "orange", "green", "blue", "purple", "white"]
        // "none" is at index 0
        assert_eq!(new_state.ui.settings.modal_selected_index, 0);
    }
}