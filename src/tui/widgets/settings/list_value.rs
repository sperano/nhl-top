/// ListValueWidget - renders a list/dropdown value
///
/// Displays "▼ current_option" format.

use ratatui::{buffer::Buffer, style::Style};
use unicode_width::UnicodeWidthStr;

/// Renders a list value showing the current selection
///
/// Returns the width consumed
pub fn render_list_value(
    options: &[String],
    current_index: usize,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> u16 {
    let current_value = options.get(current_index).map(|s| s.as_str()).unwrap_or("?");
    let text = format!("▼ {}", current_value);

    buf.set_string(x, y, &text, Style::default());
    text.width() as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

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
    fn test_list_value_first_option() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];

        let width = render_list_value(&options, 0, 0, 0, &mut buf);

        assert_eq!(width, 10); // "▼ Option 1" (▼=1, space=1, Option 1=8)
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "▼ Option 1");
    }

    #[test]
    fn test_list_value_second_option() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let options = vec!["trace".to_string(), "debug".to_string(), "info".to_string()];

        let width = render_list_value(&options, 2, 0, 0, &mut buf);

        assert_eq!(width, 6); // "▼ info" (▼=1, space=1, info=4)
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "▼ info");
    }

    #[test]
    fn test_list_value_invalid_index() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let options = vec!["Option 1".to_string()];

        let width = render_list_value(&options, 5, 0, 0, &mut buf);

        assert_eq!(width, 3); // "▼ ?"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "▼ ?");
    }

    #[test]
    fn test_list_value_empty_options() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let options: Vec<String> = vec![];

        let width = render_list_value(&options, 0, 0, 0, &mut buf);

        assert_eq!(width, 3); // "▼ ?"
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "▼ ?");
    }

    #[test]
    fn test_list_value_long_option() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let options = vec!["A very long option name here".to_string()];

        let width = render_list_value(&options, 0, 0, 0, &mut buf);

        assert_eq!(width, 30); // "▼ A very long option name here" (▼=1, space=1, text=28)
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "▼ A very long option name here");
    }

    #[test]
    fn test_list_value_at_offset() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        let options = vec!["test".to_string()];

        render_list_value(&options, 0, 10, 0, &mut buf);

        let line = buffer_to_string(&buf, 0);
        // Should have 10 spaces, then "▼ test"
        assert!(line.starts_with("          ▼ test"));
    }
}
