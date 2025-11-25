//! Document reducer for handling document navigation actions
//!
//! Handles Tab/Shift-Tab focus navigation and scrolling within documents.

use tracing::debug;

use crate::tui::action::{Action, DocumentAction};
use crate::tui::component::Effect;
use crate::tui::document::RowPosition;
use crate::tui::state::AppState;
use crate::tui::types::Tab;

/// Base number of focusable elements in the demo document
/// - 4 example links (BOS, TOR, NYR, MTL)
/// - 10 player table cells (5 forwards + 5 defensemen, 1 link column each)
const BASE_FOCUSABLE_COUNT: usize = 14;

/// Default viewport height when actual height not yet known
const DEFAULT_VIEWPORT_HEIGHT: u16 = 20;

/// Padding lines above/below focused element for autoscroll
const AUTOSCROLL_PADDING: u16 = 2;

/// Minimum page size for page up/down operations
const MIN_PAGE_SIZE: u16 = 10;


/// Calculate the total focusable count based on standings data
fn get_focusable_count(state: &AppState) -> usize {
    match state.navigation.current_tab {
        Tab::Demo => {
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
        Tab::Standings => {
            // For League view, focusable count comes from the document metadata
            state.ui.standings.focusable_positions.len()
        }
        _ => 0,
    }
}

/// Get a mutable reference to the document UI state for the current tab
fn get_doc_state_mut(state: &mut AppState) -> Option<(&mut Option<usize>, &mut u16, &Vec<u16>, &Vec<crate::tui::document::FocusableId>, &Vec<Option<RowPosition>>)> {
    match state.navigation.current_tab {
        Tab::Demo => Some((
            &mut state.ui.demo.focus_index,
            &mut state.ui.demo.scroll_offset,
            &state.ui.demo.focusable_positions,
            &state.ui.demo.focusable_ids,
            &state.ui.demo.focusable_row_positions,
        )),
        Tab::Standings if state.ui.standings.view == crate::commands::standings::GroupBy::League => Some((
            &mut state.ui.standings.focus_index,
            &mut state.ui.standings.scroll_offset,
            &state.ui.standings.focusable_positions,
            &state.ui.standings.focusable_ids,
            &state.ui.standings.focusable_row_positions,
        )),
        _ => None,
    }
}

/// Get viewport height for the current tab
fn get_viewport_height(state: &AppState) -> u16 {
    match state.navigation.current_tab {
        Tab::Demo => state.ui.demo.viewport_height,
        Tab::Standings => state.ui.standings.viewport_height,
        _ => DEFAULT_VIEWPORT_HEIGHT,
    }
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

/// Handle document actions for Standings tab (League view)
fn handle_standings_document_action(mut state: AppState, action: &DocumentAction) -> Option<(AppState, Effect)> {
    let focusable_count = state.ui.standings.focusable_positions.len();

    match action {
        DocumentAction::FocusNext => {
            if focusable_count == 0 {
                return Some((state, Effect::None));
            }

            let standings = &mut state.ui.standings;
            let wrapped = match standings.focus_index {
                None => {
                    standings.focus_index = Some(0);
                    false
                }
                Some(idx) if idx + 1 >= focusable_count => {
                    standings.focus_index = Some(0);
                    standings.scroll_offset = 0;
                    true // Wrapped - don't autoscroll
                }
                Some(idx) => {
                    standings.focus_index = Some(idx + 1);
                    false
                }
            };

            // Autoscroll to keep focused element visible (unless we wrapped)
            if !wrapped {
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.standings.focus_index,
                    focusable_positions: &state.ui.standings.focusable_positions,
                    viewport_height: state.ui.standings.viewport_height,
                    scroll_offset: &mut state.ui.standings.scroll_offset,
                });
            }

            Some((state, Effect::None))
        }

        DocumentAction::FocusPrev => {
            if focusable_count == 0 {
                return Some((state, Effect::None));
            }

            let standings = &mut state.ui.standings;
            let wrapped = match standings.focus_index {
                None => {
                    standings.focus_index = Some(focusable_count - 1);
                    standings.scroll_offset = u16::MAX;
                    true // Wrapped - don't autoscroll
                }
                Some(0) => {
                    standings.focus_index = Some(focusable_count - 1);
                    standings.scroll_offset = u16::MAX;
                    true // Wrapped - don't autoscroll
                }
                Some(idx) => {
                    standings.focus_index = Some(idx - 1);
                    false
                }
            };

            // Autoscroll to keep focused element visible (unless we wrapped)
            if !wrapped {
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.standings.focus_index,
                    focusable_positions: &state.ui.standings.focusable_positions,
                    viewport_height: state.ui.standings.viewport_height,
                    scroll_offset: &mut state.ui.standings.scroll_offset,
                });
            }

            Some((state, Effect::None))
        }

        DocumentAction::ScrollUp(lines) => {
            state.ui.standings.scroll_offset = state.ui.standings.scroll_offset.saturating_sub(*lines);
            Some((state, Effect::None))
        }

        DocumentAction::ScrollDown(lines) => {
            state.ui.standings.scroll_offset = state.ui.standings.scroll_offset.saturating_add(*lines);
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToTop => {
            state.ui.standings.scroll_offset = 0;
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToBottom => {
            state.ui.standings.scroll_offset = u16::MAX;
            Some((state, Effect::None))
        }

        DocumentAction::PageUp => {
            let page_size = state.ui.standings.viewport_height.max(MIN_PAGE_SIZE);
            state.ui.standings.scroll_offset = state.ui.standings.scroll_offset.saturating_sub(page_size);
            Some((state, Effect::None))
        }

        DocumentAction::PageDown => {
            let page_size = state.ui.standings.viewport_height.max(MIN_PAGE_SIZE);
            state.ui.standings.scroll_offset = state.ui.standings.scroll_offset.saturating_add(page_size);
            Some((state, Effect::None))
        }

        DocumentAction::ActivateFocused => {
            if let Some(idx) = state.ui.standings.focus_index {
                if let Some(id) = state.ui.standings.focusable_ids.get(idx) {
                    let display_text = id.display_name();
                    debug!("  Activating: {} (id={:?})", display_text, id);
                    state.system.set_status_message(format!("Activated: {}", display_text));
                }
            }
            Some((state, Effect::None))
        }

        // Left/Right and SyncFocusablePositions not used for simple league table yet
        _ => Some((state, Effect::None)),
    }
}

fn handle_document_action(mut state: AppState, action: &DocumentAction) -> Option<(AppState, Effect)> {
    // For Standings tab in League view, handle document actions
    if state.navigation.current_tab == Tab::Standings
        && state.ui.standings.view == crate::commands::standings::GroupBy::League {
        return handle_standings_document_action(state, action);
    }

    // For Demo tab, use the original logic below
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
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.demo.focus_index,
                    focusable_positions: &state.ui.demo.focusable_positions,
                    viewport_height: state.ui.demo.viewport_height,
                    scroll_offset: &mut state.ui.demo.scroll_offset,
                });
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
                    // Scroll to bottom to show last element - will be clamped by rendering
                    demo.scroll_offset = u16::MAX;
                    true // Treat as wrapped - don't autoscroll
                }
                Some(0) => {
                    // At first element, wrap to last - will be clamped by rendering
                    demo.focus_index = Some(focusable_count - 1);
                    demo.scroll_offset = u16::MAX;
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
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.demo.focus_index,
                    focusable_positions: &state.ui.demo.focusable_positions,
                    viewport_height: state.ui.demo.viewport_height,
                    scroll_offset: &mut state.ui.demo.scroll_offset,
                });
            }

            Some((state, Effect::None))
        }

        DocumentAction::FocusLeft => {
            debug!("Document: focus_left");
            if let Some(new_idx) = find_left_element(&state) {
                state.ui.demo.focus_index = Some(new_idx);
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.demo.focus_index,
                    focusable_positions: &state.ui.demo.focusable_positions,
                    viewport_height: state.ui.demo.viewport_height,
                    scroll_offset: &mut state.ui.demo.scroll_offset,
                });
            }
            Some((state, Effect::None))
        }

        DocumentAction::FocusRight => {
            debug!("Document: focus_right");
            if let Some(new_idx) = find_right_element(&state) {
                state.ui.demo.focus_index = Some(new_idx);
                autoscroll_to_focus(DocumentScrollState {
                    focus_index: state.ui.demo.focus_index,
                    focusable_positions: &state.ui.demo.focusable_positions,
                    viewport_height: state.ui.demo.viewport_height,
                    scroll_offset: &mut state.ui.demo.scroll_offset,
                });
            }
            Some((state, Effect::None))
        }

        DocumentAction::ActivateFocused => {
            debug!("Document: activate_focused");
            if let Some(idx) = state.ui.demo.focus_index {
                if let Some(id) = state.ui.demo.focusable_ids.get(idx) {
                    let display_text = id.display_name();
                    debug!("  Activating: {} (id={:?})", display_text, id);

                    // Set status message to show what was activated
                    state
                        .system
                        .set_status_message(format!("Activated: {}", display_text));
                }
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
            let page_size = state.ui.demo.viewport_height.max(MIN_PAGE_SIZE);
            state.ui.demo.scroll_offset = state.ui.demo.scroll_offset.saturating_sub(page_size);
            Some((state, Effect::None))
        }

        DocumentAction::PageDown => {
            debug!("Document: page_down");
            let page_size = state.ui.demo.viewport_height.max(MIN_PAGE_SIZE);
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

/// Direction for row sibling navigation
enum RowDirection {
    Left,
    Right,
}

/// Find element in adjacent Row child (with wrapping)
fn find_row_sibling(state: &AppState, direction: RowDirection) -> Option<usize> {
    let demo = &state.ui.demo;
    let current_idx = demo.focus_index?;
    let current_pos = demo.focusable_row_positions.get(current_idx).copied()??;

    let max_child_idx = demo
        .focusable_row_positions
        .iter()
        .filter_map(|pos| {
            pos.filter(|p| p.row_y == current_pos.row_y)
                .map(|p| p.child_idx)
        })
        .max()?;

    let target_child_idx = match direction {
        RowDirection::Left if current_pos.child_idx == 0 => max_child_idx,
        RowDirection::Left => current_pos.child_idx - 1,
        RowDirection::Right if current_pos.child_idx >= max_child_idx => 0,
        RowDirection::Right => current_pos.child_idx + 1,
    };

    let target_pos = RowPosition {
        row_y: current_pos.row_y,
        child_idx: target_child_idx,
        idx_within_child: current_pos.idx_within_child,
    };

    demo.focusable_row_positions
        .iter()
        .position(|pos| *pos == Some(target_pos))
}

fn find_left_element(state: &AppState) -> Option<usize> {
    find_row_sibling(state, RowDirection::Left)
}

fn find_right_element(state: &AppState) -> Option<usize> {
    find_row_sibling(state, RowDirection::Right)
}

/// Generic structure holding document scroll state
struct DocumentScrollState<'a> {
    focus_index: Option<usize>,
    focusable_positions: &'a [u16],
    viewport_height: u16,
    scroll_offset: &'a mut u16,
}

/// Autoscroll to keep the focused element visible using actual positions from FocusManager
fn autoscroll_to_focus(scroll_state: DocumentScrollState) {
    // Get the y-position of the focused element from the synced positions
    let element_y = match scroll_state.focus_index {
        Some(idx) if idx < scroll_state.focusable_positions.len() => scroll_state.focusable_positions[idx],
        _ => return, // No focus or positions not synced yet
    };

    // Use actual viewport height, with reasonable fallback
    let viewport_height = scroll_state.viewport_height.max(DEFAULT_VIEWPORT_HEIGHT);

    let viewport_top = *scroll_state.scroll_offset;
    let viewport_bottom = viewport_top.saturating_add(viewport_height);

    // Calculate new scroll offset - only scroll if element is outside viewport
    let new_offset = if element_y < viewport_top {
        // Element is above viewport - scroll up with padding
        Some(element_y.saturating_sub(AUTOSCROLL_PADDING))
    } else if element_y >= viewport_bottom {
        // Element is below viewport - scroll down with padding
        Some(element_y.saturating_sub(viewport_height - AUTOSCROLL_PADDING - 1))
    } else {
        // Element is visible - no scroll needed
        None
    };

    if let Some(offset) = new_offset {
        *scroll_state.scroll_offset = offset;
    }
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
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
    }

    #[test]
    fn test_focus_next_increments() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(0);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
    }

    #[test]
    fn test_focus_next_wraps() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        let count = test_focusable_count(&state);
        state.ui.demo.focus_index = Some(count - 1);
        state.ui.demo.scroll_offset = 50;
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
        assert_eq!(new_state.ui.demo.scroll_offset, 0); // Scrolled to top
    }

    #[test]
    fn test_focus_prev_from_none() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        let count = test_focusable_count(&state);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(count - 1));
    }

    #[test]
    fn test_focus_prev_decrements() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(5);
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();
        assert_eq!(new_state.ui.demo.focus_index, Some(4));
    }

    #[test]
    fn test_focus_prev_wraps() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        let count = test_focusable_count(&state);
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
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
    fn test_autoscroll_element_at_edge_no_scroll() {
        // When element is at the very edge but still visible, no scroll needed
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 10;
        state.ui.demo.viewport_height = 20;
        // Viewport shows lines 10-30
        // Elements at lines 10 (top edge) and 29 (bottom edge) are both visible
        state.ui.demo.focusable_positions = vec![10, 15, 29];

        // Navigate to element at line 29 (bottom edge)
        let (state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();
        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Element at y=29 is visible (viewport is 10-30), no scroll needed
        assert_eq!(new_state.ui.demo.focus_index, Some(2));
        assert_eq!(new_state.ui.demo.scroll_offset, 10);
    }

    #[test]
    fn test_autoscroll_element_below_viewport() {
        // When element is below viewport, scroll down to show it
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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

    fn rp(row_y: u16, child_idx: usize, idx_within_child: usize) -> Option<RowPosition> {
        Some(RowPosition { row_y, child_idx, idx_within_child })
    }

    #[test]
    fn test_focus_left_moves_to_adjacent_child() {
        // Setup: Element at index 3 is in row at y=10, child 1, index_within 0
        // Element at index 0 is in row at y=10, child 0, index_within 0
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(3);
        state.ui.demo.focusable_row_positions = vec![
            rp(10, 0, 0), // index 0: row y=10, child 0, idx_within 0
            rp(10, 0, 1), // index 1: row y=10, child 0, idx_within 1
            rp(10, 0, 2), // index 2: row y=10, child 0, idx_within 2
            rp(10, 1, 0), // index 3: row y=10, child 1, idx_within 0 (CURRENT)
            rp(10, 1, 1), // index 4: row y=10, child 1, idx_within 1
            rp(10, 1, 2), // index 5: row y=10, child 1, idx_within 2
        ];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusLeft).unwrap();

        // Should move to index 0 (same row, child 0, same idx_within 0)
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
    }

    #[test]
    fn test_focus_right_moves_to_adjacent_child() {
        // Setup: Element at index 1 is in row at y=10, child 0, index_within 1
        // Element at index 4 is in row at y=10, child 1, index_within 1
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(1);
        state.ui.demo.focusable_row_positions = vec![
            rp(10, 0, 0), // index 0
            rp(10, 0, 1), // index 1 (CURRENT)
            rp(10, 0, 2), // index 2
            rp(10, 1, 0), // index 3
            rp(10, 1, 1), // index 4
            rp(10, 1, 2), // index 5
        ];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusRight).unwrap();

        // Should move to index 4 (same row, child 1, same idx_within 1)
        assert_eq!(new_state.ui.demo.focus_index, Some(4));
    }

    #[test]
    fn test_focus_left_at_leftmost_child_wraps_to_rightmost() {
        // Element is at child 0, should wrap to child 1
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(1);
        state.ui.demo.focusable_row_positions = vec![
            rp(10, 0, 0),
            rp(10, 0, 1), // CURRENT - at child 0
            rp(10, 1, 0),
            rp(10, 1, 1), // Target - same idx_within (1) but child 1
        ];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusLeft).unwrap();

        // Should wrap to index 3 (child 1, idx_within 1)
        assert_eq!(new_state.ui.demo.focus_index, Some(3));
    }

    #[test]
    fn test_focus_right_at_rightmost_child_wraps_to_leftmost() {
        // Element is at child 1 (rightmost), should wrap to child 0
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(3);
        state.ui.demo.focusable_row_positions = vec![
            rp(10, 0, 0),
            rp(10, 0, 1), // Target - same idx_within (1) but child 0
            rp(10, 1, 0),
            rp(10, 1, 1), // CURRENT - at rightmost child
        ];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusRight).unwrap();

        // Should wrap to index 1 (child 0, idx_within 1)
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
    }

    #[test]
    fn test_focus_left_not_in_row_does_nothing() {
        // Element is not in a row (row_position is None)
        let mut state = AppState::default();
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.focusable_row_positions = vec![
            None, // Not in a row
            None,
            rp(10, 0, 0),
        ];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusLeft).unwrap();

        // Should stay at index 0 (no change)
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
    }
}
