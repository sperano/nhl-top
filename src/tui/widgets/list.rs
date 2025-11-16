/// Vertical list widget for focusable items
///
/// This widget provides a scrollable list of focusable items with automatic
/// focus management and keyboard navigation.

use super::focus::*;
use crate::config::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
};

/// A vertical list of focusable widgets
///
/// The List widget manages a collection of focusable items, handling:
/// - Up/Down navigation between items
/// - Automatic scrolling to keep selected item visible
/// - Focus delegation to selected item
/// - Home/End keys for jumping to first/last item
pub struct List {
    id: WidgetId,
    items: Vec<Box<dyn Focusable>>,
    selected_index: usize,
    focused: bool,
    /// Visual style
    style: ListStyle,
    /// Scroll state
    scroll_offset: usize,
    /// Number of visible items (calculated during render)
    visible_items: usize,
}

/// Visual styling for lists
#[derive(Debug, Clone)]
pub struct ListStyle {
    /// Show border around list
    pub border: bool,
    /// Symbol to show for selected item
    pub highlight_symbol: String,
    /// Spacing between items (lines)
    pub spacing: u16,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            border: false,
            highlight_symbol: "".to_string(), // Links show their own indicator
            spacing: 0,
        }
    }
}

impl List {
    /// Create a new empty list
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            items: Vec::new(),
            selected_index: 0,
            focused: false,
            style: ListStyle::default(),
            scroll_offset: 0,
            visible_items: 10,
        }
    }

    /// Create a list with items
    pub fn with_items(mut self, items: Vec<Box<dyn Focusable>>) -> Self {
        self.items = items;
        self
    }

    /// Add an item to the list
    pub fn add_item(&mut self, item: Box<dyn Focusable>) {
        self.items.push(item);
    }

    /// Set the visual style
    pub fn with_style(mut self, style: ListStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the number of items in the list
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Ensure the selected item is visible
    fn ensure_visible(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_items {
            self.scroll_offset = self.selected_index.saturating_sub(self.visible_items - 1);
        }
    }

    /// Select the next item
    fn select_next(&mut self) -> bool {
        if self.selected_index + 1 < self.items.len() {
            self.items[self.selected_index].set_focused(false);
            self.selected_index += 1;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Select the previous item
    fn select_previous(&mut self) -> bool {
        if self.selected_index > 0 {
            self.items[self.selected_index].set_focused(false);
            self.selected_index -= 1;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Select the first item
    pub fn select_first(&mut self) -> bool {
        if self.selected_index > 0 {
            self.items[self.selected_index].set_focused(false);
            self.selected_index = 0;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    /// Select the last item
    pub fn select_last(&mut self) -> bool {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.items[self.selected_index].set_focused(false);
            self.selected_index = self.items.len() - 1;
            self.items[self.selected_index].set_focused(true);
            self.ensure_visible();
            true
        } else {
            false
        }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl Focusable for List {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        !self.items.is_empty()
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;

        // Update child focus
        if focused && !self.items.is_empty() {
            self.items[self.selected_index].set_focused(true);
        } else {
            for item in &mut self.items {
                item.set_focused(false);
            }
        }
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused || self.items.is_empty() {
            return InputResult::NotHandled;
        }

        // First try to delegate to selected item
        let result = self.items[self.selected_index].handle_input(event);
        if result != InputResult::NotHandled {
            return result;
        }

        // Handle list navigation
        let is_shift = event.modifiers.contains(KeyModifiers::SHIFT);
        match event.code {
            // Up arrow or Shift+Tab (BackTab) - move up
            KeyCode::Up | KeyCode::BackTab => {
                if self.select_previous() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Shift+Tab sends as Tab with SHIFT modifier (for tests/some terminals)
            KeyCode::Tab if is_shift => {
                if self.select_previous() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Down arrow or Tab (without shift) - move down
            KeyCode::Down | KeyCode::Tab => {
                if self.select_next() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled  // Block at boundary
                }
            }
            // Home key - jump to first
            KeyCode::Home => {
                if self.select_first() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // End key - jump to last
            KeyCode::End => {
                if self.select_last() {
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // PageDown key - page down
            KeyCode::PageDown => {
                // Scroll down by visible_items
                let target = (self.selected_index + self.visible_items).min(self.items.len() - 1);
                if target != self.selected_index {
                    self.items[self.selected_index].set_focused(false);
                    self.selected_index = target;
                    self.items[self.selected_index].set_focused(true);
                    self.ensure_visible();
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // PageUp key - page up
            KeyCode::PageUp => {
                // Scroll up by visible_items
                let target = self.selected_index.saturating_sub(self.visible_items);
                if target != self.selected_index {
                    self.items[self.selected_index].set_focused(false);
                    self.selected_index = target;
                    self.items[self.selected_index].set_focused(true);
                    self.ensure_visible();
                    InputResult::Handled
                } else {
                    InputResult::NotHandled
                }
            }
            // Any other key combination
            _ => InputResult::NotHandled,
        }
    }

    fn focusable_children(&self) -> Vec<WidgetId> {
        self.items.iter().map(|item| item.widget_id()).collect()
    }

    fn find_child(&self, id: WidgetId) -> Option<&dyn Focusable> {
        self.items.iter()
            .find(|item| item.widget_id() == id)
            .map(|item| item.as_ref())
    }

    fn find_child_mut(&mut self, id: WidgetId) -> Option<&mut (dyn Focusable + '_)> {
        for item in &mut self.items {
            if item.widget_id() == id {
                return Some(item.as_mut());
            }
        }
        None
    }

    fn focus_first(&mut self) {
        self.select_first();
        self.set_focused(true);
    }

    fn focus_last(&mut self) {
        self.select_last();
        self.set_focused(true);
    }
}

impl super::RenderableWidget for List {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        // Calculate visible range
        let visible_height = area.height as usize;

        let end_index = (self.scroll_offset + visible_height).min(self.items.len());

        // Render visible items
        let mut y = area.y;
        for (idx, item) in self.items[self.scroll_offset..end_index].iter().enumerate() {
            let item_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            // Render highlight symbol if selected
            if self.scroll_offset + idx == self.selected_index && !self.style.highlight_symbol.is_empty() {
                buf.set_string(
                    area.x,
                    y,
                    &self.style.highlight_symbol,
                    Style::default().add_modifier(Modifier::BOLD),
                );
            }

            // Render the item
            item.render(item_area, buf, config);

            y += 1 + self.style.spacing;
            if y >= area.y + area.height {
                break;
            }
        }

        // Render scroll indicators if needed
        if self.scroll_offset > 0 {
            buf.set_string(area.right() - 1, area.y, "▲", Style::default());
        }
        if end_index < self.items.len() {
            buf.set_string(area.right() - 1, area.bottom() - 1, "▼", Style::default());
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        let item_height = 1 + self.style.spacing;
        Some((self.items.len() as u16) * item_height)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use super::super::link::Link;
//
//     #[test]
//     fn test_list_creation() {
//         let list = List::new();
//         assert_eq!(list.len(), 0);
//         assert!(list.is_empty());
//         assert!(!list.can_focus());
//     }
//
//     #[test]
//     fn test_list_add_items() {
//         let mut list = List::new();
//         list.add_item(Box::new(Link::player("Player 1", 1)));
//         list.add_item(Box::new(Link::player("Player 2", 2)));
//
//         assert_eq!(list.len(), 2);
//         assert!(!list.is_empty());
//         assert!(list.can_focus());
//     }
//
//     #[test]
//     fn test_list_focus_state() {
//         let mut list = List::new();
//         list.add_item(Box::new(Link::player("Player 1", 1)));
//
//         assert!(!list.is_focused());
//
//         list.set_focused(true);
//         assert!(list.is_focused());
//     }
//
//     #[test]
//     fn test_list_widget_id_unique() {
//         let list1 = List::new();
//         let list2 = List::new();
//
//         assert_ne!(list1.widget_id(), list2.widget_id());
//     }
//
//     // Regression tests for boundary navigation issues
//     #[test]
//     fn test_list_up_at_first_item_blocks() {
//         // UP at first item should return NotHandled (blocked)
//         let mut list = List::new();
//         list.add_item(Box::new(Link::new("Item 1")));
//         list.add_item(Box::new(Link::new("Item 2")));
//         list.set_focused(true);
//         assert_eq!(list.selected_index, 0);
//
//         let result = list.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
//         assert_eq!(result, InputResult::NotHandled);
//         assert_eq!(list.selected_index, 0);
//     }
//
//     #[test]
//     fn test_list_down_at_last_item_blocks() {
//         // DOWN at last item should return NotHandled (blocked)
//         let mut list = List::new();
//         list.add_item(Box::new(Link::new("Item 1")));
//         list.add_item(Box::new(Link::new("Item 2")));
//         list.set_focused(true);
//         list.selected_index = 1; // Move to last item
//
//         let result = list.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
//         assert_eq!(result, InputResult::NotHandled);
//         assert_eq!(list.selected_index, 1);
//     }
//
//     #[test]
//     fn test_list_tab_behaves_like_down() {
//         // Tab should move down like arrow down
//         let mut list = List::new();
//         list.add_item(Box::new(Link::new("Item 1")));
//         list.add_item(Box::new(Link::new("Item 2")));
//         list.add_item(Box::new(Link::new("Item 3")));
//         list.set_focused(true);
//         assert_eq!(list.selected_index, 0);
//
//         // Tab should move down
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
//         assert_eq!(result, InputResult::Handled);
//         assert_eq!(list.selected_index, 1);
//
//         // Tab again
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
//         assert_eq!(result, InputResult::Handled);
//         assert_eq!(list.selected_index, 2);
//
//         // Tab at last item should block
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE));
//         assert_eq!(result, InputResult::NotHandled);
//         assert_eq!(list.selected_index, 2);
//     }
//
//     #[test]
//     fn test_list_shift_tab_behaves_like_up() {
//         // Shift+Tab should move up like arrow up
//         let mut list = List::new();
//         list.add_item(Box::new(Link::new("Item 1")));
//         list.add_item(Box::new(Link::new("Item 2")));
//         list.add_item(Box::new(Link::new("Item 3")));
//         list.set_focused(true);
//         list.selected_index = 2; // Start at last item
//
//         // Shift+Tab should move up
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
//         assert_eq!(result, InputResult::Handled);
//         assert_eq!(list.selected_index, 1);
//
//         // Shift+Tab again
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
//         assert_eq!(result, InputResult::Handled);
//         assert_eq!(list.selected_index, 0);
//
//         // Shift+Tab at first item should block
//         let result = list.handle_input(KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT));
//         assert_eq!(result, InputResult::NotHandled);
//         assert_eq!(list.selected_index, 0);
//     }
// }
