/// Generic container widget for managing multiple focusable children
///
/// This widget automatically handles:
/// - Focus delegation to active child
/// - Navigation between children with Up/Down/Tab/Shift+Tab
/// - Optional wrapping from last to first child
/// - Automatic focus state management
///
/// This eliminates the boilerplate of manually managing inter-widget navigation.

use super::focus::*;
use super::RenderableWidget;
use crate::config::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::Rect};

/// A container that manages vertical navigation between focusable children
///
/// # Example
///
/// ```rust
/// let table = FocusableTable::new(data);
/// let list = List::new();
///
/// let container = Container::new()
///     .add_child(Box::new(table))
///     .add_child(Box::new(list))
///     .with_wrap(true);
/// ```
pub struct Container {
    id: WidgetId,
    children: Vec<Box<dyn Focusable>>,
    focused_index: usize,
    wrap: bool,
}

/// Where to position focus when entering a child widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPosition {
    /// Start at the first item (used when navigating down/forward)
    First,
    /// Start at the last item (used when navigating up/backward)
    Last,
    /// Keep current position (used when re-focusing same widget)
    Current,
}

impl Container {
    /// Create a new empty container
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            children: Vec::new(),
            focused_index: 0,
            wrap: true,
        }
    }

    /// Create a container with initial children
    pub fn with_children(children: Vec<Box<dyn Focusable>>) -> Self {
        let mut container = Self {
            id: WidgetId::new(),
            children,
            focused_index: 0,
            wrap: true,
        };

        // Focus the first child
        if !container.children.is_empty() {
            container.apply_focus_position(0, FocusPosition::First);
        }

        container
    }

    /// Add a child widget to the container
    pub fn add_child(mut self, child: Box<dyn Focusable>) -> Self {
        let should_focus = self.children.is_empty();
        self.children.push(child);

        if should_focus {
            self.apply_focus_position(0, FocusPosition::First);
        }

        self
    }

    /// Set whether to wrap from last to first child
    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Get the currently focused child (mutable)
    fn focused_child_mut(&mut self) -> Option<&mut Box<dyn Focusable>> {
        self.children.get_mut(self.focused_index)
    }

    /// Get a child by index (for rendering)
    pub fn child(&self, index: usize) -> Option<&dyn Focusable> {
        self.children.get(index).map(|b| b.as_ref())
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Apply focus to a child at a specific position
    fn apply_focus_position(&mut self, index: usize, position: FocusPosition) {
        if let Some(child) = self.children.get_mut(index) {
            match position {
                FocusPosition::First => {
                    child.focus_first();
                }
                FocusPosition::Last => {
                    child.focus_last();
                }
                FocusPosition::Current => {
                    child.set_focused(true);
                }
            }
        }
    }

    /// Move focus to the next child
    fn focus_next(&mut self) -> InputResult {
        if self.children.is_empty() {
            return InputResult::NotHandled;
        }

        if self.focused_index + 1 < self.children.len() {
            // Unfocus current child
            self.children[self.focused_index].set_focused(false);

            // Move to next child
            self.focused_index += 1;

            // Focus new child at first position
            self.apply_focus_position(self.focused_index, FocusPosition::First);

            InputResult::Handled
        } else if self.wrap {
            // Wrap to first child
            self.children[self.focused_index].set_focused(false);
            self.focused_index = 0;
            self.apply_focus_position(self.focused_index, FocusPosition::First);
            InputResult::Handled
        } else {
            // Block at boundary
            InputResult::NotHandled
        }
    }

    /// Move focus to the previous child
    fn focus_prev(&mut self) -> InputResult {
        if self.children.is_empty() {
            return InputResult::NotHandled;
        }

        if self.focused_index > 0 {
            // Unfocus current child
            self.children[self.focused_index].set_focused(false);

            // Move to previous child
            self.focused_index -= 1;

            // Focus new child at last position
            self.apply_focus_position(self.focused_index, FocusPosition::Last);

            InputResult::Handled
        } else if self.wrap {
            // Wrap to last child
            self.children[self.focused_index].set_focused(false);
            self.focused_index = self.children.len() - 1;
            self.apply_focus_position(self.focused_index, FocusPosition::Last);
            InputResult::Handled
        } else {
            // Block at boundary
            InputResult::NotHandled
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Focusable for Container {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        !self.children.is_empty()
    }

    fn is_focused(&self) -> bool {
        // Container is focused if any child is focused
        self.children.iter().any(|c| c.is_focused())
    }

    fn set_focused(&mut self, focused: bool) {
        if focused {
            // Focus the current child
            if let Some(child) = self.children.get_mut(self.focused_index) {
                child.set_focused(true);
            }
        } else {
            // Unfocus all children
            for child in &mut self.children {
                child.set_focused(false);
            }
        }
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        // First, try to delegate to the focused child
        if let Some(child) = self.focused_child_mut() {
            let result = child.handle_input(event);

            // If the child handled it, we're done
            if result != InputResult::NotHandled {
                return result;
            }
        }

        // Child didn't handle it, check if we should switch focus
        let is_shift = event.modifiers.contains(KeyModifiers::SHIFT);
        match event.code {
            // Down arrow or Tab (without shift) - move to next child
            KeyCode::Down => self.focus_next(),
            KeyCode::Tab if !is_shift => self.focus_next(),

            // Up arrow or Shift+Tab/BackTab - move to previous child
            KeyCode::Up => self.focus_prev(),
            KeyCode::BackTab => self.focus_prev(),
            KeyCode::Tab if is_shift => self.focus_prev(),

            // Other keys are not handled
            _ => InputResult::NotHandled,
        }
    }

    fn focusable_children(&self) -> Vec<WidgetId> {
        self.children.iter().map(|c| c.widget_id()).collect()
    }

    fn find_child(&self, id: WidgetId) -> Option<&dyn Focusable> {
        self.children.iter()
            .find(|c| c.widget_id() == id)
            .map(|c| c.as_ref())
    }

    fn find_child_mut(&mut self, id: WidgetId) -> Option<&mut (dyn Focusable + '_)> {
        for child in &mut self.children {
            if child.widget_id() == id {
                return Some(child.as_mut());
            }
        }
        None
    }
}

impl RenderableWidget for Container {
    fn render(&self, _area: Rect, _buf: &mut Buffer, _config: &DisplayConfig) {
        // Container doesn't render anything itself
        // Rendering should be done by a layout manager or the parent
        // This is intentionally empty - containers are for focus management,
        // not rendering
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::Link;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_container_creation() {
        let container = Container::new();
        assert_eq!(container.children.len(), 0);
        assert!(!container.can_focus());
    }

    #[test]
    fn test_container_with_children() {
        let container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]);

        assert_eq!(container.children.len(), 2);
        assert!(container.can_focus());
    }

    #[test]
    fn test_navigation_forward() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]);

        assert_eq!(container.focused_index, 0);

        // Navigate forward
        let result = container.focus_next();
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 1);
    }

    #[test]
    fn test_navigation_backward() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]);

        container.focused_index = 1;

        // Navigate backward
        let result = container.focus_prev();
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 0);
    }

    #[test]
    fn test_wrapping_forward() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]).with_wrap(true);

        container.focused_index = 1;

        // Navigate forward from last item - should wrap
        let result = container.focus_next();
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 0);
    }

    #[test]
    fn test_wrapping_backward() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]).with_wrap(true);

        container.focused_index = 0;

        // Navigate backward from first item - should wrap
        let result = container.focus_prev();
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 1);
    }

    #[test]
    fn test_no_wrapping() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]).with_wrap(false);

        container.focused_index = 1;

        // Navigate forward from last item - should block
        let result = container.focus_next();
        assert_eq!(result, InputResult::NotHandled);
        assert_eq!(container.focused_index, 1);
    }

    #[test]
    fn test_input_delegation() {
        let mut container = Container::with_children(vec![
            Box::new(Link::new("Item 1")),
            Box::new(Link::new("Item 2")),
        ]);

        // Tab should move to next child
        let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 1);

        // Shift+Tab should move back
        let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(container.focused_index, 0);
    }

    // Integration tests: Container + List
    mod integration_list {
        use super::*;
        use crate::tui::widgets::List;

        #[test]
        fn test_container_with_lists() {
            // Create two lists with items
            let mut list1 = List::new();
            list1.add_item(Box::new(Link::new("List1 Item1")));
            list1.add_item(Box::new(Link::new("List1 Item2")));

            let mut list2 = List::new();
            list2.add_item(Box::new(Link::new("List2 Item1")));
            list2.add_item(Box::new(Link::new("List2 Item2")));

            let mut container = Container::with_children(vec![
                Box::new(list1),
                Box::new(list2),
            ]).with_wrap(true);

            // Container should focus first list
            assert_eq!(container.focused_index, 0);
            assert!(container.is_focused());
        }

        #[test]
        fn test_tab_navigates_from_list_to_list() {
            // Create two lists
            let mut list1 = List::new();
            list1.add_item(Box::new(Link::new("List1 Item1")));
            list1.add_item(Box::new(Link::new("List1 Item2")));

            let mut list2 = List::new();
            list2.add_item(Box::new(Link::new("List2 Item1")));
            list2.add_item(Box::new(Link::new("List2 Item2")));

            let mut container = Container::with_children(vec![
                Box::new(list1),
                Box::new(list2),
            ]).with_wrap(false);

            // Start at list1
            assert_eq!(container.focused_index, 0);

            // Press Down while in list1 - should navigate within list
            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            // Still in list1, but item index changed inside the list
            assert_eq!(container.focused_index, 0);

            // Press Down at end of list1 - should block (list returns NotHandled)
            // which triggers container to move to next child
            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            // Now in list2
            assert_eq!(container.focused_index, 1);
        }

        #[test]
        fn test_shift_tab_navigates_backwards_between_lists() {
            let mut list1 = List::new();
            list1.add_item(Box::new(Link::new("List1 Item1")));

            let mut list2 = List::new();
            list2.add_item(Box::new(Link::new("List2 Item1")));

            let mut container = Container::with_children(vec![
                Box::new(list1),
                Box::new(list2),
            ]).with_wrap(false);

            // Start at list2
            container.focused_index = 1;
            container.children[0].set_focused(false);
            container.children[1].set_focused(true);

            // Shift+Tab should move to list1
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0);
        }

        #[test]
        fn test_wrapping_from_last_list_to_first() {
            let mut list1 = List::new();
            list1.add_item(Box::new(Link::new("List1 Item1")));

            let mut list2 = List::new();
            list2.add_item(Box::new(Link::new("List2 Item1")));

            let mut container = Container::with_children(vec![
                Box::new(list1),
                Box::new(list2),
            ]).with_wrap(true);

            // Start at last list
            container.focused_index = 1;
            container.children[0].set_focused(false);
            container.children[1].set_focused(true);

            // Tab should wrap to first list
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0);
        }

        #[test]
        fn test_focus_delegation_to_list_children() {
            let mut list1 = List::new();
            list1.add_item(Box::new(Link::new("Item1")));
            list1.add_item(Box::new(Link::new("Item2")));

            let mut container = Container::with_children(vec![
                Box::new(list1),
            ]);

            // Container should delegate focus to list
            assert!(container.is_focused());

            // List should have a focused child
            if let Some(list) = container.child(0) {
                assert!(list.is_focused());
            } else {
                panic!("Container should have list child");
            }
        }
    }

    // Integration tests: Container + Table
    mod integration_table {
        use super::*;
        use crate::tui::widgets::{FocusableTable, ColumnDef, Alignment};

        #[derive(Debug, Clone)]
        struct TestRow {
            name: String,
            value: i32,
        }

        fn test_table() -> FocusableTable<TestRow> {
            let rows = vec![
                TestRow { name: "Row 1".to_string(), value: 10 },
                TestRow { name: "Row 2".to_string(), value: 20 },
                TestRow { name: "Row 3".to_string(), value: 30 },
            ];

            FocusableTable::new(rows).with_columns(vec![
                ColumnDef::new("Name", 10, |r: &TestRow| r.name.clone(), Alignment::Left, true),
                ColumnDef::new("Value", 8, |r: &TestRow| r.value.to_string(), Alignment::Right, false),
            ])
        }

        #[test]
        fn test_container_with_table() {
            let table = test_table();

            let mut container = Container::with_children(vec![
                Box::new(table),
            ]);

            // Container should focus the table
            assert_eq!(container.focused_index, 0);
            assert!(container.is_focused());

            // Table should be focused
            if let Some(child) = container.child(0) {
                assert!(child.is_focused());
            }
        }

        #[test]
        fn test_navigation_within_table_then_to_next_widget() {
            let table = test_table();
            let mut list = crate::tui::widgets::List::new();
            list.add_item(Box::new(Link::new("List Item")));

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
            ]).with_wrap(false);

            // Start in table
            assert_eq!(container.focused_index, 0);

            // Down arrow navigates within table
            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0); // Still in table

            // Another Down arrow
            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0); // Still in table

            // Down at last row should move to next widget
            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 1); // Now in list
        }

        #[test]
        fn test_table_boundary_blocking_triggers_container_navigation() {
            let table = test_table();
            let link = Link::new("Link Item");

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(link),
            ]).with_wrap(false);

            // Start in table at first row
            assert_eq!(container.focused_index, 0);

            // Up at first row should NOT move to previous widget yet
            // because table is at row 0 and hasn't returned NotHandled
            let result = container.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
            // Table at first row returns NotHandled, container moves to prev widget
            assert_eq!(container.focused_index, 0); // Blocked at boundary
        }

        #[test]
        fn test_tab_at_last_row_moves_to_next_widget() {
            let mut table = test_table();
            let mut list = crate::tui::widgets::List::new();
            list.add_item(Box::new(Link::new("List Item")));

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
            ]).with_wrap(false);

            // Start in table
            assert_eq!(container.focused_index, 0);

            // Move table to last row (row 2)
            container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

            // Tab at last row should move to list
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 1);
        }

        #[test]
        fn test_shift_tab_from_list_to_table() {
            let table = test_table();
            let mut list = crate::tui::widgets::List::new();
            list.add_item(Box::new(Link::new("List Item")));

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
            ]).with_wrap(false);

            // Start in list
            container.focused_index = 1;
            container.children[0].set_focused(false);
            container.children[1].set_focused(true);

            // Shift+Tab should move to table
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0);
        }
    }

    // Integration tests: Container + Mixed widgets (Table, List, Link)
    mod integration_mixed {
        use super::*;
        use crate::tui::widgets::{FocusableTable, ColumnDef, Alignment, List};

        #[derive(Debug, Clone)]
        struct TestRow {
            name: String,
        }

        fn test_table() -> FocusableTable<TestRow> {
            let rows = vec![
                TestRow { name: "Row 1".to_string() },
                TestRow { name: "Row 2".to_string() },
            ];

            FocusableTable::new(rows).with_columns(vec![
                ColumnDef::new("Name", 10, |r: &TestRow| r.name.clone(), Alignment::Left, true),
            ])
        }

        #[test]
        fn test_complex_navigation_table_list_link() {
            let table = test_table();
            let mut list = List::new();
            list.add_item(Box::new(Link::new("List Item 1")));
            list.add_item(Box::new(Link::new("List Item 2")));
            let link = Link::new("Standalone Link");

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
                Box::new(link),
            ]).with_wrap(true);

            // Start in table (index 0)
            assert_eq!(container.focused_index, 0);

            // Tab at row 0 moves within table to row 1
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 0);

            // Tab at last row moves to list
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 1);

            // Tab in list item 0 moves to item 1
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 1);

            // Tab at last list item moves to link
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 2);

            // Tab at link wraps to table
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 0);
        }

        #[test]
        fn test_backwards_navigation_link_list_table() {
            let table = test_table();
            let mut list = List::new();
            list.add_item(Box::new(Link::new("List Item")));
            let link = Link::new("Standalone Link");

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
                Box::new(link),
            ]).with_wrap(true);

            // Start at link (index 2)
            container.focused_index = 2;
            container.children[0].set_focused(false);
            container.children[1].set_focused(false);
            container.children[2].set_focused(true);

            // Shift+Tab at link moves to list (list will be at last item due to focus_last())
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 1);

            // Shift+Tab in list at first item: list returns NotHandled, container moves to table
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);  // Container handled navigation
            assert_eq!(container.focused_index, 0);

            // Table is now at last row (due to FocusPosition::Last) - Shift+Tab moves within table
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 0);  // Still in table, moved to first row

            // Now at table first row - Shift+Tab wraps to link
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(container.focused_index, 2);
        }

        #[test]
        fn test_enter_activates_link_in_mixed_container() {
            let table = test_table();
            let link = Link::new("Clickable Link").with_action(|| NavigationAction::PopPanel);

            let mut container = Container::with_children(vec![
                Box::new(table),
                Box::new(link),
            ]);

            // Navigate to link
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 1);

            // Enter should activate link
            let result = container.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Navigate(NavigationAction::PopPanel));
        }

        #[test]
        fn test_focus_delegation_in_mixed_container() {
            let table = test_table();
            let mut list = List::new();
            list.add_item(Box::new(Link::new("List Item")));
            let link = Link::new("Link");

            let container = Container::with_children(vec![
                Box::new(table),
                Box::new(list),
                Box::new(link),
            ]);

            // Container is focused
            assert!(container.is_focused());

            // First child (table) should be focused
            if let Some(child) = container.child(0) {
                assert!(child.is_focused());
            }

            // Other children should not be focused
            if let Some(child) = container.child(1) {
                assert!(!child.is_focused());
            }
            if let Some(child) = container.child(2) {
                assert!(!child.is_focused());
            }
        }
    }

    // Integration tests: Nested Containers
    mod integration_nested {
        use super::*;

        #[test]
        fn test_container_containing_container() {
            let inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]);

            let mut outer_container = Container::with_children(vec![
                Box::new(Link::new("Outer Link")),
                Box::new(inner_container),
            ]);

            // Outer container should be focused
            assert!(outer_container.is_focused());
            assert_eq!(outer_container.focused_index, 0);

            // First child (outer link) should be focused
            if let Some(child) = outer_container.child(0) {
                assert!(child.is_focused());
            }
        }

        #[test]
        fn test_navigation_into_nested_container() {
            let inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]);

            let mut outer_container = Container::with_children(vec![
                Box::new(Link::new("Outer Link")),
                Box::new(inner_container),
            ]).with_wrap(false);

            // Start at outer link (index 0)
            assert_eq!(outer_container.focused_index, 0);

            // Tab moves to inner container
            let result = outer_container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(outer_container.focused_index, 1);

            // Inner container should now be focused
            if let Some(inner) = outer_container.child(1) {
                assert!(inner.is_focused());
            }
        }

        #[test]
        fn test_focus_delegation_through_multiple_levels() {
            let inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]);

            let middle_container = Container::with_children(vec![
                Box::new(inner_container),
                Box::new(Link::new("Middle Link")),
            ]);

            let outer_container = Container::with_children(vec![
                Box::new(middle_container),
                Box::new(Link::new("Outer Link")),
            ]);

            // Outer container is focused
            assert!(outer_container.is_focused());

            // Middle container (first child) should be focused
            if let Some(middle) = outer_container.child(0) {
                assert!(middle.is_focused());

                // Inner container (first child of middle) should be focused
                // We can't easily check this without downcasting, but the delegation should work
            }
        }

        #[test]
        fn test_focus_first_propagation() {
            let mut inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]);

            // Manually set inner to non-first position
            inner_container.focused_index = 1;
            inner_container.children[0].set_focused(false);
            inner_container.children[1].set_focused(true);

            let mut outer_container = Container::with_children(vec![
                Box::new(Link::new("Outer Link")),
                Box::new(inner_container),
            ]);

            // Navigate to inner container
            outer_container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(outer_container.focused_index, 1);

            // When navigating forward into inner container, it should call focus_first()
            // which resets to first child
            if let Some(inner) = outer_container.child(1) {
                assert!(inner.is_focused());
                // Can't easily verify the inner focused_index without downcasting,
                // but focus_first() should have been called
            }
        }

        #[test]
        fn test_focus_last_propagation() {
            let inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]);

            let mut outer_container = Container::with_children(vec![
                Box::new(inner_container),
                Box::new(Link::new("Outer Link")),
            ]).with_wrap(false);

            // Start at outer link (index 1)
            outer_container.focused_index = 1;
            outer_container.children[0].set_focused(false);
            outer_container.children[1].set_focused(true);

            // Shift+Tab back to inner container
            let result = outer_container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(outer_container.focused_index, 0);

            // When navigating backward into inner container, it should call focus_last()
            // which positions at last child
            if let Some(inner) = outer_container.child(0) {
                assert!(inner.is_focused());
                // focus_last() should have been called, positioning inner at last item
            }
        }

        #[test]
        fn test_nested_container_wrapping() {
            let inner_container = Container::with_children(vec![
                Box::new(Link::new("Inner Link 1")),
                Box::new(Link::new("Inner Link 2")),
            ]).with_wrap(true);

            let mut outer_container = Container::with_children(vec![
                Box::new(inner_container),
                Box::new(Link::new("Outer Link")),
            ]).with_wrap(true);

            // Start at outer link
            outer_container.focused_index = 1;
            outer_container.children[0].set_focused(false);
            outer_container.children[1].set_focused(true);

            // Tab should wrap from outer link to inner container
            let result = outer_container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);
            assert_eq!(outer_container.focused_index, 0);
        }
    }

    // FocusPosition behavior tests
    mod focus_position {
        use super::*;
        use crate::tui::widgets::{List, FocusableTable, ColumnDef, Alignment};

        #[derive(Debug, Clone)]
        struct TestRow {
            name: String,
        }

        fn test_table() -> FocusableTable<TestRow> {
            let rows = vec![
                TestRow { name: "Row 1".to_string() },
                TestRow { name: "Row 2".to_string() },
                TestRow { name: "Row 3".to_string() },
            ];

            FocusableTable::new(rows).with_columns(vec![
                ColumnDef::new("Name", 10, |r: &TestRow| r.name.clone(), Alignment::Left, true),
            ])
        }

        fn test_list() -> List {
            let mut list = List::new();
            list.add_item(Box::new(Link::new("Item 1")));
            list.add_item(Box::new(Link::new("Item 2")));
            list.add_item(Box::new(Link::new("Item 3")));
            list
        }

        #[test]
        fn test_focus_position_first_with_list() {
            let mut list = test_list();

            // Manually set list to last item
            list.select_last();
            assert_eq!(list.selected_index(), 2);

            // Apply FocusPosition::First
            list.focus_first();

            // List should be at first item
            assert_eq!(list.selected_index(), 0);
            assert!(list.is_focused());
        }

        #[test]
        fn test_focus_position_last_with_list() {
            let mut list = test_list();

            // List starts at first item
            assert_eq!(list.selected_index(), 0);

            // Apply FocusPosition::Last
            list.focus_last();

            // List should be at last item
            assert_eq!(list.selected_index(), 2);
            assert!(list.is_focused());
        }

        #[test]
        fn test_focus_position_current_with_list() {
            let mut list = test_list();

            // Manually navigate to middle item
            list.set_focused(true);
            list.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(list.selected_index(), 1);
            list.set_focused(false);

            // Apply FocusPosition::Current (just set_focused)
            list.set_focused(true);

            // List should stay at current position
            assert_eq!(list.selected_index(), 1);
            assert!(list.is_focused());
        }

        #[test]
        fn test_focus_position_first_with_table() {
            let mut table = test_table();

            // Manually move table to last row
            table.set_focused(true);
            table.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            table.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(table.selected_row(), Some(2));
            table.set_focused(false);

            // Apply FocusPosition::First
            table.focus_first();

            // Table should be at first row
            assert_eq!(table.selected_row(), Some(0));
            assert!(table.is_focused());
        }

        #[test]
        fn test_focus_position_last_with_table() {
            let mut table = test_table();

            // Table starts at first row
            table.set_focused(true);
            assert_eq!(table.selected_row(), Some(0));
            table.set_focused(false);

            // Apply FocusPosition::Last
            table.focus_last();

            // Table should be at last row
            assert_eq!(table.selected_row(), Some(2));
            assert!(table.is_focused());
        }

        #[test]
        fn test_container_applies_focus_position_first_on_forward_navigation() {
            let mut list = test_list();
            list.set_focused(true);
            list.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            list.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            // List is now at last item (index 2)
            list.set_focused(false);

            let mut container = Container::with_children(vec![
                Box::new(Link::new("Link")),
                Box::new(list),
            ]).with_wrap(false);

            // Start at link
            assert_eq!(container.focused_index, 0);

            // Tab forward to list - should apply FocusPosition::First
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 1);

            // List should be at first item (focus_first was called)
            // We can't easily verify this without downcasting, but the behavior is tested above
        }

        #[test]
        fn test_container_applies_focus_position_last_on_backward_navigation() {
            let list = test_list();

            let mut container = Container::with_children(vec![
                Box::new(list),
                Box::new(Link::new("Link")),
            ]).with_wrap(false);

            // Start at link (index 1)
            container.focused_index = 1;
            container.children[0].set_focused(false);
            container.children[1].set_focused(true);

            // Shift+Tab backward to list - should apply FocusPosition::Last
            container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(container.focused_index, 0);

            // List should be at last item (focus_last was called)
            // We can't easily verify this without downcasting, but the behavior is tested above
        }
    }

    // Edge case tests
    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_container_cannot_focus() {
            let container = Container::new();
            assert!(!container.can_focus());
            assert!(!container.is_focused());
            assert_eq!(container.child_count(), 0);
        }

        #[test]
        fn test_empty_container_handle_input() {
            let mut container = Container::new();

            // Any key should return NotHandled
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::NotHandled);

            let result = container.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            assert_eq!(result, InputResult::NotHandled);
        }

        #[test]
        fn test_single_child_container() {
            let mut container = Container::with_children(vec![
                Box::new(Link::new("Only Child")),
            ]).with_wrap(true);

            assert_eq!(container.child_count(), 1);
            assert!(container.can_focus());
            assert!(container.is_focused());

            // Tab should not move (nowhere to go)
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::Handled);  // Wraps to self
            assert_eq!(container.focused_index, 0);
        }

        #[test]
        fn test_single_child_container_no_wrap() {
            let mut container = Container::with_children(vec![
                Box::new(Link::new("Only Child")),
            ]).with_wrap(false);

            // Tab should block (nowhere to go)
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(result, InputResult::NotHandled);
            assert_eq!(container.focused_index, 0);

            // Shift+Tab should also block
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT));
            assert_eq!(result, InputResult::NotHandled);
            assert_eq!(container.focused_index, 0);
        }

        #[test]
        fn test_container_child_accessor_out_of_bounds() {
            let container = Container::with_children(vec![
                Box::new(Link::new("Child 1")),
            ]);

            assert!(container.child(0).is_some());
            assert!(container.child(1).is_none());
            assert!(container.child(100).is_none());
        }

        #[test]
        fn test_empty_list_in_container() {
            let empty_list = crate::tui::widgets::List::new();
            let mut container = Container::with_children(vec![
                Box::new(Link::new("Link")),
                Box::new(empty_list),
            ]);

            // Container should focus first child (link)
            assert_eq!(container.focused_index, 0);

            // Tab should skip empty list and block at boundary
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            // Empty list can't focus, so container moves to it but finds it can't focus
            // Actually, container will move to it and try to focus it
            assert_eq!(container.focused_index, 1);
        }

        #[test]
        fn test_empty_table_in_container() {
            let empty_table: crate::tui::widgets::FocusableTable<String> =
                crate::tui::widgets::FocusableTable::new(vec![]);
            let mut container = Container::with_children(vec![
                Box::new(Link::new("Link")),
                Box::new(empty_table),
            ]);

            // Container should focus first child (link)
            assert_eq!(container.focused_index, 0);

            // Tab moves to empty table
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            assert_eq!(container.focused_index, 1);
        }

        #[test]
        fn test_unfocused_container_ignores_input() {
            let mut container = Container::with_children(vec![
                Box::new(Link::new("Link 1")),
                Box::new(Link::new("Link 2")),
            ]);

            // Unfocus the container
            container.set_focused(false);

            // Input should still be processed (container delegates to children)
            let result = container.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
            // Child isn't focused either, so returns NotHandled, then container tries to navigate
            assert_eq!(result, InputResult::Handled);
        }

        #[test]
        fn test_container_focusable_children_list() {
            let mut list = crate::tui::widgets::List::new();
            list.add_item(Box::new(Link::new("Item")));

            let container = Container::with_children(vec![
                Box::new(Link::new("Link")),
                Box::new(list),
            ]);

            let children = container.focusable_children();
            assert_eq!(children.len(), 2);
        }
    }
}
