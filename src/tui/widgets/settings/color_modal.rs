/// ColorModalWidget - renders a centered popup modal for color selection
///
/// Features:
/// - 4x6 color grid layout
/// - Centered modal positioning
/// - Clear background behind modal
/// - Border with selection color
/// - Selection indicator (►) for current selection
/// - Current theme indicator (●) for currently-set color

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Widget},
};
use crate::config::DisplayConfig;

/// 24 professionally-chosen colors for the color picker (4x6 grid)
pub const COLORS: [(Color, &str); 24] = [
    // Row 1
    (Color::Rgb(226, 74, 74), "Deep Red"),
    (Color::Rgb(255, 107, 107), "Coral"),
    (Color::Rgb(255, 140, 66), "Burnt Orange"),
    (Color::Rgb(255, 200, 87), "Amber"),
    // Row 2
    (Color::Rgb(232, 185, 35), "Goldenrod"),
    (Color::Rgb(166, 166, 89), "Olive"),
    (Color::Rgb(140, 207, 77), "Chartreuse"),
    (Color::Rgb(88, 196, 114), "Green Apple"),
    // Row 3
    (Color::Rgb(46, 184, 114), "Emerald"),
    (Color::Rgb(42, 168, 118), "Teal"),
    (Color::Rgb(0, 184, 169), "Seafoam"),
    (Color::Rgb(77, 208, 225), "Cyan Sky"),
    // Row 4
    (Color::Rgb(33, 150, 243), "Azure"),
    (Color::Rgb(61, 90, 254), "Cobalt Blue"),
    (Color::Rgb(92, 107, 192), "Indigo"),
    (Color::Rgb(126, 87, 194), "Violet"),
    // Row 5
    (Color::Rgb(186, 104, 200), "Orchid"),
    (Color::Rgb(224, 86, 253), "Magenta"),
    (Color::Rgb(255, 119, 169), "Hot Pink"),
    (Color::Rgb(255, 158, 157), "Salmon"),
    // Row 6
    (Color::Rgb(234, 210, 172), "Beige"),
    (Color::Rgb(159, 168, 176), "Cool Gray"),
    (Color::Rgb(96, 125, 139), "Slate"),
    (Color::Rgb(55, 71, 79), "Charcoal"),
];

/// Renders a centered color picker modal with 4x6 grid
///
/// Returns the modal area that was rendered
pub fn render_color_modal(
    setting_name: &str,
    selected_color_index: usize,
    current_theme_color: Color,
    area: Rect,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> Rect {
    // Calculate modal size (4x6 color grid + borders + title + blank line)
    let modal_height = 6 + 4; // 6 rows + 2 borders + title + blank line
    let modal_width = 80; // Wide enough for 4 colors with names and margins

    // Center the modal
    let vertical_margin = (area.height.saturating_sub(modal_height)) / 2;
    let horizontal_margin = (area.width.saturating_sub(modal_width)) / 2;

    let modal_area = Rect {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Clear the area behind the modal
    Clear.render(modal_area, buf);

    // Render border
    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.selection_fg));
    border_block.render(modal_area, buf);

    // Calculate inner area (inside borders)
    let inner = Rect {
        x: modal_area.x + 1,
        y: modal_area.y + 1,
        width: modal_area.width.saturating_sub(2),
        height: modal_area.height.saturating_sub(2),
    };

    let mut y = inner.y;

    // Render title
    if y < inner.bottom() {
        let title = format!(" {} ", setting_name);
        buf.set_string(inner.x, y, &title, Style::default().fg(Color::White));
        y += 1;
    }

    // Blank line after title
    y += 1;

    // Render 4x6 color grid
    for row in 0..6 {
        if y >= inner.bottom() {
            break;
        }

        let mut x = inner.x + 1; // Left margin

        for col in 0..4 {
            let idx = row * 4 + col;
            let (color, name) = COLORS[idx];
            let is_selected = selected_color_index == idx;
            let is_current = color == current_theme_color;

            // Show indicator (selection or current)
            if is_selected {
                buf.set_string(x, y, "►", Style::default().fg(config.selection_fg));
            } else if is_current {
                buf.set_string(x, y, "●", Style::default());
            } else {
                buf.set_string(x, y, " ", Style::default());
            }
            x += 1;

            // Color block (4 characters)
            buf.set_string(x, y, "████", Style::default().fg(color).bg(color));
            x += 4;

            // Space after block
            buf.set_string(x, y, " ", Style::default());
            x += 1;

            // Color name (padded to 13 characters)
            let padded_name = format!("{:<13}", name);
            let name_style = if is_selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            buf.set_string(x, y, &padded_name, name_style);
            x += 13;
        }

        y += 1;
    }

    modal_area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::test_config;

    fn buffer_to_string(buf: &Buffer, y: u16, x_start: u16, x_end: u16) -> String {
        let mut result = String::new();
        for x in x_start..x_end {
            if let Some(cell) = buf.cell((x, y)) {
                result.push_str(cell.symbol());
            }
        }
        result.trim_end().to_string()
    }

    #[test]
    fn test_color_modal_basic_render() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Selection FG",
            0,
            Color::Red,
            area,
            &mut buf,
            &config,
        );

        // Modal should be centered
        assert!(modal_area.x > 0);
        assert!(modal_area.y > 0);
        assert_eq!(modal_area.width, 80);
        assert_eq!(modal_area.height, 10); // 6 rows + 2 borders + title + blank
    }

    #[test]
    fn test_color_modal_title() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Division Header FG",
            0,
            Color::Red,
            area,
            &mut buf,
            &config,
        );

        // Title should be visible
        let title_line = buffer_to_string(&buf, modal_area.y + 1, modal_area.x, modal_area.x + modal_area.width);
        assert!(title_line.contains("Division Header FG"));
    }

    #[test]
    fn test_color_modal_selection_first() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Test",
            0, // Select first color (top-left)
            Color::Green,
            area,
            &mut buf,
            &config,
        );

        // First row should have selection indicator (y+3: border, title, blank)
        let first_row = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(first_row.contains("►")); // Selection indicator
        assert!(first_row.contains("Deep Red")); // First color name
    }

    #[test]
    fn test_color_modal_selection_last() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Test",
            23, // Select last color (bottom-right)
            Color::Green,
            area,
            &mut buf,
            &config,
        );

        // Last row should have selection indicator (y+8: border, title, blank, 5 rows)
        let last_row = buffer_to_string(&buf, modal_area.y + 8, modal_area.x, modal_area.x + modal_area.width);
        assert!(last_row.contains("►")); // Selection indicator
        assert!(last_row.contains("Charcoal")); // Last color name
    }

    #[test]
    fn test_color_modal_current_theme_indicator() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        // Set current theme to the first color
        let (first_color, _) = COLORS[0];

        let modal_area = render_color_modal(
            "Test",
            5, // Select a different color
            first_color, // But current theme is first color
            area,
            &mut buf,
            &config,
        );

        // First row should have current theme indicator (●)
        let first_row = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(first_row.contains("●")); // Current theme indicator
    }

    #[test]
    fn test_color_modal_grid_layout() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Test",
            0,
            Color::Red,
            area,
            &mut buf,
            &config,
        );

        // Check that all 6 rows are rendered
        // Each row should contain color names
        for row in 0..6 {
            let y = modal_area.y + 3 + row; // +3 for border, title, blank
            let line = buffer_to_string(&buf, y, modal_area.x, modal_area.x + modal_area.width);

            // Each row should have 4 color names
            // We'll just check that the line is not empty
            assert!(!line.is_empty(), "Row {} should not be empty", row);
        }
    }

    #[test]
    fn test_color_modal_color_names() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let modal_area = render_color_modal(
            "Test",
            0,
            Color::Red,
            area,
            &mut buf,
            &config,
        );

        // First row should contain first 4 color names
        let first_row = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(first_row.contains("Deep Red"));
        assert!(first_row.contains("Coral"));
        assert!(first_row.contains("Burnt Orange"));
        assert!(first_row.contains("Amber"));
    }

    #[test]
    fn test_color_modal_centering() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
        let area = Rect::new(0, 0, 120, 40);

        let modal_area = render_color_modal(
            "Test",
            0,
            Color::Red,
            area,
            &mut buf,
            &config,
        );

        // Modal should be roughly centered
        let horizontal_margin = modal_area.x - area.x;
        let expected_horizontal_margin = (area.width - modal_area.width) / 2;
        assert_eq!(horizontal_margin, expected_horizontal_margin);

        let vertical_margin = modal_area.y - area.y;
        let expected_vertical_margin = (area.height - modal_area.height) / 2;
        assert_eq!(vertical_margin, expected_vertical_margin);
    }

    #[test]
    fn test_color_modal_selection_and_current_different() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        // Select index 5, current theme is index 0
        let (current_color, _) = COLORS[0];

        let modal_area = render_color_modal(
            "Test",
            5, // Selected
            current_color, // Current theme (index 0)
            area,
            &mut buf,
            &config,
        );

        // First row (row 0) should have current theme indicator (index 0 is row 0, col 0)
        let first_row = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(first_row.contains("●"));

        // Second row (row 1) should have selection indicator (index 5 is row 1, col 1)
        let second_row = buffer_to_string(&buf, modal_area.y + 4, modal_area.x, modal_area.x + modal_area.width);
        assert!(second_row.contains("►"));
    }
}
