use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Color},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use super::State;

/// Helper function to build a separator line with box-drawing connectors for tabs
fn build_tab_separator_line<'a, I>(tab_names: I, area_width: usize, style: Style) -> Line<'a>
where
    I: Iterator<Item = String>,
{
    let mut separator_spans = Vec::new();
    let mut pos = 0;

    for (i, tab_name) in tab_names.enumerate() {
        if i > 0 {
            separator_spans.push(Span::raw("─".repeat(1)));
            separator_spans.push(Span::raw("┴"));
            separator_spans.push(Span::raw("─".repeat(1)));
            pos += 3;
        }
        separator_spans.push(Span::raw("─".repeat(tab_name.len())));
        pos += tab_name.len();
    }

    if pos < area_width {
        separator_spans.push(Span::raw("─".repeat(area_width - pos)));
    }

    Line::from(separator_spans).style(style)
}

pub fn render_subtabs(
    f: &mut Frame,
    area: Rect,
    state: &State,
    game_date: &nhl_api::GameDate,
) {
    let focused = state.subtab_focused;
    let selected_index = state.selected_index;

    // Determine base style based on focus
    let base_style = if focused {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Calculate the three dates to display based on game_date and selected_index
    // game_date is always the selected date
    // The 3 visible dates depend on which position (0, 1, or 2) is selected
    let (left_date, center_date, right_date) = match selected_index {
        0 => (game_date.clone(), game_date.add_days(1), game_date.add_days(2)),
        1 => (game_date.add_days(-1), game_date.clone(), game_date.add_days(1)),
        2 => (game_date.add_days(-2), game_date.add_days(-1), game_date.clone()),
        _ => (game_date.add_days(-1), game_date.clone(), game_date.add_days(1)), // fallback
    };

    // Format dates as MM/DD
    let format_date = |date: &nhl_api::GameDate| -> String {
        match date {
            nhl_api::GameDate::Date(naive_date) => {
                naive_date.format("%m/%d").to_string()
            }
            nhl_api::GameDate::Now => {
                chrono::Local::now().date_naive().format("%m/%d").to_string()
            }
        }
    };

    let yesterday_str = format_date(&left_date);
    let today_str = format_date(&center_date);
    let tomorrow_str = format_date(&right_date);

    // Build subtab line with separators and left margin
    let mut subtab_spans = Vec::new();
    subtab_spans.push(Span::styled("  ", base_style)); // 2-space left margin

    // Left date (index 0)
    let yesterday_style = if selected_index == 0 {
        base_style.add_modifier(Modifier::REVERSED)
    } else {
        base_style
    };
    subtab_spans.push(Span::styled(yesterday_str.clone(), yesterday_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Center date (index 1)
    let today_style = if selected_index == 1 {
        base_style.add_modifier(Modifier::REVERSED)
    } else {
        base_style
    };
    subtab_spans.push(Span::styled(today_str.clone(), today_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Right date (index 2)
    let tomorrow_style = if selected_index == 2 {
        base_style.add_modifier(Modifier::REVERSED)
    } else {
        base_style
    };
    subtab_spans.push(Span::styled(tomorrow_str.clone(), tomorrow_style));

    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors
    let tab_names = vec![yesterday_str, today_str, tomorrow_str].into_iter();
    let separator_line = build_tab_separator_line(tab_names, area.width.saturating_sub(2) as usize, base_style);

    // Add left margin to separator line
    let separator_with_margin = Line::from(vec![
        Span::styled("  ", base_style),
        Span::styled(separator_line.to_string(), base_style),
    ]);

    // Render subtabs with separator line
    let subtab_widget = Paragraph::new(vec![subtab_line, separator_with_margin])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(subtab_widget, area);
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    schedule: &Option<nhl_api::DailySchedule>,
    period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
) {

    if let Some(schedule) = schedule {
        // Calculate grid dimensions
        let num_columns = if area.width >= 115 {
            3
        } else if area.width >= 76 {
            2
        } else {
            1
        };

        let total_games = schedule.games.len();
        if total_games == 0 {
            let paragraph = Paragraph::new("No games scheduled for today.")
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(paragraph, area);
            state.grid_dimensions = (0, 0);
            return;
        }

        let num_rows = (total_games + num_columns - 1) / num_columns;
        state.grid_dimensions = (num_rows, num_columns);

        // Get selected box for highlighting
        let selected_box = if state.box_selection_active {
            Some(state.selected_box)
        } else {
            None
        };

        // Render using existing formatter
        let content = crate::commands::scores_format::format_scores_for_tui_with_width(
            schedule,
            period_scores,
            game_info,
            Some(area.width as usize)
        );

        // Convert to styled Text if a box is selected
        let styled_text = if let Some((sel_row, sel_col)) = selected_box {
            apply_box_styling_ratatui(&content, sel_row, sel_col)
        } else {
            Text::raw(content)
        };

        let paragraph = Paragraph::new(styled_text).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Loading scores...").block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
        state.grid_dimensions = (0, 0);
    }
}

/// Apply teal foreground color to selected box using ratatui's styling system
fn apply_box_styling_ratatui(content: &str, sel_row: usize, sel_col: usize) -> Text<'static> {
    let lines: Vec<&str> = content.lines().collect();
    let mut styled_lines: Vec<Line> = Vec::new();

    // Each game box is 7 lines tall:
    // 1. Header line (e.g., "Final Score" or start time)
    // 2. Top border (╭─...╮)
    // 3. Header row (│ empty │ 1 │ 2 │ 3 │ T │)
    // 4. Middle border (├─┼─...┤)
    // 5. Away team row
    // 6. Home team row
    // 7. Bottom border (╰─...╯)
    // Plus 1 blank line between rows
    let lines_per_box = 7;
    let blank_line = 1;
    let lines_per_row = lines_per_box + blank_line; // 8 lines total per row

    let start_line = sel_row * lines_per_row;
    let end_line = start_line + lines_per_box; // 7 lines for the box

    // Each box is 37 chars wide + 2 spaces gap
    let box_width = 37;
    let gap = 2;
    let start_col = sel_col * (box_width + gap);
    let end_col = start_col + box_width;

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx >= start_line && line_idx < end_line {
            // This line is part of the selected box - apply cyan styling
            let mut spans = Vec::new();

            // Use char indices for UTF-8 safe slicing
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();
            let char_count = char_indices.len();

            // Find byte indices that correspond to character positions
            let byte_start = if start_col < char_count {
                char_indices[start_col].0
            } else {
                line.len()
            };

            let byte_end = if end_col < char_count {
                char_indices[end_col].0
            } else {
                line.len()
            };

            // Before the box
            if byte_start > 0 {
                spans.push(Span::raw(line[..byte_start].to_string()));
            }

            // The box content with cyan color
            if byte_start < byte_end {
                spans.push(Span::styled(
                    line[byte_start..byte_end].to_string(),
                    Style::default().fg(Color::Cyan)
                ));
            }

            // After the box
            if byte_end < line.len() {
                spans.push(Span::raw(line[byte_end..].to_string()));
            }

            // If line is too short to reach the box, just show the whole line
            if spans.is_empty() {
                spans.push(Span::raw(line.to_string()));
            }

            styled_lines.push(Line::from(spans));
        } else {
            // Normal line without styling
            styled_lines.push(Line::raw(line.to_string()));
        }
    }

    Text::from(styled_lines)
}
