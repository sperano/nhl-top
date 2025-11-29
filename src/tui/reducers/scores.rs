use crate::tui::action::{Action, ScoresAction};
use crate::tui::component::Effect;
use crate::tui::components::scores_tab::ScoresTabMsg;
use crate::tui::constants::SCORES_TAB_PATH;
use crate::tui::state::{AppState, DocumentStackEntry};
use crate::tui::types::StackedDocument;

/// Sub-reducer for scores tab
///
/// Most actions are now routed to ComponentMessage for ScoresTab.
/// Only SelectGame remains here since it modifies global navigation state (document stack).
pub fn reduce_scores(state: AppState, action: ScoresAction) -> (AppState, Effect) {
    match action {
        // Route navigation actions to component
        ScoresAction::DateLeft => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: SCORES_TAB_PATH.to_string(),
                message: Box::new(ScoresTabMsg::NavigateLeft),
            }),
        ),
        ScoresAction::DateRight => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: SCORES_TAB_PATH.to_string(),
                message: Box::new(ScoresTabMsg::NavigateRight),
            }),
        ),
        ScoresAction::EnterBoxSelection => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: SCORES_TAB_PATH.to_string(),
                message: Box::new(ScoresTabMsg::EnterBoxSelection),
            }),
        ),
        ScoresAction::ExitBoxSelection => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: SCORES_TAB_PATH.to_string(),
                message: Box::new(ScoresTabMsg::ExitBoxSelection),
            }),
        ),

        // Keep this in reducer - it modifies global navigation state
        ScoresAction::SelectGame(game_id) => handle_select_game(state, game_id),
    }
}

fn handle_select_game(state: AppState, game_id: i64) -> (AppState, Effect) {
    let mut new_state = state;

    // Push boxscore document onto stack
    new_state.navigation.document_stack.push(DocumentStackEntry {
        document: StackedDocument::Boxscore { game_id },
        selected_index: Some(0), // Start with first player selected
        scroll_offset: 0,
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
                assert_eq!(path, SCORES_TAB_PATH);
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
                assert_eq!(path, SCORES_TAB_PATH);
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
                assert_eq!(path, SCORES_TAB_PATH);
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
                assert_eq!(path, SCORES_TAB_PATH);
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    // Test actions that still modify global state (document stack)

    #[test]
    fn test_select_game() {
        let state = AppState::default();

        let (new_state, effect) = reduce_scores(state, ScoresAction::SelectGame(98765));

        assert_eq!(new_state.navigation.document_stack.len(), 1);
        match &new_state.navigation.document_stack[0].document {
            StackedDocument::Boxscore { game_id } => {
                assert_eq!(*game_id, 98765);
            }
            _ => panic!("Expected Boxscore document"),
        }
        assert!(matches!(effect, Effect::None));
    }
}
