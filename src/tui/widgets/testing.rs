/// Testing utilities for widget rendering
///
/// This module provides helper functions for testing widgets in isolation.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
};
use crate::config::DisplayConfig;
use crate::formatting::BoxChars;
use super::RenderableWidget;

/// Create a test DisplayConfig with unicode box characters
///
/// This provides consistent theming for tests.
pub fn test_config() -> DisplayConfig {
    DisplayConfig {
        use_unicode: true,
        selection_fg: Color::Rgb(255, 200, 0), // Gold
        unfocused_selection_fg: None,
        division_header_fg: Color::Rgb(159, 226, 191), // Seafoam
        error_fg: Color::Red,
        box_chars: BoxChars::unicode(),
    }
}

/// Create a test DisplayConfig with ASCII box characters
///
/// Useful for tests that want predictable ASCII-only output.
pub fn test_config_ascii() -> DisplayConfig {
    DisplayConfig {
        use_unicode: false,
        selection_fg: Color::Rgb(255, 200, 0),
        unfocused_selection_fg: None,
        division_header_fg: Color::Rgb(159, 226, 191),
        error_fg: Color::Red,
        box_chars: BoxChars::ascii(),
    }
}

/// Render a widget to a buffer and return it for testing
///
/// # Example
///
/// ```rust
/// let widget = MyWidget { text: "Hello" };
/// let buf = render_widget(&widget, 40, 10);
/// assert_eq!(get_cell(&buf, 0, 0).symbol(), "H");
/// ```
pub fn render_widget(
    widget: &impl RenderableWidget,
    width: u16,
    height: u16,
) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    let config = test_config();
    widget.render(buf.area, &mut buf, &config);
    buf
}

/// Render a widget to a buffer with a custom config
pub fn render_widget_with_config(
    widget: &impl RenderableWidget,
    width: u16,
    height: u16,
    config: &DisplayConfig,
) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    widget.render(buf.area, &mut buf, config);
    buf
}

/// Convert a buffer to a string representation for snapshot testing
///
/// Each line of the buffer is converted to a string, preserving spacing.
/// Useful for visual regression testing.
pub fn buffer_to_string(buf: &Buffer) -> String {
    let area = buf.area();
    let mut output = String::new();

    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            output.push_str(cell.symbol());
        }
        if y < area.height - 1 {
            output.push('\n');
        }
    }

    output
}

/// Get the text content of a specific line in the buffer
///
/// # Example
///
/// ```rust
/// let buf = render_widget(&widget, 80, 24);
/// assert_eq!(buffer_line(&buf, 0), "╭─────┬─────╮");
/// ```
pub fn buffer_line(buf: &Buffer, line: u16) -> String {
    let area = buf.area();
    let mut output = String::new();

    for x in 0..area.width {
        let cell = &buf[(x, line)];
        output.push_str(cell.symbol());
    }

    output
}

/// Get all lines from the buffer as a vector of strings
///
/// Each string is exactly the width of the buffer area.
///
/// # Example
///
/// ```rust
/// let buf = render_widget(&widget, 80, 3);
/// let lines = buffer_lines(&buf);
/// assert_eq!(lines, vec![
///     "Line 1 content...                                                               ",
///     "Line 2 content...                                                               ",
///     "Line 3 content...                                                               ",
/// ]);
/// ```
pub fn buffer_lines(buf: &Buffer) -> Vec<String> {
    let area = buf.area();
    let mut lines = Vec::new();

    for y in 0..area.height {
        lines.push(buffer_line(buf, y));
    }

    lines
}

