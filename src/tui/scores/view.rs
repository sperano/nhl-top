use ratatui::{
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use super::State;
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};

// Subtab Layout Constants
/// Left margin for subtab bar (spaces before date tabs)
const SUBTAB_LEFT_MARGIN: usize = 2;

pub fn render_subtabs(
    f: &mut Frame,
    area: Rect,
    state: &State,
    game_date: &nhl_api::GameDate,
    selection_fg: Color,
    unfocused_selection_fg: Color,
) {
    let focused = state.subtab_focused;
    let selected_index = state.selected_index;

    let base_style = base_tab_style(focused);

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
    subtab_spans.push(Span::styled(" ".repeat(SUBTAB_LEFT_MARGIN), base_style)); // Left margin

    // Left date (index 0)
    let yesterday_style = selection_style(
        base_style,
        selected_index == 0,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(yesterday_str.clone(), yesterday_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Center date (index 1)
    let today_style = selection_style(
        base_style,
        selected_index == 1,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(today_str.clone(), today_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Right date (index 2)
    let tomorrow_style = selection_style(
        base_style,
        selected_index == 2,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(tomorrow_str.clone(), tomorrow_style));

    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors (adjust width for left margin)
    let tab_names = vec![yesterday_str, today_str, tomorrow_str].into_iter();
    let separator_line = build_tab_separator_line(
        tab_names,
        area.width.saturating_sub(SUBTAB_LEFT_MARGIN as u16) as usize,
        base_style
    );

    // Add left margin to separator line
    let separator_with_margin = Line::from(vec![
        Span::styled(" ".repeat(SUBTAB_LEFT_MARGIN), base_style),
        Span::styled(separator_line.to_string(), base_style),
    ]);

    // Render subtabs with separator line
    let subtab_widget = Paragraph::new(vec![subtab_line, separator_with_margin])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(subtab_widget, area);
}

// Layout Constants - must match scores_format.rs
/// Terminal width threshold for 3-column layout
const THREE_COLUMN_WIDTH: u16 = 115;

/// Terminal width threshold for 2-column layout
const TWO_COLUMN_WIDTH: u16 = 76;

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    schedule: &Option<nhl_api::DailySchedule>,
    period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
    selection_fg: Color,
) {

    if let Some(schedule) = schedule {
        // Calculate grid dimensions
        let num_columns = if area.width >= THREE_COLUMN_WIDTH {
            3
        } else if area.width >= TWO_COLUMN_WIDTH {
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
            apply_box_styling_ratatui(&content, sel_row, sel_col, selection_fg)
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

// Constants for box layout
const LINES_PER_BOX: usize = 7;
const BLANK_LINE_BETWEEN_BOXES: usize = 1;
const BOX_WIDTH: usize = 37;
const BOX_GAP: usize = 2;

/// Calculate the line range (start, end) for a box at the given row
fn calculate_box_line_range(sel_row: usize) -> (usize, usize) {
    let lines_per_row = LINES_PER_BOX + BLANK_LINE_BETWEEN_BOXES; // 8 lines total per row
    let start_line = sel_row * lines_per_row;
    let end_line = start_line + LINES_PER_BOX; // 7 lines for the box
    (start_line, end_line)
}

/// Calculate the column range (start, end) for a box at the given column
fn calculate_box_column_range(sel_col: usize) -> (usize, usize) {
    let start_col = sel_col * (BOX_WIDTH + BOX_GAP);
    let end_col = start_col + BOX_WIDTH;
    (start_col, end_col)
}

/// Convert character positions to byte indices for UTF-8 safe string slicing
fn char_positions_to_byte_indices(line: &str, start_col: usize, end_col: usize) -> (usize, usize) {
    let char_indices: Vec<(usize, char)> = line.char_indices().collect();
    let char_count = char_indices.len();

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

    (byte_start, byte_end)
}

/// Create styled spans for a line, applying selection color to the specified byte range
fn create_styled_spans(line: &str, byte_start: usize, byte_end: usize, selection_fg: Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Before the box
    if byte_start > 0 {
        spans.push(Span::raw(line[..byte_start].to_string()));
    }

    // The box content with selection color
    if byte_start < byte_end {
        spans.push(Span::styled(
            line[byte_start..byte_end].to_string(),
            Style::default().fg(selection_fg)
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

    spans
}

/// Apply selection foreground color to selected box using ratatui's styling system
///
/// Each game box is 7 lines tall:
/// 1. Header line (e.g., "Final Score" or start time)
/// 2. Top border (╭─...╮)
/// 3. Header row (│ empty │ 1 │ 2 │ 3 │ T │)
/// 4. Middle border (├─┼─...┤)
/// 5. Away team row
/// 6. Home team row
/// 7. Bottom border (╰─...╯)
/// Plus 1 blank line between rows
fn apply_box_styling_ratatui(content: &str, sel_row: usize, sel_col: usize, selection_fg: Color) -> Text<'static> {
    let lines: Vec<&str> = content.lines().collect();
    let mut styled_lines: Vec<Line> = Vec::new();

    let (start_line, end_line) = calculate_box_line_range(sel_row);
    let (start_col, end_col) = calculate_box_column_range(sel_col);

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx >= start_line && line_idx < end_line {
            // This line is part of the selected box - apply selection styling
            let (byte_start, byte_end) = char_positions_to_byte_indices(line, start_col, end_col);
            let spans = create_styled_spans(line, byte_start, byte_end, selection_fg);
            styled_lines.push(Line::from(spans));
        } else {
            // Normal line without styling
            styled_lines.push(Line::raw(line.to_string()));
        }
    }

    Text::from(styled_lines)
}
