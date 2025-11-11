/// Widget-based rendering infrastructure for TUI
///
/// This module provides a trait-based architecture for composable, testable widgets.
/// Inspired by OO UI frameworks, widgets are small, focused components that can be
/// composed together to build complex interfaces.

#[cfg(test)]
pub mod testing;

// Focus management system
pub mod focus;
pub mod tree;
pub mod container;
pub mod link;
pub mod list;
pub mod table;
pub mod breadcrumb_focusable;

pub use container::{Container, FocusPosition};
pub use link::{Link, LinkBuilder, LinkStyle};
pub use list::{List, ListStyle};
pub use table::{FocusableTable, ColumnDef, Alignment, TableStyle, HighlightMode};
pub use breadcrumb_focusable::{BreadcrumbWidget, BreadcrumbSegment, BreadcrumbStyle};

// Small reusable widgets
pub mod section_header;
pub mod horizontal_separator;
pub mod settings;

// Widget implementations
pub mod scoring_table;
pub use scoring_table::ScoringTable;

pub mod score_table;
pub use score_table::ScoreTable;

pub mod game_box;
pub use game_box::{GameBox, GameState};

pub mod game_grid;
pub use game_grid::GameGrid;

pub mod standings_table;
pub use standings_table::StandingsTable;

pub mod player_stats_table;
pub use player_stats_table::PlayerStatsTable;

pub mod goalie_stats_table;
pub use goalie_stats_table::GoalieStatsTable;

pub mod career_stats_table;
pub use career_stats_table::CareerStatsTable;

pub mod player_bio_card;
pub use player_bio_card::PlayerBioCard;

pub mod game_skater_stats_table;
pub use game_skater_stats_table::GameSkaterStatsTable;

pub mod game_goalie_stats_table;
pub use game_goalie_stats_table::GameGoalieStatsTable;

pub mod team_stats_panel;
pub use team_stats_panel::TeamStatsPanel;

pub mod action_bar;
pub use action_bar::{ActionBar, Action};

pub mod enhanced_breadcrumb;
pub use enhanced_breadcrumb::EnhancedBreadcrumb;

pub mod command_palette;
pub use command_palette::{CommandPalette, SearchResult};

pub mod tab_bar;
pub use tab_bar::TabBar;
// Note: Tab from tab_bar is not exported at the module level to keep it scoped

pub mod status_bar;
pub use status_bar::{StatusBar, KeyHint, KeyHintStyle};

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
/// This trait is object-safe, meaning you can use trait objects:
/// ```rust
/// let widgets: Vec<Box<dyn RenderableWidget>> = vec![
///     Box::new(GameBox { ... }),
///     Box::new(ScoreTable { ... }),
/// ];
///
/// for widget in widgets {
///     widget.render(area, buf, config);
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use ratatui::{buffer::Buffer, layout::Rect, style::Style};
/// use crate::config::DisplayConfig;
/// use crate::tui::widgets::RenderableWidget;
///
/// struct MyWidget {
///     text: String,
/// }
///
/// impl RenderableWidget for MyWidget {
///     fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
///         buf.set_string(area.x, area.y, &self.text, Style::default());
///     }
/// }
/// ```
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
