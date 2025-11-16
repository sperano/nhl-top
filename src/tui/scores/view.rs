use super::State;
use crate::config::DisplayConfig;
use crate::tui::widgets::{GameBox, GameGrid, GameState, RenderableWidget};
use crate::commands::scores_format::PeriodScores;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Format a GameDate as MM/DD
fn format_date_mmdd(date: &nhl_api::GameDate) -> String {
    match date {
        nhl_api::GameDate::Date(naive_date) => naive_date.format("%m/%d").to_string(),
        nhl_api::GameDate::Now => chrono::Local::now().date_naive().format("%m/%d").to_string(),
    }
}

/// Format a GameDate as "Mon DD, YYYY" for breadcrumb
fn format_date_full(date: &nhl_api::GameDate) -> String {
    match date {
        nhl_api::GameDate::Date(naive_date) => naive_date.format("%b %d, %Y").to_string(),
        nhl_api::GameDate::Now => chrono::Local::now().date_naive().format("%b %d, %Y").to_string(),
    }
}

/// Convert schedule data to GameBox widgets
fn create_game_boxes(
    schedule: &nhl_api::DailySchedule,
    period_scores: &HashMap<i64, PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
    selected_game_index: Option<usize>,
) -> Vec<GameBox> {
    schedule.games.iter().enumerate().map(|(index, game)| {
        // Determine game state
        let state = if game.game_state.is_final() {
            GameState::Final
        } else if game.game_state.has_started() {
            // Get period text and time from game_info
            if let Some(info) = game_info.get(&game.id) {
                let period_text = crate::commands::scores_format::format_period_text(
                    &info.period_descriptor.period_type,
                    info.period_descriptor.number
                );

                let (time_remaining, in_intermission) = if let Some(clock) = &info.clock {
                    (Some(clock.time_remaining.clone()), clock.in_intermission)
                } else {
                    (None, false)
                };

                GameState::Live {
                    period_text,
                    time_remaining,
                    in_intermission,
                }
            } else {
                // Fallback for live game without info
                GameState::Live {
                    period_text: "In Progress".to_string(),
                    time_remaining: None,
                    in_intermission: false,
                }
            }
        } else {
            // Scheduled game - format start time
            let start_time = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
                let local_time: chrono::DateTime<chrono::Local> = parsed.into();
                local_time.format("%I:%M %p").to_string()
            } else {
                game.start_time_utc.clone()
            };
            GameState::Scheduled { start_time }
        };

        // Get period scores if available
        let (has_ot, has_so, away_periods, home_periods) = if let Some(scores) = period_scores.get(&game.id) {
            (
                scores.has_ot,
                scores.has_so,
                Some(scores.away_periods.clone()),
                Some(scores.home_periods.clone()),
            )
        } else {
            (false, false, None, None)
        };

        // Determine current period for live games
        let current_period = if game.game_state.has_started() && !game.game_state.is_final() {
            game_info.get(&game.id).and_then(|info| {
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

        GameBox::new(
            game.away_team.abbrev.clone(),
            game.home_team.abbrev.clone(),
            game.away_team.score,
            game.home_team.score,
            away_periods,
            home_periods,
            has_ot,
            has_so,
            current_period,
            state,
            selected_game_index == Some(index),
        )
    }).collect()
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    schedule: &Option<nhl_api::DailySchedule>,
    period_scores: &HashMap<i64, PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
    display: &Arc<DisplayConfig>,
    _boxscore: &Option<nhl_api::Boxscore>,
    _boxscore_loading: bool,
    _player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
) {

    // Handle empty schedule (loading state)
    if schedule.is_none() {
        let paragraph = Paragraph::new("Loading games...")
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
        return;
    }

    let schedule = schedule.as_ref().unwrap();

    // Handle no games for this date
    if schedule.games.is_empty() {
        let paragraph = Paragraph::new("No games scheduled for this date")
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
        return;
    }

    // Update grid dimensions based on terminal size and game count
    state.update_grid_dimensions(area.width, schedule.games.len());

    // Get selected game index if in selection mode
    let selected_game_index = state.get_selected_game_index();

    // Create game boxes with selection
    let game_boxes = create_game_boxes(schedule, period_scores, game_info, selected_game_index);

    // Render game grid
    let game_grid = GameGrid::new(game_boxes);
    game_grid.render(area, f.buffer_mut(), display);
}

// ============================================================================
// === COMMENTED OUT FOR REFACTORING - WILL REACTIVATE LATER ===
// ============================================================================
//
// This entire file represents the old state-based scores view implementation.
// Keep for reference when rebuilding scores functionality with Container widgets.
//
// Key functionality to rebuild:
// - Date window with 5-date sliding navigation
// - Game grid with responsive 1-3 column layout
// - GameBox widgets showing scores, period details, game status
// - Boxscore detailed view with player stats
// - Player detail panels with bio and career stats
// - Complex scrolling and selection state management
//
// ============================================================================

// use ratatui::{
//     layout::Rect,
//     style::Style,
//     text::{Line, Span, Text},
//     widgets::{Block, Borders, Paragraph},
//     Frame,
// };
// use std::collections::HashMap;
// use std::sync::Arc;
// use crate::tui::widgets::{GameBox, GameGrid, GameState, TeamStatsPanel, RenderableWidget};
// use crate::config::DisplayConfig;
// use crate::formatting::format_header;
// use super::State;
// use super::state::DATE_WINDOW_SIZE;
// use crate::tui::common::separator::build_tab_separator_line;
// use crate::tui::common::styling::{base_tab_style, selection_style};
//
// /// Calculate the date window based on game_date and selected_index
// /// The window has a fixed base date (leftmost date) that only shifts when reaching edges
// /// Relationship: window_base_date = game_date - selected_index
// /// Window: [base, base+1, base+2, base+3, base+4]
// fn calculate_date_window(game_date: &nhl_api::GameDate, selected_index: usize) -> [nhl_api::GameDate; DATE_WINDOW_SIZE] {
//     // Calculate window base: the leftmost date in the window
//     let window_base_date = game_date.add_days(-(selected_index as i64));
//
//     // Window is always [base, base+1, base+2, base+3, base+4]
//     [
//         window_base_date.add_days(0),
//         window_base_date.add_days(1),
//         window_base_date.add_days(2),
//         window_base_date.add_days(3),
//         window_base_date.add_days(4),
//     ]
// }
//
// /// Format a GameDate as MM/DD
// fn format_date_mmdd(date: &nhl_api::GameDate) -> String {
//     match date {
//         nhl_api::GameDate::Date(naive_date) => naive_date.format("%m/%d").to_string(),
//         nhl_api::GameDate::Now => chrono::Local::now().date_naive().format("%m/%d").to_string(),
//     }
// }
//
// /// Build subtab spans for date navigation
// fn build_date_subtab_spans(
//     date_strings: &[String],
//     selected_index: usize,
//     base_style: Style,
//     focused: bool,
//     theme: &Arc<DisplayConfig>,
// ) -> Vec<Span<'static>> {
//     let separator = format!(" {} ", theme.box_chars.vertical);
//     let mut spans = Vec::new();
//
//     for (i, date_str) in date_strings.iter().enumerate() {
//         if i > 0 {
//             spans.push(Span::styled(separator.clone(), base_style));
//         }
//
//         let style = selection_style(
//             base_style,
//             i == selected_index,
//             focused,
//             theme.selection_fg,
//             theme.unfocused_selection_fg(),
//         );
//         spans.push(Span::styled(date_str.clone(), style));
//     }
//
//     spans
// }
//
// pub fn render_subtabs(
//     f: &mut Frame,
//     area: Rect,
//     state: &State,
//     game_date: &nhl_api::GameDate,
//     theme: &Arc<DisplayConfig>,
// ) {
//     let focused = state.subtab_focused && !state.box_selection_active;
//     let base_style = base_tab_style(focused);
//
//     // Old implementation (before widget migration)
//     // Now using render_subtabs_with_breadcrumb() instead
//
//     let subtab_spans = build_date_subtab_spans(
//         &date_strings,
//         state.selected_index,
//         base_style,
//         focused,
//         theme,
//     );
//     let subtab_line = Line::from(subtab_spans);
//
//     let separator_line = build_tab_separator_line(
//         date_strings.into_iter(),
//         area.width as usize,
//         base_style,
//         &theme.box_chars,
//     );
//
//     let subtab_widget = Paragraph::new(vec![subtab_line, separator_line])
//         .block(Block::default().borders(Borders::NONE));
//
//     f.render_widget(subtab_widget, area);
// }
//
// /// Terminal width threshold for 3-column layout
// const THREE_COLUMN_WIDTH: u16 = 115;
//
// /// Terminal width threshold for 2-column layout
// const TWO_COLUMN_WIDTH: u16 = 76;
//
// /// Convert schedule data to GameBox widgets
// fn create_game_boxes(
//     schedule: &nhl_api::DailySchedule,
//     period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
//     game_info: &HashMap<i64, nhl_api::GameMatchup>,
//     selected_game_index: Option<usize>,
// ) -> Vec<GameBox> {
//     schedule.games.iter().enumerate().map(|(index, game)| {
//         // Determine game state
//         let state = if game.game_state.is_final() {
//             GameState::Final
//         } else if game.game_state.has_started() {
//             // Get period text and time from game_info
//             if let Some(info) = game_info.get(&game.id) {
//                 let period_text = crate::commands::scores_format::format_period_text(
//                     &info.period_descriptor.period_type,
//                     info.period_descriptor.number
//                 );
//
//                 let (time_remaining, in_intermission) = if let Some(clock) = &info.clock {
//                     (Some(clock.time_remaining.clone()), clock.in_intermission)
//                 } else {
//                     (None, false)
//                 };
//
//                 GameState::Live {
//                     period_text,
//                     time_remaining,
//                     in_intermission,
//                 }
//             } else {
//                 // Fallback for live game without info
//                 GameState::Live {
//                     period_text: "In Progress".to_string(),
//                     time_remaining: None,
//                     in_intermission: false,
//                 }
//             }
//         } else {
//             // Scheduled game - format start time
//             let start_time = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
//                 let local_time: chrono::DateTime<chrono::Local> = parsed.into();
//                 local_time.format("%I:%M %p").to_string()
//             } else {
//                 game.start_time_utc.clone()
//             };
//             GameState::Scheduled { start_time }
//         };
//
//         // Get period scores if available
//         let (has_ot, has_so, away_periods, home_periods) = if let Some(scores) = period_scores.get(&game.id) {
//             (
//                 scores.has_ot,
//                 scores.has_so,
//                 Some(scores.away_periods.clone()),
//                 Some(scores.home_periods.clone()),
//             )
//         } else {
//             (false, false, None, None)
//         };
//
//         // Determine current period for live games
//         let current_period = if game.game_state.has_started() && !game.game_state.is_final() {
//             game_info.get(&game.id).and_then(|info| {
//                 match info.period_descriptor.period_type.as_str() {
//                     "REG" => Some(info.period_descriptor.number),
//                     "OT" => Some(4),
//                     "SO" => Some(5),
//                     _ => Some(info.period_descriptor.number),
//                 }
//             })
//         } else {
//             None
//         };
//
//         GameBox::new(
//             game.away_team.abbrev.clone(),
//             game.home_team.abbrev.clone(),
//             game.away_team.score,
//             game.home_team.score,
//             away_periods,
//             home_periods,
//             has_ot,
//             has_so,
//             current_period,
//             state,
//             selected_game_index == Some(index),
//         )
//     }).collect()
// }
//
// pub fn render_content(
//     f: &mut Frame,
//     area: Rect,
//     state: &mut State,
//     schedule: &Option<nhl_api::DailySchedule>,
//     period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
//     game_info: &HashMap<i64, nhl_api::GameMatchup>,
//     display: &Arc<DisplayConfig>,
//     boxscore: &Option<nhl_api::Boxscore>,
//     boxscore_loading: bool,
//     player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
// ) {
//     // Check if we should render a navigation panel (player details)
//     let panel_to_render = state.navigation.as_ref()
//         .and_then(|nav_ctx| nav_ctx.stack.current())
//         .cloned();
//
//     if let Some(panel) = panel_to_render {
//         render_panel(f, area, state, &panel, display, player_info);
//         return;
//     }
//
//     // If boxscore view is active, render boxscore instead of game list
//     if state.boxscore_view_active {
//         render_boxscore_content(f, area, state, boxscore, boxscore_loading, period_scores, game_info, display);
//         return;
//     }
//
//     if let Some(schedule) = schedule {
//         // Calculate grid dimensions
//         let num_columns = if area.width >= THREE_COLUMN_WIDTH {
//             3
//         } else if area.width >= TWO_COLUMN_WIDTH {
//             2
//         } else {
//             1
//         };
//
//         let total_games = schedule.games.len();
//         if total_games == 0 {
//             let paragraph = Paragraph::new("No games scheduled for today.")
//                 .block(Block::default().borders(Borders::NONE));
//             f.render_widget(paragraph, area);
//             state.grid_dimensions = (0, 0);
//             return;
//         }
//
//         let num_rows = (total_games + num_columns - 1) / num_columns;
//         state.grid_dimensions = (num_rows, num_columns);
//
//         // Calculate selected game index from (row, col) grid position
//         let selected_game_index = if state.box_selection_active {
//             let (row, col) = state.selected_box;
//             Some(row * num_columns + col)
//         } else {
//             None
//         };
//
//         // Create game boxes and render using GameGrid widget
//         let game_boxes = create_game_boxes(schedule, period_scores, game_info, selected_game_index);
//         let game_grid = GameGrid::new(game_boxes);
//
//         // Render the widget directly to the frame
//         use crate::tui::widgets::RenderableWidget;
//         let buf = f.buffer_mut();
//         game_grid.render(area, buf, display);
//
//         // Update scrollable state (for future scroll support)
//         state.grid_scrollable.update_viewport_height(area.height);
//         // Height = num_rows * 7 (each box is 7 lines tall)
//         state.grid_scrollable.update_content_height(num_rows * 7);
//     } else {
//         let paragraph = Paragraph::new("Loading scores...").block(Block::default().borders(Borders::NONE));
//         f.render_widget(paragraph, area);
//         state.grid_dimensions = (0, 0);
//     }
// }
//
// // Constants for box layout
// const LINES_PER_BOX: usize = 7;
// const BLANK_LINE_BETWEEN_BOXES: usize = 1;
// const LINES_PER_ROW: usize = LINES_PER_BOX + BLANK_LINE_BETWEEN_BOXES; // 8 lines total per row
// const BOX_WIDTH: usize = 37;
// const BOX_GAP: usize = 2;
//
// /// Calculate the line range (start, end) for a box at the given row
// fn calculate_box_line_range(sel_row: usize) -> (usize, usize) {
//     let start_line = sel_row * LINES_PER_ROW;
//     let end_line = start_line + LINES_PER_BOX; // 7 lines for the box
//     (start_line, end_line)
// }
//
// /// Ensure the selected box is fully visible by adjusting scroll offset
// fn ensure_box_visible(state: &mut State, viewport_height: u16) {
//     if !state.box_selection_active {
//         return;
//     }
//
//     let (sel_row, _) = state.selected_box;
//     let (start_line, end_line) = calculate_box_line_range(sel_row);
//
//     let scroll_offset = state.grid_scrollable.scroll_offset as usize;
//     let viewport_end = scroll_offset + viewport_height as usize;
//
//     // If box top is above viewport, scroll up to show it
//     if start_line < scroll_offset {
//         state.grid_scrollable.scroll_offset = start_line as u16;
//     }
//     // If box bottom is below viewport, scroll down to show it
//     else if end_line > viewport_end {
//         let new_offset = end_line.saturating_sub(viewport_height as usize);
//         state.grid_scrollable.scroll_offset = new_offset as u16;
//     }
// }
//
// /// Calculate the column range (start, end) for a box at the given column
// fn calculate_box_column_range(sel_col: usize) -> (usize, usize) {
//     let start_col = sel_col * (BOX_WIDTH + BOX_GAP);
//     let end_col = start_col + BOX_WIDTH;
//     (start_col, end_col)
// }
//
// /// Convert character positions to byte indices for UTF-8 safe string slicing
// fn char_positions_to_byte_indices(line: &str, start_col: usize, end_col: usize) -> (usize, usize) {
//     let char_indices: Vec<(usize, char)> = line.char_indices().collect();
//     let char_count = char_indices.len();
//
//     let byte_start = if start_col < char_count {
//         char_indices[start_col].0
//     } else {
//         line.len()
//     };
//
//     let byte_end = if end_col < char_count {
//         char_indices[end_col].0
//     } else {
//         line.len()
//     };
//
//     (byte_start, byte_end)
// }
//
// /// Create styled spans for a line, applying selection color to the specified byte range
// fn create_styled_spans(line: &str, byte_start: usize, byte_end: usize, theme: &Arc<DisplayConfig>) -> Vec<Span<'static>> {
//     let mut spans = Vec::new();
//
//     // Before the box
//     if byte_start > 0 {
//         spans.push(Span::raw(line[..byte_start].to_string()));
//     }
//
//     // The box content with selection color
//     if byte_start < byte_end {
//         spans.push(Span::styled(
//             line[byte_start..byte_end].to_string(),
//             Style::default().fg(theme.selection_fg)
//         ));
//     }
//
//     // After the box
//     if byte_end < line.len() {
//         spans.push(Span::raw(line[byte_end..].to_string()));
//     }
//
//     // If line is too short to reach the box, just show the whole line
//     if spans.is_empty() {
//         spans.push(Span::raw(line.to_string()));
//     }
//
//     spans
// }
//
// /// Apply selection foreground color to selected box using ratatui's styling system
// ///
// /// Each game box is 7 lines tall:
// /// 1. Header line (e.g., "Final Score" or start time)
// /// 2. Top border (╭─...╮)
// /// 3. Header row (│ empty │ 1 │ 2 │ 3 │ T │)
// /// 4. Middle border (├─┼─...┤)
// /// 5. Away team row
// /// 6. Home team row
// /// 7. Bottom border (╰─...╯)
// /// Plus 1 blank line between rows
// fn apply_box_styling_ratatui(content: &str, sel_row: usize, sel_col: usize, theme: &Arc<DisplayConfig>) -> Text<'static> {
//     let lines: Vec<&str> = content.lines().collect();
//     let mut styled_lines: Vec<Line> = Vec::new();
//
//     let (start_line, end_line) = calculate_box_line_range(sel_row);
//     let (start_col, end_col) = calculate_box_column_range(sel_col);
//
//     for (line_idx, line) in lines.iter().enumerate() {
//         if line_idx >= start_line && line_idx < end_line {
//             // This line is part of the selected box - apply selection styling
//             let (byte_start, byte_end) = char_positions_to_byte_indices(line, start_col, end_col);
//             let spans = create_styled_spans(line, byte_start, byte_end, theme);
//             styled_lines.push(Line::from(spans));
//         } else {
//             // Normal line without styling
//             styled_lines.push(Line::raw(line.to_string()));
//         }
//     }
//
//     Text::from(styled_lines)
// }
//
// /// Combine two tables side by side with headers above each
// fn combine_tables_with_headers(
//     left_header: &str,
//     left_table: &str,
//     right_header: &str,
//     right_table: &str,
// ) -> String {
//     let mut output = String::new();
//
//     // Split tables into lines
//     let left_lines: Vec<&str> = left_table.lines().collect();
//     let right_lines: Vec<&str> = right_table.lines().collect();
//
//     // Add headers (assuming each table is 37 chars wide)
//     output.push_str(left_header);
//     output.push_str(&" ".repeat(37 - left_header.len() + 2)); // Padding + gap
//     output.push_str(right_header);
//     output.push('\n');
//
//     // Combine tables line by line
//     let max_lines = left_lines.len().max(right_lines.len());
//
//     for i in 0..max_lines {
//         // Get left line or pad with spaces
//         if i < left_lines.len() {
//             output.push_str(left_lines[i]);
//         } else {
//             output.push_str(&" ".repeat(37));
//         }
//
//         // Add gap between tables
//         output.push_str("  ");
//
//         // Get right line or pad with spaces
//         if i < right_lines.len() {
//             output.push_str(right_lines[i]);
//         } else {
//             output.push_str(&" ".repeat(37));
//         }
//
//         output.push('\n');
//     }
//
//     output
// }
//
// /// Column widths structure for scoring summary table
// // Old scoring summary implementation removed - now using ScoringTable widget
// // See src/tui/widgets/scoring_table.rs for the widget-based implementation
// /// Render scoring summary using the ScoringTable widget
// ///
// /// This is a bridge function during migration from string-based to widget-based rendering.
// /// It renders the ScoringTable widget to a buffer and converts it to a string.
// fn format_scoring_summary(scoring: &[nhl_api::PeriodScoring], display: &DisplayConfig) -> String {
//     use crate::tui::widgets::{RenderableWidget, ScoringTable};
//     use ratatui::{buffer::Buffer, layout::Rect};
//
//     if scoring.is_empty() {
//         return String::new();
//     }
//
//     // Create the widget
//     let widget = ScoringTable::new(scoring.to_vec());
//
//     // Get preferred dimensions
//     let width = widget.preferred_width().unwrap_or(80);
//     let height = widget.preferred_height().unwrap_or(20);
//
//     // Create a buffer to render into
//     let area = Rect::new(0, 0, width, height);
//     let mut buf = Buffer::empty(area);
//
//     // Render the widget
//     widget.render(area, &mut buf, display);
//
//     // Convert buffer to string
//     let mut output = String::new();
//     for y in 0..height {
//         let mut line = String::new();
//         for x in 0..width {
//             let cell = buf.cell((x, y)).unwrap();
//             line.push_str(cell.symbol());
//         }
//         // Trim trailing spaces from each line
//         let trimmed = line.trim_end();
//         if !trimmed.is_empty() || y < height - 1 {
//             output.push_str(trimmed);
//             output.push('\n');
//         }
//     }
//
//     output
// }
//
// /// Helper function to render TeamStatsPanel widget to string
// fn render_team_stats_panel_to_string(
//     team_abbrev: &str,
//     stats: &nhl_api::TeamPlayerStats,
//     display: &DisplayConfig,
// ) -> String {
//     use ratatui::buffer::Buffer;
//
//     // Create the widget
//     let panel = TeamStatsPanel::new(team_abbrev, stats, 0);
//
//     // Calculate dimensions
//     let height = panel.preferred_height().unwrap_or(0);
//     let width = panel.preferred_width().unwrap_or(80);
//
//     // Render to buffer
//     let area = Rect::new(0, 0, width, height);
//     let mut buf = Buffer::empty(area);
//     panel.render(area, &mut buf, display);
//
//     // Convert buffer to string
//     let mut output = String::new();
//     for y in 0..buf.area.height {
//         for x in 0..buf.area.width {
//             if let Some(cell) = buf.cell((x, y)) {
//                 output.push_str(cell.symbol());
//             }
//         }
//         if y < buf.area.height - 1 {
//             output.push('\n');
//         }
//     }
//
//     output
// }
//
// /// Format boxscore with period score box at the top
// fn format_boxscore_with_period_box(
//     boxscore: &nhl_api::Boxscore,
//     period_scores: Option<&crate::commands::scores_format::PeriodScores>,
//     game_info: Option<&nhl_api::GameMatchup>,
//     display: &DisplayConfig,
// ) -> String {
//     let mut output = String::new();
//
//     // Display game header
//     let header = format!("{} @ {}",
//         boxscore.away_team.common_name.default,
//         boxscore.home_team.common_name.default
//     );
//     output.push_str(&format!("\n{}", format_header(&header, true, display)));
//     output.push_str(&format!("Date: {} | Venue: {}\n",
//         boxscore.game_date,
//         boxscore.venue.default
//     ));
//     output.push_str(&format!("Status: {} | Period: {}\n",
//         boxscore.game_state,
//         boxscore.period_descriptor.number
//     ));
//     if boxscore.clock.running || !boxscore.clock.in_intermission {
//         output.push_str(&format!("Time: {}\n", boxscore.clock.time_remaining));
//     }
//
//     // Add period score and shots boxes side by side
//     output.push_str("\n");
//
//     // Determine if game has OT or SO
//     let (has_ot, has_so, away_periods, home_periods) = if let Some(scores) = period_scores {
//         (scores.has_ot, scores.has_so, Some(&scores.away_periods), Some(&scores.home_periods))
//     } else {
//         (false, false, None, None)
//     };
//
//     // Determine current period for in-progress games
//     let current_period_num = if boxscore.game_state.has_started() && !boxscore.game_state.is_final() {
//         game_info.and_then(|info| {
//             match info.period_descriptor.period_type.as_str() {
//                 "REG" => Some(info.period_descriptor.number),
//                 "OT" => Some(4),
//                 "SO" => Some(5),
//                 _ => Some(info.period_descriptor.number),
//             }
//         })
//     } else {
//         None
//     };
//
//     // Build both tables
//     let score_table = crate::commands::scores_format::build_score_table(
//         &boxscore.away_team.abbrev,
//         &boxscore.home_team.abbrev,
//         Some(boxscore.away_team.score),
//         Some(boxscore.home_team.score),
//         has_ot,
//         has_so,
//         away_periods,
//         home_periods,
//         current_period_num,
//         &display.box_chars,
//     );
//
//     let shots_table = crate::commands::scores_format::build_shots_table(
//         &boxscore.away_team.abbrev,
//         &boxscore.home_team.abbrev,
//         Some(boxscore.away_team.sog),
//         Some(boxscore.home_team.sog),
//         has_ot,
//         has_so,
//         &display.box_chars,
//     );
//
//     // Combine tables side by side with headers
//     let combined = combine_tables_with_headers(
//         "Scores",
//         &score_table,
//         "Shots on goal",
//         &shots_table,
//     );
//
//     output.push_str(&combined);
//
//     if let Some(game_matchup) = game_info {
//         if let Some(ref summary) = game_matchup.summary {
//             output.push_str("\n");
//             output.push_str(&format_scoring_summary(&summary.scoring, display));
//         }
//     }
//
//     #[cfg(feature = "game_stats")]
//     {
//         let away_team_stats = nhl_api::TeamGameStats::from_team_player_stats(&boxscore.player_by_game_stats.away_team);
//         let home_team_stats = nhl_api::TeamGameStats::from_team_player_stats(&boxscore.player_by_game_stats.home_team);
//         let game_stats_table = crate::commands::boxscore::format_game_stats_table(
//             &boxscore.away_team.abbrev,
//             &boxscore.home_team.abbrev,
//             &away_team_stats,
//             &home_team_stats,
//         );
//         output.push_str(&game_stats_table);
//     }
//
//     // Display player stats using TeamStatsPanel widget
//     output.push('\n');
//     output.push_str(&render_team_stats_panel_to_string(
//         &boxscore.away_team.abbrev,
//         &boxscore.player_by_game_stats.away_team,
//         display
//     ));
//     output.push('\n');
//     output.push_str(&render_team_stats_panel_to_string(
//         &boxscore.home_team.abbrev,
//         &boxscore.player_by_game_stats.home_team,
//         display
//     ));
//
//     output
// }
//
// /// Render boxscore content in place of game list (scrollable)
// fn render_boxscore_content(
//     f: &mut Frame,
//     area: Rect,
//     state: &mut State,
//     boxscore: &Option<nhl_api::Boxscore>,
//     loading: bool,
//     period_scores: &HashMap<i64, crate::commands::scores_format::PeriodScores>,
//     game_info: &HashMap<i64, nhl_api::GameMatchup>,
//     display: &DisplayConfig,
// ) {
//     // Render the boxscore content
//     let content_text = if loading {
//         "Loading boxscore...".to_string()
//     } else if let Some(ref bs) = boxscore {
//         format_boxscore_with_period_box(bs, period_scores.get(&bs.id), game_info.get(&bs.id), display)
//     } else {
//         "No boxscore available".to_string()
//     };
//
//     // Use game_details rendering for enhanced navigation support
//     crate::tui::scores::game_details::view::render(
//         f,
//         area,
//         &mut state.game_details,
//         &content_text,
//         boxscore.as_ref(),
//         display,
//     );
// }
//
// /// Public function to format boxscore as text for exporting
// pub fn format_boxscore_text(
//     boxscore: &nhl_api::Boxscore,
//     period_scores: Option<&crate::commands::scores_format::PeriodScores>,
//     game_info: Option<&nhl_api::GameMatchup>,
//     display: &DisplayConfig,
// ) -> String {
//     format_boxscore_with_period_box(boxscore, period_scores, game_info, display)
// }
//
// /// Render header lines with formatting
// fn render_header_lines(header_text: &str, margin: usize, display: &Arc<DisplayConfig>) -> Vec<Line<'static>> {
//     let mut lines = Vec::new();
//     let header = format_header(header_text, true, display);
//     for line in header.lines() {
//         if !line.is_empty() {
//             lines.push(Line::from(vec![
//                 Span::raw(" ".repeat(margin)),
//                 Span::styled(line.to_string(), Style::default().fg(display.division_header_fg))
//             ]));
//         } else {
//             lines.push(Line::raw(""));
//         }
//     }
//     lines
// }
//
// /// Render a navigation panel (currently only player details)
// fn render_panel(
//     f: &mut Frame,
//     area: Rect,
//     state: &mut State,
//     panel: &crate::tui::scores::panel::ScoresPanel,
//     display: &Arc<DisplayConfig>,
//     player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
// ) {
//     match panel {
//         crate::tui::scores::panel::ScoresPanel::PlayerDetail { player_id, player_name, .. } => {
//             render_player_panel(f, area, state, *player_id, player_name, display, player_info);
//         }
//     }
// }
//
// /// Render player details panel (adapted from standings)
// fn render_player_panel(
//     f: &mut Frame,
//     area: Rect,
//     state: &mut State,
//     player_id: i64,
//     player_name: &str,
//     display: &Arc<DisplayConfig>,
//     player_info_map: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
// ) {
//     use crate::tui::widgets::{RenderableWidget, PlayerBioCard, CareerStatsTable};
//     use crate::types::NHL_LEAGUE_ABBREV;
//
//     let mut lines = Vec::new();
//
//     lines.push(Line::raw(""));
//     lines.extend(render_header_lines(player_name, 2, display));
//
//     // Get real player data
//     let player_data = player_info_map.get(&player_id);
//
//     if player_data.is_none() {
//         lines.push(Line::raw("  Loading player information..."));
//         lines.push(Line::raw(""));
//
//         state.panel_scrollable.update_viewport_height(area.height);
//         state.panel_scrollable.update_content_height(lines.len());
//
//         let paragraph = Paragraph::new(lines)
//             .scroll((state.panel_scrollable.scroll_offset, 0));
//         f.render_widget(paragraph, area);
//         return;
//     }
//
//     let player = player_data.unwrap();
//
//     // Filter to only NHL seasons
//     let nhl_seasons: Vec<nhl_api::SeasonTotal> = if let Some(season_totals) = &player.season_totals {
//         season_totals.iter()
//             .filter(|s| s.league_abbrev == NHL_LEAGUE_ABBREV)
//             .cloned()
//             .collect()
//     } else {
//         vec![]
//     };
//
//     // Create bio card widget
//     let bio_card = PlayerBioCard::new(player, Some("Player Information"), 0);
//
//     // Create career stats table
//     let career_table = CareerStatsTable::new(&nhl_seasons, Some("NHL Career Statistics"), None, 0);
//
//     // Render widgets to buffer and convert to lines
//     let buf_width = area.width.max(80);
//     let bio_height = bio_card.preferred_height().unwrap_or(12);
//     let career_height = career_table.preferred_height().unwrap_or(10);
//
//     let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, buf_width, bio_height + career_height + 2));
//
//     // Render bio card
//     bio_card.render(Rect::new(0, 0, buf_width, bio_height), &mut buf, display);
//
//     // Add blank line
//     lines.push(Line::raw(""));
//
//     // Render career stats below bio
//     career_table.render(Rect::new(0, bio_height + 1, buf_width, career_height), &mut buf, display);
//
//     // Convert buffer to lines
//     for y in 0..buf.area.height {
//         let mut current_style = Style::default();
//         let mut current_text = String::new();
//         let mut spans = Vec::new();
//
//         for x in 0..buf.area.width {
//             let cell = buf.get(x, y);
//             if cell.style() != current_style {
//                 if !current_text.is_empty() {
//                     spans.push(Span::styled(current_text.clone(), current_style));
//                     current_text.clear();
//                 }
//                 current_style = cell.style();
//             }
//             current_text.push_str(cell.symbol());
//         }
//
//         if !current_text.is_empty() {
//             spans.push(Span::styled(current_text, current_style));
//         }
//
//         lines.push(Line::from(spans));
//     }
//
//     state.panel_scrollable.update_viewport_height(area.height);
//     state.panel_scrollable.update_content_height(lines.len());
//
//     let paragraph = Paragraph::new(lines)
//         .scroll((state.panel_scrollable.scroll_offset, 0));
//     f.render_widget(paragraph, area);
// }
//
// #[cfg(test)]
// mod tests {
//     // Tests for the old scoring summary implementation have been removed.
//     // The ScoringTable widget has its own comprehensive test suite in
//     // src/tui/widgets/scoring_table.rs
//
// }
