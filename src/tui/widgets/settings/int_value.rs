/// IntValueWidget - renders an integer value with optional edit cursor
///
/// Displays the number normally, or shows edit buffer with cursor when editing.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use unicode_width::UnicodeWidthStr;

/// Renders an integer value, optionally showing edit buffer with cursor
///
/// Returns the width consumed
pub fn render_int_value(
    value: u32,
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
    fn test_int_value_normal() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        let width = render_int_value(60, false, None, 0, 0, &mut buf);

        assert_eq!(width, 2); // "60"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "60");
    }

    #[test]
    fn test_int_value_large_number() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        let width = render_int_value(12345, false, None, 0, 0, &mut buf);

        assert_eq!(width, 5); // "12345"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "12345");
    }

    #[test]
    fn test_int_value_editing_with_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        let width = render_int_value(60, true, Some("12"), 0, 0, &mut buf);

        assert_eq!(width, 3); // "12█"
        let line = buffer_to_string(&buf, 0);
        assert!(line.starts_with("12"));
        assert!(line.contains("█"));
    }

    #[test]
    fn test_int_value_editing_empty_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        let width = render_int_value(60, true, Some(""), 0, 0, &mut buf);

        assert_eq!(width, 1); // "█"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "█");
    }

    #[test]
    fn test_int_value_editing_no_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        // When editing but no buffer provided, show original value with cursor
        let width = render_int_value(60, true, None, 0, 0, &mut buf);

        assert_eq!(width, 3); // "60█"
        let line = buffer_to_string(&buf, 0);
        assert!(line.starts_with("60"));
        assert!(line.contains("█"));
    }

    #[test]
    fn test_int_value_at_offset() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        render_int_value(42, false, None, 10, 0, &mut buf);

        let line = buffer_to_string(&buf, 0);
        // Should have 10 spaces, then "42"
        assert!(line.starts_with("          42"));
    }
}
