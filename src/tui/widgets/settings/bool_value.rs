/// BoolValueWidget - renders a boolean value as a checkbox
///
/// Displays checked ([✔]) or unchecked ([ ]) based on value.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};

/// Renders a boolean value as a checkbox
///
/// Returns the width consumed (3 characters: "[X]" or "[ ]")
pub fn render_bool_value(
    value: bool,
    use_unicode: bool,
    selection_fg: ratatui::style::Color,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> u16 {
    if value {
        // Checked: brackets in default, checkmark/X in selection color
        let check_char = if use_unicode { "✔" } else { "X" };
        buf.set_string(x, y, "[", Style::default());
        buf.set_string(x + 1, y, check_char, Style::default().fg(selection_fg));
        buf.set_string(x + 2, y, "]", Style::default());
    } else {
        // Unchecked: all in default color
        buf.set_string(x, y, "[ ]", Style::default());
    }
    3 // Width consumed
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

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
    fn test_bool_value_checked_unicode() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        let width = render_bool_value(true, true, Color::Green, 0, 0, &mut buf);

        assert_eq!(width, 3);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("["));
        assert!(line.contains("✔"));
        assert!(line.contains("]"));
    }

    #[test]
    fn test_bool_value_checked_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        let width = render_bool_value(true, false, Color::Green, 0, 0, &mut buf);

        assert_eq!(width, 3);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("[X]"));
    }

    #[test]
    fn test_bool_value_unchecked() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        let width = render_bool_value(false, true, Color::Green, 0, 0, &mut buf);

        assert_eq!(width, 3);
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "[ ]");
    }

    #[test]
    fn test_bool_value_at_offset() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        render_bool_value(true, true, Color::Green, 5, 0, &mut buf);

        let line = buffer_to_string(&buf, 0);
        // Should have 5 spaces, then the checkbox
        assert!(line.starts_with("     "));
        assert!(line.contains("✔"));
    }
}
