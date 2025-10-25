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
    let mut output = String::new();

    // Display header
    output.push_str(&format!("{}\n", schedule.date));
    //output.push_str(&format!("{}\n\n", "═".repeat(60)));

    if schedule.number_of_games == 0 {
        output.push_str("No games scheduled for today.\n");
        return output;
    }

    // Display each game
    for (i, game) in schedule.games.iter().enumerate() {
        if i > 0 {
            output.push_str("\n");
        }
        output.push_str(&format_game_table(game, period_scores.get(&game.id)));
    }

    output
}

fn format_game_table(game: &ScheduleGame, period_scores: Option<&PeriodScores>) -> String {
    let mut output = String::new();

    // Determine if game has started
    let game_started = game.game_state.has_started();
    let game_final = game.game_state.is_final();

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
                away_score,
                home_score,
                has_ot,
                has_so,
                away_periods,
                home_periods,
            ));
        } else {
            // Game started but no scores yet - show simple info
            output.push_str(&format!(
                "{} @ {} ({})\n",
                game.away_team.abbrev, game.home_team.abbrev, game.game_state
            ));
        }
    } else {
        // Game hasn't started - show simple info
        output.push_str(&format!(
            "{} @ {} - Scheduled: {}\n",
            game.away_team.abbrev, game.home_team.abbrev, game.start_time_utc
        ));
    }

    output
}

fn build_score_table(
    away_team: &str,
    home_team: &str,
    away_score: i32,
    home_score: i32,
    has_ot: bool,
    has_so: bool,
    away_periods: Option<&Vec<i32>>,
    home_periods: Option<&Vec<i32>>,
) -> String {
    let mut output = String::new();

    // Calculate column count (empty + 3 periods + OT? + SO? + Total)
    let base_cols = 5; // empty, 1, 2, 3, T
    let ot_cols = if has_ot { 1 } else { 0 };
    let so_cols = if has_so { 1 } else { 0 };
    let total_cols = base_cols + ot_cols + so_cols;

    // Top border
    output.push('╭');
    output.push_str(&"─".repeat(5)); // team name column
    for _ in 1..total_cols {
        output.push('┬');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("╮\n");

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
    output.push_str("│\n");

    // Middle border
    output.push('├');
    output.push_str(&"─".repeat(5));
    for _ in 1..total_cols {
        output.push('┼');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("┤\n");

    // Away team row
    output.push('│');
    output.push_str(&format!("{:^5}", away_team));
    output.push('│');

    // Period scores or placeholders
    if let Some(periods) = away_periods {
        output.push_str(&format!("{:^4}", periods.get(0).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        output.push('│');
        output.push_str(&format!("{:^4}", periods.get(1).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        output.push('│');
        output.push_str(&format!("{:^4}", periods.get(2).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));

        if has_ot {
            output.push('│');
            output.push_str(&format!("{:^4}", periods.get(3).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        }

        if has_so {
            output.push('│');
            output.push_str(&format!("{:^4}", periods.get(4).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
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
    output.push_str(&format!("{:^4}", away_score)); // Total
    output.push_str("│\n");

    // Home team row
    output.push('│');
    output.push_str(&format!("{:^5}", home_team));
    output.push('│');

    // Period scores or placeholders
    if let Some(periods) = home_periods {
        output.push_str(&format!("{:^4}", periods.get(0).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        output.push('│');
        output.push_str(&format!("{:^4}", periods.get(1).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        output.push('│');
        output.push_str(&format!("{:^4}", periods.get(2).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));

        if has_ot {
            output.push('│');
            output.push_str(&format!("{:^4}", periods.get(3).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
        }

        if has_so {
            output.push('│');
            output.push_str(&format!("{:^4}", periods.get(4).map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())));
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
    output.push_str(&format!("{:^4}", home_score)); // Total
    output.push_str("│\n");

    // Bottom border
    output.push('╰');
    output.push_str(&"─".repeat(5));
    for _ in 1..total_cols {
        output.push('┴');
        output.push_str(&"─".repeat(4));
    }
    output.push_str("╯\n");

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
