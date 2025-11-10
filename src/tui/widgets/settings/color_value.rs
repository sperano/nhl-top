/// ColorValueWidget - renders a color value as a colored block
///
/// Displays "██████" (6 block characters) in the specified color.

use ratatui::{buffer::Buffer, style::{Color, Style}};

/// Renders a color value as a colored block
///
/// Returns the width consumed (6 characters)
pub fn render_color_value(
    color: Color,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) -> u16 {
    let block = "██████";
    buf.set_string(x, y, block, Style::default().fg(color).bg(color));
    6 // Width consumed
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
    fn test_color_value_renders_block() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        let width = render_color_value(Color::Red, 0, 0, &mut buf);

        assert_eq!(width, 6);
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "██████");
    }

    #[test]
    fn test_color_value_different_colors() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));

        render_color_value(Color::Blue, 0, 0, &mut buf);
        let line = buffer_to_string(&buf, 0);
        assert_eq!(line, "██████");

        // Test with RGB color
        let mut buf2 = Buffer::empty(Rect::new(0, 0, 20, 1));
        render_color_value(Color::Rgb(255, 100, 50), 0, 0, &mut buf2);
        let line2 = buffer_to_string(&buf2, 0);
        assert_eq!(line2, "██████");
    }

    #[test]
    fn test_color_value_at_offset() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));

        render_color_value(Color::Green, 10, 0, &mut buf);

        let line = buffer_to_string(&buf, 0);
        // Should have 10 spaces, then the color block
        assert!(line.starts_with("          ██████"));
    }

    #[test]
    fn test_color_value_has_correct_style() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        let test_color = Color::Rgb(100, 150, 200);

        render_color_value(test_color, 0, 0, &mut buf);

        // Check that the first cell has the correct color applied
        if let Some(cell) = buf.cell((0, 0)) {
            assert_eq!(cell.style().fg, Some(test_color));
            assert_eq!(cell.style().bg, Some(test_color));
        } else {
            panic!("Cell not found");
        }
    }
}
