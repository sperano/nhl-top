/// Tests for experimental mode integration
///
/// These tests verify that the Runtime and keyboard handling work correctly

#[cfg(test)]
mod tests {
    use crate::tui::framework::{Action, Runtime, DataEffects, AppState};
    use crate::tui::framework::keys::key_to_action;
    use crate::tui::testing::create_client;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    fn create_test_runtime() -> Runtime {
        let client = create_client();
        let data_effects = Arc::new(DataEffects::new(client));
        Runtime::new(AppState::default(), data_effects)
    }

    #[tokio::test]
    async fn test_runtime_initializes() {
        let runtime = create_test_runtime();
        let state = runtime.state();

        // Should start on Scores tab
        assert_eq!(state.navigation.current_tab, crate::tui::framework::action::Tab::Scores);
    }

    #[tokio::test]
    async fn test_tab_navigation_keys() {
        let runtime = create_test_runtime();
        let state = runtime.state();

        // Test number keys - should work on any tab regardless of focus
        let key1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        let action1 = key_to_action(key1, state);
        assert!(matches!(action1, Some(Action::NavigateTab(_))));

        // With tab bar focused (default), arrows should navigate tabs
        let key_right = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        let action_right = key_to_action(key_right, state);
        assert!(matches!(action_right, Some(Action::NavigateTabRight)));

        let key_left = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
        let action_left = key_to_action(key_left, state);
        assert!(matches!(action_left, Some(Action::NavigateTabLeft)));
    }

    #[tokio::test]
    async fn test_quit_key() {
        let runtime = create_test_runtime();
        let state = runtime.state();

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let action = key_to_action(key, state);

        assert!(matches!(action, Some(Action::Quit)));
    }

    #[tokio::test]
    async fn test_focus_level_keys() {
        let mut runtime = create_test_runtime();
        let state = runtime.state();

        // Start with tab bar focused - Down should enter content focus
        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        let action_down = key_to_action(key_down, state);
        assert!(matches!(action_down, Some(Action::EnterContentFocus)));

        // After entering content focus, arrows should be context-sensitive
        runtime.dispatch(Action::EnterContentFocus);
        let state = runtime.state();
        assert!(state.navigation.content_focused);

        // Now arrows should navigate dates on Scores tab
        let key_right = KeyEvent::new(KeyCode::Right, KeyModifiers::empty());
        let action_right = key_to_action(key_right, state);
        assert!(matches!(action_right, Some(Action::ScoresAction(_))));

        // Up should return to tab bar
        let key_up = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        let action_up = key_to_action(key_up, state);
        assert!(matches!(action_up, Some(Action::ExitContentFocus)));
    }

    #[tokio::test]
    async fn test_action_dispatching() {
        let mut runtime = create_test_runtime();

        // Dispatch a NavigateTabRight action
        runtime.dispatch(Action::NavigateTabRight);

        // State should have changed
        let state = runtime.state();
        assert_eq!(state.navigation.current_tab, crate::tui::framework::action::Tab::Standings);
    }

    #[tokio::test]
    async fn test_tab_cycling() {
        let mut runtime = create_test_runtime();

        // Start on Scores, go right to Standings
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Standings);

        // Go right to Stats
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Stats);

        // Go right to Players
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Players);

        // Go right to Settings
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Settings);

        // Go right to Browser
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Browser);

        // Go right to wrap around to Scores
        runtime.dispatch(Action::NavigateTabRight);
        assert_eq!(runtime.state().navigation.current_tab, crate::tui::framework::action::Tab::Scores);
    }

}
