use std::sync::Arc;

use crate::commands::standings::GroupBy;
use crate::tui::action::StandingsAction;
use crate::tui::component::Effect;
use crate::tui::components::{
    ConferenceStandingsDocument, DivisionStandingsDocument, LeagueStandingsDocument,
    WildcardStandingsDocument,
};
use crate::tui::document::Document;
use crate::tui::state::AppState;

/// Sub-reducer for standings tab
pub fn reduce_standings(state: AppState, action: StandingsAction) -> (AppState, Effect) {
    match action {
        StandingsAction::CycleViewLeft => handle_cycle_view_left(state),
        StandingsAction::CycleViewRight => handle_cycle_view_right(state),
        StandingsAction::EnterBrowseMode => handle_enter_browse_mode(state),
        StandingsAction::ExitBrowseMode => handle_exit_browse_mode(state),
    }
}

fn handle_cycle_view_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = new_state.ui.standings.view.prev();

    // Rebuild focusable metadata for the new view
    rebuild_focusable_metadata(&mut new_state);

    // Reset document focus when changing views
    new_state.ui.standings_doc.focus_index = None;
    new_state.ui.standings_doc.scroll_offset = 0;

    (new_state, Effect::None)
}

fn handle_cycle_view_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.view = new_state.ui.standings.view.next();

    // Rebuild focusable metadata for the new view
    rebuild_focusable_metadata(&mut new_state);

    // Reset document focus when changing views
    new_state.ui.standings_doc.focus_index = None;
    new_state.ui.standings_doc.scroll_offset = 0;

    (new_state, Effect::None)
}

fn handle_enter_browse_mode(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.browse_mode = true;

    // Initialize document focus to first team and reset scroll
    new_state.ui.standings_doc.focus_index = Some(0);
    new_state.ui.standings_doc.scroll_offset = 0;

    (new_state, Effect::None)
}

fn handle_exit_browse_mode(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.standings.browse_mode = false;
    (new_state, Effect::None)
}

/// Rebuild focusable metadata for document-based views
pub fn rebuild_focusable_metadata(state: &mut AppState) {
    if let Some(standings) = state.data.standings.as_ref().as_ref() {
        match state.ui.standings.view {
            GroupBy::Conference => {
                let conference_doc = ConferenceStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                state.ui.standings_doc.focusable_positions = conference_doc.focusable_positions();
                state.ui.standings_doc.focusable_ids = conference_doc.focusable_ids();
                state.ui.standings_doc.focusable_row_positions =
                    conference_doc.focusable_row_positions();
            }
            GroupBy::Division => {
                let division_doc = DivisionStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                state.ui.standings_doc.focusable_positions = division_doc.focusable_positions();
                state.ui.standings_doc.focusable_ids = division_doc.focusable_ids();
                state.ui.standings_doc.focusable_row_positions =
                    division_doc.focusable_row_positions();
            }
            GroupBy::League => {
                let league_doc = LeagueStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                state.ui.standings_doc.focusable_positions = league_doc.focusable_positions();
                state.ui.standings_doc.focusable_ids = league_doc.focusable_ids();
                state.ui.standings_doc.focusable_row_positions =
                    league_doc.focusable_row_positions();
            }
            GroupBy::Wildcard => {
                let wildcard_doc = WildcardStandingsDocument::new(
                    Arc::new(standings.clone()),
                    state.system.config.clone(),
                );
                state.ui.standings_doc.focusable_positions = wildcard_doc.focusable_positions();
                state.ui.standings_doc.focusable_ids = wildcard_doc.focusable_ids();
                state.ui.standings_doc.focusable_row_positions =
                    wildcard_doc.focusable_row_positions();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_view_left_wildcard_to_league() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;

        let (new_state, effect) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::League);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_left_division_to_wildcard() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_cycle_view_left_conference_to_division() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
    }

    #[test]
    fn test_cycle_view_left_league_to_conference() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewLeft);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_cycle_view_right_wildcard_to_division() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Wildcard;

        let (new_state, effect) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_right_division_to_conference() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }

    #[test]
    fn test_cycle_view_right_conference_to_league() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::League);
    }

    #[test]
    fn test_cycle_view_right_league_to_wildcard() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    #[test]
    fn test_enter_browse_mode() {
        let mut state = AppState::default();
        state.ui.standings.browse_mode = false;

        let (new_state, effect) = reduce_standings(state, StandingsAction::EnterBrowseMode);

        assert!(new_state.ui.standings.browse_mode);
        assert_eq!(new_state.ui.standings_doc.focus_index, Some(0));
        assert_eq!(new_state.ui.standings_doc.scroll_offset, 0);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_exit_browse_mode() {
        let mut state = AppState::default();
        state.ui.standings.browse_mode = true;

        let (new_state, effect) = reduce_standings(state, StandingsAction::ExitBrowseMode);

        assert!(!new_state.ui.standings.browse_mode);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_cycle_view_resets_document_focus() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Division;
        state.ui.standings_doc.focus_index = Some(5);
        state.ui.standings_doc.scroll_offset = 10;

        let (new_state, _) = reduce_standings(state, StandingsAction::CycleViewRight);

        // Document focus should be reset when changing views
        assert_eq!(new_state.ui.standings_doc.focus_index, None);
        assert_eq!(new_state.ui.standings_doc.scroll_offset, 0);
        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);
    }
}
