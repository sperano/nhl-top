//! Generic document navigation logic
//!
//! This module provides reusable navigation logic for components that display
//! scrollable, focusable document-like content (e.g., StandingsTab, DemoTab).

use crate::tui::component::Effect;
use crate::tui::document::{FocusableId, RowPosition};

/// Minimum viewport height - if smaller than this, autoscroll may behave oddly
const MIN_VIEWPORT_HEIGHT: u16 = 5;

/// Padding lines above/below focused element for autoscroll
const AUTOSCROLL_PADDING: u16 = 3;

/// Minimum page size for page up/down operations
const MIN_PAGE_SIZE: u16 = 10;

/// Direction for finding sibling in row navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowDirection {
    Left,
    Right,
}

/// Document navigation state
///
/// Components can embed this struct to get document navigation behavior.
/// All navigation functions in this module operate on this state.
#[derive(Debug, Clone, Default)]
pub struct DocumentNavState {
    pub focus_index: Option<usize>,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    pub focusable_positions: Vec<u16>,
    pub focusable_ids: Vec<FocusableId>,
    pub focusable_row_positions: Vec<Option<RowPosition>>,
}

/// Document navigation messages
///
/// Components that use DocumentNavState can include these messages in their
/// own message enum to handle document navigation.
#[derive(Clone, Debug)]
pub enum DocumentNavMsg {
    FocusNext,
    FocusPrev,
    FocusLeft,
    FocusRight,
    ScrollUp(u16),
    ScrollDown(u16),
    ScrollToTop,
    ScrollToBottom,
    PageUp,
    PageDown,
    UpdateViewportHeight(u16),
}

/// Handle a document navigation message
///
/// Call this from your component's update() method to handle document navigation.
/// Returns Effect::None (navigation doesn't trigger side effects).
pub fn handle_message(state: &mut DocumentNavState, msg: &DocumentNavMsg) -> Effect {
    match msg {
        DocumentNavMsg::FocusNext => {
            let _wrapped = focus_next(state);
            autoscroll_to_focus(state);
        }
        DocumentNavMsg::FocusPrev => {
            let _wrapped = focus_prev(state);
            autoscroll_to_focus(state);
        }
        DocumentNavMsg::FocusLeft => {
            if let Some(new_idx) = find_row_sibling(state, RowDirection::Left) {
                state.focus_index = Some(new_idx);
                autoscroll_to_focus(state);
            }
        }
        DocumentNavMsg::FocusRight => {
            if let Some(new_idx) = find_row_sibling(state, RowDirection::Right) {
                state.focus_index = Some(new_idx);
                autoscroll_to_focus(state);
            }
        }
        DocumentNavMsg::ScrollUp(lines) => {
            scroll_up(state, *lines);
        }
        DocumentNavMsg::ScrollDown(lines) => {
            scroll_down(state, *lines);
        }
        DocumentNavMsg::ScrollToTop => {
            scroll_to_top(state);
        }
        DocumentNavMsg::ScrollToBottom => {
            scroll_to_bottom(state);
        }
        DocumentNavMsg::PageUp => {
            page_up(state);
        }
        DocumentNavMsg::PageDown => {
            page_down(state);
        }
        DocumentNavMsg::UpdateViewportHeight(height) => {
            state.viewport_height = *height;
        }
    }
    Effect::None
}

// ============================================================================
// Focus Navigation
// ============================================================================

/// Move focus to next element, wrapping from last to first
/// Returns true if wrapped around
pub fn focus_next(state: &mut DocumentNavState) -> bool {
    let focusable_count = state.focusable_positions.len();
    if focusable_count == 0 {
        return false;
    }

    match state.focus_index {
        None => {
            state.focus_index = Some(0);
            false // didn't wrap
        }
        Some(idx) if idx + 1 >= focusable_count => {
            state.focus_index = Some(0);
            state.scroll_offset = 0;
            true // wrapped
        }
        Some(idx) => {
            state.focus_index = Some(idx + 1);
            false
        }
    }
}

/// Move focus to previous element, wrapping from first to last
/// Returns true if wrapped around
pub fn focus_prev(state: &mut DocumentNavState) -> bool {
    let focusable_count = state.focusable_positions.len();
    if focusable_count == 0 {
        return false;
    }

    match state.focus_index {
        None => {
            state.focus_index = Some(focusable_count - 1);
            state.scroll_offset = u16::MAX;
            true // wrapped
        }
        Some(0) => {
            state.focus_index = Some(focusable_count - 1);
            state.scroll_offset = u16::MAX;
            true // wrapped
        }
        Some(idx) => {
            state.focus_index = Some(idx - 1);
            false
        }
    }
}

/// Find sibling element in the same row (left or right)
pub fn find_row_sibling(state: &DocumentNavState, direction: RowDirection) -> Option<usize> {
    let focus_idx = state.focus_index?;
    let row_positions = &state.focusable_row_positions;
    let current_row = row_positions.get(focus_idx)?.as_ref()?;

    // Find element in same row at same idx_within_child but different child_idx
    let target_child_idx = match direction {
        RowDirection::Left => {
            if current_row.child_idx == 0 {
                // Wrap to rightmost child
                row_positions
                    .iter()
                    .filter_map(|r| r.as_ref())
                    .filter(|r| r.row_y == current_row.row_y)
                    .map(|r| r.child_idx)
                    .max()?
            } else {
                current_row.child_idx - 1
            }
        }
        RowDirection::Right => {
            let max_child_idx = row_positions
                .iter()
                .filter_map(|r| r.as_ref())
                .filter(|r| r.row_y == current_row.row_y)
                .map(|r| r.child_idx)
                .max()?;

            if current_row.child_idx >= max_child_idx {
                // Wrap to leftmost child
                0
            } else {
                current_row.child_idx + 1
            }
        }
    };

    // Find element with matching row_y, target child_idx, and same idx_within_child
    row_positions
        .iter()
        .enumerate()
        .find(|(_, r)| {
            if let Some(row) = r.as_ref() {
                row.row_y == current_row.row_y
                    && row.child_idx == target_child_idx
                    && row.idx_within_child == current_row.idx_within_child
            } else {
                false
            }
        })
        .map(|(idx, _)| idx)
}

// ============================================================================
// Scrolling
// ============================================================================

/// Scroll up by N lines
pub fn scroll_up(state: &mut DocumentNavState, lines: u16) {
    state.scroll_offset = state.scroll_offset.saturating_sub(lines);
}

/// Scroll down by N lines
pub fn scroll_down(state: &mut DocumentNavState, lines: u16) {
    state.scroll_offset = state.scroll_offset.saturating_add(lines);
}

/// Scroll to top
pub fn scroll_to_top(state: &mut DocumentNavState) {
    state.scroll_offset = 0;
}

/// Scroll to bottom
pub fn scroll_to_bottom(state: &mut DocumentNavState) {
    state.scroll_offset = u16::MAX;
}

/// Page up (scroll by viewport height)
pub fn page_up(state: &mut DocumentNavState) {
    let page_size = state.viewport_height.max(MIN_PAGE_SIZE);
    state.scroll_offset = state.scroll_offset.saturating_sub(page_size);
}

/// Page down (scroll by viewport height)
pub fn page_down(state: &mut DocumentNavState) {
    let page_size = state.viewport_height.max(MIN_PAGE_SIZE);
    state.scroll_offset = state.scroll_offset.saturating_add(page_size);
}

/// Autoscroll to keep focused element visible
pub fn autoscroll_to_focus(state: &mut DocumentNavState) {
    let focus_idx = match state.focus_index {
        Some(idx) => idx,
        None => return,
    };

    let focused_y = match state.focusable_positions.get(focus_idx) {
        Some(&y) => y,
        None => return,
    };

    let viewport_height = state.viewport_height.max(MIN_VIEWPORT_HEIGHT);
    let scroll_offset = state.scroll_offset;

    // Calculate visible range with padding
    let visible_start = scroll_offset.saturating_add(AUTOSCROLL_PADDING);
    let visible_end = scroll_offset
        .saturating_add(viewport_height)
        .saturating_sub(AUTOSCROLL_PADDING);

    // Check if focused element is outside visible range
    if focused_y < visible_start {
        // Scroll up to show focused element with padding
        let new_offset = focused_y.saturating_sub(AUTOSCROLL_PADDING);
        state.scroll_offset = new_offset;
    } else if focused_y >= visible_end {
        // Scroll down to show focused element with padding
        let new_offset = focused_y
            .saturating_sub(viewport_height)
            .saturating_add(AUTOSCROLL_PADDING)
            .saturating_add(1);
        state.scroll_offset = new_offset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_next_wraps_around() {
        let mut state = DocumentNavState {
            focus_index: Some(2),
            scroll_offset: 0,
            viewport_height: 20,
            focusable_positions: vec![0, 5, 10],
            focusable_ids: vec![],
            focusable_row_positions: vec![None, None, None],
        };

        let wrapped = focus_next(&mut state);
        assert!(wrapped);
        assert_eq!(state.focus_index, Some(0));
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_focus_next_advances() {
        let mut state = DocumentNavState {
            focus_index: Some(0),
            scroll_offset: 0,
            viewport_height: 20,
            focusable_positions: vec![0, 5, 10],
            focusable_ids: vec![],
            focusable_row_positions: vec![None, None, None],
        };

        let wrapped = focus_next(&mut state);
        assert!(!wrapped);
        assert_eq!(state.focus_index, Some(1));
    }

    #[test]
    fn test_focus_prev_wraps_around() {
        let mut state = DocumentNavState {
            focus_index: Some(0),
            scroll_offset: 5,
            viewport_height: 20,
            focusable_positions: vec![0, 5, 10],
            focusable_ids: vec![],
            focusable_row_positions: vec![None, None, None],
        };

        let wrapped = focus_prev(&mut state);
        assert!(wrapped);
        assert_eq!(state.focus_index, Some(2));
        assert_eq!(state.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_focus_prev_from_none() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 0,
            viewport_height: 20,
            focusable_positions: vec![0, 5, 10],
            focusable_ids: vec![],
            focusable_row_positions: vec![None, None, None],
        };

        let wrapped = focus_prev(&mut state);
        assert!(wrapped);
        assert_eq!(state.focus_index, Some(2));
    }

    #[test]
    fn test_scroll_up() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 10,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        scroll_up(&mut state, 5);
        assert_eq!(state.scroll_offset, 5);

        scroll_up(&mut state, 10); // Should saturate at 0
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 10,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        scroll_down(&mut state, 5);
        assert_eq!(state.scroll_offset, 15);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 100,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        scroll_to_top(&mut state);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 0,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        scroll_to_bottom(&mut state);
        assert_eq!(state.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_page_up() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 50,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        page_up(&mut state);
        assert_eq!(state.scroll_offset, 30);
    }

    #[test]
    fn test_page_down() {
        let mut state = DocumentNavState {
            focus_index: None,
            scroll_offset: 30,
            viewport_height: 20,
            focusable_positions: vec![],
            focusable_ids: vec![],
            focusable_row_positions: vec![],
        };

        page_down(&mut state);
        assert_eq!(state.scroll_offset, 50);
    }

    #[test]
    fn test_autoscroll_to_focus_scrolls_down() {
        let mut state = DocumentNavState {
            focus_index: Some(5),
            scroll_offset: 0,
            viewport_height: 10,
            focusable_positions: vec![0, 2, 4, 6, 8, 20, 22], // Element 5 is at y=20
            focusable_ids: vec![],
            focusable_row_positions: vec![None; 7],
        };

        autoscroll_to_focus(&mut state);
        // Should scroll so element at y=20 is visible
        // visible_end needs to be > 20 + PADDING
        assert!(state.scroll_offset > 0);
    }

    #[test]
    fn test_autoscroll_to_focus_scrolls_up() {
        let mut state = DocumentNavState {
            focus_index: Some(0),
            scroll_offset: 10,
            viewport_height: 10,
            focusable_positions: vec![0, 2, 4, 6, 8, 20, 22], // Element 0 is at y=0
            focusable_ids: vec![],
            focusable_row_positions: vec![None; 7],
        };

        autoscroll_to_focus(&mut state);
        // Should scroll so element at y=0 is visible
        assert_eq!(state.scroll_offset, 0);
    }
}
