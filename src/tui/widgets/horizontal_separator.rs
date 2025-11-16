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
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_horizontal_separator_basic() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, RENDER_WIDTH, 1));
        let area = Rect::new(0, 0, RENDER_WIDTH, 1);

        let lines = render_horizontal_separator(50, 0, area, 0, &mut buf, &config);

        assert_eq!(lines, 1);

        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────",
        ]);
    }

    #[test]
    fn test_horizontal_separator_with_margin() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, RENDER_WIDTH, 1));
        let area = Rect::new(0, 0, RENDER_WIDTH, 1);

        let lines = render_horizontal_separator(50, 4, area, 0, &mut buf, &config);

        assert_eq!(lines, 1);

        assert_buffer(&buf, &[
            "    ──────────────────────────────────────────────",
        ]);
    }

    #[test]
    fn test_horizontal_separator_at_bottom() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, RENDER_WIDTH, 5));
        let area = Rect::new(0, 0, RENDER_WIDTH, 5);

        // Try to render at y=5 (at bottom)
        let lines = render_horizontal_separator(50, 0, area, 5, &mut buf, &config);

        // Should not render anything when at bottom
        assert_eq!(lines, 0);
    }

    #[test]
    fn test_horizontal_separator_width() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, RENDER_WIDTH, 1));
        let area = Rect::new(0, 0, RENDER_WIDTH, 1);

        let lines = render_horizontal_separator(20, 0, area, 0, &mut buf, &config);

        assert_eq!(lines, 1);

        assert_buffer(&buf, &[
            "────────────────────",
        ]);
    }

    #[test]
    fn test_horizontal_separator_width_with_margin() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, RENDER_WIDTH, 1));
        let area = Rect::new(0, 0, RENDER_WIDTH, 1);

        // Total width 30, margin 5 means 25 separator chars
        let lines = render_horizontal_separator(30, 5, area, 0, &mut buf, &config);

        assert_eq!(lines, 1);

        assert_buffer(&buf, &[
            "     ─────────────────────────",
        ]);
    }
}
