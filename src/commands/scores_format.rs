use nhl_api::{DailySchedule, ScheduleGame, GameSummary};
use std::collections::HashMap;

// Layout Constants
/// Terminal width threshold for 3-column layout (37*3 + 2*2 gaps = 115)
const THREE_COLUMN_WIDTH: usize = 115;

/// Terminal width threshold for 2-column layout (37*2 + 2 gap = 76)
const TWO_COLUMN_WIDTH: usize = 76;

/// Width of a single game box to accommodate all 5 periods (1, 2, 3, OT, SO)
const GAME_BOX_WIDTH: usize = 37;

/// Gap between game boxes when displayed side-by-side
const GAME_BOX_GAP: usize = 2;

// Score Table Constants
/// Number of base columns in score table (empty, 1, 2, 3, T)
const BASE_SCORE_COLUMNS: usize = 5;

/// Width of team abbreviation column in score table
const TEAM_ABBREV_COL_WIDTH: usize = 5;

/// Width of each period column in score table
const PERIOD_COL_WIDTH: usize = 4;

/// Maximum width of score table with all periods
const SCORE_TABLE_MAX_WIDTH: usize = 37;

/// Header text left padding (1 space)
const HEADER_LEFT_PADDING: usize = 1;

/// Header text width (36 characters for content)
const HEADER_CONTENT_WIDTH: usize = 36;

// Period Index Constants
/// Array index for period 1 scores
const PERIOD_1_INDEX: usize = 0;

/// Array index for period 2 scores
const PERIOD_2_INDEX: usize = 1;

/// Array index for period 3 scores
const PERIOD_3_INDEX: usize = 2;

/// Array index for overtime scores
const OVERTIME_INDEX: usize = 3;

/// Array index for shootout scores
const SHOOTOUT_INDEX: usize = 4;

/// Period number for overtime (used in game state)
const OVERTIME_PERIOD_NUM: i32 = 4;

/// Period number for shootout (used in game state)
const SHOOTOUT_PERIOD_NUM: i32 = 5;

/// Period-by-period score data
#[derive(Debug, Clone)]
pub struct PeriodScores {
    pub away_periods: Vec<i32>,
    pub home_periods: Vec<i32>,
    pub has_ot: bool,
    pub has_so: bool,
}

/// Format scores for TUI display with period-by-period breakdown
// pub fn format_scores_for_tui(
//     schedule: &DailySchedule,
//     period_scores: &HashMap<i64, PeriodScores>,
// ) -> String {
//     let empty_game_info = HashMap::new();
//     format_scores_for_tui_with_width(schedule, period_scores, &empty_game_info, None)
// }

/// Calculate the number of columns to display based on terminal width.
/// Each game box is 37 characters wide to accommodate all 5 periods (1, 2, 3, OT, SO).
///
/// Returns:
/// - 3 columns for wide terminals (width >= 115)
/// - 2 columns for medium terminals (width >= 76)
/// - 1 column for narrow terminals or when width is not provided
fn calculate_columns_from_width(terminal_width: Option<usize>) -> usize {
    if let Some(width) = terminal_width {
        if width >= THREE_COLUMN_WIDTH {
            3
        } else if width >= TWO_COLUMN_WIDTH {
            2
        } else {
            1
        }
    } else {
        1 // Default to 1 column if width not provided
    }
}

/// Format scores with specific terminal width for column layout
pub fn format_scores_for_tui_with_width(
    schedule: &DailySchedule,
    period_scores: &HashMap<i64, PeriodScores>,
    game_info: &HashMap<i64, nhl_api::GameMatchup>,
    terminal_width: Option<usize>,
) -> String {
    let mut output = String::new();

    // Display header
    //output.push_str(&format!("{}\n", schedule.date));

    if schedule.number_of_games == 0 {
        output.push_str("No games scheduled for today.\n");
        return output;
    }

    // Determine number of columns based on terminal width
    let num_columns = calculate_columns_from_width(terminal_width);

    // Group games into rows
    let games: Vec<_> = schedule.games.iter().collect();
    let rows: Vec<_> = games.chunks(num_columns).collect();

    for (row_idx, row) in rows.iter().enumerate() {
        if row_idx > 0 {
            output.push('\n');
        }

        // Format each game in the row as a table
        let formatted_games: Vec<String> = row
            .iter()
            .map(|game| format_game_table(game, period_scores.get(&game.id), game_info.get(&game.id)))
            .collect();

        // Combine games horizontally
        output.push_str(&combine_tables_horizontally(&formatted_games));
    }

    output
}

/// Combine multiple game tables horizontally (side-by-side)
fn combine_tables_horizontally(tables: &[String]) -> String {
    if tables.is_empty() {
        return String::new();
    }

    // Split each table into lines
    let table_lines: Vec<Vec<&str>> = tables
        .iter()
        .map(|t| t.lines().collect())
        .collect();

    // Find the maximum number of lines
    let max_lines = table_lines.iter().map(|t| t.len()).max().unwrap_or(0);

    let mut output = String::new();

    // Combine line by line
    for line_idx in 0..max_lines {
        for (table_idx, lines) in table_lines.iter().enumerate() {
            if table_idx > 0 {
                output.push_str(&" ".repeat(GAME_BOX_GAP)); // Gap between tables
            }

            // Get the line or use empty space if this table is shorter
            if line_idx < lines.len() {
                output.push_str(lines[line_idx]);
            } else {
                // Pad with spaces to match table width (all 5 periods)
                output.push_str(&" ".repeat(GAME_BOX_WIDTH));
            }
        }
        output.push('\n');
    }

    output
}

/// Format period text (e.g., "1st Period", "Overtime", "Shootout")
fn format_period_text(period_type: &str, period_number: i32) -> String {
    match period_type {
        "REG" => {
            let ordinal = match period_number {
                1 => "1st",
                2 => "2nd",
                3 => "3rd",
                n => return format!("{}th Period", n),
            };
            format!("{} Period", ordinal)
        },
        "OT" => "Overtime".to_string(),
        "SO" => "Shootout".to_string(),
        _ => format!("Period {}", period_number),
    }
}

/// Format header for live game (with period and time info)
fn format_live_game_header(info: &nhl_api::GameMatchup) -> String {
    let period_text = format_period_text(
        &info.period_descriptor.period_type,
        info.period_descriptor.number
    );

    if let Some(clock) = &info.clock {
        if clock.in_intermission {
            format!("{} - Intermission", period_text)
        } else {
            format!("{} - {}", period_text, clock.time_remaining)
        }
    } else {
        period_text
    }
}

/// Format header for scheduled game (showing start time)
fn format_scheduled_game_header(start_time_utc: &str) -> String {
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(start_time_utc) {
        let local_time: chrono::DateTime<chrono::Local> = parsed.into();
        local_time.format("%I:%M %p").to_string()
    } else {
        start_time_utc.to_string()
    }
}

/// Generate appropriate header text based on game state
fn generate_game_header(
    game: &ScheduleGame,
    game_info: Option<&nhl_api::GameMatchup>
) -> String {
    if game.game_state.is_final() {
        "Final Score".to_string()
    } else if game.game_state.has_started() {
        game_info
            .map(format_live_game_header)
            .unwrap_or_else(|| "In Progress".to_string())
    } else {
        format_scheduled_game_header(&game.start_time_utc)
    }
}

fn format_game_table(game: &ScheduleGame, period_scores: Option<&PeriodScores>, game_info: Option<&nhl_api::GameMatchup>) -> String {
    let mut output = String::new();

    // Determine if game has started
    let game_started = game.game_state.has_started();

    // Generate header using extracted function
    let header = generate_game_header(game, game_info);

    // Add left padding, then left-align the header
    output.push_str(&format!("{}{:<width$}\n", " ".repeat(HEADER_LEFT_PADDING), header, width = HEADER_CONTENT_WIDTH));

    // Determine current period for in-progress games
    let current_period_num = if game_started && !game.game_state.is_final() {
        game_info.and_then(|info| {
            // Get the current period number based on period type
            match info.period_descriptor.period_type.as_str() {
                "REG" => Some(info.period_descriptor.number),
                "OT" => Some(OVERTIME_PERIOD_NUM),
                "SO" => Some(SHOOTOUT_PERIOD_NUM),
                _ => Some(info.period_descriptor.number),
            }
        })
    } else {
        None
    };

    if game_started {
        if let (Some(away_score), Some(home_score)) = (game.away_team.score, game.home_team.score) {
            // Use period scores if available
            let (has_ot, has_so, away_periods, home_periods) = if let Some(scores) = period_scores {
                (scores.has_ot, scores.has_so, Some(&scores.away_periods), Some(&scores.home_periods))
            } else {
                (false, false, None, None)
            };

            output.push_str(&build_score_table(
                &game.away_team.abbrev,
                &game.home_team.abbrev,
                Some(away_score),
                Some(home_score),
                has_ot,
                has_so,
                away_periods,
                home_periods,
                current_period_num,
            ));
        } else {
            // Game started but no scores yet - show table with dashes
            output.push_str(&build_score_table(
                &game.away_team.abbrev,
                &game.home_team.abbrev,
                None,
                None,
                false,
                false,
                None,
                None,
                current_period_num,
            ));
        }
    } else {
        // Game hasn't started - show table with dashes
        output.push_str(&build_score_table(
            &game.away_team.abbrev,
            &game.home_team.abbrev,
            None,
            None,
            false,
            false,
            None,
            None,
            None,
        ));
    }

    output
}

fn build_score_table(
    away_team: &str,
    home_team: &str,
    away_score: Option<i32>,
    home_score: Option<i32>,
    has_ot: bool,
    has_so: bool,
    away_periods: Option<&Vec<i32>>,
    home_periods: Option<&Vec<i32>>,
    current_period_num: Option<i32>,
) -> String {
    let mut output = String::new();

    // Calculate column count based on actual periods, but we'll pad to max width later
    let base_cols = BASE_SCORE_COLUMNS; // empty, 1, 2, 3, T
    let ot_cols = if has_ot { 1 } else { 0 };
    let so_cols = if has_so { 1 } else { 0 };
    let total_cols = base_cols + ot_cols + so_cols;
    let max_width = SCORE_TABLE_MAX_WIDTH; // Width with all 5 periods

    // Helper to check if a period should show score or dash
    let should_show_period = |period: i32| -> bool {
        current_period_num.map_or(true, |current| period <= current)
    };

    // Build table components
    output.push_str(&build_top_border(total_cols, max_width));
    output.push_str(&build_header_row(has_ot, has_so, total_cols, max_width));
    output.push_str(&build_middle_border(total_cols, max_width));
    output.push_str(&build_team_row(
        away_team,
        away_score,
        away_periods,
        has_ot,
        has_so,
        total_cols,
        max_width,
        &should_show_period,
    ));
    output.push_str(&build_team_row(
        home_team,
        home_score,
        home_periods,
        has_ot,
        has_so,
        total_cols,
        max_width,
        &should_show_period,
    ));
    output.push_str(&build_bottom_border(total_cols, max_width));

    output
}

/// Calculate padding needed to reach max width of 37 characters
fn calculate_padding(total_cols: usize, max_width: usize) -> usize {
    // Calculate actual width: 1 (left border) + 5 (team column) + (total_cols-1) * (1 separator + 4 chars) + 1 (right border)
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        max_width - current_width
    } else {
        0
    }
}

/// Build top border for the score table
fn build_top_border(total_cols: usize, max_width: usize) -> String {
    let mut border = String::new();
    border.push('╭');
    border.push_str(&"─".repeat(TEAM_ABBREV_COL_WIDTH)); // team name column
    for _ in 1..total_cols {
        border.push('┬');
        border.push_str(&"─".repeat(PERIOD_COL_WIDTH));
    }
    border.push_str("╮");

    let padding = calculate_padding(total_cols, max_width);
    if padding > 0 {
        border.push_str(&" ".repeat(padding));
    }
    border.push('\n');
    border
}

/// Build middle border for the score table
fn build_middle_border(total_cols: usize, max_width: usize) -> String {
    let mut border = String::new();
    border.push('├');
    border.push_str(&"─".repeat(TEAM_ABBREV_COL_WIDTH));
    for _ in 1..total_cols {
        border.push('┼');
        border.push_str(&"─".repeat(PERIOD_COL_WIDTH));
    }
    border.push_str("┤");

    let padding = calculate_padding(total_cols, max_width);
    if padding > 0 {
        border.push_str(&" ".repeat(padding));
    }
    border.push('\n');
    border
}

/// Build bottom border for the score table
fn build_bottom_border(total_cols: usize, max_width: usize) -> String {
    let mut border = String::new();
    border.push('╰');
    border.push_str(&"─".repeat(TEAM_ABBREV_COL_WIDTH));
    for _ in 1..total_cols {
        border.push('┴');
        border.push_str(&"─".repeat(PERIOD_COL_WIDTH));
    }
    border.push_str("╯");

    let padding = calculate_padding(total_cols, max_width);
    if padding > 0 {
        border.push_str(&" ".repeat(padding));
    }
    border.push('\n');
    border
}

/// Build header row showing period numbers (1, 2, 3, OT, SO, T)
fn build_header_row(has_ot: bool, has_so: bool, total_cols: usize, max_width: usize) -> String {
    let mut row = String::new();
    row.push('│');
    row.push_str(&format!("{:^5}", ""));
    row.push('│');
    row.push_str(&format!("{:^4}", "1"));
    row.push('│');
    row.push_str(&format!("{:^4}", "2"));
    row.push('│');
    row.push_str(&format!("{:^4}", "3"));

    if has_ot {
        row.push('│');
        row.push_str(&format!("{:^4}", "OT"));
    }

    if has_so {
        row.push('│');
        row.push_str(&format!("{:^4}", "SO"));
    }

    row.push('│');
    row.push_str(&format!("{:^4}", "T"));
    row.push_str("│");

    let padding = calculate_padding(total_cols, max_width);
    if padding > 0 {
        row.push_str(&" ".repeat(padding));
    }
    row.push('\n');
    row
}

/// Render period scores for a team
fn render_team_periods(
    output: &mut String,
    periods: Option<&Vec<i32>>,
    has_ot: bool,
    has_so: bool,
    should_show_period: &impl Fn(i32) -> bool,
) {
    if let Some(periods) = periods {
        // Period 1
        let p1_value = if should_show_period(1) {
            periods.get(PERIOD_1_INDEX).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^width$}", p1_value, width = PERIOD_COL_WIDTH));
        output.push('│');

        // Period 2
        let p2_value = if should_show_period(2) {
            periods.get(PERIOD_2_INDEX).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^width$}", p2_value, width = PERIOD_COL_WIDTH));
        output.push('│');

        // Period 3
        let p3_value = if should_show_period(3) {
            periods.get(PERIOD_3_INDEX).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^width$}", p3_value, width = PERIOD_COL_WIDTH));

        if has_ot {
            output.push('│');
            let ot_value = if should_show_period(OVERTIME_PERIOD_NUM) {
                periods.get(OVERTIME_INDEX).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^width$}", ot_value, width = PERIOD_COL_WIDTH));
        }

        if has_so {
            output.push('│');
            let so_value = if should_show_period(SHOOTOUT_PERIOD_NUM) {
                periods.get(SHOOTOUT_INDEX).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^width$}", so_value, width = PERIOD_COL_WIDTH));
        }
    } else {
        output.push_str(&format!("{:^width$}", "-", width = PERIOD_COL_WIDTH)); // P1
        output.push('│');
        output.push_str(&format!("{:^width$}", "-", width = PERIOD_COL_WIDTH)); // P2
        output.push('│');
        output.push_str(&format!("{:^width$}", "-", width = PERIOD_COL_WIDTH)); // P3

        if has_ot {
            output.push('│');
            output.push_str(&format!("{:^width$}", "-", width = PERIOD_COL_WIDTH)); // OT
        }

        if has_so {
            output.push('│');
            output.push_str(&format!("{:^width$}", "-", width = PERIOD_COL_WIDTH)); // SO
        }
    }
}

/// Build a team row with period scores
fn build_team_row(
    team_abbrev: &str,
    team_score: Option<i32>,
    team_periods: Option<&Vec<i32>>,
    has_ot: bool,
    has_so: bool,
    total_cols: usize,
    max_width: usize,
    should_show_period: &impl Fn(i32) -> bool,
) -> String {
    let mut row = String::new();
    row.push('│');
    row.push_str(&format!("{:^5}", team_abbrev));
    row.push('│');

    render_team_periods(&mut row, team_periods, has_ot, has_so, should_show_period);

    row.push('│');
    row.push_str(&format!("{:^4}", team_score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
    row.push_str("│");

    let padding = calculate_padding(total_cols, max_width);
    if padding > 0 {
        row.push_str(&" ".repeat(padding));
    }
    row.push('\n');
    row
}

/// Extract period scores from GameSummary
pub fn extract_period_scores(summary: &GameSummary) -> PeriodScores {
    let mut away_periods = vec![0, 0, 0]; // P1, P2, P3
    let mut home_periods = vec![0, 0, 0];
    let mut has_ot = false;
    let mut has_so = false;

    let mut prev_away_score = 0;
    let mut prev_home_score = 0;

    for period in &summary.scoring {
        let period_num = period.period_descriptor.number as usize;

        // Determine if this is OT or SO
        if period.period_descriptor.period_type == "OT" {
            has_ot = true;
            // Ensure we have enough slots (up to OVERTIME_INDEX + 1)
            if away_periods.len() < OVERTIME_INDEX + 1 {
                away_periods.push(0);
                home_periods.push(0);
            }
        } else if period.period_descriptor.period_type == "SO" {
            has_so = true;
            // Ensure we have enough slots (up to SHOOTOUT_INDEX + 1)
            while away_periods.len() < SHOOTOUT_INDEX + 1 {
                away_periods.push(0);
                home_periods.push(0);
            }
        }

        // Get the final score after this period (from last goal)
        if let Some(last_goal) = period.goals.last() {
            let period_away_score = last_goal.away_score;
            let period_home_score = last_goal.home_score;

            // Calculate goals scored in this period
            let away_goals_in_period = period_away_score - prev_away_score;
            let home_goals_in_period = period_home_score - prev_home_score;

            // Store in the appropriate slot
            let idx = if period.period_descriptor.period_type == "REG" {
                (period_num - 1).min(PERIOD_3_INDEX) // P1=0, P2=1, P3=2
            } else if period.period_descriptor.period_type == "OT" {
                OVERTIME_INDEX // OT slot
            } else if period.period_descriptor.period_type == "SO" {
                SHOOTOUT_INDEX // SO slot
            } else {
                continue;
            };

            if idx < away_periods.len() {
                away_periods[idx] = away_goals_in_period;
                home_periods[idx] = home_goals_in_period;
            }

            prev_away_score = period_away_score;
            prev_home_score = period_home_score;
        }
    }

    PeriodScores {
        away_periods,
        home_periods,
        has_ot,
        has_so,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_columns_from_width() {
        // Wide terminal should give 3 columns
        assert_eq!(calculate_columns_from_width(Some(120)), 3);
        assert_eq!(calculate_columns_from_width(Some(115)), 3);

        // Medium terminal should give 2 columns
        assert_eq!(calculate_columns_from_width(Some(100)), 2);
        assert_eq!(calculate_columns_from_width(Some(76)), 2);

        // Narrow terminal should give 1 column
        assert_eq!(calculate_columns_from_width(Some(75)), 1);
        assert_eq!(calculate_columns_from_width(Some(50)), 1);

        // None should default to 1 column
        assert_eq!(calculate_columns_from_width(None), 1);
    }

    #[test]
    fn test_calculate_padding() {
        // Test basic padding calculation
        // With 5 total_cols: current_width = 1 + 5 + (5-1)*5 + 1 = 27
        // Padding = 37 - 27 = 10
        assert_eq!(calculate_padding(5, 37), 10);

        // With 7 total_cols: current_width = 1 + 5 + (7-1)*5 + 1 = 37
        // Padding = 37 - 37 = 0
        assert_eq!(calculate_padding(7, 37), 0);
    }

    #[test]
    fn test_build_top_border() {
        let border = build_top_border(5, 37);
        assert!(border.starts_with('╭'));
        assert!(border.contains('┬'));
        assert!(border.contains('╮'));
        assert!(border.ends_with('\n'));
    }

    #[test]
    fn test_build_middle_border() {
        let border = build_middle_border(5, 37);
        assert!(border.starts_with('├'));
        assert!(border.contains('┼'));
        assert!(border.contains('┤'));
        assert!(border.ends_with('\n'));
    }

    #[test]
    fn test_build_bottom_border() {
        let border = build_bottom_border(5, 37);
        assert!(border.starts_with('╰'));
        assert!(border.contains('┴'));
        assert!(border.contains('╯'));
        assert!(border.ends_with('\n'));
    }

    #[test]
    fn test_build_header_row_basic() {
        let header = build_header_row(false, false, 5, 37);
        assert!(header.contains('│'));
        assert!(header.contains('1'));
        assert!(header.contains('2'));
        assert!(header.contains('3'));
        assert!(header.contains('T'));
        assert!(!header.contains("OT"));
        assert!(!header.contains("SO"));
    }

    #[test]
    fn test_build_header_row_with_ot() {
        let header = build_header_row(true, false, 6, 37);
        assert!(header.contains("OT"));
        assert!(!header.contains("SO"));
    }

    #[test]
    fn test_build_header_row_with_shootout() {
        let header = build_header_row(true, true, 7, 37);
        assert!(header.contains("OT"));
        assert!(header.contains("SO"));
    }

    #[test]
    fn test_build_score_table_no_scores() {
        let table = build_score_table(
            "TOR",
            "MTL",
            None,
            None,
            false,
            false,
            None,
            None,
            None,
        );

        // Should contain team names
        assert!(table.contains("TOR"));
        assert!(table.contains("MTL"));

        // Should contain dashes for no scores
        assert!(table.contains('-'));

        // Should have proper box-drawing characters
        assert!(table.contains('╭'));
        assert!(table.contains('╰'));
        assert!(table.contains('│'));
    }

    #[test]
    fn test_build_score_table_with_scores() {
        let away_periods = vec![1, 2, 0];
        let home_periods = vec![0, 1, 2];

        let table = build_score_table(
            "BOS",
            "NYR",
            Some(3),
            Some(3),
            false,
            false,
            Some(&away_periods),
            Some(&home_periods),
            None,
        );

        // Should contain team names
        assert!(table.contains("BOS"));
        assert!(table.contains("NYR"));

        // Should contain final scores
        assert!(table.contains('3'));

        // Should have period scores
        assert!(table.contains('1'));
        assert!(table.contains('2'));
        assert!(table.contains('0'));
    }

    #[test]
    fn test_build_score_table_with_overtime() {
        let away_periods = vec![1, 1, 1, 1];
        let home_periods = vec![1, 1, 1, 0];

        let table = build_score_table(
            "EDM",
            "VAN",
            Some(4),
            Some(3),
            true,
            false,
            Some(&away_periods),
            Some(&home_periods),
            None,
        );

        // Should contain OT header
        assert!(table.contains("OT"));

        // Should show OT score
        assert!(table.contains('4')); // Away total
        assert!(table.contains('3')); // Home total
    }

    #[test]
    fn test_period_scores_struct() {
        let scores = PeriodScores {
            away_periods: vec![1, 2, 3],
            home_periods: vec![0, 1, 2],
            has_ot: false,
            has_so: false,
        };

        assert_eq!(scores.away_periods.len(), 3);
        assert_eq!(scores.home_periods.len(), 3);
        assert!(!scores.has_ot);
        assert!(!scores.has_so);
    }

    #[test]
    fn test_combine_tables_horizontally_empty() {
        let result = combine_tables_horizontally(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_combine_tables_horizontally_single() {
        let table = "Line 1\nLine 2\nLine 3\n".to_string();
        let result = combine_tables_horizontally(&[table.clone()]);
        assert_eq!(result, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_combine_tables_horizontally_multiple() {
        let table1 = "A1\nA2\nA3".to_string();
        let table2 = "B1\nB2\nB3".to_string();
        let result = combine_tables_horizontally(&[table1, table2]);

        // Should have both tables side by side with 2-space gap
        assert!(result.contains("A1  B1"));
        assert!(result.contains("A2  B2"));
        assert!(result.contains("A3  B3"));
    }

    #[test]
    fn test_format_period_text_regular() {
        assert_eq!(format_period_text("REG", 1), "1st Period");
        assert_eq!(format_period_text("REG", 2), "2nd Period");
        assert_eq!(format_period_text("REG", 3), "3rd Period");
        assert_eq!(format_period_text("REG", 4), "4th Period");
    }

    #[test]
    fn test_format_period_text_overtime() {
        assert_eq!(format_period_text("OT", 4), "Overtime");
    }

    #[test]
    fn test_format_period_text_shootout() {
        assert_eq!(format_period_text("SO", 5), "Shootout");
    }

    #[test]
    fn test_format_period_text_unknown() {
        assert_eq!(format_period_text("UNKNOWN", 1), "Period 1");
    }
}
