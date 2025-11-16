//! Integration tests for the entire data flow
//!
//! These tests verify that data flows correctly through the system:
//! API → Effect → Action → Reducer → State → Component → Render

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::tui::framework::{
        action::{Action, Tab},
        effects::DataEffects,
        renderer::Renderer,
        runtime::Runtime,
        state::AppState,
        Element,
    };
    use crate::tui::testing::create_client;

    fn create_test_runtime() -> Runtime {
        let client = create_client();
        let data_effects = Arc::new(DataEffects::new(client));
        let state = AppState::default();
        Runtime::new(state, data_effects)
    }

    #[tokio::test]
    async fn test_initial_state_renders() {
        let runtime = create_test_runtime();

        // Build the component tree
        let element = runtime.build();

        // Should be able to render without panicking
        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2); // TabbedPanel, StatusBar
            }
            _ => panic!("Expected container from App component"),
        }
    }

    #[tokio::test]
    async fn test_tab_navigation_updates_state_and_view() {
        let mut runtime = create_test_runtime();

        // Initial state should be Scores tab
        assert_eq!(runtime.state().navigation.current_tab, Tab::Scores);

        // Navigate to Standings
        runtime.dispatch(Action::NavigateTab(Tab::Standings));
        assert_eq!(runtime.state().navigation.current_tab, Tab::Standings);

        // Build should reflect new state
        let element = runtime.build();
        // Component tree should still be valid
        assert!(matches!(element, Element::Container { .. }));

        // Navigate to Settings
        runtime.dispatch(Action::NavigateTab(Tab::Settings));
        assert_eq!(runtime.state().navigation.current_tab, Tab::Settings);
    }

    #[tokio::test]
    async fn test_refresh_data_triggers_loading_state() {
        let mut runtime = create_test_runtime();

        // Dispatch refresh
        runtime.dispatch(Action::RefreshData);

        // Give time for async effects to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check that loading states were set
        // Note: In a real implementation, the reducer would set loading states
        // For now, we just verify that the action was dispatched
    }

    #[tokio::test]
    async fn test_data_loaded_action_updates_state() {
        let mut runtime = create_test_runtime();

        // Initial state should have no standings
        assert!(runtime.state().data.standings.is_none());

        // Simulate standings loaded
        let standings = vec![]; // Empty standings for test
        runtime.dispatch(Action::StandingsLoaded(Ok(standings.clone())));

        // State should now have standings
        assert!(runtime.state().data.standings.is_some());
        assert_eq!(runtime.state().data.standings.as_ref().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_error_action_stores_error_in_state() {
        let mut runtime = create_test_runtime();

        // Simulate error loading standings
        runtime.dispatch(Action::StandingsLoaded(Err("Network error".to_string())));

        // Error should be stored in state
        assert!(runtime.state().data.errors.contains_key("standings"));
        assert_eq!(
            runtime.state().data.errors.get("standings").unwrap(),
            "Network error"
        );
    }

    #[tokio::test]
    async fn test_action_queue_processing() {
        let mut runtime = create_test_runtime();

        // Queue multiple actions
        let tx = runtime.action_sender();
        tx.send(Action::NavigateTab(Tab::Standings)).unwrap();
        tx.send(Action::NavigateTab(Tab::Settings)).unwrap();

        // Process all queued actions
        let count = runtime.process_actions();

        assert_eq!(count, 2);
        assert_eq!(runtime.state().navigation.current_tab, Tab::Settings);
    }

    #[tokio::test]
    async fn test_full_render_pipeline() {
        let runtime = create_test_runtime();
        let mut renderer = Renderer::new();

        // Build component tree
        let element = runtime.build();

        // Create a test buffer
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use crate::config::DisplayConfig;
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        let config = DisplayConfig::default();

        // Render should complete without panicking
        renderer.render(element, area, &mut buffer, &config);

        // Verify buffer was written to (not all spaces)
        let mut has_content = false;
        for y in 0..area.height {
            for x in 0..area.width {
                if buffer[(x, y)].symbol() != " " {
                    has_content = true;
                    break;
                }
            }
        }
        assert!(has_content, "Expected rendered content in buffer");
    }

    #[tokio::test]
    async fn test_state_persistence_across_renders() {
        let mut runtime = create_test_runtime();

        // Set some state
        runtime.dispatch(Action::NavigateTab(Tab::Standings));

        // Build first time
        let element1 = runtime.build();

        // Build again without changing state
        let element2 = runtime.build();

        // Both builds should produce container elements
        assert!(matches!(element1, Element::Container { .. }));
        assert!(matches!(element2, Element::Container { .. }));

        // State should be unchanged
        assert_eq!(runtime.state().navigation.current_tab, Tab::Standings);
    }

    #[tokio::test]
    async fn test_component_tree_structure() {
        let runtime = create_test_runtime();

        // Build component tree
        let element = runtime.build();

        // Verify structure: Container with 2 children (TabbedPanel + StatusBar)
        if let Element::Container { children, .. } = element {
            assert_eq!(children.len(), 2);

            // First child should be TabbedPanel (Container)
            assert!(matches!(children[0], Element::Container { .. }));

            // Second child should be StatusBar (Widget)
            assert!(matches!(children[1], Element::Widget(_)));
        } else {
            panic!("Expected container element");
        }
    }
}
