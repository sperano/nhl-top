/// Widget-based rendering infrastructure for TUI
///
/// This module provides a trait-based architecture for composable, testable widgets.
/// Inspired by OO UI frameworks, widgets are small, focused components that can be
/// composed together to build complex interfaces.

#[cfg(test)]
pub mod testing;

// Small reusable widgets
pub mod list_modal;
pub use list_modal::{render_list_modal, ListModalWidget};

// Widget implementations
pub mod score_table;
pub use score_table::ScoreTable;

pub mod game_box;
pub use game_box::{GameBox, GameState};

pub mod settings_list;
pub use settings_list::SettingsListWidget;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
};
use crate::config::DisplayConfig;

/// Core trait for renderable widgets
///
/// Widgets render themselves directly to a ratatui Buffer, avoiding string-based
/// intermediate representations. This enables:
/// - Direct styling without character position calculations
/// - Composability (widgets can contain other widgets)
/// - Testability (can render to test buffers)
/// - Type safety (compiler catches layout errors)
///
/// # Object Safety
///
/// This trait is object-safe, meaning you can use trait objects to store
/// different widget types in collections.
pub trait RenderableWidget {
    /// Render this widget into the provided buffer
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render into
    /// * `buf` - The buffer to write to
    /// * `config` - Display configuration (colors, box chars, etc.)
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig);

    /// Get the preferred height of this widget
    ///
    /// Returns None if the widget can adapt to any height.
    /// Returns Some(height) if the widget has a fixed or preferred height.
    ///
    /// This is useful for layout calculations but is not enforced.
    fn preferred_height(&self) -> Option<u16> {
        None
    }

    /// Get the preferred width of this widget
    ///
    /// Returns None if the widget can adapt to any width.
    /// Returns Some(width) if the widget has a fixed or preferred width.
    ///
    /// This is useful for layout calculations but is not enforced.
    fn preferred_width(&self) -> Option<u16> {
        None
    }
}
