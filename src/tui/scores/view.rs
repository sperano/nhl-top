use ratatui::{
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame,
};
use std::collections::HashMap;
use super::State;
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};

// Subtab Layout Constants
/// Left margin for subtab bar (spaces before date tabs) - REMOVED
const SUBTAB_LEFT_MARGIN: usize = 0;

pub fn render_subtabs(
    f: &mut Frame,
    area: Rect,
    state: &State,
    game_date: &nhl_api::GameDate,
    selection_fg: Color,
    unfocused_selection_fg: Color,
) {
    // Subtabs are only focused when subtab_focused is true AND box selection is not active
    let focused = state.subtab_focused && !state.box_selection_active;
    let selected_index = state.selected_index;

    let base_style = base_tab_style(focused);

    // Calculate the five dates to display based on game_date and selected_index
    // game_date is always the selected date
    // The 5 visible dates with current day in the middle (index 2)
    let (date0, date1, date2, date3, date4) = match selected_index {
        0 => (
            game_date.clone(),
            game_date.add_days(1),
            game_date.add_days(2),
            game_date.add_days(3),
            game_date.add_days(4),
        ),
        1 => (
            game_date.add_days(-1),
            game_date.clone(),
            game_date.add_days(1),
            game_date.add_days(2),
            game_date.add_days(3),
        ),
        2 => (
            game_date.add_days(-2),
            game_date.add_days(-1),
            game_date.clone(),
            game_date.add_days(1),
            game_date.add_days(2),
        ),
        3 => (
            game_date.add_days(-3),
            game_date.add_days(-2),
            game_date.add_days(-1),
            game_date.clone(),
            game_date.add_days(1),
        ),
        4 => (
            game_date.add_days(-4),
            game_date.add_days(-3),
            game_date.add_days(-2),
            game_date.add_days(-1),
            game_date.clone(),
        ),
        _ => (
            game_date.add_days(-2),
            game_date.add_days(-1),
            game_date.clone(),
            game_date.add_days(1),
            game_date.add_days(2),
        ), // fallback to middle
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

    let date0_str = format_date(&date0);
    let date1_str = format_date(&date1);
    let date2_str = format_date(&date2);
    let date3_str = format_date(&date3);
    let date4_str = format_date(&date4);

    // Build subtab line with separators (no left margin)
    let mut subtab_spans = Vec::new();

    // Date 0 (index 0)
    let date0_style = selection_style(
        base_style,
        selected_index == 0,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(date0_str.clone(), date0_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Date 1 (index 1)
    let date1_style = selection_style(
        base_style,
        selected_index == 1,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(date1_str.clone(), date1_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Date 2 (index 2) - CENTER (current day)
    let date2_style = selection_style(
        base_style,
        selected_index == 2,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(date2_str.clone(), date2_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Date 3 (index 3)
    let date3_style = selection_style(
        base_style,
        selected_index == 3,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(date3_str.clone(), date3_style));
    subtab_spans.push(Span::styled(" │ ", base_style));

    // Date 4 (index 4)
    let date4_style = selection_style(
        base_style,
        selected_index == 4,
        focused,
        selection_fg,
        unfocused_selection_fg,
    );
    subtab_spans.push(Span::styled(date4_str.clone(), date4_style));

    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors (no left margin)
    let tab_names = vec![date0_str, date1_str, date2_str, date3_str, date4_str].into_iter();
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
    boxscore: &Option<nhl_api::Boxscore>,
    boxscore_loading: bool,
) {
    // If boxscore view is active, render boxscore instead of game list
    if state.boxscore_view_active {
        render_boxscore_content(f, area, state, boxscore, boxscore_loading);
        return;
    }

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

        // Update viewport and content height
        state.grid_scrollable.update_viewport_height(area.height);
        state.grid_scrollable.update_content_height(content.lines().count());

        // Ensure selected box is visible
        ensure_box_visible(state, area.height);

        // If a box is selected, apply styling and render with scroll
        if let Some((sel_row, sel_col)) = selected_box {
            let styled_text = apply_box_styling_ratatui(&content, sel_row, sel_col, selection_fg);

            let paragraph = Paragraph::new(styled_text)
                .block(Block::default().borders(Borders::NONE))
                .scroll((state.grid_scrollable.scroll_offset, 0));
            f.render_widget(paragraph, area);
        } else {
            // No selection, render normally with scroll
            let paragraph = Paragraph::new(content)
                .block(Block::default().borders(Borders::NONE))
                .scroll((state.grid_scrollable.scroll_offset, 0));
            f.render_widget(paragraph, area);
        }
    } else {
        let paragraph = Paragraph::new("Loading scores...").block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
        state.grid_dimensions = (0, 0);
    }
}

// Constants for box layout
const LINES_PER_BOX: usize = 7;
const BLANK_LINE_BETWEEN_BOXES: usize = 1;
const LINES_PER_ROW: usize = LINES_PER_BOX + BLANK_LINE_BETWEEN_BOXES; // 8 lines total per row
const BOX_WIDTH: usize = 37;
const BOX_GAP: usize = 2;

/// Calculate the line range (start, end) for a box at the given row
fn calculate_box_line_range(sel_row: usize) -> (usize, usize) {
    let start_line = sel_row * LINES_PER_ROW;
    let end_line = start_line + LINES_PER_BOX; // 7 lines for the box
    (start_line, end_line)
}

/// Ensure the selected box is fully visible by adjusting scroll offset
fn ensure_box_visible(state: &mut State, viewport_height: u16) {
    if !state.box_selection_active {
        return;
    }

    let (sel_row, _) = state.selected_box;
    let (start_line, end_line) = calculate_box_line_range(sel_row);

    let scroll_offset = state.grid_scrollable.scroll_offset as usize;
    let viewport_end = scroll_offset + viewport_height as usize;

    // If box top is above viewport, scroll up to show it
    if start_line < scroll_offset {
        state.grid_scrollable.scroll_offset = start_line as u16;
    }
    // If box bottom is below viewport, scroll down to show it
    else if end_line > viewport_end {
        let new_offset = end_line.saturating_sub(viewport_height as usize);
        state.grid_scrollable.scroll_offset = new_offset as u16;
    }
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

/// Render boxscore content in place of game list (scrollable)
fn render_boxscore_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    boxscore: &Option<nhl_api::Boxscore>,
    loading: bool,
) {
    // Render the boxscore content
    let content_text = if loading {
        "Loading boxscore...".to_string()
    } else if let Some(ref bs) = boxscore {
        crate::commands::boxscore::format_boxscore(bs)
    } else {
        "No boxscore available".to_string()
    };

    state.boxscore_scrollable.render_paragraph(f, area, content_text, None);
}
