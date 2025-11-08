/// Testing utilities for widget rendering
///
/// This module provides helper functions for testing widgets in isolation.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
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

/// Get a single cell from the buffer
///
/// This is a convenience wrapper around Buffer indexing that's easier to use in tests.
#[allow(dead_code)]
pub fn get_cell(buf: &Buffer, x: u16, y: u16) -> &ratatui::buffer::Cell {
    &buf[(x, y)]
}

/// Assert that a buffer line matches the expected string
///
/// This is a convenience macro-like function for common test patterns.
///
/// # Example
///
/// ```rust
/// let buf = render_widget(&widget, 40, 10);
/// assert_buffer_line(&buf, 0, "╭─────╮");
/// assert_buffer_line(&buf, 1, "│ Hi! │");
/// ```
#[allow(dead_code)]
pub fn assert_buffer_line(buf: &Buffer, line: u16, expected: &str) {
    let actual = buffer_line(buf, line);
    assert_eq!(
        actual, expected,
        "\nLine {} mismatch:\nExpected: {}\nActual:   {}",
        line, expected, actual
    );
}

/// Assert that a buffer matches expected multi-line string
///
/// Useful for snapshot-style testing.
///
/// # Example
///
/// ```rust
/// let buf = render_widget(&widget, 10, 3);
/// assert_buffer_eq(&buf, "\
/// ╭────────╮
/// │ Hello  │
/// ╰────────╯");
/// ```
#[allow(dead_code)]
pub fn assert_buffer_eq(buf: &Buffer, expected: &str) {
    let actual = buffer_to_string(buf);
    assert_eq!(
        actual, expected,
        "\nBuffer mismatch:\n\nExpected:\n{}\n\nActual:\n{}",
        expected, actual
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    /// Simple test widget for testing the testing utilities
    struct TestWidget {
        text: String,
    }

    impl RenderableWidget for TestWidget {
        fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
            buf.set_string(area.x, area.y, &self.text, Style::default());
        }
    }

    #[test]
    fn test_render_widget() {
        let widget = TestWidget {
            text: "Hello".to_string(),
        };

        let buf = render_widget(&widget, 10, 1);

        assert_eq!(buf[(0, 0)].symbol(), "H");
        assert_eq!(buf[(1, 0)].symbol(), "e");
        assert_eq!(buf[(2, 0)].symbol(), "l");
        assert_eq!(buf[(3, 0)].symbol(), "l");
        assert_eq!(buf[(4, 0)].symbol(), "o");
    }

    #[test]
    fn test_buffer_to_string() {
        let widget = TestWidget {
            text: "Hi".to_string(),
        };

        let buf = render_widget(&widget, 5, 2);
        let output = buffer_to_string(&buf);

        // Buffer is 5 wide, 2 tall, "Hi" at top-left
        assert_eq!(output, "Hi   \n     ");
    }

    #[test]
    fn test_buffer_line() {
        let widget = TestWidget {
            text: "Test".to_string(),
        };

        let buf = render_widget(&widget, 10, 1);
        let line = buffer_line(&buf, 0);

        assert_eq!(line, "Test      ");
    }

    #[test]
    fn test_config_creates_unicode() {
        let config = test_config();
        assert_eq!(config.box_chars.horizontal, "─");
        assert_eq!(config.box_chars.vertical, "│");
        assert_eq!(config.box_chars.top_left, "╭");
    }

    #[test]
    fn test_config_creates_ascii() {
        let config = test_config_ascii();
        assert_eq!(config.box_chars.horizontal, "-");
        assert_eq!(config.box_chars.vertical, "|");
        assert_eq!(config.box_chars.top_left, "+");
    }
}
