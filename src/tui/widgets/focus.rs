/// Focus management system for interactive widgets
///
/// This module provides the core infrastructure for hierarchical focus management
/// in the TUI. It enables keyboard navigation through widget trees, automatic focus
/// delegation, and navigation actions.

use super::RenderableWidget;
use crossterm::event::KeyEvent;
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique identifier for widgets in the tree
///
/// Widget IDs are automatically generated and guaranteed to be unique within
/// the application lifetime. They are used to track focus state and navigate
/// the widget hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub usize);

impl WidgetId {
    /// Generate a new unique widget ID
    ///
    /// Uses an atomic counter to ensure uniqueness across threads.
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        WidgetId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of handling an input event
///
/// This enum describes how a widget processed an input event and what
/// action should be taken by the parent/focus manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputResult {
    /// Input was handled by the widget, stop propagation
    Handled,
    /// Input was not handled, continue propagation to parent
    NotHandled,
    /// Request focus to move in a direction
    MoveFocus(FocusDirection),
    /// Request navigation to a new panel/page
    Navigate(NavigationAction),
}

/// Focus movement direction
///
/// Describes the direction in which focus should move within the widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Tab or Down - move to next focusable widget
    Next,
    /// Shift+Tab or Up - move to previous focusable widget
    Previous,
    /// Left arrow - move focus left (in 2D layouts)
    Left,
    /// Right arrow - move focus right (in 2D layouts)
    Right,
    /// Enter - focus into a container widget
    In,
    /// Esc - focus out of a container widget
    Out,
}

/// Navigation action to perform
///
/// These actions represent high-level navigation events that are handled
/// by the application's navigation system, not the widget focus system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    /// Pop the current panel from the navigation stack
    PopPanel,
    /// Navigate to a specific team by abbreviation
    NavigateToTeam(String),
    /// Navigate to a specific player by ID
    NavigateToPlayer(i64),
    /// Navigate to a specific game by ID
    NavigateToGame(i64),
}

/// Trait for widgets that can receive focus
///
/// Widgets implementing this trait can participate in keyboard navigation,
/// receive input events, and be part of the focus hierarchy.
///
/// # Focus Hierarchy
///
/// Widgets form a tree structure where:
/// - Container widgets (List, Table) have focusable children
/// - Leaf widgets (Link, Button) have no children
/// - Focus can be delegated from parent to child
pub trait Focusable: RenderableWidget {
    /// Get the unique ID of this widget
    fn widget_id(&self) -> WidgetId;

    /// Returns true if this widget can receive focus
    ///
    /// Widgets that are disabled, hidden, or otherwise non-interactive
    /// should return false.
    fn can_focus(&self) -> bool;

    /// Returns true if this widget currently has focus
    fn is_focused(&self) -> bool;

    /// Set whether this widget has focus
    ///
    /// When focus changes, the widget should update its visual state
    /// and potentially notify its children.
    fn set_focused(&mut self, focused: bool);

    /// Handle keyboard input when focused
    ///
    /// Returns an InputResult indicating how the event was processed.
    /// Container widgets should delegate to their focused child first,
    /// then handle their own navigation if the child doesn't handle it.
    fn handle_input(&mut self, event: KeyEvent) -> InputResult;

    /// Get focusable children for navigation
    ///
    /// Container widgets should return the IDs of their focusable children.
    /// Leaf widgets return an empty vector.
    fn focusable_children(&self) -> Vec<WidgetId> {
        vec![]
    }

    /// Find a child widget by ID
    ///
    /// Used for focus navigation and delegation. Container widgets should
    /// search their children and return a reference if found.
    fn find_child(&self, _id: WidgetId) -> Option<&dyn Focusable> {
        None
    }

    /// Find a mutable child widget by ID
    ///
    /// Used for focus management and input routing. Container widgets should
    /// search their children and return a mutable reference if found.
    fn find_child_mut(&mut self, _id: WidgetId) -> Option<&mut dyn Focusable> {
        None
    }

    /// Focus this widget and reset selection to the first item
    ///
    /// Used by containers when navigating forward into this widget.
    /// Default implementation just calls `set_focused(true)`.
    /// Widgets like List and Table can override to reset their selection position.
    fn focus_first(&mut self) {
        self.set_focused(true);
    }

    /// Focus this widget and reset selection to the last item
    ///
    /// Used by containers when navigating backward into this widget.
    /// Default implementation just calls `set_focused(true)`.
    /// Widgets like List and Table can override to reset their selection position.
    fn focus_last(&mut self) {
        self.set_focused(true);
    }

    /// Get the current selection index (for widgets that have selection)
    ///
    /// Returns None for widgets that don't support selection, or Some(index)
    /// for widgets like List and Table that maintain a selected item.
    /// This is read-only - used for testing and debugging.
    fn selected_index(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id_uniqueness() {
        let id1 = WidgetId::new();
        let id2 = WidgetId::new();
        let id3 = WidgetId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_widget_id_default() {
        let id1 = WidgetId::default();
        let id2 = WidgetId::default();

        // Default should create new unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_input_result_equality() {
        assert_eq!(InputResult::Handled, InputResult::Handled);
        assert_eq!(InputResult::NotHandled, InputResult::NotHandled);
        assert_eq!(
            InputResult::MoveFocus(FocusDirection::Next),
            InputResult::MoveFocus(FocusDirection::Next)
        );
        assert_ne!(InputResult::Handled, InputResult::NotHandled);
    }

    #[test]
    fn test_navigation_action_equality() {
        assert_eq!(
            NavigationAction::NavigateToTeam("MTL".to_string()),
            NavigationAction::NavigateToTeam("MTL".to_string())
        );
        assert_ne!(
            NavigationAction::NavigateToTeam("MTL".to_string()),
            NavigationAction::NavigateToTeam("TOR".to_string())
        );
    }
}
