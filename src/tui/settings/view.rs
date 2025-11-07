use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::{Line, Span},
    style::{Color, Style},
    Frame,
};
use super::{State, build_settings_list, SettingValue, KEY_VALUE_MARGIN};
use std::sync::Arc;
use crate::config::{Config, DisplayConfig};

/// 24 beautiful, professionally-chosen colors for the color picker (4x6 grid)
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

use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::widgets::Clear;

pub fn render_content(f: &mut Frame, area: Rect, state: &State, config: &Arc<Config>) {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));

    if state.subtab_focused {
        lines.push(Line::raw("  Use Up/Down to navigate, Enter to select, Up/Esc to exit"));
    } else {
        lines.push(Line::raw("  Press Down/Enter to edit settings"));
    }
    lines.push(Line::raw(""));

    // Build settings list from config
    let settings = build_settings_list(config);

    // Calculate the maximum key width for alignment
    let max_key_width = settings.iter()
        .map(|s| s.key.len())
        .max()
        .unwrap_or(0) + KEY_VALUE_MARGIN;

    // Render each setting
    for (idx, setting) in settings.iter().enumerate() {
        let is_selected = state.subtab_focused && state.selected_setting_index == idx;

        let mut line_spans = vec![Span::raw("  ")]; // Left margin

        // Show selection pointer
        if is_selected {
            line_spans.push(Span::styled("► ", Style::default().fg(config.display.selection_fg)));
        } else {
            line_spans.push(Span::raw("  "));
        }

        // Render key (left-aligned with padding)
        let padded_key = format!("{:<width$}", setting.key, width = max_key_width);
        line_spans.push(Span::raw(padded_key));

        // Render value based on type
        match &setting.value {
            SettingValue::Bool(value) => {
                if *value {
                    // Checked: brackets in default color, checkmark/X in selectionFG
                    let check_char = if config.display.use_unicode { "✔" } else { "X" };
                    line_spans.push(Span::raw("["));
                    line_spans.push(Span::styled(check_char, Style::default().fg(config.display.selection_fg)));
                    line_spans.push(Span::raw("]"));
                } else {
                    // Unchecked: all in default color
                    line_spans.push(Span::raw("[ ]"));
                }
            }
            SettingValue::Int(value) => {
                // Check if this setting is being edited
                if let Some((editing_name, edit_buffer)) = &state.editing {
                    if editing_name == &setting.key {
                        // Show edit buffer with cursor
                        line_spans.push(Span::raw(format!("{}█", edit_buffer)));
                    } else {
                        line_spans.push(Span::raw(value.to_string()));
                    }
                } else {
                    line_spans.push(Span::raw(value.to_string()));
                }
            }
            SettingValue::String(value) => {
                // Check if this setting is being edited
                if let Some((editing_name, edit_buffer)) = &state.editing {
                    if editing_name == &setting.key {
                        // Show edit buffer with cursor
                        line_spans.push(Span::raw(format!("{}█", edit_buffer)));
                    } else {
                        line_spans.push(Span::raw(value.clone()));
                    }
                } else {
                    line_spans.push(Span::raw(value.clone()));
                }
            }
            SettingValue::List { options, current_index } => {
                let current_value = options.get(*current_index).map(|s| s.as_str()).unwrap_or("?");
                line_spans.push(Span::raw(format!("▼ {}", current_value)));
            }
            SettingValue::Color(color) => {
                let block = "██████";
                line_spans.push(Span::styled(block, Style::default().fg(*color).bg(*color)));
            }
        }

        lines.push(Line::from(line_spans));
    }

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);

    // Render list modal if open
    if let Some((setting_name, options, selected_index)) = &state.list_modal {
        render_list_modal(f, area, setting_name, options, *selected_index, config);
    }

    // Render color picker modal if open
    if let Some(setting_name) = &state.color_modal {
        render_color_modal(f, area, setting_name, state.selected_color_index, config);
    }
}

/// Render a centered popup modal for list selection
fn render_list_modal(
    f: &mut Frame,
    area: Rect,
    setting_name: &str,
    options: &[String],
    selected_index: usize,
    config: &Arc<Config>,
) {
    // Calculate modal size
    let modal_height = options.len() as u16 + 4; // +4 for borders and title
    let modal_width = options.iter().map(|s| s.len()).max().unwrap_or(20).max(setting_name.len()) as u16 + 6;

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
    f.render_widget(Clear, modal_area);

    // Create modal content
    let mut modal_lines = Vec::new();
    modal_lines.push(Line::from(vec![
        Span::styled(format!(" {} ", setting_name), Style::default().fg(Color::White)),
    ]));
    modal_lines.push(Line::raw(""));

    for (idx, option) in options.iter().enumerate() {
        let is_selected = idx == selected_index;
        let line = if is_selected {
            Line::from(vec![
                Span::styled(" ► ", Style::default().fg(config.display.selection_fg)),
                Span::styled(option, Style::default().fg(Color::White)),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(option, Style::default().fg(Color::Gray)),
            ])
        };
        modal_lines.push(line);
    }

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.display.selection_fg));

    let modal_paragraph = Paragraph::new(modal_lines).block(modal_block);
    f.render_widget(modal_paragraph, modal_area);
}

/// Render a centered popup modal for color picker
fn render_color_modal(
    f: &mut Frame,
    area: Rect,
    setting_name: &str,
    selected_color_index: usize,
    config: &Arc<Config>,
) {
    // Calculate modal size (4x6 color grid + borders + title + instructions)
    let modal_height = 6 + 5; // 6 rows of colors + 5 for borders/title/instructions
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
    f.render_widget(Clear, modal_area);

    // Create modal content
    let mut modal_lines = Vec::new();
    modal_lines.push(Line::from(vec![
        Span::styled(format!(" {} ", setting_name), Style::default().fg(Color::White)),
    ]));
    modal_lines.push(Line::raw(""));

    // Render 4x6 color grid
    for row in 0..6 {
        let mut line_spans = vec![Span::raw(" ")]; // Left margin

        for col in 0..4 {
            let idx = row * 4 + col;
            let (color, name) = COLORS[idx];
            let is_selected = selected_color_index == idx;
            let is_current = if setting_name == "Selection FG" {
                color == config.display.selection_fg
            } else if setting_name == "Division Header FG" {
                color == config.display.division_header_fg
            } else if setting_name == "Error FG" {
                color == config.display.error_fg
            } else {
                false
            };

            // Show selection indicator
            if is_selected {
                line_spans.push(Span::styled("►", Style::default().fg(config.display.selection_fg)));
            } else if is_current {
                line_spans.push(Span::raw("●")); // Current theme color
            } else {
                line_spans.push(Span::raw(" "));
            }

            // Color block
            let block = "████";
            line_spans.push(Span::styled(block, Style::default().fg(color).bg(color)));
            line_spans.push(Span::raw(" "));

            // Color name
            let padded_name = format!("{:<13}", name);
            let name_style = if is_selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            line_spans.push(Span::styled(padded_name, name_style));
        }

        modal_lines.push(Line::from(line_spans));
    }

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.display.selection_fg));

    let modal_paragraph = Paragraph::new(modal_lines).block(modal_block);
    f.render_widget(modal_paragraph, modal_area);
}

// ============================================================================
// COLOR PICKER CODE (DEACTIVATED FOR PHASE 1 - WILL BE USED IN PHASE 2.5)
// ============================================================================
/*
pub fn render_content_color_picker(f: &mut Frame, area: Rect, state: &State, theme: &Arc<DisplayConfig>) {
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
*/
