/// HorizontalSeparator widget - renders a horizontal separator line
///
/// This widget provides a consistent way to render separator lines in tables,
/// eliminating code duplication and ensuring visual consistency.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;

/// Renders a horizontal separator line using box-drawing characters
///
/// Returns 1 if rendered, 0 if no space available
pub fn render_horizontal_separator(
    width: usize,
    margin: u16,
    area: Rect,
    y: u16,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> u16 {
    if y >= area.bottom() {
        return 0;
    }

    let separator = format!(
        "{}{}",
        " ".repeat(margin as usize),
        config.box_chars.horizontal.repeat(width.saturating_sub(margin as usize))
    );

    buf.set_string(area.x, y, &separator, Style::default());
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_horizontal_separator_basic() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_horizontal_separator(50, 0, area, 2, &mut buf, &config);

        assert_eq!(lines, 1);

        // Check that separator appears
        let line = buffer_line(&buf, 2);
        assert!(line.contains(&config.box_chars.horizontal));
    }

    #[test]
    fn test_horizontal_separator_with_margin() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_horizontal_separator(50, 4, area, 2, &mut buf, &config);

        assert_eq!(lines, 1);

        // Check that line is indented
        let line = buffer_line(&buf, 2);
        assert!(line.starts_with("    ")); // 4 spaces
    }

    #[test]
    fn test_horizontal_separator_at_bottom() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        let area = Rect::new(0, 0, 80, 5);

        // Try to render at y=5 (at bottom)
        let lines = render_horizontal_separator(50, 0, area, 5, &mut buf, &config);

        // Should not render anything when at bottom
        assert_eq!(lines, 0);
    }

    #[test]
    fn test_horizontal_separator_width() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_horizontal_separator(20, 0, area, 2, &mut buf, &config);

        assert_eq!(lines, 1);

        // The separator should use box chars and be visible
        let line = buffer_line(&buf, 2);
        assert!(line.contains(&config.box_chars.horizontal));
    }

    #[test]
    fn test_horizontal_separator_width_with_margin() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        // Total width 30, margin 5 means 25 separator chars
        let lines = render_horizontal_separator(30, 5, area, 2, &mut buf, &config);

        assert_eq!(lines, 1);

        let line = buffer_line(&buf, 2);
        // Should start with 5 spaces
        assert!(line.starts_with("     "));
        // Should contain separator chars
        assert!(line.contains(&config.box_chars.horizontal));
    }
}
