
use crate::tui::action::{Action, ScoresAction};
use crate::tui::component::Effect;
use crate::tui::components::scores_tab::ScoresTabMsg;
use crate::tui::state::{AppState, PanelState};
use crate::tui::types::Panel;

/// Sub-reducer for scores tab
///
/// Phase 3.5: Most actions are now routed to ComponentMessage for ScoresTab.
/// Only SelectGame and SelectGameById remain here since they modify global navigation state.
pub fn reduce_scores(state: AppState, action: ScoresAction) -> (AppState, Effect) {
    match action {
        // Route navigation actions to component
        ScoresAction::DateLeft => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::NavigateLeft),
            }),
        ),
        ScoresAction::DateRight => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::NavigateRight),
            }),
        ),
        ScoresAction::EnterBoxSelection => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::EnterBoxSelection),
            }),
        ),
        ScoresAction::ExitBoxSelection => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::ExitBoxSelection),
            }),
        ),
        ScoresAction::MoveGameSelectionUp => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::MoveGameSelectionUp),
            }),
        ),
        ScoresAction::MoveGameSelectionDown => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::MoveGameSelectionDown),
            }),
        ),
        ScoresAction::MoveGameSelectionLeft => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::MoveGameSelectionLeft),
            }),
        ),
        ScoresAction::MoveGameSelectionRight => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::MoveGameSelectionRight),
            }),
        ),
        ScoresAction::UpdateBoxesPerRow(boxes_per_row) => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: "app/scores_tab".to_string(),
                message: Box::new(ScoresTabMsg::UpdateBoxesPerRow(boxes_per_row)),
            }),
        ),

        // Keep these in reducer - they modify global navigation state
        ScoresAction::SelectGame => handle_select_game(state),
        ScoresAction::SelectGameById(game_id) => handle_select_game_by_id(state, game_id),
    }
}

fn handle_select_game(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    if let Some(selected_index) = new_state.ui.scores.selected_game_index {
        if let Some(schedule) = new_state.data.schedule.as_ref().as_ref() {
            if let Some(game) = schedule.games.get(selected_index) {
                let game_id = game.id;

                // Push boxscore panel onto stack
                new_state.navigation.panel_stack.push(PanelState {
                    panel: Panel::Boxscore { game_id },
                    selected_index: Some(0), // Start with first player selected
                });

                return (new_state, Effect::None);
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_select_game_by_id(state: AppState, game_id: i64) -> (AppState, Effect) {
    let mut new_state = state;

    // Push boxscore panel onto stack
    new_state.navigation.panel_stack.push(PanelState {
        panel: Panel::Boxscore { game_id },
        selected_index: Some(0), // Start with first player selected
    });

    (new_state, Effect::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Phase 3.5: Most actions now route to ComponentMessage
    // The actual behavior is tested in scores_tab.rs component tests

    #[test]
    fn test_date_left_dispatches_component_message() {
        let state = AppState::default();
        let (new_state, effect) = reduce_scores(state.clone(), ScoresAction::DateLeft);

        // State should not be modified - action is routed to component
        assert_eq!(new_state.ui.scores, state.ui.scores);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/scores_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_date_right_dispatches_component_message() {
        let state = AppState::default();
        let (new_state, effect) = reduce_scores(state.clone(), ScoresAction::DateRight);

        // State should not be modified
        assert_eq!(new_state.ui.scores, state.ui.scores);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/scores_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_enter_box_selection_dispatches_component_message() {
        let state = AppState::default();
        let (new_state, effect) = reduce_scores(state.clone(), ScoresAction::EnterBoxSelection);

        // State should not be modified
        assert_eq!(new_state.ui.scores, state.ui.scores);

        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/scores_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_exit_box_selection_dispatches_component_message() {
        let state = AppState::default();
        let (new_state, effect) = reduce_scores(state.clone(), ScoresAction::ExitBoxSelection);

        // State should not be modified
        assert_eq!(new_state.ui.scores, state.ui.scores);

        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, "app/scores_tab");
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    // Test actions that still modify global state (panel stack)

    #[test]
    fn test_select_game_with_no_selection() {
        let state = AppState::default();

        let (new_state, effect) = reduce_scores(state, ScoresAction::SelectGame);

        assert_eq!(new_state.navigation.panel_stack.len(), 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_select_game_by_id() {
        let state = AppState::default();

        let (new_state, effect) = reduce_scores(state, ScoresAction::SelectGameById(98765));

        assert_eq!(new_state.navigation.panel_stack.len(), 1);
        match &new_state.navigation.panel_stack[0].panel {
            Panel::Boxscore { game_id } => {
                assert_eq!(*game_id, 98765);
            }
            _ => panic!("Expected Boxscore panel"),
        }
        assert!(matches!(effect, Effect::None));
    }
}
