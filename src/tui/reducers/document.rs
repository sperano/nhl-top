//! Document reducer for handling document navigation actions
//!
//! Handles Tab/Shift-Tab focus navigation and scrolling within documents.

use tracing::debug;

use crate::tui::action::{Action, DocumentAction};
use crate::tui::component::Effect;
use crate::tui::document::RowPosition;
use crate::tui::state::{AppState, DocumentState};
use crate::tui::types::Tab;

/// Base number of focusable elements in the demo document
/// - 4 example links (BOS, TOR, NYR, MTL)
/// - 10 player table cells (5 forwards + 5 defensemen, 1 link column each)
const BASE_FOCUSABLE_COUNT: usize = 14;

/// Minimum viewport height - if smaller than this, autoscroll may behave oddly
/// but we still use the actual value rather than clamping (which causes worse bugs)
const MIN_VIEWPORT_HEIGHT: u16 = 5;

/// Padding lines above/below focused element for autoscroll
/// This ensures scrolling starts before the element reaches the edge
const AUTOSCROLL_PADDING: u16 = 3;

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
            state.ui.standings_doc.focusable_positions.len()
        }
        _ => 0,
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

// ============================================================================
// Generic document navigation helpers
// ============================================================================

/// Move focus to next element, wrapping from last to first
fn doc_focus_next(doc: &mut DocumentState, focusable_count: usize) -> bool {
    if focusable_count == 0 {
        return false;
    }

    match doc.focus_index {
        None => {
            doc.focus_index = Some(0);
            false // didn't wrap
        }
        Some(idx) if idx + 1 >= focusable_count => {
            doc.focus_index = Some(0);
            doc.scroll_offset = 0;
            true // wrapped
        }
        Some(idx) => {
            doc.focus_index = Some(idx + 1);
            false
        }
    }
}

/// Move focus to previous element, wrapping from first to last
fn doc_focus_prev(doc: &mut DocumentState, focusable_count: usize) -> bool {
    if focusable_count == 0 {
        return false;
    }

    match doc.focus_index {
        None => {
            doc.focus_index = Some(focusable_count - 1);
            doc.scroll_offset = u16::MAX;
            true // wrapped
        }
        Some(0) => {
            doc.focus_index = Some(focusable_count - 1);
            doc.scroll_offset = u16::MAX;
            true // wrapped
        }
        Some(idx) => {
            doc.focus_index = Some(idx - 1);
            false
        }
    }
}

/// Scroll document up by N lines
fn doc_scroll_up(doc: &mut DocumentState, lines: u16) {
    doc.scroll_offset = doc.scroll_offset.saturating_sub(lines);
}

/// Scroll document down by N lines
fn doc_scroll_down(doc: &mut DocumentState, lines: u16) {
    doc.scroll_offset = doc.scroll_offset.saturating_add(lines);
}

/// Page up (scroll by viewport height)
fn doc_page_up(doc: &mut DocumentState) {
    let page_size = doc.viewport_height.max(MIN_PAGE_SIZE);
    doc.scroll_offset = doc.scroll_offset.saturating_sub(page_size);
}

/// Page down (scroll by viewport height)
fn doc_page_down(doc: &mut DocumentState) {
    let page_size = doc.viewport_height.max(MIN_PAGE_SIZE);
    doc.scroll_offset = doc.scroll_offset.saturating_add(page_size);
}

/// Activate the currently focused element, returning the display text if any
fn doc_activate_focused(doc: &DocumentState) -> Option<String> {
    let idx = doc.focus_index?;
    let id = doc.focusable_ids.get(idx)?;
    Some(id.display_name())
}

/// Create autoscroll state from a document state
fn autoscroll_state_from_doc(doc: &mut DocumentState) -> DocumentScrollState<'_> {
    DocumentScrollState {
        focus_index: doc.focus_index,
        focusable_positions: &doc.focusable_positions,
        viewport_height: doc.viewport_height,
        scroll_offset: &mut doc.scroll_offset,
    }
}

// ============================================================================
// Main document action handler
// ============================================================================

fn handle_document_action(mut state: AppState, action: &DocumentAction) -> Option<(AppState, Effect)> {
    // UpdateViewportHeight applies globally to all document-based tabs
    if let DocumentAction::UpdateViewportHeight { demo, standings } = action {
        let demo_changed = state.ui.demo.viewport_height != *demo;
        let standings_changed = state.ui.standings_doc.viewport_height != *standings;
        if demo_changed || standings_changed {
            debug!(
                "Document: update_viewport_height demo={}->{}, standings={}->{}",
                state.ui.demo.viewport_height, demo,
                state.ui.standings_doc.viewport_height, standings
            );
            state.ui.demo.viewport_height = *demo;
            state.ui.standings_doc.viewport_height = *standings;
        }
        return Some((state, Effect::None));
    }

    // Determine which document state to use based on current tab
    let is_standings_doc = state.navigation.current_tab == Tab::Standings
        && (state.ui.standings.view == crate::commands::standings::GroupBy::League
            || state.ui.standings.view == crate::commands::standings::GroupBy::Conference);

    let is_demo = state.navigation.current_tab == Tab::Demo;

    // Get focusable count for the appropriate document
    let focusable_count = if is_standings_doc {
        state.ui.standings_doc.focusable_count()
    } else if is_demo {
        get_focusable_count(&state)
    } else {
        return None; // Not a document-based tab
    };

    match action {
        DocumentAction::FocusNext => {
            debug!("Document: focus_next (count={})", focusable_count);

            let wrapped = if is_standings_doc {
                doc_focus_next(&mut state.ui.standings_doc, focusable_count)
            } else {
                doc_focus_next(&mut state.ui.demo, focusable_count)
            };

            if !wrapped && focusable_count > 0 {
                if is_standings_doc {
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.standings_doc));
                } else {
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.demo));
                }
            }

            Some((state, Effect::None))
        }

        DocumentAction::FocusPrev => {
            debug!("Document: focus_prev (count={})", focusable_count);

            let wrapped = if is_standings_doc {
                doc_focus_prev(&mut state.ui.standings_doc, focusable_count)
            } else {
                doc_focus_prev(&mut state.ui.demo, focusable_count)
            };

            if !wrapped && focusable_count > 0 {
                if is_standings_doc {
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.standings_doc));
                } else {
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.demo));
                }
            }

            Some((state, Effect::None))
        }

        DocumentAction::FocusLeft => {
            debug!("Document: focus_left");
            let new_idx = if is_standings_doc {
                find_standings_left_element(&state)
            } else {
                find_left_element(&state)
            };

            if let Some(idx) = new_idx {
                if is_standings_doc {
                    state.ui.standings_doc.focus_index = Some(idx);
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.standings_doc));
                } else {
                    state.ui.demo.focus_index = Some(idx);
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.demo));
                }
            }
            Some((state, Effect::None))
        }

        DocumentAction::FocusRight => {
            debug!("Document: focus_right");
            let new_idx = if is_standings_doc {
                find_standings_right_element(&state)
            } else {
                find_right_element(&state)
            };

            if let Some(idx) = new_idx {
                if is_standings_doc {
                    state.ui.standings_doc.focus_index = Some(idx);
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.standings_doc));
                } else {
                    state.ui.demo.focus_index = Some(idx);
                    autoscroll_to_focus(autoscroll_state_from_doc(&mut state.ui.demo));
                }
            }
            Some((state, Effect::None))
        }

        DocumentAction::ActivateFocused => {
            debug!("Document: activate_focused");
            let display_text = if is_standings_doc {
                doc_activate_focused(&state.ui.standings_doc)
            } else {
                doc_activate_focused(&state.ui.demo)
            };

            if let Some(text) = display_text {
                debug!("  Activating: {}", text);
                state.system.set_status_message(format!("Activated: {}", text));
            }
            Some((state, Effect::None))
        }

        DocumentAction::ScrollUp(lines) => {
            debug!("Document: scroll_up {}", lines);
            if is_standings_doc {
                doc_scroll_up(&mut state.ui.standings_doc, *lines);
            } else {
                doc_scroll_up(&mut state.ui.demo, *lines);
            }
            Some((state, Effect::None))
        }

        DocumentAction::ScrollDown(lines) => {
            debug!("Document: scroll_down {}", lines);
            if is_standings_doc {
                doc_scroll_down(&mut state.ui.standings_doc, *lines);
            } else {
                doc_scroll_down(&mut state.ui.demo, *lines);
            }
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToTop => {
            debug!("Document: scroll_to_top");
            if is_standings_doc {
                state.ui.standings_doc.scroll_offset = 0;
            } else {
                state.ui.demo.scroll_offset = 0;
            }
            Some((state, Effect::None))
        }

        DocumentAction::ScrollToBottom => {
            debug!("Document: scroll_to_bottom");
            if is_standings_doc {
                state.ui.standings_doc.scroll_offset = u16::MAX;
            } else {
                state.ui.demo.scroll_offset = u16::MAX;
            }
            Some((state, Effect::None))
        }

        DocumentAction::PageUp => {
            debug!("Document: page_up");
            if is_standings_doc {
                doc_page_up(&mut state.ui.standings_doc);
            } else {
                doc_page_up(&mut state.ui.demo);
            }
            Some((state, Effect::None))
        }

        DocumentAction::PageDown => {
            debug!("Document: page_down");
            if is_standings_doc {
                doc_page_down(&mut state.ui.standings_doc);
            } else {
                doc_page_down(&mut state.ui.demo);
            }
            Some((state, Effect::None))
        }

        DocumentAction::SyncFocusablePositions(positions, viewport_height) => {
            debug!(
                "Document: sync_focusable_positions (count={}, viewport={})",
                positions.len(),
                viewport_height
            );
            // Currently only used by demo tab
            state.ui.demo.focusable_positions = positions.clone();
            state.ui.demo.viewport_height = *viewport_height;
            Some((state, Effect::None))
        }

        // UpdateViewportHeight is handled at the top of this function before tab dispatch
        DocumentAction::UpdateViewportHeight { .. } => unreachable!("handled earlier in function"),
    }
}

/// Direction for row sibling navigation
enum RowDirection {
    Left,
    Right,
}

/// Find element in adjacent Row child (with wrapping)
///
/// This function navigates left/right within a Row element, finding the
/// corresponding element in an adjacent child column while preserving
/// the row position (idx_within_child).
fn find_row_sibling_in_doc(doc: &DocumentState, direction: RowDirection) -> Option<usize> {
    let current_idx = doc.focus_index?;
    let current_pos = doc.focusable_row_positions.get(current_idx).copied()??;

    let max_child_idx = doc
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

    doc.focusable_row_positions
        .iter()
        .position(|pos| *pos == Some(target_pos))
}

fn find_left_element(state: &AppState) -> Option<usize> {
    find_row_sibling_in_doc(&state.ui.demo, RowDirection::Left)
}

fn find_right_element(state: &AppState) -> Option<usize> {
    find_row_sibling_in_doc(&state.ui.demo, RowDirection::Right)
}

fn find_standings_left_element(state: &AppState) -> Option<usize> {
    find_row_sibling_in_doc(&state.ui.standings_doc, RowDirection::Left)
}

fn find_standings_right_element(state: &AppState) -> Option<usize> {
    find_row_sibling_in_doc(&state.ui.standings_doc, RowDirection::Right)
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

    let focus_idx = scroll_state.focus_index.unwrap();

    // Use actual viewport height (must be at least MIN_VIEWPORT_HEIGHT for sane behavior)
    let viewport_height = scroll_state.viewport_height.max(MIN_VIEWPORT_HEIGHT);

    // Calculate viewport bounds (without padding - we only scroll when truly outside)
    let viewport_top = *scroll_state.scroll_offset;
    let viewport_bottom = viewport_top.saturating_add(viewport_height);

    debug!(
        "Autoscroll: idx={}, element_y={}, scroll={}, viewport={}, top={}, bottom={}",
        focus_idx, element_y, scroll_state.scroll_offset, viewport_height, viewport_top, viewport_bottom
    );

    // Calculate new scroll offset - only scroll if element is outside viewport
    let new_offset = if element_y < viewport_top {
        // Element is above viewport - scroll up with padding
        let offset = element_y.saturating_sub(AUTOSCROLL_PADDING);
        debug!("  -> Scrolling UP to {}", offset);
        Some(offset)
    } else if element_y >= viewport_bottom {
        // Element is below viewport - scroll down with padding
        let offset = element_y.saturating_sub(viewport_height - AUTOSCROLL_PADDING - 1);
        debug!("  -> Scrolling DOWN to {}", offset);
        Some(offset)
    } else {
        // Element is visible - no scroll needed
        debug!("  -> No scroll needed (element visible)");
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
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.scroll_offset = 10;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollUp(3)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 7);
    }

    #[test]
    fn test_scroll_up_clamps() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.scroll_offset = 2;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollUp(10)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.scroll_offset = 10;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollDown(5)).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 15);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.scroll_offset = 50;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollToTop).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        let (new_state, _) = handle_document_action(state, &DocumentAction::ScrollToBottom).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_page_up() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.scroll_offset = 30;
        state.ui.demo.viewport_height = 20;
        let (new_state, _) = handle_document_action(state, &DocumentAction::PageUp).unwrap();
        assert_eq!(new_state.ui.demo.scroll_offset, 10);
    }

    #[test]
    fn test_page_down() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
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
        // Scroll offset should be adjusted: element_y + AUTOSCROLL_PADDING + 1 - viewport_height
        // = 25 + 3 + 1 - 20 = 9
        assert_eq!(new_state.ui.demo.scroll_offset, 9);
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
        // Scroll offset should be adjusted: element_y - AUTOSCROLL_PADDING = 5 - 3 = 2
        assert_eq!(new_state.ui.demo.scroll_offset, 2);
    }

    #[test]
    fn test_sync_focusable_positions() {
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
        state.navigation.current_tab = Tab::Demo;
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
    fn test_update_viewport_height() {
        let mut state = AppState::default();
        state.ui.demo.viewport_height = 0;
        state.ui.standings_doc.viewport_height = 0;

        let (new_state, _) =
            handle_document_action(state, &DocumentAction::UpdateViewportHeight { demo: 31, standings: 29 }).unwrap();

        // Each tab gets its own viewport height
        assert_eq!(new_state.ui.demo.viewport_height, 31);
        assert_eq!(new_state.ui.standings_doc.viewport_height, 29);
    }

    #[test]
    fn test_update_viewport_height_no_change_when_same() {
        let mut state = AppState::default();
        state.ui.demo.viewport_height = 31;
        state.ui.standings_doc.viewport_height = 29;

        // Same values should still return state (no-op but handled)
        let (new_state, _) =
            handle_document_action(state, &DocumentAction::UpdateViewportHeight { demo: 31, standings: 29 }).unwrap();

        assert_eq!(new_state.ui.demo.viewport_height, 31);
        assert_eq!(new_state.ui.standings_doc.viewport_height, 29);
    }

    #[test]
    fn test_standings_viewport_is_smaller_due_to_subtabs() {
        // This test documents why standings has a smaller viewport than demo.
        // Standings has nested TabbedPanel (Wildcard/Division/Conference/League subtabs)
        // which adds 2 extra lines of chrome.
        //
        // Terminal height: 35
        // Base chrome (main tabs + status bar): 4 lines
        // Standings subtab chrome: 2 lines
        //
        // Demo viewport: 35 - 4 = 31
        // Standings viewport: 35 - 6 = 29
        let mut state = AppState::default();
        state.ui.demo.viewport_height = 0;
        state.ui.standings_doc.viewport_height = 0;

        // Simulating terminal height of 35
        let terminal_height: u16 = 35;
        let base_chrome: u16 = 4;
        let standings_subtab_chrome: u16 = 2;

        let demo_viewport = terminal_height - base_chrome;
        let standings_viewport = terminal_height - base_chrome - standings_subtab_chrome;

        let (new_state, _) = handle_document_action(
            state,
            &DocumentAction::UpdateViewportHeight { demo: demo_viewport, standings: standings_viewport }
        ).unwrap();

        assert_eq!(new_state.ui.demo.viewport_height, 31);
        assert_eq!(new_state.ui.standings_doc.viewport_height, 29);

        // The 2-line difference is critical for correct down-scroll behavior
        // Without this, autoscroll triggers late because viewport_bottom is too far down
    }

    #[test]
    fn test_autoscroll_down_with_correct_viewport_height() {
        // This test verifies the fix for the autoscroll asymmetry bug.
        // With viewport_height=31, an element at y=20 should be visible
        // and NOT trigger scrolling when navigating to it.
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 0;
        state.ui.demo.viewport_height = 31; // Realistic viewport (not default 20)
        // Element at y=5 (first), element at y=20 (second)
        // With viewport [0, 31), element at y=20 is visible - no scroll needed
        state.ui.demo.focusable_positions = vec![5, 20];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Focus moves to index 1 (y=20)
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
        // Element at y=20 is within viewport [0, 31), so NO scroll
        assert_eq!(new_state.ui.demo.scroll_offset, 0);
    }

    #[test]
    fn test_autoscroll_down_with_wrong_viewport_height_would_scroll() {
        // This demonstrates the bug: with viewport_height=20 (the wrong value),
        // an element at y=20 triggers scrolling because 20 >= 20.
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(0);
        state.ui.demo.scroll_offset = 0;
        state.ui.demo.viewport_height = 20; // Wrong/default value
        state.ui.demo.focusable_positions = vec![5, 20];

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusNext).unwrap();

        // Focus moves to index 1 (y=20)
        assert_eq!(new_state.ui.demo.focus_index, Some(1));
        // With wrong viewport_height=20, element at y=20 is AT the boundary
        // viewport_bottom = 0 + 20 = 20, and 20 >= 20 triggers scroll
        // new_offset = element_y - (viewport_height - padding - 1) = 20 - (20 - 3 - 1) = 20 - 16 = 4
        assert_eq!(new_state.ui.demo.scroll_offset, 4);
    }

    #[test]
    fn test_autoscroll_up_works_regardless_of_viewport_height() {
        // UP navigation doesn't use viewport_height in its condition,
        // so it works correctly even with the wrong viewport_height value.
        // This explains the asymmetry observed in the bug.
        let mut state = AppState::default();
        state.navigation.current_tab = Tab::Demo;
        state.ui.demo.focus_index = Some(1);
        state.ui.demo.scroll_offset = 10; // Viewing lines 10-30 (with height 20) or 10-41 (with height 31)
        state.ui.demo.viewport_height = 20; // Wrong value, but doesn't matter for UP
        state.ui.demo.focusable_positions = vec![5, 15]; // y=5 is above viewport, y=15 is visible

        let (new_state, _) = handle_document_action(state, &DocumentAction::FocusPrev).unwrap();

        // Focus moves to index 0 (y=5), which is above viewport
        assert_eq!(new_state.ui.demo.focus_index, Some(0));
        // Condition: element_y < viewport_top => 5 < 10 => TRUE => scroll
        // new_offset = element_y - padding = 5 - 3 = 2
        assert_eq!(new_state.ui.demo.scroll_offset, 2);
    }

    // =========================================================================
    // Direct unit tests for helper functions
    // =========================================================================

    #[test]
    fn test_doc_focus_next_empty_count() {
        let mut doc = DocumentState::default();
        let wrapped = doc_focus_next(&mut doc, 0);
        assert!(!wrapped);
        assert_eq!(doc.focus_index, None);
    }

    #[test]
    fn test_doc_focus_next_from_none_sets_zero() {
        let mut doc = DocumentState::default();
        let wrapped = doc_focus_next(&mut doc, 5);
        assert!(!wrapped);
        assert_eq!(doc.focus_index, Some(0));
    }

    #[test]
    fn test_doc_focus_next_increments() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(2);
        let wrapped = doc_focus_next(&mut doc, 5);
        assert!(!wrapped);
        assert_eq!(doc.focus_index, Some(3));
    }

    #[test]
    fn test_doc_focus_next_wraps_at_end() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(4);
        doc.scroll_offset = 100;
        let wrapped = doc_focus_next(&mut doc, 5);
        assert!(wrapped);
        assert_eq!(doc.focus_index, Some(0));
        assert_eq!(doc.scroll_offset, 0);
    }

    #[test]
    fn test_doc_focus_prev_empty_count() {
        let mut doc = DocumentState::default();
        let wrapped = doc_focus_prev(&mut doc, 0);
        assert!(!wrapped);
        assert_eq!(doc.focus_index, None);
    }

    #[test]
    fn test_doc_focus_prev_from_none_sets_last() {
        let mut doc = DocumentState::default();
        let wrapped = doc_focus_prev(&mut doc, 5);
        assert!(wrapped);
        assert_eq!(doc.focus_index, Some(4));
        assert_eq!(doc.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_doc_focus_prev_decrements() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(3);
        let wrapped = doc_focus_prev(&mut doc, 5);
        assert!(!wrapped);
        assert_eq!(doc.focus_index, Some(2));
    }

    #[test]
    fn test_doc_focus_prev_wraps_at_start() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(0);
        let wrapped = doc_focus_prev(&mut doc, 5);
        assert!(wrapped);
        assert_eq!(doc.focus_index, Some(4));
        assert_eq!(doc.scroll_offset, u16::MAX);
    }

    #[test]
    fn test_doc_scroll_up() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc_scroll_up(&mut doc, 10);
        assert_eq!(doc.scroll_offset, 40);
    }

    #[test]
    fn test_doc_scroll_up_saturates() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 5;
        doc_scroll_up(&mut doc, 10);
        assert_eq!(doc.scroll_offset, 0);
    }

    #[test]
    fn test_doc_scroll_down() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc_scroll_down(&mut doc, 10);
        assert_eq!(doc.scroll_offset, 60);
    }

    #[test]
    fn test_doc_page_up() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc.viewport_height = 20;
        doc_page_up(&mut doc);
        assert_eq!(doc.scroll_offset, 30);
    }

    #[test]
    fn test_doc_page_up_uses_min_page_size() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc.viewport_height = 5; // Less than MIN_PAGE_SIZE
        doc_page_up(&mut doc);
        // Should use MIN_PAGE_SIZE (10) instead of viewport_height
        assert_eq!(doc.scroll_offset, 40);
    }

    #[test]
    fn test_doc_page_down() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc.viewport_height = 20;
        doc_page_down(&mut doc);
        assert_eq!(doc.scroll_offset, 70);
    }

    #[test]
    fn test_doc_page_down_uses_min_page_size() {
        let mut doc = DocumentState::default();
        doc.scroll_offset = 50;
        doc.viewport_height = 5; // Less than MIN_PAGE_SIZE
        doc_page_down(&mut doc);
        // Should use MIN_PAGE_SIZE (10) instead of viewport_height
        assert_eq!(doc.scroll_offset, 60);
    }

    #[test]
    fn test_doc_activate_focused_none() {
        let doc = DocumentState::default();
        assert_eq!(doc_activate_focused(&doc), None);
    }

    #[test]
    fn test_doc_activate_focused_out_of_bounds() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(5);
        doc.focusable_ids = vec![]; // Empty
        assert_eq!(doc_activate_focused(&doc), None);
    }

    #[test]
    fn test_doc_activate_focused_returns_display_name() {
        use crate::tui::document::FocusableId;
        let mut doc = DocumentState::default();
        doc.focus_index = Some(0);
        doc.focusable_ids = vec![FocusableId::link("test_link")];
        let result = doc_activate_focused(&doc);
        assert!(result.is_some());
        assert!(result.unwrap().contains("test_link"));
    }

    #[test]
    fn test_find_row_sibling_in_doc_no_focus() {
        let doc = DocumentState::default();
        assert_eq!(find_row_sibling_in_doc(&doc, RowDirection::Left), None);
        assert_eq!(find_row_sibling_in_doc(&doc, RowDirection::Right), None);
    }

    #[test]
    fn test_find_row_sibling_in_doc_not_in_row() {
        let mut doc = DocumentState::default();
        doc.focus_index = Some(0);
        doc.focusable_row_positions = vec![None]; // Not in a row
        assert_eq!(find_row_sibling_in_doc(&doc, RowDirection::Left), None);
    }

    #[test]
    fn test_document_state_focusable_count() {
        let mut doc = DocumentState::default();
        assert_eq!(doc.focusable_count(), 0);
        doc.focusable_positions = vec![1, 2, 3];
        assert_eq!(doc.focusable_count(), 3);
    }
}
