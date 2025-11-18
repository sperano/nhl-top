use tracing::debug;

use super::action::{Action, SettingsAction};
use super::component::Effect;
use super::state::AppState;
use super::types::{Panel, SettingsCategory};

// Import sub-reducers from the parent framework module
use crate::tui::reducers::{
    reduce_navigation,
    reduce_panels,
    reduce_data_loading,
    reduce_scores,
    reduce_standings,
};

/// Pure state reducer - like Redux reducer
///
/// Takes current state and an action, returns new state and optional effect.
/// This function is PURE - no side effects, no I/O, no async.
/// All side effects are returned as `Effect` to be executed separately.
pub fn reduce(state: AppState, action: Action) -> (AppState, Effect) {
    // Try each sub-reducer in order
    // Each returns Option<(AppState, Effect)> - None means it didn't handle the action

    // Navigation actions
    if let Some(result) = reduce_navigation(state.clone(), &action) {
        return result;
    }

    // Panel management actions
    if let Some(result) = reduce_panels(state.clone(), &action) {
        return result;
    }

    // Data loading actions
    if let Some(result) = reduce_data_loading(state.clone(), &action) {
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

        // For now, stub out the other settings actions
        // TODO: Implement these properly when moving to separate module
        _ => (state, Effect::None),
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
}