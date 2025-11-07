use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;
use ratatui::text::ToLine;
use crate::config::DisplayConfig;
use crate::formatting::format_header;
use super::State;
use super::state::DATE_WINDOW_SIZE;
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};

/// Calculate the date window based on game_date and selected_index
/// The window has a fixed base date (leftmost date) that only shifts when reaching edges
/// Relationship: window_base_date = game_date - selected_index
/// Window: [base, base+1, base+2, base+3, base+4]
fn calculate_date_window(game_date: &nhl_api::GameDate, selected_index: usize) -> [nhl_api::GameDate; DATE_WINDOW_SIZE] {
    // Calculate window base: the leftmost date in the window
    let window_base_date = game_date.add_days(-(selected_index as i64));

    // Window is always [base, base+1, base+2, base+3, base+4]
    [
        window_base_date.add_days(0),
        window_base_date.add_days(1),
        window_base_date.add_days(2),
        window_base_date.add_days(3),
        window_base_date.add_days(4),
    ]
}

/// Format a GameDate as MM/DD
fn format_date_mmdd(date: &nhl_api::GameDate) -> String {
    match date {
        nhl_api::GameDate::Date(naive_date) => naive_date.format("%m/%d").to_string(),
        nhl_api::GameDate::Now => chrono::Local::now().date_naive().format("%m/%d").to_string(),
    }
}

/// Build subtab spans for date navigation
fn build_date_subtab_spans(
    date_strings: &[String],
    selected_index: usize,
    base_style: Style,
    focused: bool,
    theme: &Arc<DisplayConfig>,
) -> Vec<Span<'static>> {
    let separator = format!(" {} ", theme.box_chars.vertical);
    let mut spans = Vec::new();

    for (i, date_str) in date_strings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(separator.clone(), base_style));
        }

        let style = selection_style(
            base_style,
            i == selected_index,
            focused,
            theme.selection_fg,
            theme.unfocused_selection_fg(),
        );
        spans.push(Span::styled(date_str.clone(), style));
    }

    spans
}

pub fn render_subtabs(
    f: &mut Frame,
    area: Rect,
    state: &State,
    game_date: &nhl_api::GameDate,
    theme: &Arc<DisplayConfig>,
) {
    let focused = state.subtab_focused && !state.box_selection_active;
    let base_style = base_tab_style(focused);

    let dates = calculate_date_window(game_date, state.selected_index);
    let date_strings: Vec<String> = dates.iter().map(format_date_mmdd).collect();

    let subtab_spans = build_date_subtab_spans(
        &date_strings,
        state.selected_index,
        base_style,
        focused,
        theme,
    );
    let subtab_line = Line::from(subtab_spans);

    let separator_line = build_tab_separator_line(
        date_strings.into_iter(),
        area.width as usize,
        base_style,
        &theme.box_chars,
    );

    let subtab_widget = Paragraph::new(vec![subtab_line, separator_line])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(subtab_widget, area);
}

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
    display: &Arc<DisplayConfig>,
    boxscore: &Option<nhl_api::Boxscore>,
    boxscore_loading: bool,
) {
    // If boxscore view is active, render boxscore instead of game list
    if state.boxscore_view_active {
        render_boxscore_content(f, area, state, boxscore, boxscore_loading, period_scores, game_info, display);
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
            Some(area.width as usize),
            &display.box_chars,
        );

        // Update viewport and content height
        state.grid_scrollable.update_viewport_height(area.height);
        state.grid_scrollable.update_content_height(content.lines().count());

        // Ensure selected box is visible
        ensure_box_visible(state, area.height);

        // If a box is selected, apply styling and render with scroll
        if let Some((sel_row, sel_col)) = selected_box {
            let styled_text = apply_box_styling_ratatui(&content, sel_row, sel_col, display);

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
fn create_styled_spans(line: &str, byte_start: usize, byte_end: usize, theme: &Arc<DisplayConfig>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Before the box
    if byte_start > 0 {
        spans.push(Span::raw(line[..byte_start].to_string()));
    }

    // The box content with selection color
    if byte_start < byte_end {
        spans.push(Span::styled(
            line[byte_start..byte_end].to_string(),
            Style::default().fg(theme.selection_fg)
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
fn apply_box_styling_ratatui(content: &str, sel_row: usize, sel_col: usize, theme: &Arc<DisplayConfig>) -> Text<'static> {
    let lines: Vec<&str> = content.lines().collect();
    let mut styled_lines: Vec<Line> = Vec::new();

    let (start_line, end_line) = calculate_box_line_range(sel_row);
    let (start_col, end_col) = calculate_box_column_range(sel_col);

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx >= start_line && line_idx < end_line {
            // This line is part of the selected box - apply selection styling
            let (byte_start, byte_end) = char_positions_to_byte_indices(line, start_col, end_col);
            let spans = create_styled_spans(line, byte_start, byte_end, theme);
            styled_lines.push(Line::from(spans));
        } else {
            // Normal line without styling
            styled_lines.push(Line::raw(line.to_string()));
        }
    }

    Text::from(styled_lines)
}

/// Combine two tables side by side with headers above each
fn combine_tables_with_headers(
    left_header: &str,
    left_table: &str,
    right_header: &str,
    right_table: &str,
) -> String {
    let mut output = String::new();

    // Split tables into lines
    let left_lines: Vec<&str> = left_table.lines().collect();
    let right_lines: Vec<&str> = right_table.lines().collect();

    // Add headers (assuming each table is 37 chars wide)
    output.push_str(left_header);
    output.push_str(&" ".repeat(37 - left_header.len() + 2)); // Padding + gap
    output.push_str(right_header);
    output.push('\n');

    // Combine tables line by line
    let max_lines = left_lines.len().max(right_lines.len());

    for i in 0..max_lines {
        // Get left line or pad with spaces
        if i < left_lines.len() {
            output.push_str(left_lines[i]);
        } else {
            output.push_str(&" ".repeat(37));
        }

        // Add gap between tables
        output.push_str("  ");

        // Get right line or pad with spaces
        if i < right_lines.len() {
            output.push_str(right_lines[i]);
        } else {
            output.push_str(&" ".repeat(37));
        }

        output.push('\n');
    }

    output
}

/// Format the scoring summary by period
fn format_scoring_summary(scoring: &[nhl_api::PeriodScoring], display: &DisplayConfig) -> String {
    if scoring.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    for period_scoring in scoring {
        if period_scoring.goals.is_empty() {
            continue;
        }

        let period_name = match period_scoring.period_descriptor.period_type.as_str() {
            "REG" => format!("{}st Period", period_scoring.period_descriptor.number)
                .replace("1st", "1st")
                .replace("2st", "2nd")
                .replace("3st", "3rd"),
            "OT" => "Overtime".to_string(),
            "SO" => "Shootout".to_string(),
            _ => format!("Period {}", period_scoring.period_descriptor.number),
        };

        output.push_str(&format_header(&period_name, false, display));
        output.push('\n');

        for goal in &period_scoring.goals {
            let scorer = format!("{} ({})",
                goal.name.default,
                goal.goals_to_date.unwrap_or(0)
            );

            let assists = if goal.assists.is_empty() {
                String::new()
            } else {
                let assist_names: Vec<String> = goal.assists.iter()
                    .map(|a| format!("{} ({})", a.name.default, a.assists_to_date))
                    .collect();
                format!("\n    {}", assist_names.join(", "))
            };

            let score_line = format!("{}-{} {}",
                goal.away_score,
                goal.home_score,
                goal.team_abbrev.default
            );

            let strength_modifier = match (goal.strength.as_str(), goal.goal_modifier.as_str()) {
                (s, m) if s != "EV" || m != "NONE" => {
                    let mut parts = Vec::new();
                    if s == "PP" { parts.push("PPG"); }
                    else if s == "SH" { parts.push("SHG"); }
                    if m == "empty-net" { parts.push("EN"); }
                    if !parts.is_empty() {
                        format!(" {}", parts.join(", "))
                    } else {
                        String::new()
                    }
                }
                _ => String::new()
            };

            output.push_str(&format!("{:<15} {} {:<10} {:<15} {}\n",
                score_line,
                scorer,
                strength_modifier,
                goal.time_in_period,
                goal.shot_type
            ));

            if !assists.is_empty() {
                output.push_str(&assists);
                output.push('\n');
            }
        }

        output.push('\n');
    }

    output
}

/// Format boxscore with period score box at the top
fn format_boxscore_with_period_box(
    boxscore: &nhl_api::Boxscore,
    period_scores: Option<&crate::commands::scores_format::PeriodScores>,
    game_info: Option<&nhl_api::GameMatchup>,
    display: &DisplayConfig,
) -> String {
    let mut output = String::new();

    // Display game header
    let header = format!("{} @ {}",
        boxscore.away_team.common_name.default,
        boxscore.home_team.common_name.default
    );
    output.push_str(&format!("\n{}", format_header(&header, true, display)));
    output.push_str(&format!("Date: {} | Venue: {}\n",
        boxscore.game_date,
        boxscore.venue.default
    ));
    output.push_str(&format!("Status: {} | Period: {}\n",
        boxscore.game_state,
        boxscore.period_descriptor.number
    ));
    if boxscore.clock.running || !boxscore.clock.in_intermission {
        output.push_str(&format!("Time: {}\n", boxscore.clock.time_remaining));
    }

    // Add period score and shots boxes side by side
    output.push_str("\n");

    // Determine if game has OT or SO
    let (has_ot, has_so, away_periods, home_periods) = if let Some(scores) = period_scores {
        (scores.has_ot, scores.has_so, Some(&scores.away_periods), Some(&scores.home_periods))
    } else {
        (false, false, None, None)
    };

    // Determine current period for in-progress games
    let current_period_num = if boxscore.game_state.has_started() && !boxscore.game_state.is_final() {
        game_info.and_then(|info| {
            match info.period_descriptor.period_type.as_str() {
                "REG" => Some(info.period_descriptor.number),
                "OT" => Some(4),
                "SO" => Some(5),
                _ => Some(info.period_descriptor.number),
            }
        })
    } else {
        None
    };

    // Build both tables
    let score_table = crate::commands::scores_format::build_score_table(
        &boxscore.away_team.abbrev,
        &boxscore.home_team.abbrev,
        Some(boxscore.away_team.score),
        Some(boxscore.home_team.score),
        has_ot,
        has_so,
        away_periods,
        home_periods,
        current_period_num,
        &display.box_chars,
    );

    let shots_table = crate::commands::scores_format::build_shots_table(
        &boxscore.away_team.abbrev,
        &boxscore.home_team.abbrev,
        Some(boxscore.away_team.sog),
        Some(boxscore.home_team.sog),
        has_ot,
        has_so,
        &display.box_chars,
    );

    // Combine tables side by side with headers
    let combined = combine_tables_with_headers(
        "Scores",
        &score_table,
        "Shots on goal",
        &shots_table,
    );

    output.push_str(&combined);

    if let Some(game_matchup) = game_info {
        if let Some(ref summary) = game_matchup.summary {
            output.push_str("\n");
            output.push_str(&format_scoring_summary(&summary.scoring, display));
        }
    }

    #[cfg(feature = "game_stats")]
    {
        let away_team_stats = nhl_api::TeamGameStats::from_team_player_stats(&boxscore.player_by_game_stats.away_team);
        let home_team_stats = nhl_api::TeamGameStats::from_team_player_stats(&boxscore.player_by_game_stats.home_team);
        let game_stats_table = crate::commands::boxscore::format_game_stats_table(
            &boxscore.away_team.abbrev,
            &boxscore.home_team.abbrev,
            &away_team_stats,
            &home_team_stats,
        );
        output.push_str(&game_stats_table);
    }

    // Display player stats using the existing helper functions from boxscore module
    crate::commands::boxscore::format_team_stats(&mut output, &boxscore.away_team.abbrev, &boxscore.player_by_game_stats.away_team, display);
    crate::commands::boxscore::format_team_stats(&mut output, &boxscore.home_team.abbrev, &boxscore.player_by_game_stats.home_team, display);

    output
}

/// Render boxscore content in place of game list (scrollable)
fn render_boxscore_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    boxscore: &Option<nhl_api::Boxscore>,
    loading: bool,
    period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
    display: &DisplayConfig,
) {
    // Render the boxscore content
    let content_text = if loading {
        "Loading boxscore...".to_string()
    } else if let Some(ref bs) = boxscore {
        format_boxscore_with_period_box(bs, period_scores.get(&bs.id), game_info.get(&bs.id), display)
    } else {
        "No boxscore available".to_string()
    };

    state.boxscore_scrollable.render_paragraph(f, area, content_text, None);
}
