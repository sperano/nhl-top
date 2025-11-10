/// StringValueWidget - renders a string value with optional edit cursor
///
/// Displays the string normally, or shows edit buffer with cursor when editing.

use ratatui::{buffer::Buffer, style::Style};
use unicode_width::UnicodeWidthStr;

/// Renders a string value, optionally showing edit buffer with cursor
///
/// Returns the width consumed
pub fn render_string_value(
    value: &str,
    is_editing: bool,
    edit_buffer: Option<&str>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> u16 {
    let text = if is_editing {
        if let Some(buffer) = edit_buffer {
            format!("{}█", buffer)
        } else {
            format!("{}█", value)
        }
    } else {
        value.to_string()
    };

    buf.set_string(x, y, &text, Style::default());
    text.width() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer_to_string(buf: &Buffer, y: u16) -> String {
        let mut result = String::new();
        for x in 0..buf.area.width {
            if let Some(cell) = buf.cell((x, y)) {
                result.push_str(cell.symbol());
            }
        }
        result.trim_end().to_string()
    }

    #[test]
    fn test_string_value_normal() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        let width = render_string_value("info", false, None, 0, 0, &mut buf);

        assert_eq!(width, 4); // "info"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "info");
    }

    #[test]
    fn test_string_value_long_text() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));

        let text = "This is a longer string";
        let width = render_string_value(text, false, None, 0, 0, &mut buf);

        assert_eq!(width, text.len() as u16);
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, text);
    }

    #[test]
    fn test_string_value_editing_with_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        let width = render_string_value("original", true, Some("new"), 0, 0, &mut buf);

        assert_eq!(width, 4); // "new█"
        let line = buffer_to_string(&buf, 0);
        assert!(line.starts_with("new"));
        assert!(line.contains("█"));
    }

    #[test]
    fn test_string_value_editing_empty_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        let width = render_string_value("original", true, Some(""), 0, 0, &mut buf);

        assert_eq!(width, 1); // "█"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "█");
    }

    #[test]
    fn test_string_value_editing_no_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        // When editing but no buffer provided, show original value with cursor
        let width = render_string_value("test", true, None, 0, 0, &mut buf);

        assert_eq!(width, 5); // "test█"
        let line = buffer_to_string(&buf, 0);
        assert!(line.starts_with("test"));
        assert!(line.contains("█"));
    }

    #[test]
    fn test_string_value_empty_string() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        let width = render_string_value("", false, None, 0, 0, &mut buf);

        assert_eq!(width, 0);
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "");
    }

    #[test]
    fn test_string_value_at_offset() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));

        render_string_value("hello", false, None, 10, 0, &mut buf);

        let line = buffer_to_string(&buf, 0);
        // Should have 10 spaces, then "hello"
        assert!(line.starts_with("          hello"));
    }
}
