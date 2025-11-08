/// SectionHeader widget - renders a formatted section header with box characters
///
/// This widget provides a consistent way to render section headers across all widgets,
/// eliminating code duplication and ensuring visual consistency.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::formatting::format_header;

/// Renders a section header with optional box-drawing characters
///
/// Returns the number of lines rendered (height consumed)
pub fn render_section_header(
    text: &str,
    double_line: bool,
    margin: u16,
    area: Rect,
    y: u16,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> u16 {
    if y >= area.bottom() {
        return 0;
    }

    let header_line = format_header(text, double_line, config);
    let mut lines_rendered = 0;

    for line in header_line.lines() {
        if y + lines_rendered >= area.bottom() {
            break;
        }
        if !line.is_empty() {
            let formatted = format!("{}{}", " ".repeat(margin as usize), line);
            buf.set_string(
                area.x,
                y + lines_rendered,
                &formatted,
                Style::default().fg(config.division_header_fg),
            );
        }
        lines_rendered += 1;
    }

    lines_rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_section_header_single_line() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_section_header("Test Header", false, 0, area, 0, &mut buf, &config);

        // Single-line header should render 2 lines
        assert_eq!(lines, 2);

        // Check that header text appears
        let line0 = buffer_line(&buf, 0);
        assert!(line0.contains("Test Header"));
    }

    #[test]
    fn test_section_header_double_line() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_section_header("Test Header", true, 0, area, 0, &mut buf, &config);

        // Double-line header should render 2 lines (text + separator)
        assert_eq!(lines, 2);

        // Check that header text appears on first line
        let line0 = buffer_line(&buf, 0);
        assert!(line0.contains("Test Header"));
    }

    #[test]
    fn test_section_header_with_margin() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let lines = render_section_header("Test Header", false, 4, area, 0, &mut buf, &config);

        assert_eq!(lines, 2);

        // Check that lines are indented with margin
        let line0 = buffer_line(&buf, 0);
        assert!(line0.starts_with("    ")); // 4 spaces
    }

    #[test]
    fn test_section_header_at_bottom() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        let area = Rect::new(0, 0, 80, 5);

        // Try to render at y=5 (at bottom)
        let lines = render_section_header("Test Header", false, 0, area, 5, &mut buf, &config);

        // Should not render anything when at bottom
        assert_eq!(lines, 0);
    }

    #[test]
    fn test_section_header_partial_render() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        let area = Rect::new(0, 0, 80, 5);

        // Try to render double-line header (2 lines) starting at y=4, only 1 line of space
        let lines = render_section_header("Test Header", true, 0, area, 4, &mut buf, &config);

        // Should only render 1 line (whatever fits)
        assert_eq!(lines, 1);
    }
}
