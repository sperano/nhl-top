//! Document reducer for handling document navigation actions
//!
//! Handles Tab/Shift-Tab focus navigation and scrolling within documents.

use tracing::debug;

use crate::tui::action::{Action, DocumentAction};
use crate::tui::component::Effect;
use crate::tui::state::AppState;

/// Base number of focusable elements in the demo document (example links section)
const BASE_FOCUSABLE_COUNT: usize = 4; // BOS, TOR, NYR, MTL links

/// Calculate the total focusable count based on standings data
fn get_focusable_count(state: &AppState) -> usize {
    let standings_count = state
        .data
        .standings
        .as_ref()
        .as_ref()
        .map(|s| s.len())
        .unwrap_or(0);

    // standings links + example links
    standings_count + BASE_FOCUSABLE_COUNT
}

/// Handle all document-related actions
pub fn reduce_document(state: &AppState, action: &Action) -> Option<(AppState, Effect)> {
    match action {
        Action::DocumentAction(doc_action) => {
            handle_document_action(state.clone(), doc_action)
        }
        _ => None,
    }
}

fn handle_document_action(mut state: AppState, action: &DocumentAction) -> Option<(AppState, Effect)> {
    let focusable_count = get_focusable_count(&state);

    match action {
        DocumentAction::FocusNext => {
            debug!("Document: focus_next (count={})", focusable_count);

            if focusable_count == 0 {
                return Some((state, Effect::None));
            }

            let demo = &mut state.ui.demo;

            let wrapped = match demo.focus_index {
                None => {
                    // No focus yet, focus first element
                    demo.focus_index = Some(0);
                    false
                }
                Some(idx) if idx + 1 >= focusable_count => {
                    // At last element, wrap to first and scroll to top
                    demo.focus_index = Some(0);
                    demo.scroll_offset = 0;
                    true // Wrapped - don't autoscroll
                }
                Some(idx) => {
                    // Move to next element
                    demo.focus_index = Some(idx + 1);
                    false
                }
            };

            // Autoscroll to keep focused element visible (unless we wrapped)
            if !wrapped {
                autoscroll_to_focus(&mut state);
            }

            Some((state, Effect::None))
        }

        DocumentAction::FocusPrev => {
            debug!("Document: focus_prev (count={})", focusable_count);

            if focusable_count == 0 {
                return Some((state, Effect::None));
            }

            let demo = &mut state.ui.demo;

            let wrapped = match demo.focus_index {
                None => {
                    // No focus yet, focus last element
                    demo.focus_index = Some(focusable_count - 1);
                    // Scroll to bottom to show last element
                    // This is a rough estimate - proper implementation would use actual content height
                    demo.scroll_offset = 100; // Will be clamped by rendering
                    true // Treat as wrapped - don't autoscroll
                }
                Some(0) => {
                    // At first element, wrap to last
                    demo.focus_index = Some(focusable_count - 1);
                    demo.scroll_offset = 100; // Will be clamped
                    true // Wrapped - don't autoscroll
                }
                Some(idx) => {
                    // Move to previous element
                    demo.focus_index = Some(idx - 1);
                    false
                }
            };

            // Autoscroll to keep focused element visible (unless we wrapped)
            if !wrapped {
                autoscroll_to_focus(&mut state);
            }

            Some((state, Effect::None))
        }

        DocumentAction::ActivateFocused => {
            debug!("Document: activate_focused");
            if let Some(idx) = state.ui.demo.focus_index {
                debug!("  Activating link at index {}", idx);
                // TODO: Handle link activation (e.g., navigate to team panel)
            }
            Some((state, Effect::None))
        }

        DocumentAction::ScrollUp(lines) => {
            debug!("Document: scroll_up {}", lines);
            state.ui.demo.scroll_offset = state.ui.demo.scroll_offset.saturating_sub(*lines);
            Some((state, Effect::None))
        }

        DocumentAction::ScrollDown(lines) => {
            debug!("Document: scroll_down {}", lines);
            state.ui.demo.scroll_offset = state.ui.demo.scroll_offset.saturating_add(*lines);
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToTop => {
            debug!("Document: scroll_to_top");
            state.ui.demo.scroll_offset = 0;
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToBottom => {
            debug!("Document: scroll_to_bottom");
            // Set to a large value - rendering will clamp it
            state.ui.demo.scroll_offset = u16::MAX;
            Some((state, Effect::None))
        }

        DocumentAction::PageUp => {
            debug!("Document: page_up");
            let page_size = state.ui.demo.viewport_height.max(10);
            state.ui.demo.scroll_offset = state.ui.demo.scroll_offset.saturating_sub(page_size);
            Some((state, Effect::None))
        }

        DocumentAction::PageDown => {
            debug!("Document: page_down");
            let page_size = state.ui.demo.viewport_height.max(10);
            state.ui.demo.scroll_offset = state.ui.demo.scroll_offset.saturating_add(page_size);
            Some((state, Effect::None))
        }

        DocumentAction::SyncFocusablePositions(positions, viewport_height) => {
            debug!(
                "Document: sync_focusable_positions (count={}, viewport={})",
                positions.len(),
                viewport_height
            );
            state.ui.demo.focusable_positions = positions.clone();
            state.ui.demo.viewport_height = *viewport_height;
            Some((state, Effect::None))
        }
    }
}

/// Autoscroll to keep the focused element visible using actual positions from FocusManager
fn autoscroll_to_focus(state: &mut AppState) {
    let demo = &state.ui.demo;

    // Get the y-position of the focused element from the synced positions
    let element_y = match demo.focus_index {
        Some(idx) if idx < demo.focusable_positions.len() => demo.focusable_positions[idx],
        _ => return, // No focus or positions not synced yet
    };

    // Use actual viewport height, with reasonable fallback
    let viewport_height = demo.viewport_height.max(20);
    let padding: u16 = 2;

    let scroll_offset = demo.scroll_offset;

    // If element is above viewport (with padding), scroll up
    if element_y < scroll_offset.saturating_add(padding) {
        state.ui.demo.scroll_offset = element_y.saturating_sub(padding);
    }
    // If element is below viewport (with padding), scroll down
    else if element_y >= scroll_offset + viewport_height.saturating_sub(padding) {
        state.ui.demo.scroll_offset = element_y.saturating_sub(viewport_height - padding - 1);
    }
    // Otherwise, element is visible - no scroll needed
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to get focusable count for test state
    fn test_focusable_count(state: &AppState) -> usize {
        get_focusable_count(state)
    }

    #[test]
    fn test_focus_next_from_none() {
        let state = AppState::default();
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
    }

    #[test]
    fn test_focus_next_increments() {
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
    }

    #[test]
    fn test_focus_next_wraps() {
        let state = AppState::default();
        let count = test_focusable_count(&state);
        let mut state = state;
        state.ui.demo.focus_index = Some(count - 1);
        state.ui.demo.scroll_offset = 50;
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
        assert_eq!(new_state.ui.demo.scroll_offset, 0); // Scrolled to top
    }

    #[test]
    fn test_focus_prev_from_none() {
        let state = AppState::default();
        let count = test_focusable_count(&state);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(count - 1));
    }

    #[test]
    fn test_focus_prev_decrements() {
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(5);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(4));
    }

    #[test]
    fn test_focus_prev_wraps() {
        let state = AppState::default();
        let count = test_focusable_count(&state);
        let mut state = state;
        state.ui.demo.focus_index = Some(0);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(count - 1));
    }

    #[test]
    fn test_scroll_up() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 10;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollUp(3)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 7);
    }

    #[test]
    fn test_scroll_up_clamps() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 2;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollUp(10)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 10;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollDown(5)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 15);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 50;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollToTop).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let state = AppState::default();
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollToBottom).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_page_up() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 30;
        state.ui.demo.viewport_height = 20;
        let (new_state, _) = handle_document_action(state, &DocumentAction::PageUp).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 10);
    }

    #[test]
    fn test_page_down() {
        let mut state = AppState::default();
        state.ui.demo.scroll_offset = 10;
        state.ui.demo.viewport_height = 20;
        let (new_state, _) = handle_document_action(state, &DocumentAction::PageDown).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 30);
    }

    #[test]
    fn test_autoscroll_no_positions() {
        // When focusable_positions is empty, autoscroll should not change scroll_offset
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 0;
        state.ui.demo.focusable_positions = vec![]; // No positions synced yet

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Focus should advance, but scroll should not change (positions empty)
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_autoscroll_element_visible_no_scroll() {
        // When element is already visible, no scroll needed
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 0;
        state.ui.demo.viewport_height = 30;
        // Positions at lines 10, 12, 14 (all visible in viewport 0-30)
        state.ui.demo.focusable_positions = vec![10, 12, 14];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Focus moves to index 1 (y=12), which is visible - no scroll needed
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_autoscroll_element_below_viewport() {
        // When element is below viewport, scroll down to show it
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 0;
        state.ui.demo.viewport_height = 20;
        // Position at line 25 is below viewport (0-20)
        state.ui.demo.focusable_positions = vec![5, 25];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Focus moves to index 1 (y=25), should scroll to make it visible
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
        // Scroll offset should be adjusted: element_y - (viewport_height - padding - 1)
        // = 25 - (20 - 2 - 1) = 25 - 17 = 8
        assert_eq!(new_state.ui.demo.scroll_offset, 8);
    }

    #[test]
    fn test_autoscroll_element_above_viewport() {
        // When element is above viewport, scroll up to show it
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(1);
        state.ui.demo.scroll_offset = 20; // Viewing lines 20-40
        state.ui.demo.viewport_height = 20;
        // Position at line 5 is above viewport
        state.ui.demo.focusable_positions = vec![5, 25];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();

        // Focus moves to index 0 (y=5), should scroll up to make it visible
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
        // Scroll offset should be adjusted: element_y - padding = 5 - 2 = 3
        assert_eq!(new_state.ui.demo.scroll_offset, 3);
    }

    #[test]
    fn test_sync_focusable_positions() {
        let state = AppState::default();
        let positions = vec![10, 20, 30, 40];
        let viewport_height = 25;

        let (new_state, _) = handle_document_action(
            state,
            &DocumentAction::SyncFocusablePositions(positions.clone(), viewport_height),
        )
        .unwrap();

        assert_eq!(new_state.ui.demo.focusable_positions, positions);
        assert_eq!(new_state.ui.demo.viewport_height, viewport_height);
    }
}
