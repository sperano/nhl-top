/// Widget tree manager for hierarchical focus management
///
/// This module provides the WidgetTree struct which manages the widget hierarchy,
/// tracks focus state, and routes input events to the appropriate widgets.

use super::focus::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Manages the widget tree and focus state
///
/// The WidgetTree maintains the hierarchy of focusable widgets and provides
/// automatic focus management including:
/// - Tab/Shift+Tab navigation through all focusable widgets
/// - Arrow key navigation (delegated to widgets)
/// - Focus path tracking for breadcrumbs
/// - Input routing to focused widgets
///
/// # Example
///
/// ```rust
/// let mut tree = WidgetTree::new();
/// tree.set_root(Box::new(my_widget));
///
/// // Handle keyboard input
/// if tree.handle_input(key_event) {
///     // Input was handled
/// }
/// ```
pub struct WidgetTree {
    /// Root widget of the tree
    root: Option<Box<dyn Focusable>>,
    /// Currently focused widget ID
    focused_id: Option<WidgetId>,
    /// Focus path from root to focused widget
    focus_path: Vec<WidgetId>,
    /// Widget cache for quick lookup (widget ID -> cached data)
    widget_cache: HashMap<WidgetId, WidgetCacheEntry>,
    /// Pending navigation action from a widget
    pending_navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone)]
struct WidgetCacheEntry {
    /// Position in depth-first traversal
    dfs_index: usize,
    /// Can this widget receive focus?
    can_focus: bool,
}

impl WidgetTree {
    /// Create a new empty widget tree
    pub fn new() -> Self {
        Self {
            root: None,
            focused_id: None,
            focus_path: vec![],
            widget_cache: HashMap::new(),
            pending_navigation: None,
        }
    }

    /// Set the root widget
    ///
    /// This will rebuild the cache and focus the first focusable widget.
    pub fn set_root(&mut self, root: Box<dyn Focusable>) {
        self.root = Some(root);
        self.rebuild_cache();

        // Focus first focusable widget
        if self.focused_id.is_none() {
            self.focus_first();
        }
    }

    /// Get a reference to the root widget
    pub fn root(&self) -> Option<&dyn Focusable> {
        self.root.as_ref().map(|r| r.as_ref())
    }

    /// Get a mutable reference to the root widget
    pub fn root_mut(&mut self) -> Option<&mut (dyn Focusable + '_)> {
        match &mut self.root {
            Some(root) => Some(root.as_mut()),
            None => None,
        }
    }

    /// Get the currently focused widget ID
    pub fn focused_id(&self) -> Option<WidgetId> {
        self.focused_id
    }

    /// Get the focus path from root to focused widget
    pub fn focus_path(&self) -> &[WidgetId] {
        &self.focus_path
    }

    /// Take any pending navigation action
    ///
    /// This should be called after handling input to check if a widget
    /// requested navigation.
    pub fn take_navigation_action(&mut self) -> Option<NavigationAction> {
        self.pending_navigation.take()
    }

    /// Route input to the focused widget
    ///
    /// Returns true if the input was handled.
    pub fn handle_input(&mut self, event: KeyEvent) -> bool {
        if let Some(focused_id) = self.focused_id {
            if let Some(widget) = self.find_widget_mut(focused_id) {
                match widget.handle_input(event) {
                    InputResult::Handled => return true,
                    InputResult::NotHandled => {
                        // Try default navigation
                        return self.handle_default_navigation(event);
                    }
                    InputResult::MoveFocus(direction) => {
                        return self.move_focus(direction);
                    }
                    InputResult::Navigate(action) => {
                        self.pending_navigation = Some(action);
                        return true;
                    }
                }
            }
        }

        // No focused widget, try default navigation
        self.handle_default_navigation(event)
    }

    /// Move focus in the specified direction
    pub fn move_focus(&mut self, direction: FocusDirection) -> bool {
        match direction {
            FocusDirection::Next => self.focus_next(),
            FocusDirection::Previous => self.focus_previous(),
            FocusDirection::In => self.focus_into(),
            FocusDirection::Out => self.focus_out(),
            FocusDirection::Left => self.focus_left(),
            FocusDirection::Right => self.focus_right(),
        }
    }

    /// Focus the first focusable widget
    pub fn focus_first(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if let Some(&first_id) = focusable_widgets.first() {
            self.set_focus(first_id);
            true
        } else {
            false
        }
    }

    /// Focus the last focusable widget
    pub fn focus_last(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if let Some(&last_id) = focusable_widgets.last() {
            self.set_focus(last_id);
            true
        } else {
            false
        }
    }

    /// Focus the next focusable widget
    fn focus_next(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if focusable_widgets.is_empty() {
            return false;
        }

        let current_index = self.focused_id
            .and_then(|id| focusable_widgets.iter().position(|&w| w == id))
            .unwrap_or(0);

        let next_index = (current_index + 1) % focusable_widgets.len();
        self.set_focus(focusable_widgets[next_index]);
        true
    }

    /// Focus the previous focusable widget
    fn focus_previous(&mut self) -> bool {
        let focusable_widgets = self.collect_focusable_widgets();
        if focusable_widgets.is_empty() {
            return false;
        }

        let current_index = self.focused_id
            .and_then(|id| focusable_widgets.iter().position(|&w| w == id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            focusable_widgets.len() - 1
        } else {
            current_index - 1
        };
        self.set_focus(focusable_widgets[prev_index]);
        true
    }

    /// Focus into the current widget (if it's a container)
    fn focus_into(&mut self) -> bool {
        // For now, just delegate to the widget's input handler
        // In the future, this could automatically focus the first child
        false
    }

    /// Focus out of the current widget (return to parent)
    fn focus_out(&mut self) -> bool {
        // If we have a focus path, move to the parent
        if self.focus_path.len() > 1 {
            let parent_id = self.focus_path[self.focus_path.len() - 2];
            self.set_focus(parent_id);
            true
        } else {
            false
        }
    }

    /// Focus left (delegate to widget)
    fn focus_left(&mut self) -> bool {
        // Most widgets will handle this themselves
        // This is here as a fallback
        false
    }

    /// Focus right (delegate to widget)
    fn focus_right(&mut self) -> bool {
        // Most widgets will handle this themselves
        // This is here as a fallback
        false
    }

    /// Set focus to a specific widget
    fn set_focus(&mut self, widget_id: WidgetId) {
        // Clear old focus
        if let Some(old_id) = self.focused_id {
            if let Some(widget) = self.find_widget_mut(old_id) {
                widget.set_focused(false);
            }
        }

        // Set new focus
        if let Some(widget) = self.find_widget_mut(widget_id) {
            widget.set_focused(true);
            self.focused_id = Some(widget_id);
            self.update_focus_path(widget_id);
        }
    }

    /// Update the focus path for the given widget ID
    fn update_focus_path(&mut self, widget_id: WidgetId) {
        let mut new_path = Vec::new();

        // Build path from root to widget
        if let Some(root) = &self.root {
            new_path.push(root.widget_id());
            if root.widget_id() != widget_id {
                Self::find_path_to_widget_static(root.as_ref(), widget_id, &mut new_path);
            }
        }

        self.focus_path = new_path;
    }

    /// Recursively find path to widget (static version to avoid borrowing issues)
    fn find_path_to_widget_static(widget: &dyn Focusable, target_id: WidgetId, path: &mut Vec<WidgetId>) -> bool {
        for child_id in widget.focusable_children() {
            if child_id == target_id {
                path.push(child_id);
                return true;
            }

            if let Some(child) = widget.find_child(child_id) {
                path.push(child_id);
                if Self::find_path_to_widget_static(child, target_id, path) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }

    /// Collect all focusable widgets in tree order (depth-first)
    fn collect_focusable_widgets(&self) -> Vec<WidgetId> {
        let mut widgets = vec![];
        if let Some(root) = &self.root {
            Self::collect_focusable_recursive_static(root.as_ref(), &mut widgets);
        }
        widgets
    }

    fn collect_focusable_recursive_static(widget: &dyn Focusable, widgets: &mut Vec<WidgetId>) {
        if widget.can_focus() {
            widgets.push(widget.widget_id());
        }
        for child_id in widget.focusable_children() {
            if let Some(child) = widget.find_child(child_id) {
                Self::collect_focusable_recursive_static(child, widgets);
            }
        }
    }

    /// Find a widget by ID (immutable)
    fn find_widget(&self, id: WidgetId) -> Option<&dyn Focusable> {
        if let Some(root) = &self.root {
            if root.widget_id() == id {
                return Some(root.as_ref());
            }
            Self::find_widget_recursive_static(root.as_ref(), id)
        } else {
            None
        }
    }

    fn find_widget_recursive_static(widget: &dyn Focusable, id: WidgetId) -> Option<&dyn Focusable> {
        for child_id in widget.focusable_children() {
            if child_id == id {
                return widget.find_child(id);
            }
            if let Some(child) = widget.find_child(child_id) {
                if let Some(found) = Self::find_widget_recursive_static(child, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Find a widget by ID (mutable)
    ///
    /// Note: Due to Rust's borrowing rules, this uses an iterative approach
    /// that may not work well with deeply nested widget trees. For most use
    /// cases (1-2 levels deep), this is sufficient.
    fn find_widget_mut(&mut self, id: WidgetId) -> Option<&mut dyn Focusable> {
        // This is a simplified implementation that only searches direct children
        // A full recursive implementation would require unsafe code or RefCell
        if let Some(root) = &mut self.root {
            if root.widget_id() == id {
                return Some(root.as_mut());
            }

            // Try to find in direct children
            let child_ids: Vec<WidgetId> = root.focusable_children();
            for child_id in child_ids {
                if child_id == id {
                    return root.find_child_mut(id);
                }
            }
        }
        None
    }

    /// Rebuild the widget cache
    fn rebuild_cache(&mut self) {
        self.widget_cache.clear();

        if let Some(root) = &self.root {
            let mut dfs_index = 0;
            Self::cache_widget_recursive_static(root.as_ref(), &mut self.widget_cache, &mut dfs_index);
        }
    }

    fn cache_widget_recursive_static(
        widget: &dyn Focusable,
        cache: &mut HashMap<WidgetId, WidgetCacheEntry>,
        dfs_index: &mut usize,
    ) {
        let id = widget.widget_id();
        let can_focus = widget.can_focus();

        cache.insert(id, WidgetCacheEntry {
            dfs_index: *dfs_index,
            can_focus,
        });

        *dfs_index += 1;

        for child_id in widget.focusable_children() {
            if let Some(child) = widget.find_child(child_id) {
                Self::cache_widget_recursive_static(child, cache, dfs_index);
            }
        }
    }
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Default keyboard navigation mappings
impl WidgetTree {
    fn handle_default_navigation(&mut self, event: KeyEvent) -> bool {
        match (event.code, event.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => self.move_focus(FocusDirection::Next),
            (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                self.move_focus(FocusDirection::Previous)
            }
            (KeyCode::Down, KeyModifiers::NONE) => self.move_focus(FocusDirection::Next),
            (KeyCode::Up, KeyModifiers::NONE) => self.move_focus(FocusDirection::Previous),
            (KeyCode::Left, KeyModifiers::NONE) => self.move_focus(FocusDirection::Left),
            (KeyCode::Right, KeyModifiers::NONE) => self.move_focus(FocusDirection::Right),
            (KeyCode::Enter, KeyModifiers::NONE) => self.move_focus(FocusDirection::In),
            (KeyCode::Esc, KeyModifiers::NONE) => self.move_focus(FocusDirection::Out),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::link::Link;
    use super::super::list::List;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_widget_tree_creation() {
        let tree = WidgetTree::new();
        assert!(tree.root().is_none());
        assert!(tree.focused_id().is_none());
        assert_eq!(tree.focus_path().len(), 0);
    }

    #[test]
    fn test_widget_tree_set_root() {
        let mut tree = WidgetTree::new();
        let link = Link::player("Test Player", 123);
        tree.set_root(Box::new(link));

        assert!(tree.root().is_some());
    }

    #[test]
    fn test_widget_tree_auto_focus_first() {
        let mut tree = WidgetTree::new();
        let link = Link::player("Test Player", 123);
        let link_id = link.widget_id();

        tree.set_root(Box::new(link));

        // Should automatically focus the first focusable widget
        assert_eq!(tree.focused_id(), Some(link_id));
    }

    #[test]
    fn test_widget_tree_handle_tab() {
        let mut tree = WidgetTree::new();

        let mut list = List::new();
        list.add_item(Box::new(Link::player("Player 1", 1)));
        list.add_item(Box::new(Link::player("Player 2", 2)));

        tree.set_root(Box::new(list));

        // Press Tab
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let handled = tree.handle_input(tab);

        assert!(handled);
        assert!(tree.focused_id().is_some());
    }

    #[test]
    fn test_widget_tree_navigation_action() {
        let mut tree = WidgetTree::new();
        let link = Link::player("Test Player", 123);

        tree.set_root(Box::new(link));

        // Press Enter to activate
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let handled = tree.handle_input(enter);

        assert!(handled);

        // Check for navigation action
        let action = tree.take_navigation_action();
        assert!(action.is_some());

        match action.unwrap() {
            NavigationAction::NavigateToPlayer(id) => {
                assert_eq!(id, 123);
            }
            _ => panic!("Expected NavigateToPlayer"),
        }

        // Taking again should return None
        assert!(tree.take_navigation_action().is_none());
    }

    #[test]
    fn test_widget_tree_default() {
        let tree = WidgetTree::default();
        assert!(tree.root().is_none());
    }
}
