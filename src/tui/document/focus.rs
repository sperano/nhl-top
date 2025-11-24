//! Focus management for document navigation
//!
//! Provides Tab/Shift-Tab navigation through focusable elements within documents.
//! Tracks focus state and provides methods for navigating and activating elements.

use ratatui::layout::Rect;

use super::link::LinkTarget;
use crate::tui::focus_helpers;

/// Type-safe identifier for focusable elements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FocusableId {
    /// A table cell identified by table name, row, and column
    TableCell {
        table_name: String,
        row: usize,
        col: usize,
    },
    /// A standalone link with a string identifier
    Link(String),
}

impl FocusableId {
    /// Create a table cell ID
    pub fn table_cell(table_name: impl Into<String>, row: usize, col: usize) -> Self {
        Self::TableCell {
            table_name: table_name.into(),
            row,
            col,
        }
    }

    /// Create a link ID
    pub fn link(id: impl Into<String>) -> Self {
        Self::Link(id.into())
    }

    /// Get the table row if this is a table cell
    pub fn table_row(&self) -> Option<usize> {
        match self {
            Self::TableCell { row, .. } => Some(*row),
            Self::Link(_) => None,
        }
    }

    /// Get the table name if this is a table cell
    pub fn table_name(&self) -> Option<&str> {
        match self {
            Self::TableCell { table_name, .. } => Some(table_name),
            Self::Link(_) => None,
        }
    }

    /// Format for user-friendly display
    pub fn display_name(&self) -> String {
        match self {
            Self::TableCell { row, .. } => format!("Table row {}", row + 1),
            Self::Link(id) => format_link_id(id),
        }
    }
}

/// Format a link ID for user-friendly display
fn format_link_id(id: &str) -> String {
    match id {
        "bos" => "Boston Bruins".to_string(),
        "tor" => "Toronto Maple Leafs".to_string(),
        "nyr" => "New York Rangers".to_string(),
        "mtl" => "Montreal Canadiens".to_string(),
        _ => id.to_string(),
    }
}

/// A focusable element within a document
#[derive(Debug, Clone)]
pub struct FocusableElement {
    /// Unique ID for this focusable element
    pub id: FocusableId,
    /// Y position in the document (for scrolling)
    pub y: u16,
    /// Height of the element
    pub height: u16,
    /// Rectangle of the focusable area (for highlighting)
    pub rect: Rect,
    /// Optional link target if this is a link
    pub link_target: Option<LinkTarget>,
    /// Tab order (lower numbers get focus first)
    pub tab_order: i32,
}

impl FocusableElement {
    /// Create a new focusable element
    pub fn new(
        id: FocusableId,
        y: u16,
        height: u16,
        rect: Rect,
        link_target: Option<LinkTarget>,
        tab_order: i32,
    ) -> Self {
        Self {
            id,
            y,
            height,
            rect,
            link_target,
            tab_order,
        }
    }

    /// Create a focusable link element
    pub fn link(id: impl Into<String>, y: u16, width: u16, target: LinkTarget) -> Self {
        Self {
            id: FocusableId::link(id),
            y,
            height: 1,
            rect: Rect::new(0, y, width, 1),
            link_target: Some(target),
            tab_order: 0,
        }
    }

    /// Create a focusable table cell element
    pub fn table_cell(
        table_name: impl Into<String>,
        row: usize,
        col: usize,
        rect: Rect,
        target: Option<LinkTarget>,
    ) -> Self {
        Self {
            id: FocusableId::table_cell(table_name, row, col),
            y: rect.y,
            height: rect.height,
            rect,
            link_target: target,
            tab_order: (row * 100 + col) as i32,
        }
    }
}

/// Manages focus state within a document
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// All focusable elements in tab order
    elements: Vec<FocusableElement>,
    /// Currently focused element index (None = no focus)
    current_focus: Option<usize>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new empty focus manager
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            current_focus: None,
        }
    }

    /// Build a focus manager from a list of focusable elements
    ///
    /// Elements are sorted by y position (document order).
    pub fn from_elements(elements: &[super::elements::DocumentElement]) -> Self {
        let mut focusable = Vec::new();
        let mut y_offset = 0u16;

        for element in elements {
            element.collect_focusable(&mut focusable, y_offset);
            y_offset += element.height();
        }

        // Sort by y position (document order) for natural reading flow
        focusable.sort_by_key(|e| e.y);

        Self {
            elements: focusable,
            current_focus: None,
        }
    }

    /// Add a focusable element
    pub fn add_element(&mut self, element: FocusableElement) {
        self.elements.push(element);
    }

    /// Get the number of focusable elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if there are no focusable elements
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Navigate to next focusable element (Tab)
    ///
    /// Returns true if focus changed, false if no elements to focus.
    /// Wraps from last to first element.
    pub fn focus_next(&mut self) -> bool {
        let new_focus = focus_helpers::focus_next(self.current_focus, self.elements.len());
        let changed = new_focus.is_some();
        self.current_focus = new_focus;
        changed
    }

    /// Navigate to previous focusable element (Shift-Tab)
    ///
    /// Returns true if focus changed, false if no elements to focus.
    /// Wraps from first to last element.
    pub fn focus_prev(&mut self) -> bool {
        let new_focus = focus_helpers::focus_prev(self.current_focus, self.elements.len());
        let changed = new_focus.is_some();
        self.current_focus = new_focus;
        changed
    }

    /// Get the currently focused element index
    pub fn current_index(&self) -> Option<usize> {
        self.current_focus
    }

    /// Get the currently focused element
    pub fn current_element(&self) -> Option<&FocusableElement> {
        self.current_focus.map(|idx| &self.elements[idx])
    }

    /// Get the currently focused element's ID
    pub fn get_current_id(&self) -> Option<&FocusableId> {
        self.current_focus.map(|idx| &self.elements[idx].id)
    }

    /// Get the currently focused element's position (y coordinate)
    pub fn get_focused_position(&self) -> Option<u16> {
        self.current_focus.map(|idx| self.elements[idx].y)
    }

    /// Get the currently focused element's height
    pub fn get_focused_height(&self) -> Option<u16> {
        self.current_focus.map(|idx| self.elements[idx].height)
    }

    /// Get the currently focused element's rectangle
    pub fn get_focused_rect(&self) -> Option<Rect> {
        self.current_focus.map(|idx| self.elements[idx].rect)
    }

    /// Activate the currently focused element
    ///
    /// Returns the link target if the focused element has one.
    pub fn activate_current(&self) -> Option<LinkTarget> {
        self.current_focus
            .and_then(|idx| self.elements[idx].link_target.clone())
    }

    /// Get the current link target without activating
    pub fn get_current_link(&self) -> Option<&LinkTarget> {
        self.current_focus
            .and_then(|idx| self.elements[idx].link_target.as_ref())
    }

    /// Clear focus (no element focused)
    pub fn clear_focus(&mut self) {
        self.current_focus = None;
    }

    /// Focus a specific element by ID
    ///
    /// Returns true if element was found and focused.
    pub fn focus_by_id(&mut self, id: &FocusableId) -> bool {
        self.current_focus = self.elements.iter().position(|e| &e.id == id);
        self.current_focus.is_some()
    }

    /// Focus a specific element by index
    ///
    /// Returns true if index was valid.
    pub fn focus_by_index(&mut self, index: usize) -> bool {
        if index < self.elements.len() {
            self.current_focus = Some(index);
            true
        } else {
            false
        }
    }

    /// Check if focus wrapped from last to first (for autoscrolling)
    ///
    /// Call this after focus_next() to detect wrap-around.
    pub fn did_wrap_forward(&self, prev_focus: Option<usize>) -> bool {
        focus_helpers::did_wrap_forward(prev_focus, self.current_focus, self.elements.len())
    }

    /// Check if focus wrapped from first to last (for autoscrolling)
    ///
    /// Call this after focus_prev() to detect wrap-around.
    pub fn did_wrap_backward(&self, prev_focus: Option<usize>) -> bool {
        focus_helpers::did_wrap_backward(prev_focus, self.current_focus, self.elements.len())
    }

    /// Get all focusable elements
    pub fn elements(&self) -> &[FocusableElement] {
        &self.elements
    }

    /// Get y-positions of all focusable elements
    ///
    /// Useful for storing positions in state for autoscrolling in reducers.
    pub fn y_positions(&self) -> Vec<u16> {
        self.elements.iter().map(|e| e.y).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::document::link::{DocumentLink, LinkTarget};

    fn create_test_elements(count: usize) -> Vec<FocusableElement> {
        (0..count)
            .map(|i| FocusableElement {
                id: FocusableId::link(format!("elem_{}", i)),
                y: i as u16 * 2,
                height: 1,
                rect: Rect::new(0, i as u16 * 2, 10, 1),
                link_target: Some(LinkTarget::Action(format!("action_{}", i))),
                tab_order: i as i32,
            })
            .collect()
    }

    #[test]
    fn test_focus_manager_new() {
        let fm = FocusManager::new();

        assert!(fm.is_empty());
        assert_eq!(fm.len(), 0);
        assert_eq!(fm.current_index(), None);
    }

    #[test]
    fn test_add_element() {
        let mut fm = FocusManager::new();
        fm.add_element(FocusableElement::link(
            "link1",
            0,
            10,
            LinkTarget::Action("test".to_string()),
        ));

        assert_eq!(fm.len(), 1);
        assert!(!fm.is_empty());
    }

    #[test]
    fn test_focus_next_empty() {
        let mut fm = FocusManager::new();
        assert!(!fm.focus_next());
        assert_eq!(fm.current_index(), None);
    }

    #[test]
    fn test_focus_next_single_element() {
        let mut fm = FocusManager::new();
        fm.add_element(FocusableElement::link(
            "link1",
            0,
            10,
            LinkTarget::Action("test".to_string()),
        ));

        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(0));

        // Tab again should stay on the same element (wrap)
        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(0));
    }

    #[test]
    fn test_focus_next_multiple_elements() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(0));

        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(1));

        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(2));

        // Wrap to first
        assert!(fm.focus_next());
        assert_eq!(fm.current_index(), Some(0));
    }

    #[test]
    fn test_focus_prev_empty() {
        let mut fm = FocusManager::new();
        assert!(!fm.focus_prev());
        assert_eq!(fm.current_index(), None);
    }

    #[test]
    fn test_focus_prev_multiple_elements() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        // First prev goes to last element
        assert!(fm.focus_prev());
        assert_eq!(fm.current_index(), Some(2));

        assert!(fm.focus_prev());
        assert_eq!(fm.current_index(), Some(1));

        assert!(fm.focus_prev());
        assert_eq!(fm.current_index(), Some(0));

        // Wrap to last
        assert!(fm.focus_prev());
        assert_eq!(fm.current_index(), Some(2));
    }

    #[test]
    fn test_get_focused_position() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        assert_eq!(fm.get_focused_position(), None);

        fm.focus_next();
        assert_eq!(fm.get_focused_position(), Some(0));

        fm.focus_next();
        assert_eq!(fm.get_focused_position(), Some(2));
    }

    #[test]
    fn test_get_focused_rect() {
        let mut fm = FocusManager::new();
        fm.add_element(FocusableElement {
            id: FocusableId::link("test"),
            y: 5,
            height: 2,
            rect: Rect::new(3, 5, 15, 2),
            link_target: None,
            tab_order: 0,
        });

        fm.focus_next();
        let rect = fm.get_focused_rect().unwrap();
        assert_eq!(rect, Rect::new(3, 5, 15, 2));
    }

    #[test]
    fn test_activate_current() {
        let mut fm = FocusManager::new();
        let target = LinkTarget::Document(DocumentLink::team("BOS"));
        fm.add_element(FocusableElement {
            id: FocusableId::link("team_link"),
            y: 0,
            height: 1,
            rect: Rect::new(0, 0, 10, 1),
            link_target: Some(target.clone()),
            tab_order: 0,
        });

        assert_eq!(fm.activate_current(), None); // No focus yet

        fm.focus_next();
        assert_eq!(fm.activate_current(), Some(target));
    }

    #[test]
    fn test_get_current_link() {
        let mut fm = FocusManager::new();
        let target = LinkTarget::Action("test".to_string());
        fm.add_element(FocusableElement {
            id: FocusableId::link("action_link"),
            y: 0,
            height: 1,
            rect: Rect::new(0, 0, 10, 1),
            link_target: Some(target.clone()),
            tab_order: 0,
        });

        assert_eq!(fm.get_current_link(), None);

        fm.focus_next();
        assert_eq!(fm.get_current_link(), Some(&target));
    }

    #[test]
    fn test_clear_focus() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        fm.focus_next();
        assert!(fm.current_index().is_some());

        fm.clear_focus();
        assert_eq!(fm.current_index(), None);
    }

    #[test]
    fn test_focus_by_id() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        assert!(fm.focus_by_id(&FocusableId::link("elem_1")));
        assert_eq!(fm.current_index(), Some(1));

        assert!(!fm.focus_by_id(&FocusableId::link("nonexistent")));
        assert_eq!(fm.current_index(), None);
    }

    #[test]
    fn test_focus_by_index() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        assert!(fm.focus_by_index(2));
        assert_eq!(fm.current_index(), Some(2));

        // Invalid index should return false but not change current focus
        assert!(!fm.focus_by_index(10));
        assert_eq!(fm.current_index(), Some(2)); // Focus unchanged
    }

    #[test]
    fn test_did_wrap_forward() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        // Navigate to last element
        fm.focus_by_index(2);
        let prev = fm.current_index();

        fm.focus_next();
        assert!(fm.did_wrap_forward(prev));

        // Normal forward navigation shouldn't be a wrap
        let prev = fm.current_index();
        fm.focus_next();
        assert!(!fm.did_wrap_forward(prev));
    }

    #[test]
    fn test_did_wrap_backward() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        // Navigate to first element
        fm.focus_by_index(0);
        let prev = fm.current_index();

        fm.focus_prev();
        assert!(fm.did_wrap_backward(prev));

        // Normal backward navigation shouldn't be a wrap
        let prev = fm.current_index();
        fm.focus_prev();
        assert!(!fm.did_wrap_backward(prev));
    }

    #[test]
    fn test_focusable_element_link() {
        let target = LinkTarget::Action("test".to_string());
        let elem = FocusableElement::link("my_link", 10, 20, target.clone());

        assert_eq!(elem.id, FocusableId::link("my_link"));
        assert_eq!(elem.y, 10);
        assert_eq!(elem.height, 1);
        assert_eq!(elem.rect, Rect::new(0, 10, 20, 1));
        assert_eq!(elem.link_target, Some(target));
        assert_eq!(elem.tab_order, 0);
    }

    #[test]
    fn test_focusable_element_table_cell() {
        let target = LinkTarget::Document(DocumentLink::player(12345));
        let rect = Rect::new(5, 10, 15, 1);
        let elem = FocusableElement::table_cell("standings", 3, 2, rect, Some(target.clone()));

        assert_eq!(elem.id, FocusableId::table_cell("standings", 3, 2));
        assert_eq!(elem.y, 10);
        assert_eq!(elem.height, 1);
        assert_eq!(elem.rect, rect);
        assert_eq!(elem.link_target, Some(target));
        assert_eq!(elem.tab_order, 302); // row * 100 + col
    }

    #[test]
    fn test_current_element() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        assert!(fm.current_element().is_none());

        fm.focus_next();
        let current = fm.current_element().unwrap();
        assert_eq!(current.id, FocusableId::link("elem_0"));
    }

    #[test]
    fn test_get_focused_height() {
        let mut fm = FocusManager::new();
        fm.add_element(FocusableElement {
            id: FocusableId::link("test"),
            y: 5,
            height: 3,
            rect: Rect::new(0, 5, 10, 3),
            link_target: None,
            tab_order: 0,
        });

        assert_eq!(fm.get_focused_height(), None);

        fm.focus_next();
        assert_eq!(fm.get_focused_height(), Some(3));
    }

    #[test]
    fn test_elements_accessor() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        let elements = fm.elements();
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0].id, FocusableId::link("elem_0"));
        assert_eq!(elements[1].id, FocusableId::link("elem_1"));
        assert_eq!(elements[2].id, FocusableId::link("elem_2"));
    }

    #[test]
    fn test_y_positions() {
        let mut fm = FocusManager::new();
        for elem in create_test_elements(3) {
            fm.add_element(elem);
        }

        // Test elements have y positions at 0, 2, 4 (i * 2)
        let positions = fm.y_positions();
        assert_eq!(positions, vec![0, 2, 4]);
    }

    #[test]
    fn test_y_positions_empty() {
        let fm = FocusManager::new();
        assert!(fm.y_positions().is_empty());
    }
}
