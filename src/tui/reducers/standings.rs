use std::sync::Arc;

use crate::commands::standings::GroupBy;
use crate::tui::action::{Action, StandingsAction};
use crate::tui::component::Effect;
use crate::tui::components::standings_tab::StandingsTabMsg;
use crate::tui::constants::STANDINGS_TAB_PATH;
use crate::tui::components::{
    ConferenceStandingsDocument, DivisionStandingsDocument, LeagueStandingsDocument,
    WildcardStandingsDocument,
};
use crate::tui::document::Document;
use crate::tui::state::AppState;

/// Sub-reducer for standings tab
///
/// Phase 4: Actions are now routed to ComponentMessage for StandingsTab.
pub fn reduce_standings(
    state: AppState,
    action: StandingsAction,
    component_states: &mut crate::tui::component_store::ComponentStateStore,
) -> (AppState, Effect) {
    match action {
        // Route all actions to component
        StandingsAction::CycleViewLeft => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: STANDINGS_TAB_PATH.to_string(),
                message: Box::new(StandingsTabMsg::CycleViewLeft),
            }),
        ),
        StandingsAction::CycleViewRight => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: STANDINGS_TAB_PATH.to_string(),
                message: Box::new(StandingsTabMsg::CycleViewRight),
            }),
        ),
        StandingsAction::EnterBrowseMode => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: STANDINGS_TAB_PATH.to_string(),
                message: Box::new(StandingsTabMsg::EnterBrowseMode),
            }),
        ),
        StandingsAction::ExitBrowseMode => (
            state,
            Effect::Action(Action::ComponentMessage {
                path: STANDINGS_TAB_PATH.to_string(),
                message: Box::new(StandingsTabMsg::ExitBrowseMode),
            }),
        ),
        StandingsAction::RebuildFocusableMetadata => {
            // Rebuild focusable metadata for current view
            rebuild_focusable_metadata(&state, component_states);
            (state, Effect::None)
        }
    }
}

/// Rebuild focusable metadata for document-based views (Phase 7: Component state)
/// Called from data loading reducer when standings data changes
pub fn rebuild_focusable_metadata(
    state: &AppState,
    component_states: &mut crate::tui::component_store::ComponentStateStore,
) {
    use crate::tui::components::standings_tab::StandingsTabState;

    if let Some(standings) = state.data.standings.as_ref().as_ref() {
        // Get current view from component state
        let view = component_states
            .get::<StandingsTabState>(STANDINGS_TAB_PATH)
            .map(|s| s.view.clone())
            .unwrap_or(GroupBy::Wildcard);

        // Build document for current view and extract metadata
        let (positions, ids, row_positions, link_targets) = match view {
            GroupBy::Conference => {
                let doc = ConferenceStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                (
                    doc.focusable_positions(),
                    doc.focusable_ids(),
                    doc.focusable_row_positions(),
                    doc.focusable_link_targets(),
                )
            }
            GroupBy::Division => {
                let doc = DivisionStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                (
                    doc.focusable_positions(),
                    doc.focusable_ids(),
                    doc.focusable_row_positions(),
                    doc.focusable_link_targets(),
                )
            }
            GroupBy::League => {
                let doc = LeagueStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                (
                    doc.focusable_positions(),
                    doc.focusable_ids(),
                    doc.focusable_row_positions(),
                    doc.focusable_link_targets(),
                )
            }
            GroupBy::Wildcard => {
                let doc = WildcardStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                (
                    doc.focusable_positions(),
                    doc.focusable_ids(),
                    doc.focusable_row_positions(),
                    doc.focusable_link_targets(),
                )
            }
        };

        // Update component state with new metadata
        if let Some(standings_state) = component_states.get_mut::<StandingsTabState>(STANDINGS_TAB_PATH) {
            standings_state.doc_nav.focusable_positions = positions;
            standings_state.doc_nav.focusable_ids = ids;
            standings_state.doc_nav.focusable_row_positions = row_positions;
            standings_state.doc_nav.link_targets = link_targets;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::component_store::ComponentStateStore;

    // Phase 4: Actions now route to ComponentMessage
    // The actual behavior is tested in standings_tab.rs component tests

    #[test]
    fn test_cycle_view_left_dispatches_component_message() {
        let state = AppState::default();
        let mut component_states = ComponentStateStore::new();
        let (new_state, effect) = reduce_standings(state.clone(), StandingsAction::CycleViewLeft, &mut component_states);

        // State should not be modified - action is routed to component (StandingsUiState removed in Phase 7)
        assert_eq!(new_state.data.standings, state.data.standings);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, STANDINGS_TAB_PATH);
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_cycle_view_right_dispatches_component_message() {
        let state = AppState::default();
        let mut component_states = ComponentStateStore::new();
        let (new_state, effect) = reduce_standings(state.clone(), StandingsAction::CycleViewRight, &mut component_states);

        // State should not be modified (StandingsUiState removed in Phase 7)
        assert_eq!(new_state.data.standings, state.data.standings);

        // Should dispatch ComponentMessage
        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, STANDINGS_TAB_PATH);
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_enter_browse_mode_dispatches_component_message() {
        let state = AppState::default();
        let mut component_states = ComponentStateStore::new();
        let (new_state, effect) = reduce_standings(state.clone(), StandingsAction::EnterBrowseMode, &mut component_states);

        // State should not be modified (StandingsUiState removed in Phase 7)
        assert_eq!(new_state.data.standings, state.data.standings);

        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, STANDINGS_TAB_PATH);
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }

    #[test]
    fn test_exit_browse_mode_dispatches_component_message() {
        let state = AppState::default();
        let mut component_states = ComponentStateStore::new();
        let (new_state, effect) = reduce_standings(state.clone(), StandingsAction::ExitBrowseMode, &mut component_states);

        // State should not be modified (StandingsUiState removed in Phase 7)
        assert_eq!(new_state.data.standings, state.data.standings);

        match effect {
            Effect::Action(Action::ComponentMessage { path, .. }) => {
                assert_eq!(path, STANDINGS_TAB_PATH);
            }
            _ => panic!("Expected ComponentMessage effect"),
        }
    }
}
