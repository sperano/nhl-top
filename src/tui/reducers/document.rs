//! Document reducer for handling document navigation actions
//!
//! Handles Tab/Shift-Tab focus navigation and scrolling within documents.

use tracing::debug;

use crate::tui::action::{Action, DocumentAction};
use crate::tui::component::Effect;
use crate::tui::state::AppState;

/// Number of focusable elements in the demo document
/// This should match the number of links in DemoDocument::build()
const DEMO_FOCUSABLE_COUNT: usize = 10;

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
    match action {
        DocumentAction::FocusNext => {
            debug!("Document: focus_next");
            let demo = &mut state.ui.demo;

            let wrapped = match demo.focus_index {
                None => {
                    // No focus yet, focus first element
                    demo.focus_index = Some(0);
                    false
                }
                Some(idx) if idx + 1 >= DEMO_FOCUSABLE_COUNT => {
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
            debug!("Document: focus_prev");
            let demo = &mut state.ui.demo;

            let wrapped = match demo.focus_index {
                None => {
                    // No focus yet, focus last element
                    demo.focus_index = Some(DEMO_FOCUSABLE_COUNT - 1);
                    // Scroll to bottom to show last element
                    // This is a rough estimate - proper implementation would use actual content height
                    demo.scroll_offset = 100; // Will be clamped by rendering
                    true // Treat as wrapped - don't autoscroll
                }
                Some(0) => {
                    // At first element, wrap to last
                    demo.focus_index = Some(DEMO_FOCUSABLE_COUNT - 1);
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
    }
}

/// Autoscroll to keep the focused element visible
/// This is a simplified version - proper implementation would use FocusManager positions
fn autoscroll_to_focus(state: &mut AppState) {
    // For now, this is a rough approximation based on focus index
    // The actual y-position depends on the document layout
    // Each link in demo is approximately 2 lines apart (link + spacer)
    if let Some(focus_idx) = state.ui.demo.focus_index {
        // Rough estimate: each focusable is ~2 lines, plus header content
        let header_lines: u16 = 15; // Approximate header/intro content
        let estimated_y = header_lines + (focus_idx as u16 * 2);

        let viewport_height = state.ui.demo.viewport_height.max(10);
        let padding: u16 = 2;

        // If element is above viewport, scroll up
        if estimated_y < state.ui.demo.scroll_offset + padding {
            state.ui.demo.scroll_offset = estimated_y.saturating_sub(padding);
        }
        // If element is below viewport, scroll down
        else if estimated_y >= state.ui.demo.scroll_offset + viewport_height - padding {
            state.ui.demo.scroll_offset = estimated_y.saturating_sub(viewport_height - padding - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(DEMO_FOCUSABLE_COUNT - 1);
        state.ui.demo.scroll_offset = 50;
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
        assert_eq!(new_state.ui.demo.scroll_offset, 0); // Scrolled to top
    }

    #[test]
    fn test_focus_prev_from_none() {
        let state = AppState::default();
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(DEMO_FOCUSABLE_COUNT - 1));
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
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(DEMO_FOCUSABLE_COUNT - 1));
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
}
