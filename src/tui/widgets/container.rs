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
}
