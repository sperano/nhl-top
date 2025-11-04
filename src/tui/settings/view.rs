use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::{Line, Span},
    style::{Color, Style},
    Frame,
};
use super::State;
use std::sync::Arc;
use crate::config::DisplayConfig;

/// 24 beautiful, professionally-chosen colors for the color picker (4x6 grid)
pub const COLORS: [(Color, &str); 24] = [
    // Row 1 - Vibrant primary colors
    (Color::Rgb(255, 107, 107), "Coral Red"),      // Soft coral red
    (Color::Rgb(255, 165, 0), "Bright Orange"),    // Vibrant orange (like current selection)
    (Color::Rgb(255, 215, 0), "Golden Yellow"),    // Rich gold
    (Color::Rgb(144, 238, 144), "Mint Green"),     // Soft mint green

    // Row 2 - Cool colors
    (Color::Rgb(100, 200, 255), "Sky Blue"),       // Bright sky blue
    (Color::Rgb(138, 112, 229), "Claude Purple"),  // Purple like Claude Code
    (Color::Rgb(255, 105, 180), "Hot Pink"),       // Vibrant pink
    (Color::Rgb(64, 224, 208), "Turquoise"),       // Beautiful turquoise

    // Row 3 - Pastel/soft colors
    (Color::Rgb(255, 182, 193), "Light Pink"),     // Soft pink
    (Color::Rgb(176, 224, 230), "Powder Blue"),    // Powder blue
    (Color::Rgb(221, 160, 221), "Plum"),           // Soft plum
    (Color::Rgb(240, 230, 140), "Khaki"),          // Warm khaki

    // Row 4 - Deep/rich colors
    (Color::Rgb(72, 201, 176), "Teal"),            // Modern teal
    (Color::Rgb(156, 89, 182), "Amethyst"),        // Deep purple
    (Color::Rgb(230, 126, 34), "Pumpkin"),         // Rich pumpkin
    (Color::Rgb(52, 152, 219), "Ocean Blue"),      // Deep ocean blue

    // Row 5 - Earth tones & warm colors
    (Color::Rgb(210, 105, 30), "Chocolate"),       // Rich chocolate brown
    (Color::Rgb(188, 143, 143), "Rosy Brown"),     // Warm dusty rose
    (Color::Rgb(255, 140, 105), "Salmon"),         // Peachy salmon
    (Color::Rgb(189, 183, 107), "Olive"),          // Muted olive green

    // Row 6 - Jewel tones & saturated colors
    (Color::Rgb(220, 20, 60), "Crimson"),          // Deep crimson
    (Color::Rgb(0, 206, 209), "Cyan"),             // Pure cyan
    (Color::Rgb(75, 0, 130), "Indigo"),            // Deep indigo
    (Color::Rgb(50, 205, 50), "Lime Green"),       // Electric lime
];

pub fn render_content(f: &mut Frame, area: Rect, state: &State, theme: &Arc<DisplayConfig>) {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  Current Theme Color: "),
        Span::styled("████", Style::default().fg(theme.selection_fg).bg(theme.selection_fg)),
    ]));
    lines.push(Line::raw("  ───────────────"));
    lines.push(Line::raw(""));

    if state.subtab_focused {
        lines.push(Line::raw("  Use arrow keys to navigate, Enter to select, Up/Esc to exit"));
    } else {
        lines.push(Line::raw("  Press Down/Enter to activate color picker"));
    }
    lines.push(Line::raw(""));

    // Render 4x6 color grid
    for row in 0..6 {
        let mut line_spans = vec![Span::raw("  ")]; // Left margin

        for col in 0..4 {
            let idx = row * 4 + col;
            let (color, name) = COLORS[idx];
            let is_selected = state.subtab_focused && state.selected_color_index == idx;
            let is_current_theme = color == theme.selection_fg;

            // Show selection indicator
            if is_selected {
                line_spans.push(Span::raw("► "));
            } else if is_current_theme {
                line_spans.push(Span::raw("● ")); // Show dot for current theme color
            } else {
                line_spans.push(Span::raw("  "));
            }

            // Create a colored block: [████] ColorName
            // Block uses the actual color as background
            let block = "████";
            let block_style = Style::default().fg(color).bg(color);
            line_spans.push(Span::styled(block, block_style));
            line_spans.push(Span::raw(" "));

            // Color name in default white/gray text
            let padded_name = format!("{:<14}", name);
            let name_style = if is_selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            line_spans.push(Span::styled(padded_name, name_style));
        }

        lines.push(Line::from(line_spans));
    }

    lines.push(Line::raw(""));
    lines.push(Line::raw(""));

    // Show currently selected color
    let (selected_color, selected_name) = COLORS[state.selected_color_index];
    lines.push(Line::from(vec![
        Span::raw("  Current selection: "),
        Span::styled(selected_name, Style::default().fg(selected_color)),
    ]));

    // Show status message if present
    if let Some(msg) = &state.status_message {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(msg.clone(), Style::default().fg(Color::Green)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
