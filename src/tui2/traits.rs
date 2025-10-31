use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

/// Result of key handling by a view
pub enum KeyResult {
    /// The view consumed the key event
    Handled,
    /// The view didn't handle this key, pass to parent
    NotHandled,
    /// Request to drill down into a child view
    DrillDown(Box<dyn View>),
    /// Request to go back up one level
    GoBack,
    /// Request to quit the application
    Quit,
}

/// Core trait for all views in the hierarchical TUI
pub trait View {
    /// Render the view to the terminal
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool);

    /// Handle a key event
    /// Returns KeyResult indicating what action should be taken
    fn handle_key(&mut self, key: KeyEvent) -> KeyResult;

    /// Check if this view can drill down into a child view
    fn can_drill_down(&self) -> bool {
        false
    }

    /// Get the breadcrumb label for this view
    fn breadcrumb_label(&self) -> String {
        "Unknown".to_string()
    }
}
