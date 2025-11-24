use tracing::{debug, trace};

use crate::tui::action::Action;
use crate::tui::component::Effect;
use crate::tui::state::AppState;
use crate::tui::types::Tab;

/// Handle all navigation-related actions
pub fn reduce_navigation(state: &AppState, action: &Action) -> Option<(AppState, Effect)> {
    match action {
        Action::NavigateTab(tab) => Some(navigate_to_tab(state.clone(), *tab)),
        Action::NavigateTabLeft => Some(navigate_tab_left(state.clone())),
        Action::NavigateTabRight => Some(navigate_tab_right(state.clone())),
        Action::EnterContentFocus => Some(enter_content_focus(state.clone())),
        Action::ExitContentFocus => Some(exit_content_focus(state.clone())),
        Action::ToggleCommandPalette => {
            // TODO: Implement command palette toggling
            Some((state.clone(), Effect::None))
        }
        _ => None,
    }
}

fn navigate_to_tab(state: AppState, tab: Tab) -> (AppState, Effect) {
    trace!("Navigating to tab: {:?}", tab);
    let mut new_state = state;
    new_state.navigation.current_tab = tab;
    new_state.navigation.panel_stack.clear();
    new_state.navigation.content_focused = false; // Return focus to tab bar
    trace!("  Cleared panel stack and returned focus to tab bar");
    (new_state, Effect::None)
}

fn navigate_tab_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.navigation.current_tab = match new_state.navigation.current_tab {
        Tab::Scores => Tab::Demo,
        Tab::Standings => Tab::Scores,
        Tab::Stats => Tab::Standings,
        Tab::Players => Tab::Stats,
        Tab::Settings => Tab::Players,
        Tab::Demo => Tab::Settings,
    };
    new_state.navigation.panel_stack.clear();
    new_state.navigation.content_focused = false; // Return focus to tab bar
    (new_state, Effect::None)
}

fn navigate_tab_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.navigation.current_tab = match new_state.navigation.current_tab {
        Tab::Scores => Tab::Standings,
        Tab::Standings => Tab::Stats,
        Tab::Stats => Tab::Players,
        Tab::Players => Tab::Settings,
        Tab::Settings => Tab::Demo,
        Tab::Demo => Tab::Scores,
    };
    new_state.navigation.panel_stack.clear();
    new_state.navigation.content_focused = false; // Return focus to tab bar
    (new_state, Effect::None)
}

fn enter_content_focus(state: AppState) -> (AppState, Effect) {
    debug!("FOCUS: Entering content focus (Down key from tab bar)");
    let mut new_state = state;
    new_state.navigation.content_focused = true;

    // Set tab-specific status message
    if new_state.navigation.current_tab == Tab::Demo {
        new_state
            .system
            .set_status_message("↑↓: move selection  Shift+↑↓: scroll  Esc: go back".to_string());
    }

    (new_state, Effect::None)
}

fn exit_content_focus(state: AppState) -> (AppState, Effect) {
    debug!("FOCUS: Exiting content focus (Up key to tab bar)");
    let mut new_state = state;

    // Reset status message if exiting from Demo tab
    if new_state.navigation.current_tab == Tab::Demo {
        new_state.system.reset_status_message();
    }

    new_state.navigation.content_focused = false;

    // Also exit any tab-specific modes when returning to tab bar
    new_state.ui.scores.box_selection_active = false;
    new_state.ui.standings.browse_mode = false;
    new_state.ui.settings.settings_mode = false;

    (new_state, Effect::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_to_tab() {
        let state = AppState::default();
        let (new_state, _) = navigate_to_tab(state, Tab::Settings);

        assert_eq!(new_state.navigation.current_tab, Tab::Settings);
        assert!(new_state.navigation.panel_stack.is_empty());
        assert!(!new_state.navigation.content_focused);
    }

    #[test]
    fn test_tab_left_navigation_cycles() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Scores;

        let (state, _) = navigate_tab_left(state);
        assert_eq!(state.navigation.current_tab, Tab::Demo);

        let (state, _) = navigate_tab_left(state);
        assert_eq!(state.navigation.current_tab, Tab::Settings);
    }

    #[test]
    fn test_tab_right_navigation_cycles() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;

        let (state, _) = navigate_tab_right(state);
        assert_eq!(state.navigation.current_tab, Tab::Scores);

        let (state, _) = navigate_tab_right(state);
        assert_eq!(state.navigation.current_tab, Tab::Standings);
    }

    #[test]
    fn test_content_focus_resets_tab_modes() {
        let mut state = AppState::default();
        state.ui.scores.box_selection_active = true;
        state.ui.standings.browse_mode = true;
        state.ui.settings.settings_mode = true;

        let (new_state, _) = exit_content_focus(state);

        assert!(!new_state.ui.scores.box_selection_active);
        assert!(!new_state.ui.standings.browse_mode);
        assert!(!new_state.ui.settings.settings_mode);
        assert!(!new_state.navigation.content_focused);
    }
}
