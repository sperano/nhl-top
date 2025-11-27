//! Document reducer for handling document navigation actions
//!
//! Phase 7: This reducer is now obsolete. Document navigation is handled by components
//! (StandingsTab, DemoTab) using DocumentNavState and DocumentNavMsg.
//!
//! The only remaining action handled here is UpdateViewportHeight, which dispatches
//! ComponentMessages to update viewport height in component state.

use crate::tui::action::{Action, DocumentAction};
use crate::tui::component::Effect;
use crate::tui::state::AppState;

/// Handle all document-related actions (Phase 7: Route to components)
pub fn reduce_document(
    state: &AppState,
    action: &Action,
    _component_states: &mut crate::tui::component_store::ComponentStateStore,
) -> Option<(AppState, Effect)> {
    match action {
        Action::DocumentAction(DocumentAction::UpdateViewportHeight { demo, standings }) => {
            // Dispatch to both demo and standings components
            Some((
                state.clone(),
                Effect::Batch(vec![
                    Effect::Action(Action::ComponentMessage {
                        path: "app/demo_tab".to_string(),
                        message: Box::new(
                            crate::tui::components::demo_tab::DemoTabMessage::UpdateViewportHeight(
                                *demo,
                            ),
                        ),
                    }),
                    Effect::Action(Action::ComponentMessage {
                        path: "app/standings_tab".to_string(),
                        message: Box::new(
                            crate::tui::components::standings_tab::StandingsTabMsg::UpdateViewportHeight(
                                *standings,
                            ),
                        ),
                    }),
                ]),
            ))
        }
        // All other DocumentActions are now handled by components via ComponentMessage
        Action::DocumentAction(_) => None,
        _ => None,
    }
}
