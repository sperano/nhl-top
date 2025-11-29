use tracing::debug;

use super::action::{Action, SettingsAction};
use super::component::Effect;
use super::component_store::ComponentStateStore;
//use super::settings_helpers::find_initial_modal_index;
use super::state::AppState;
use super::types::SettingsCategory;
use crate::config::Config;

// Import sub-reducers from the parent framework module
use crate::tui::reducers::{
    reduce_data_loading, reduce_navigation, reduce_panels, reduce_scores, reduce_standings,
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
/// Takes current state, component state store, and an action, returns new state and optional effect.
/// This function is PURE - no side effects, no I/O, no async (except for component state updates).
/// All side effects are returned as `Effect` to be executed separately.
///
/// Note: component_states is passed as &mut to allow ComponentMessage actions to update
/// component-local state, but the reducer itself doesn't create side effects through this.
pub fn reduce(
    state: AppState,
    action: Action,
    component_states: &mut ComponentStateStore,
) -> (AppState, Effect) {
    // Try each sub-reducer in order
    // Each returns Option<(AppState, Effect)> - None means it didn't handle the action

    // Component message dispatch (React-like component system)
    if let Action::ComponentMessage { path, message } = &action {
        if let Some(component_state) = component_states.get_mut_any(path) {
            debug!("COMPONENT: Dispatching message to {}: {:?}", path, message);
            let effect = message.apply(component_state);
            return (state, effect);
        } else {
            debug!("COMPONENT: No state found for path: {}", path);
            return (state, Effect::None);
        }
    }

    // Navigation actions
    if let Some(result) = reduce_navigation(&state, &action) {
        return result;
    }

    // Panel management actions
    if let Some(result) = reduce_panels(&state, &action) {
        return result;
    }

    // Data loading actions (Phase 7: Pass component_states for focusable metadata)
    if let Some(result) = reduce_data_loading(&state, &action, component_states) {
        return result;
    }

    // Document actions removed in Phase 10 - now handled by component messages

    // Tab-specific action delegation
    match action {
        Action::ScoresAction(scores_action) => reduce_scores(state, scores_action),
        Action::StandingsAction(standings_action) => reduce_standings(state, standings_action),
        Action::SettingsAction(settings_action) => reduce_settings(state, settings_action, component_states),

        // // Special cases that don't fit cleanly into sub-modules
        // Action::SelectPlayer(player_id) => {
        //     debug!(
        //         "PLAYER: Opening player detail panel for player_id={}",
        //         player_id
        //     );
        //     let mut new_state = state;
        //
        //     // Push PlayerDetail panel onto stack
        //     new_state
        //         .navigation
        //         .panel_stack
        //         .push(super::state::PanelState {
        //             panel: Panel::PlayerDetail { player_id },
        //             selected_index: Some(0), // Start with first season selected
        //         });
        //
        //     (new_state, Effect::None)
        // }

        // Action::SelectTeam(team_abbrev) => {
        //     debug!("TEAM: Opening team detail panel for team={}", team_abbrev);
        //     let mut new_state = state;
        //
        //     // Push TeamDetail panel onto stack
        //     new_state
        //         .navigation
        //         .panel_stack
        //         .push(super::state::PanelState {
        //             panel: Panel::TeamDetail {
        //                 abbrev: team_abbrev,
        //             },
        //             selected_index: Some(0), // Start with first player selected
        //         });
        //
        //     (new_state, Effect::None)
        // }

        Action::SetStatusMessage { message, is_error } => {
            let mut new_state = state;
            if is_error {
                new_state.system.set_status_error_message(message);
            } else {
                new_state.system.set_status_message(message);
            }
            (new_state, Effect::None)
        }

        Action::UpdateTerminalWidth(width) => {
            let mut new_state = state;
            new_state.system.terminal_width = width;
            (new_state, Effect::None)
        }

        Action::Quit | Action::Error(_) => (state, Effect::None),

        _ => (state, Effect::None),
    }
}

/// Sub-reducer for settings tab
/// TODO: Move this to its own module once refactoring is complete
fn reduce_settings(
    state: AppState,
    action: SettingsAction,
    component_states: &mut ComponentStateStore,
) -> (AppState, Effect) {
    match action {
        SettingsAction::NavigateCategoryLeft => {
            use crate::tui::components::{SettingsDocument, SettingsTabState};
            use crate::tui::document::Document;
            let mut new_state = state;
            let old_category = new_state.ui.settings.selected_category;
            new_state.ui.settings.selected_category = match old_category {
                SettingsCategory::Logging => SettingsCategory::Data,
                SettingsCategory::Display => SettingsCategory::Logging,
                SettingsCategory::Data => SettingsCategory::Display,
            };

            // Update focusable metadata in component state
            if let Some(settings_state) = component_states.get_mut::<SettingsTabState>("app/settings_tab") {
                let doc = SettingsDocument::new(new_state.ui.settings.selected_category, new_state.system.config.clone());
                settings_state.doc_nav = Default::default(); // Reset navigation
                settings_state.doc_nav.focusable_positions = doc.focusable_positions();
                settings_state.doc_nav.focusable_ids = doc.focusable_ids();
                settings_state.doc_nav.focusable_row_positions = doc.focusable_row_positions();
            }

            (new_state, Effect::None)
        }

        SettingsAction::NavigateCategoryRight => {
            use crate::tui::components::{SettingsDocument, SettingsTabState};
            use crate::tui::document::Document;
            let mut new_state = state;
            let old_category = new_state.ui.settings.selected_category;
            new_state.ui.settings.selected_category = match old_category {
                SettingsCategory::Logging => SettingsCategory::Display,
                SettingsCategory::Display => SettingsCategory::Data,
                SettingsCategory::Data => SettingsCategory::Logging,
            };

            // Update focusable metadata in component state
            if let Some(settings_state) = component_states.get_mut::<SettingsTabState>("app/settings_tab") {
                let doc = SettingsDocument::new(new_state.ui.settings.selected_category, new_state.system.config.clone());
                settings_state.doc_nav = Default::default(); // Reset navigation
                settings_state.doc_nav.focusable_positions = doc.focusable_positions();
                settings_state.doc_nav.focusable_ids = doc.focusable_ids();
                settings_state.doc_nav.focusable_row_positions = doc.focusable_row_positions();
            }

            (new_state, Effect::None)
        }

        SettingsAction::ToggleBoolean(key) => {
            debug!("SETTINGS: Toggling boolean setting: {}", key);
            let mut new_state = state;
            match key.as_str() {
                "use_unicode" => {
                    new_state.system.config.display.use_unicode =
                        !new_state.system.config.display.use_unicode;
                    // Update box_chars based on use_unicode
                    new_state.system.config.display.box_chars =
                        crate::formatting::BoxChars::from_use_unicode(
                            new_state.system.config.display.use_unicode,
                        );
                }
                "western_teams_first" => {
                    new_state.system.config.display_standings_western_first =
                        !new_state.system.config.display_standings_western_first;
                }
                _ => {
                    debug!("SETTINGS: Unknown boolean setting: {}", key);
                }
            }
            let config = new_state.system.config.clone();
            let effect = save_config_effect(config);
            (new_state, effect)
        }

        SettingsAction::UpdateSetting { key, value } => {
            debug!("SETTINGS: Updating setting: {} = {}", key, value);
            let mut new_state = state;
            match key.as_str() {
                "log_level" => {
                    new_state.system.config.log_level = value;
                }
                "theme" => {
                    if value == "none" {
                        new_state.system.config.display.theme_name = None;
                        new_state.system.config.display.theme = None;
                    } else {
                        use crate::config::{
                            THEME_BLUE, THEME_CYAN, THEME_GREEN, THEME_ORANGE, THEME_PURPLE,
                            THEME_RED, THEME_WHITE, THEME_YELLOW,
                        };
                        let theme = match value.as_str() {
                            "orange" => Some(THEME_ORANGE.clone()),
                            "green" => Some(THEME_GREEN.clone()),
                            "blue" => Some(THEME_BLUE.clone()),
                            "purple" => Some(THEME_PURPLE.clone()),
                            "white" => Some(THEME_WHITE.clone()),
                            "red" => Some(THEME_RED.clone()),
                            "yellow" => Some(THEME_YELLOW.clone()),
                            "cyan" => Some(THEME_CYAN.clone()),
                            _ => None,
                        };
                        new_state.system.config.display.theme_name = Some(value);
                        new_state.system.config.display.theme = theme;
                    }
                }
                _ => {
                    debug!("SETTINGS: Unknown setting key: {}", key);
                }
            }
            let config = new_state.system.config.clone();
            let effect = save_config_effect(config);
            (new_state, effect)
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

    // Test helper that creates a ComponentStateStore for each test
    fn test_reduce(state: AppState, action: Action) -> (AppState, Effect) {
        let mut component_states = ComponentStateStore::new();
        reduce(state, action, &mut component_states)
    }

    #[test]
    fn test_navigation_actions_are_handled() {
        let state = AppState::default();
        let action = Action::NavigateTab(Tab::Settings);

        let (new_state, _) = test_reduce(state, action);

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

        let (new_state, _) = test_reduce(state, action);

        assert_eq!(new_state.navigation.panel_stack.len(), 1);
    }

    #[test]
    fn test_scores_actions_are_delegated() {
        // Phase 3.5: ScoresAction now routes to ComponentMessage
        let state = AppState::default();
        let action = Action::ScoresAction(ScoresAction::DateLeft);
        let (new_state, effect) = test_reduce(state.clone(), action);

        // State should not be modified by the reducer
        assert_eq!(new_state.ui.scores.game_date, state.ui.scores.game_date);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/scores_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_standings_actions_are_delegated() {
        // Phase 7: StandingsAction now routes to ComponentMessage
        let state = AppState::default();
        let action = Action::StandingsAction(StandingsAction::CycleViewRight);
        let (new_state, effect) = test_reduce(state.clone(), action);

        // State should not be modified by the reducer (StandingsUiState removed in Phase 7)
        assert_eq!(new_state.data.standings, state.data.standings);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/standings_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_data_loading_actions_are_handled() {
        let state = AppState::default();
        let action = Action::RefreshData;

        let (new_state, _) = test_reduce(state, action);

        assert!(new_state.system.last_refresh.is_some());
    }

    // #[test]
    // fn test_select_player_opens_panel() {
    //     let state = AppState::default();
    //     let action = Action::SelectPlayer(8471214);
    //
    //     let (new_state, effect) = test_reduce(state, action);
    //
    //     assert_eq!(new_state.navigation.panel_stack.len(), 1);
    //     match &new_state.navigation.panel_stack[0].panel {
    //         Panel::PlayerDetail { player_id } => {
    //             assert_eq!(*player_id, 8471214);
    //         }
    //         _ => panic!("Expected PlayerDetail panel"),
    //     }
    //     assert_eq!(new_state.navigation.panel_stack[0].selected_index, Some(0));
    //     assert!(matches!(effect, Effect::None));
    // }
    //
    // #[test]
    // fn test_select_team_opens_panel() {
    //     let state = AppState::default();
    //     let action = Action::SelectTeam("BOS".to_string());
    //
    //     let (new_state, effect) = test_reduce(state, action);
    //
    //     assert_eq!(new_state.navigation.panel_stack.len(), 1);
    //     match &new_state.navigation.panel_stack[0].panel {
    //         Panel::TeamDetail { abbrev } => {
    //             assert_eq!(abbrev, "BOS");
    //         }
    //         _ => panic!("Expected TeamDetail panel"),
    //     }
    //     assert_eq!(new_state.navigation.panel_stack[0].selected_index, Some(0));
    //     assert!(matches!(effect, Effect::None));
    // }

    #[test]
    fn test_quit_action_does_nothing_to_state() {
        let state = AppState::default();
        let action = Action::Quit;

        let (new_state, effect) = test_reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(
            new_state.navigation.current_tab,
            state.navigation.current_tab
        );
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_error_action_does_nothing_to_state() {
        let state = AppState::default();
        let action = Action::Error("test error".to_string());

        let (new_state, effect) = test_reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(
            new_state.navigation.current_tab,
            state.navigation.current_tab
        );
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_unknown_action_does_nothing() {
        let state = AppState::default();
        let action = Action::ToggleCommandPalette;

        let (new_state, effect) = test_reduce(state.clone(), action);

        // State should remain unchanged
        assert_eq!(
            new_state.navigation.current_tab,
            state.navigation.current_tab
        );
        assert!(matches!(effect, Effect::None));
    }

    // Settings reducer tests
    #[test]
    fn test_settings_navigate_category_left_from_logging() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Logging;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, _effect) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Data
        );
    }

    #[test]
    fn test_settings_navigate_category_left_from_display() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Display;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, _) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Logging
        );
    }

    #[test]
    fn test_settings_navigate_category_left_from_data() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Data;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryLeft);
        let (new_state, _) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Display
        );
    }

    #[test]
    fn test_settings_navigate_category_right_from_logging() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Logging;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, _effect) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Display
        );
    }

    #[test]
    fn test_settings_navigate_category_right_from_display() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Display;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, _) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Data
        );
    }

    #[test]
    fn test_settings_navigate_category_right_from_data() {
        let mut state = AppState::default();
        state.ui.settings.selected_category = SettingsCategory::Data;

        let action = Action::SettingsAction(SettingsAction::NavigateCategoryRight);
        let (new_state, _) = test_reduce(state, action);

        assert_eq!(
            new_state.ui.settings.selected_category,
            SettingsCategory::Logging
        );
    }

    // TODO: Obsolete tests removed - settings now use document system
    // These tests were for the old modal/editing/selection system which has been replaced

    #[test]
    fn test_set_status_message_with_error() {
        let state = AppState::default();
        let action = Action::SetStatusMessage {
            message: "Test error message".to_string(),
            is_error: true,
        };

        let (new_state, effect) = test_reduce(state, action);

        assert_eq!(
            new_state.system.status_message,
            Some("Test error message".to_string())
        );
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

        let (new_state, effect) = test_reduce(state, action);

        assert_eq!(
            new_state.system.status_message,
            Some("Configuration saved".to_string())
        );
        assert!(!new_state.system.status_is_error);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_toggle_boolean_returns_save_effect() {
        let state = AppState::default();
        let action =
            Action::SettingsAction(SettingsAction::ToggleBoolean("use_unicode".to_string()));

        let (new_state, effect) = test_reduce(state.clone(), action);

        // Config should be toggled
        assert_eq!(
            new_state.system.config.display.use_unicode,
            !state.system.config.display.use_unicode
        );

        // Should return an Async effect (save_config_effect)
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_toggle_boolean_use_unicode_updates_box_chars() {
        let mut state = AppState::default();
        state.system.config.display.use_unicode = true;

        let action =
            Action::SettingsAction(SettingsAction::ToggleBoolean("use_unicode".to_string()));

        let (new_state, _) = test_reduce(state, action);

        // Should toggle to false
        assert!(!new_state.system.config.display.use_unicode);
        // box_chars should be updated to ASCII
        assert_eq!(
            new_state.system.config.display.box_chars,
            crate::formatting::BoxChars::ascii()
        );
    }

    #[test]
    fn test_toggle_boolean_western_teams_first() {
        let state = AppState::default();
        let action = Action::SettingsAction(SettingsAction::ToggleBoolean(
            "western_teams_first".to_string(),
        ));

        let (new_state, _) = test_reduce(state.clone(), action);

        assert_eq!(
            new_state.system.config.display_standings_western_first,
            !state.system.config.display_standings_western_first
        );
    }

    #[test]
    fn test_toggle_boolean_unknown_setting() {
        let state = AppState::default();
        let action =
            Action::SettingsAction(SettingsAction::ToggleBoolean("unknown_setting".to_string()));

        let (new_state, _) = test_reduce(state.clone(), action);

        // State should not change for unknown settings
        assert_eq!(
            new_state.system.config.display.use_unicode,
            state.system.config.display.use_unicode
        );
        assert_eq!(
            new_state.system.config.display_standings_western_first,
            state.system.config.display_standings_western_first
        );
    }

    // TODO: Obsolete editing/modal tests removed - settings now use document system
}
