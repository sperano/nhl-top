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

/// Column widths structure for scoring summary table
struct ScoringColumnWidths {
    team: usize,      // Column 1: always 5 (space + 3-letter abbrev + space)
    description: usize, // Column 2: dynamic based on longest name/assists
    score: usize,     // Column 3: dynamic based on max score digits
    time: usize,      // Column 4: always 7 (space + MM:SS + space)
    shot_type: usize, // Column 5: dynamic based on longest shot type
}

impl ScoringColumnWidths {
    fn new(scoring: &[nhl_api::PeriodScoring]) -> Self {
        let mut max_desc_width = 0;
        let mut max_score_width = 0;
        let mut max_shot_type_width = 0;

        for period in scoring {
            for goal in &period.goals {
                let scorer = format!("{} ({})", goal.name.default, goal.goals_to_date.unwrap_or(0));
                max_desc_width = max_desc_width.max(scorer.len());

                let assists_str = if goal.assists.is_empty() {
                    "Unassisted".to_string()
                } else {
                    goal.assists.iter()
                        .map(|a| format!("{} ({})", a.name.default, a.assists_to_date))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                max_desc_width = max_desc_width.max(assists_str.len());

                // Track the actual formatted score string length
                let score_str = format!("{}-{}", goal.away_score, goal.home_score);
                max_score_width = max_score_width.max(score_str.len());

                max_shot_type_width = max_shot_type_width.max(goal.shot_type.len());
            }
        }

        Self {
            team: 5,
            description: max_desc_width + 6,
            score: max_score_width + 2,
            time: 7,
            shot_type: max_shot_type_width + 2,
        }
    }
}

/// Build a horizontal border for the scoring table
fn build_scoring_border(
    widths: &ScoringColumnWidths,
    left: &str,
    mid: &str,
    right: &str,
    horiz: &str,
) -> String {
    let mut line = String::new();
    line.push_str(left);
    line.push_str(&horiz.repeat(widths.team));
    line.push_str(mid);
    line.push_str(&horiz.repeat(widths.description));
    line.push_str(mid);
    line.push_str(&horiz.repeat(widths.score));
    line.push_str(mid);
    line.push_str(&horiz.repeat(widths.time));
    line.push_str(mid);
    line.push_str(&horiz.repeat(widths.shot_type));
    line.push_str(right);
    line.push('\n');
    line
}

/// Format the goal scorer row
fn format_goal_row(
    goal: &nhl_api::GoalSummary,
    widths: &ScoringColumnWidths,
    vert: &str,
) -> String {
    let scorer = format!("{} ({})", goal.name.default, goal.goals_to_date.unwrap_or(0));
    let score_str = format!("{}-{}", goal.away_score, goal.home_score);

    // Capitalize the first letter of the shot type
    let shot_type_capitalized = {
        let mut chars = goal.shot_type.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    format!(
        "{} {:3} {} {:<desc_w$} {} {:<score_w$} {} {:5} {} {:<shot_w$} {}\n",
        vert,
        goal.team_abbrev.default,
        vert,
        scorer,
        vert,
        score_str,
        vert,
        goal.time_in_period,
        vert,
        shot_type_capitalized,
        vert,
        desc_w = widths.description - 2,
        score_w = widths.score - 2,
        shot_w = widths.shot_type - 2,
    )
}

/// Format the assists row
fn format_assists_row(
    goal: &nhl_api::GoalSummary,
    widths: &ScoringColumnWidths,
    vert: &str,
) -> String {
    let assists_str = if goal.assists.is_empty() {
        "Unassisted".to_string()
    } else {
        goal.assists.iter()
            .map(|a| format!("{} ({})", a.name.default, a.assists_to_date))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "{} {:3} {} {:<desc_w$} {} {:<score_w$} {} {:5} {} {:shot_w$} {}\n",
        vert,
        "",
        vert,
        assists_str,
        vert,
        goal.team_abbrev.default,
        vert,
        "",
        vert,
        "",
        vert,
        desc_w = widths.description - 2,
        score_w = widths.score - 2,
        shot_w = widths.shot_type - 2,
    )
}

/// Format a single goal's data rows (without any borders)
fn format_goal_rows(
    goal: &nhl_api::GoalSummary,
    widths: &ScoringColumnWidths,
    bc: &crate::formatting::BoxChars,
) -> String {
    let mut output = String::new();
    output.push_str(&format_goal_row(goal, widths, &bc.vertical));
    output.push_str(&format_assists_row(goal, widths, &bc.vertical));
    output
}

/// Format the scoring summary by period
fn format_scoring_summary(scoring: &[nhl_api::PeriodScoring], display: &DisplayConfig) -> String {
    if scoring.is_empty() {
        return String::new();
    }

    let widths = ScoringColumnWidths::new(scoring);
    let mut output = String::new();

    for period in scoring {
        let period_name = match period.period_descriptor.period_type.as_str() {
            "REG" => format!("{}st Period", period.period_descriptor.number)
                .replace("1st", "1st")
                .replace("2st", "2nd")
                .replace("3st", "3rd"),
            "OT" => "Overtime".to_string(),
            "SO" => "Shootout".to_string(),
            _ => format!("Period {}", period.period_descriptor.number),
        };

        output.push_str(&period_name);
        output.push_str("\n\n");

        if period.goals.is_empty() {
            output.push_str("No Goals\n\n");
        } else {
            // Add top border once before all goals in this period
            output.push_str(&build_scoring_border(&widths, &display.box_chars.top_left, &display.box_chars.top_junction, &display.box_chars.top_right, &display.box_chars.horizontal));

            for (i, goal) in period.goals.iter().enumerate() {
                output.push_str(&format_goal_rows(goal, &widths, &display.box_chars));

                // Add separator after each goal
                if i < period.goals.len() - 1 {
                    // Middle separator between goals
                    output.push_str(&build_scoring_border(&widths, &display.box_chars.left_junction, &display.box_chars.cross, &display.box_chars.right_junction, &display.box_chars.horizontal));
                } else {
                    // Bottom border after last goal
                    output.push_str(&build_scoring_border(&widths, &display.box_chars.bottom_left, &display.box_chars.bottom_junction, &display.box_chars.bottom_right, &display.box_chars.horizontal));
                }
            }

            output.push('\n');
        }
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

/// Public function to format boxscore as text for exporting
pub fn format_boxscore_text(
    boxscore: &nhl_api::Boxscore,
    period_scores: Option<&crate::commands::scores_format::PeriodScores>,
    game_info: Option<&nhl_api::GameMatchup>,
    display: &DisplayConfig,
) -> String {
    format_boxscore_with_period_box(boxscore, period_scores, game_info, display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::{GoalSummary, AssistSummary, LocalizedString, PeriodDescriptor, PeriodScoring};

    fn create_localized_string(s: &str) -> LocalizedString {
        LocalizedString { default: s.to_string() }
    }

    fn create_test_goal(
        team: &str,
        scorer_name: &str,
        goals_to_date: i32,
        assists: Vec<(&str, i32)>,
        away_score: i32,
        home_score: i32,
        time: &str,
        shot_type: &str,
    ) -> GoalSummary {
        let assist_summaries = assists.into_iter().map(|(name, total)| {
            AssistSummary {
                player_id: 0,
                first_name: create_localized_string(""),
                last_name: create_localized_string(name),
                name: create_localized_string(name),
                assists_to_date: total,
                sweater_number: 0,
            }
        }).collect();

        GoalSummary {
            situation_code: "".to_string(),
            event_id: 0,
            strength: "EV".to_string(),
            player_id: 0,
            first_name: create_localized_string(""),
            last_name: create_localized_string(scorer_name),
            name: create_localized_string(scorer_name),
            team_abbrev: create_localized_string(team),
            headshot: "".to_string(),
            highlight_clip_sharing_url: None,
            highlight_clip: None,
            discrete_clip: None,
            goals_to_date: Some(goals_to_date),
            away_score,
            home_score,
            leading_team_abbrev: None,
            time_in_period: time.to_string(),
            shot_type: shot_type.to_string(),
            goal_modifier: "NONE".to_string(),
            assists: assist_summaries,
            home_team_defending_side: "".to_string(),
            is_home: false,
        }
    }

    // ===== ScoringColumnWidths Tests =====

    #[test]
    fn test_widths_scorer_longer_than_assists() {
        // Scorer name is longer than assists line
        let goal = create_test_goal(
            "TOR",
            "Alexander Ovechkin-Malkin III",
            50,
            vec![("A. B", 1)],
            1,
            0,
            "5:42",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "Alexander Ovechkin-Malkin III (50)" = 34 chars + 6 padding = 40
        assert_eq!(widths.description, 40);
        assert_eq!(widths.team, 5);
        assert_eq!(widths.time, 7);
    }

    #[test]
    fn test_widths_assists_longer_than_scorer() {
        // Assists line is longer than scorer name
        let goal = create_test_goal(
            "OTT",
            "M. Amadio",
            4,
            vec![("S. Pinto", 5), ("C. Giroux", 7), ("T. Stutzle", 12)],
            1,
            0,
            "5:42",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "S. Pinto (5), C. Giroux (7), T. Stutzle (12)" = 44 chars + 6 padding = 50
        assert_eq!(widths.description, 50);
    }

    #[test]
    fn test_widths_unassisted_longer_than_scorer() {
        // "Unassisted" is longer than short scorer name
        let goal = create_test_goal(
            "TOR",
            "A. B",
            1,
            vec![], // Unassisted
            1,
            0,
            "12:34",
            "Wrist",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "Unassisted" (10) is longer than "A. B (1)" (7), so 10 + 6 = 16
        assert_eq!(widths.description, 16);
    }

    #[test]
    fn test_widths_scorer_longer_than_unassisted() {
        // Scorer name is longer than "Unassisted"
        let goal = create_test_goal(
            "TOR",
            "A. Matthews",
            50,
            vec![], // Unassisted
            1,
            0,
            "12:34",
            "Wrist",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "A. Matthews (50)" (16) is longer than "Unassisted" (10), so 16 + 6 = 22
        assert_eq!(widths.description, 22);
    }

    #[test]
    fn test_widths_score_single_digits() {
        // Both scores are single digits: 1-1
        let goal = create_test_goal(
            "TOR",
            "Player",
            1,
            vec![],
            1,
            1,
            "01:00",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "1-1" format: 1 space + 1 digit + 1 dash + 1 digit + 1 space = 5
        assert_eq!(widths.score, 5);
    }

    #[test]
    fn test_widths_score_mixed_digits() {
        // One score is double digit: 10-1
        let goal = create_test_goal(
            "TOR",
            "Player",
            1,
            vec![],
            10,
            1,
            "01:00",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "10-1" format: 1 space + 2 digits + 1 dash + 1 digit + 1 space = 6
        assert_eq!(widths.score, 6);
    }

    #[test]
    fn test_widths_score_both_double_digits() {
        // Both scores are double digits: 10-10
        let goal = create_test_goal(
            "TOR",
            "Player",
            1,
            vec![],
            10,
            10,
            "01:00",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "10-10" format: 1 space + 2 digits + 1 dash + 2 digits + 1 space = 7
        assert_eq!(widths.score, 7);
    }

    #[test]
    fn test_widths_shot_type_variations() {
        // Test different shot type lengths
        let goal1 = create_test_goal("TOR", "P1", 1, vec![], 1, 0, "01:00", "Snap");
        let goal2 = create_test_goal("TOR", "P2", 1, vec![], 2, 0, "02:00", "Wrist");
        let goal3 = create_test_goal("TOR", "P3", 1, vec![], 3, 0, "03:00", "Backhand");
        let goal4 = create_test_goal("TOR", "P4", 1, vec![], 4, 0, "04:00", "Slap");

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal1, goal2, goal3, goal4],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // "Backhand" (8) is the longest, + 2 padding = 10
        assert_eq!(widths.shot_type, 10);
    }

    #[test]
    fn test_widths_multiple_goals_max_of_all() {
        // Test that widths are the maximum across all goals
        let goal1 = create_test_goal(
            "OTT",
            "Short Name",
            1,
            vec![("Long Assist Name", 99)],
            1,
            0,
            "5:42",
            "Snap",
        );

        let goal2 = create_test_goal(
            "BOS",
            "Very Long Scorer Name Here",
            100,
            vec![("A", 1)],
            10,
            10,
            "01:22",
            "Deflected",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal1, goal2],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // Description: max of "Very Long Scorer Name Here (100)" (32)
        assert_eq!(widths.description, 38); // 32 + 6 padding

        // Score: "10-10" needs 7
        assert_eq!(widths.score, 7);

        // Shot type: "Deflected" (9) + 2 padding = 11
        assert_eq!(widths.shot_type, 11);
    }

    #[test]
    fn test_widths_fixed_columns() {
        // Verify that team and time are always fixed
        let goal = create_test_goal(
            "TOR",
            "Player",
            1,
            vec![],
            1,
            1,
            "01:00",
            "Snap",
        );

        let period = PeriodScoring {
            period_descriptor: PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widths = ScoringColumnWidths::new(&[period]);

        // Team is always 5 (space + 3 chars + space)
        assert_eq!(widths.team, 5);

        // Time is always 7 (space + MM:SS + space)
        assert_eq!(widths.time, 7);
    }

    #[test]
    fn test_scoring_summary_single_goal() {
        let goal = create_test_goal(
            "OTT",
            "M. Amadio",
            4,
            vec![("S. Pinto", 5), ("C. Giroux", 7)],
            1,
            0,
            "5:42",
            "Snap",
        );

        let bc = crate::formatting::BoxChars::unicode();
        let widths = ScoringColumnWidths {
            team: 5,
            description: 33,
            score: 5,
            time: 7,
            shot_type: 6,
        };

        let mut result = String::new();
        // Add top border before the goal
        result.push_str(&build_scoring_border(&widths, &bc.top_left, &bc.top_junction, &bc.top_right, &bc.horizontal));
        result.push_str(&format_goal_rows(&goal, &widths, &bc));
        result.push_str(&build_scoring_border(&widths, &bc.bottom_left, &bc.bottom_junction, &bc.bottom_right, &bc.horizontal));

        let expected = "\
╭─────┬─────────────────────────────────┬─────┬───────┬──────╮
│ OTT │ M. Amadio (4)                   │ 1-0 │ 5:42  │ Snap │
│     │ S. Pinto (5), C. Giroux (7)     │ OTT │       │      │
╰─────┴─────────────────────────────────┴─────┴───────┴──────╯
";

        assert_eq!(result, expected, "\n\nExpected:\n{}\n\nGot:\n{}\n", expected, result);
    }

    #[test]
    fn test_scoring_summary_two_goals() {
        let goal1 = create_test_goal(
            "BOS",
            "M.Geekie",
            10,
            vec![("A. Peeke", 3)],
            9,
            1,
            "01:22",
            "Poke",
        );

        let goal2 = create_test_goal(
            "BOS",
            "S. Kuraly",
            2,
            vec![("T. Jeannot", 4), ("A. Peeke", 4)],
            10,
            1,
            "16:03",
            "Wrist",
        );

        let bc = crate::formatting::BoxChars::unicode();
        let widths = ScoringColumnWidths {
            team: 5,
            description: 33,
            score: 6, // "10-1" is 4 chars + 2 padding
            time: 7,
            shot_type: 7,
        };

        let mut result = String::new();
        // Add top border once before all goals
        result.push_str(&build_scoring_border(&widths, &bc.top_left, &bc.top_junction, &bc.top_right, &bc.horizontal));
        result.push_str(&format_goal_rows(&goal1, &widths, &bc));
        // Middle separator between goals
        result.push_str(&build_scoring_border(&widths, &bc.left_junction, &bc.cross, &bc.right_junction, &bc.horizontal));
        result.push_str(&format_goal_rows(&goal2, &widths, &bc));
        // Bottom border after last goal
        result.push_str(&build_scoring_border(&widths, &bc.bottom_left, &bc.bottom_junction, &bc.bottom_right, &bc.horizontal));

        let expected = "\
╭─────┬─────────────────────────────────┬──────┬───────┬───────╮
│ BOS │ M.Geekie (10)                   │ 9-1  │ 01:22 │ Poke  │
│     │ A. Peeke (3)                    │ BOS  │       │       │
├─────┼─────────────────────────────────┼──────┼───────┼───────┤
│ BOS │ S. Kuraly (2)                   │ 10-1 │ 16:03 │ Wrist │
│     │ T. Jeannot (4), A. Peeke (4)    │ BOS  │       │       │
╰─────┴─────────────────────────────────┴──────┴───────┴───────╯
";

        assert_eq!(result, expected, "\n\nExpected:\n{}\n\nGot:\n{}\n", expected, result);
    }

    #[test]
    fn test_scoring_summary_unassisted_goal() {
        let goal = create_test_goal(
            "MTL",
            "N. Suzuki",
            15,
            vec![], // Unassisted
            2,
            1,
            "10:15",
            "Wrist",
        );

        let bc = crate::formatting::BoxChars::unicode();
        let widths = ScoringColumnWidths {
            team: 5,
            description: 16,
            score: 5,
            time: 7,
            shot_type: 7,
        };

        let mut result = String::new();
        // Add top border before the goal
        result.push_str(&build_scoring_border(&widths, &bc.top_left, &bc.top_junction, &bc.top_right, &bc.horizontal));
        result.push_str(&format_goal_rows(&goal, &widths, &bc));
        result.push_str(&build_scoring_border(&widths, &bc.bottom_left, &bc.bottom_junction, &bc.bottom_right, &bc.horizontal));

        let expected = "\
╭─────┬────────────────┬─────┬───────┬───────╮
│ MTL │ N. Suzuki (15) │ 2-1 │ 10:15 │ Wrist │
│     │ Unassisted     │ MTL │       │       │
╰─────┴────────────────┴─────┴───────┴───────╯
";

        assert_eq!(result, expected, "\n\nExpected:\n{}\n\nGot:\n{}\n", expected, result);
    }
}
