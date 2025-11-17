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

