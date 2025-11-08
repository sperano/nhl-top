/// Buffer utilities for drawing borders, boxes, and lines
///
/// This module provides low-level drawing primitives for the TUI widget system.
/// All functions work with both ASCII and Unicode box characters.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};
use crate::formatting::BoxChars;

/// Draw a simple box border around an area
///
/// # Example
/// ```rust
/// draw_box(buf, area, &config.box_chars, Style::default());
/// // Draws: ╭───╮
/// //        │   │
/// //        ╰───╯
/// ```
pub fn draw_box(
    buf: &mut Buffer,
    area: Rect,
    box_chars: &BoxChars,
    style: Style,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let x = area.x;
    let y = area.y;
    let width = area.width;
    let height = area.height;

    // Draw corners
    buf.set_string(x, y, &box_chars.top_left, style);
    buf.set_string(x + width - 1, y, &box_chars.top_right, style);
    buf.set_string(x, y + height - 1, &box_chars.bottom_left, style);
    buf.set_string(x + width - 1, y + height - 1, &box_chars.bottom_right, style);

    // Draw horizontal borders (top and bottom)
    if width > 2 {
        for i in 1..width - 1 {
            buf.set_string(x + i, y, &box_chars.horizontal, style);
            buf.set_string(x + i, y + height - 1, &box_chars.horizontal, style);
        }
    }

    // Draw vertical borders (left and right)
    if height > 2 {
        for i in 1..height - 1 {
            buf.set_string(x, y + i, &box_chars.vertical, style);
            buf.set_string(x + width - 1, y + i, &box_chars.vertical, style);
        }
    }
}

/// Draw a box with a title
///
/// # Example
/// ```rust
/// draw_titled_box(buf, area, "Settings", &config.box_chars, Style::default());
/// // Draws: ╭─Settings─╮
/// //        │          │
/// //        ╰──────────╯
/// ```
pub fn draw_titled_box(
    buf: &mut Buffer,
    area: Rect,
    title: &str,
    box_chars: &BoxChars,
    style: Style,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let x = area.x;
    let y = area.y;
    let width = area.width;
    let height = area.height;

    // Draw corners
    buf.set_string(x, y, &box_chars.top_left, style);
    buf.set_string(x + width - 1, y, &box_chars.top_right, style);
    buf.set_string(x, y + height - 1, &box_chars.bottom_left, style);
    buf.set_string(x + width - 1, y + height - 1, &box_chars.bottom_right, style);

    // Calculate title position (left-aligned after corner and one horizontal char)
    let title_start_x = x + 1;
    let available_width = if width > 2 { width - 2 } else { 0 };

    // Draw title with one horizontal char before it
    if available_width > 0 && !title.is_empty() {
        buf.set_string(title_start_x, y, &box_chars.horizontal, style);

        let title_x = title_start_x + 1;
        let max_title_len = if available_width > 1 { available_width - 1 } else { 0 };

        if max_title_len > 0 {
            // Truncate title if needed
            let displayed_title = if title.len() > max_title_len as usize {
                &title[..max_title_len as usize]
            } else {
                title
            };

            buf.set_string(title_x, y, displayed_title, style);

            // Fill remaining space with horizontal lines
            let after_title_x = title_x + displayed_title.len() as u16;
            for i in after_title_x..x + width - 1 {
                buf.set_string(i, y, &box_chars.horizontal, style);
            }
        }
    } else if available_width > 0 {
        // No title, just draw horizontal line
        for i in 1..width - 1 {
            buf.set_string(x + i, y, &box_chars.horizontal, style);
        }
    }

    // Draw bottom horizontal border
    if width > 2 {
        for i in 1..width - 1 {
            buf.set_string(x + i, y + height - 1, &box_chars.horizontal, style);
        }
    }

    // Draw vertical borders (left and right)
    if height > 2 {
        for i in 1..height - 1 {
            buf.set_string(x, y + i, &box_chars.vertical, style);
            buf.set_string(x + width - 1, y + i, &box_chars.vertical, style);
        }
    }
}

/// Draw a horizontal line
///
/// # Example
/// ```rust
/// draw_horizontal_line(buf, 2, 10, 5, &box_chars, Style::default());
/// // Draws 5 characters starting at (2, 10): ─────
/// ```
pub fn draw_horizontal_line(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    box_chars: &BoxChars,
    style: Style,
) {
    for i in 0..width {
        buf.set_string(x + i, y, &box_chars.horizontal, style);
    }
}

/// Draw a vertical line
pub fn draw_vertical_line(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    height: u16,
    box_chars: &BoxChars,
    style: Style,
) {
    for i in 0..height {
        buf.set_string(x, y + i, &box_chars.vertical, style);
    }
}

/// Draw a double horizontal line (for emphasis)
pub fn draw_double_horizontal_line(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    box_chars: &BoxChars,
    style: Style,
) {
    for i in 0..width {
        buf.set_string(x + i, y, &box_chars.double_horizontal, style);
    }
}

/// Draw corner characters
pub fn draw_corner(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    corner_type: CornerType,
    box_chars: &BoxChars,
    style: Style,
) {
    let ch = match corner_type {
        CornerType::TopLeft => &box_chars.top_left,
        CornerType::TopRight => &box_chars.top_right,
        CornerType::BottomLeft => &box_chars.bottom_left,
        CornerType::BottomRight => &box_chars.bottom_right,
    };
    buf.set_string(x, y, ch, style);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CornerType {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Draw a junction (where lines meet)
pub fn draw_junction(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    junction_type: JunctionType,
    box_chars: &BoxChars,
    style: Style,
) {
    let ch = match junction_type {
        JunctionType::Top => &box_chars.top_junction,
        JunctionType::Bottom => &box_chars.bottom_junction,
        JunctionType::Left => &box_chars.left_junction,
        JunctionType::Right => &box_chars.right_junction,
        JunctionType::Cross => &box_chars.cross,
    };
    buf.set_string(x, y, ch, style);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JunctionType {
    Top,    // ┬
    Bottom, // ┴
    Left,   // ├
    Right,  // ┤
    Cross,  // ┼
}

#[cfg(test)]
mod border_tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_draw_box_unicode() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 3));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_box(&mut buf, area, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "╭───╮");
        assert_eq!(buffer_line(&buf, 1), "│   │");
        assert_eq!(buffer_line(&buf, 2), "╰───╯");
    }

    #[test]
    fn test_draw_box_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 3));
        let area = buf.area;
        let box_chars = BoxChars::ascii();
        draw_box(&mut buf, area, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "+---+");
        assert_eq!(buffer_line(&buf, 1), "|   |");
        assert_eq!(buffer_line(&buf, 2), "+---+");
    }

    #[test]
    fn test_draw_titled_box() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_titled_box(&mut buf, area, "Test", &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "╭─Test───╮");
        assert_eq!(buffer_line(&buf, 1), "│        │");
        assert_eq!(buffer_line(&buf, 2), "╰────────╯");
    }

    #[test]
    fn test_lines_and_junctions() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 5));
        let box_chars = BoxChars::unicode();

        // Draw a cross pattern
        draw_horizontal_line(&mut buf, 0, 2, 5, &box_chars, Style::default());
        draw_vertical_line(&mut buf, 2, 0, 5, &box_chars, Style::default());
        draw_junction(&mut buf, 2, 2, JunctionType::Cross, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 2), "──┼──");
        // Check vertical line exists at x=2 for all rows
        for y in 0..5 {
            let line = buffer_line(&buf, y);
            let char_at_x2 = &line[2..3];
            assert!(char_at_x2 == "│" || char_at_x2 == "┼");
        }
    }

    #[test]
    fn test_draw_box_minimum_size() {
        // Test 2x2 box (minimum size)
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_box(&mut buf, area, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "╭╮");
        assert_eq!(buffer_line(&buf, 1), "╰╯");
    }

    #[test]
    fn test_draw_box_too_small() {
        // Test box that's too small (should do nothing)
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_box(&mut buf, area, &box_chars, Style::default());

        // Buffer should remain empty/default
        assert_eq!(buffer_line(&buf, 0), " ");
    }

    #[test]
    fn test_draw_titled_box_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 3));
        let area = buf.area;
        let box_chars = BoxChars::ascii();
        draw_titled_box(&mut buf, area, "Title", &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "+-Title----+");
        assert_eq!(buffer_line(&buf, 1), "|          |");
        assert_eq!(buffer_line(&buf, 2), "+----------+");
    }

    #[test]
    fn test_draw_titled_box_long_title() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_titled_box(&mut buf, area, "VeryLongTitle", &box_chars, Style::default());

        // Title should be truncated to fit
        let top_line = buffer_line(&buf, 0);
        assert_eq!(top_line.chars().next().unwrap(), '╭');
        assert_eq!(top_line.chars().last().unwrap(), '╮');
    }

    #[test]
    fn test_draw_titled_box_empty_title() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 8, 3));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_titled_box(&mut buf, area, "", &box_chars, Style::default());

        // Should draw like a regular box
        assert_eq!(buffer_line(&buf, 0), "╭──────╮");
        assert_eq!(buffer_line(&buf, 1), "│      │");
        assert_eq!(buffer_line(&buf, 2), "╰──────╯");
    }

    #[test]
    fn test_horizontal_line() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 8, 1));
        let box_chars = BoxChars::unicode();
        draw_horizontal_line(&mut buf, 1, 0, 5, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), " ─────  ");
    }

    #[test]
    fn test_vertical_line() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 5));
        let box_chars = BoxChars::unicode();
        draw_vertical_line(&mut buf, 0, 0, 5, &box_chars, Style::default());

        for y in 0..5 {
            assert_eq!(buffer_line(&buf, y), "│");
        }
    }

    #[test]
    fn test_double_horizontal_line() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
        let box_chars = BoxChars::unicode();
        draw_double_horizontal_line(&mut buf, 0, 0, 6, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "══════");
    }

    #[test]
    fn test_double_horizontal_line_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
        let box_chars = BoxChars::ascii();
        draw_double_horizontal_line(&mut buf, 0, 0, 6, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "======");
    }

    #[test]
    fn test_corners_unicode() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
        let box_chars = BoxChars::unicode();

        draw_corner(&mut buf, 0, 0, CornerType::TopLeft, &box_chars, Style::default());
        draw_corner(&mut buf, 1, 0, CornerType::TopRight, &box_chars, Style::default());
        draw_corner(&mut buf, 0, 1, CornerType::BottomLeft, &box_chars, Style::default());
        draw_corner(&mut buf, 1, 1, CornerType::BottomRight, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "╭╮");
        assert_eq!(buffer_line(&buf, 1), "╰╯");
    }

    #[test]
    fn test_corners_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
        let box_chars = BoxChars::ascii();

        draw_corner(&mut buf, 0, 0, CornerType::TopLeft, &box_chars, Style::default());
        draw_corner(&mut buf, 1, 0, CornerType::TopRight, &box_chars, Style::default());
        draw_corner(&mut buf, 0, 1, CornerType::BottomLeft, &box_chars, Style::default());
        draw_corner(&mut buf, 1, 1, CornerType::BottomRight, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "++");
        assert_eq!(buffer_line(&buf, 1), "++");
    }

    #[test]
    fn test_junctions_unicode() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let box_chars = BoxChars::unicode();

        draw_junction(&mut buf, 0, 0, JunctionType::Top, &box_chars, Style::default());
        draw_junction(&mut buf, 1, 0, JunctionType::Bottom, &box_chars, Style::default());
        draw_junction(&mut buf, 2, 0, JunctionType::Left, &box_chars, Style::default());
        draw_junction(&mut buf, 3, 0, JunctionType::Right, &box_chars, Style::default());
        draw_junction(&mut buf, 4, 0, JunctionType::Cross, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "┬┴├┤┼");
    }

    #[test]
    fn test_junctions_ascii() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
        let box_chars = BoxChars::ascii();

        draw_junction(&mut buf, 0, 0, JunctionType::Top, &box_chars, Style::default());
        draw_junction(&mut buf, 1, 0, JunctionType::Bottom, &box_chars, Style::default());
        draw_junction(&mut buf, 2, 0, JunctionType::Left, &box_chars, Style::default());
        draw_junction(&mut buf, 3, 0, JunctionType::Right, &box_chars, Style::default());
        draw_junction(&mut buf, 4, 0, JunctionType::Cross, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "+++++");
    }

    #[test]
    fn test_draw_box_large() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));
        let area = buf.area;
        let box_chars = BoxChars::unicode();
        draw_box(&mut buf, area, &box_chars, Style::default());

        assert_eq!(buffer_line(&buf, 0), "╭────────╮");
        assert_eq!(buffer_line(&buf, 1), "│        │");
        assert_eq!(buffer_line(&buf, 2), "│        │");
        assert_eq!(buffer_line(&buf, 3), "│        │");
        assert_eq!(buffer_line(&buf, 4), "╰────────╯");
    }

    #[test]
    fn test_titled_box_with_partial_offset() {
        // Test with non-zero offset
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 4));
        let box_chars = BoxChars::unicode();
        let area = Rect::new(2, 1, 10, 3);
        draw_titled_box(&mut buf, area, "Test", &box_chars, Style::default());

        // Line 0 should be empty
        assert_eq!(buffer_line(&buf, 0), "               ");
        // Line 1 should have the box starting at x=2
        let line1 = buffer_line(&buf, 1);
        assert_eq!(&line1[2..12], "╭─Test───╮");
    }
}
