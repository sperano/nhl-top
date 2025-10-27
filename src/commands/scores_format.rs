use nhl_api::{DailySchedule, ScheduleGame, GameSummary};
use std::collections::HashMap;

/// Period-by-period score data
#[derive(Debug, Clone)]
pub struct PeriodScores {
    pub away_periods: Vec<i32>,
    pub home_periods: Vec<i32>,
    pub has_ot: bool,
    pub has_so: bool,
}

/// Format scores for TUI display with period-by-period breakdown
pub fn format_scores_for_tui(
    schedule: &DailySchedule,
    period_scores: &HashMap<i64, PeriodScores>,
) -> String {
    let empty_game_info = HashMap::new();
    format_scores_for_tui_with_width(schedule, period_scores, &empty_game_info, None)
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
    // Each game box is 37 characters wide to accommodate all 5 periods (1, 2, 3, OT, SO)
    let num_columns = if let Some(width) = terminal_width {
        if width >= 115 {
            3 // 3 columns for wide terminals (115 = 37*3 + 2*2 gaps)
        } else if width >= 76 {
            2 // 2 columns for medium terminals (76 = 37*2 + 2 gap)
        } else {
            1 // 1 column for narrow terminals
        }
    } else {
        1 // Default to 1 column if width not provided
    };

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
                output.push_str("  "); // 2-space gap between tables
            }

            // Get the line or use empty space if this table is shorter
            if line_idx < lines.len() {
                output.push_str(lines[line_idx]);
            } else {
                // Pad with spaces to match table width (all 5 periods)
                output.push_str(&" ".repeat(37));
            }
        }
        output.push('\n');
    }

    output
}

fn format_game_table(game: &ScheduleGame, period_scores: Option<&PeriodScores>, game_info: Option<&nhl_api::GameMatchup>) -> String {
    let mut output = String::new();

    // Determine if game has started
    let game_started = game.game_state.has_started();

    // Add header based on game state
    let header = if game.game_state.is_final() {
        "Final Score".to_string()
    } else if game_started {
        // Game is in progress - show period and time
        if let Some(info) = game_info {
            let period_text = match info.period_descriptor.period_type.as_str() {
                "REG" => {
                    let ordinal = match info.period_descriptor.number {
                        1 => "1st",
                        2 => "2nd",
                        3 => "3rd",
                        n => return format!("{}th Period", n),
                    };
                    format!("{} Period", ordinal)
                },
                "OT" => "Overtime".to_string(),
                "SO" => "Shootout".to_string(),
                _ => format!("Period {}", info.period_descriptor.number),
            };

            if let Some(clock) = &info.clock {
                if clock.in_intermission {
                    format!("{} - Intermission", period_text)
                } else {
                    format!("{} - {}", period_text, clock.time_remaining)
                }
            } else {
                period_text
            }
        } else {
            "In Progress".to_string()
        }
    } else {
        // Game hasn't started - show start time
        // Parse and format the UTC time (format: "2024-10-25T23:00:00Z")
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
            let local_time: chrono::DateTime<chrono::Local> = parsed.into();
            local_time.format("%I:%M %p").to_string()
        } else {
            game.start_time_utc.clone()
        }
    };

    // Add 1 char left padding, then left-align the header and pad to 37 chars
    output.push_str(&format!(" {:<36}\n", header));

    // Determine current period for in-progress games
    let current_period_num = if game_started && !game.game_state.is_final() {
        game_info.and_then(|info| {
            // Get the current period number based on period type
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
    let base_cols = 5; // empty, 1, 2, 3, T
    let ot_cols = if has_ot { 1 } else { 0 };
    let so_cols = if has_so { 1 } else { 0 };
    let total_cols = base_cols + ot_cols + so_cols;

    // Top border
    let max_width = 37; // Width with all 5 periods
    output.push('╭');
    output.push_str(&"─".repeat(5)); // team name column
    for _ in 1..total_cols {
        output.push('┬');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("╮");

    // Pad to max width
    // Calculate actual width: 1 (╭) + 5 (team) + (total_cols-1) * (1 connector + 4 dashes) + 1 (╮)
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    // Header row
    output.push('│');
    output.push_str(&format!("{:^5}", ""));
    output.push('│');
    output.push_str(&format!("{:^4}", "1"));
    output.push('│');
    output.push_str(&format!("{:^4}", "2"));
    output.push('│');
    output.push_str(&format!("{:^4}", "3"));

    if has_ot {
        output.push('│');
        output.push_str(&format!("{:^4}", "OT"));
    }

    if has_so {
        output.push('│');
        output.push_str(&format!("{:^4}", "SO"));
    }

    output.push('│');
    output.push_str(&format!("{:^4}", "T"));
    output.push_str("│");

    // Pad to max width
    // Calculate: 1 (│) + 5 (team) + (total_cols-1) * (1 │ + 4 chars) + 1 (│)
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    // Middle border
    output.push('├');
    output.push_str(&"─".repeat(5));
    for _ in 1..total_cols {
        output.push('┼');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("┤");

    // Pad to max width
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    // Away team row
    output.push('│');
    output.push_str(&format!("{:^5}", away_team));
    output.push('│');

    // Helper to check if a period should show score or dash
    let should_show_period = |period: i32| -> bool {
        current_period_num.map_or(true, |current| period <= current)
    };

    // Period scores or placeholders
    if let Some(periods) = away_periods {
        // Period 1
        let p1_value = if should_show_period(1) {
            periods.get(0).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p1_value));
        output.push('│');

        // Period 2
        let p2_value = if should_show_period(2) {
            periods.get(1).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p2_value));
        output.push('│');

        // Period 3
        let p3_value = if should_show_period(3) {
            periods.get(2).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p3_value));

        if has_ot {
            output.push('│');
            let ot_value = if should_show_period(4) {
                periods.get(3).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^4}", ot_value));
        }

        if has_so {
            output.push('│');
            let so_value = if should_show_period(5) {
                periods.get(4).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^4}", so_value));
        }
    } else {
        output.push_str(&format!("{:^4}", "-")); // P1
        output.push('│');
        output.push_str(&format!("{:^4}", "-")); // P2
        output.push('│');
        output.push_str(&format!("{:^4}", "-")); // P3

        if has_ot {
            output.push('│');
            output.push_str(&format!("{:^4}", "-")); // OT
        }

        if has_so {
            output.push('│');
            output.push_str(&format!("{:^4}", "-")); // SO
        }
    }

    output.push('│');
    output.push_str(&format!("{:^4}", away_score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string()))); // Total
    output.push_str("│");

    // Pad to max width
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    // Home team row
    output.push('│');
    output.push_str(&format!("{:^5}", home_team));
    output.push('│');

    // Period scores or placeholders
    if let Some(periods) = home_periods {
        // Period 1
        let p1_value = if should_show_period(1) {
            periods.get(0).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p1_value));
        output.push('│');

        // Period 2
        let p2_value = if should_show_period(2) {
            periods.get(1).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p2_value));
        output.push('│');

        // Period 3
        let p3_value = if should_show_period(3) {
            periods.get(2).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        output.push_str(&format!("{:^4}", p3_value));

        if has_ot {
            output.push('│');
            let ot_value = if should_show_period(4) {
                periods.get(3).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^4}", ot_value));
        }

        if has_so {
            output.push('│');
            let so_value = if should_show_period(5) {
                periods.get(4).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
            } else {
                "-".to_string()
            };
            output.push_str(&format!("{:^4}", so_value));
        }
    } else {
        output.push_str(&format!("{:^4}", "-")); // P1
        output.push('│');
        output.push_str(&format!("{:^4}", "-")); // P2
        output.push('│');
        output.push_str(&format!("{:^4}", "-")); // P3

        if has_ot {
            output.push('│');
            output.push_str(&format!("{:^4}", "-")); // OT
        }

        if has_so {
            output.push('│');
            output.push_str(&format!("{:^4}", "-")); // SO
        }
    }

    output.push('│');
    output.push_str(&format!("{:^4}", home_score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string()))); // Total
    output.push_str("│");

    // Pad to max width
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    // Bottom border
    output.push('╰');
    output.push_str(&"─".repeat(5));
    for _ in 1..total_cols {
        output.push('┴');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("╯");

    // Pad to max width
    let current_width = 1 + 5 + (total_cols - 1) * 5 + 1;
    if current_width < max_width {
        output.push_str(&" ".repeat(max_width - current_width));
    }
    output.push('\n');

    output
}

/// Extract period scores from GameSummary
pub fn extract_period_scores(summary: &GameSummary, away_team_id: i64, home_team_id: i64) -> PeriodScores {
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
            // Ensure we have enough slots
            if away_periods.len() < 4 {
                away_periods.push(0);
                home_periods.push(0);
            }
        } else if period.period_descriptor.period_type == "SO" {
            has_so = true;
            // Ensure we have enough slots
            while away_periods.len() < 5 {
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
                (period_num - 1).min(2) // P1=0, P2=1, P3=2
            } else if period.period_descriptor.period_type == "OT" {
                3 // OT slot
            } else if period.period_descriptor.period_type == "SO" {
                4 // SO slot
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
