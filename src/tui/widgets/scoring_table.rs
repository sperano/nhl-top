/// ScoringTable widget - displays goal scoring summary by period
///
/// This widget renders a formatted table showing goals scored in each period,
/// including scorer, assists, time, and shot type information.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Widget for displaying goal scoring summary
#[derive(Debug, Clone)]
pub struct ScoringTable {
    /// Scoring data by period
    pub scoring: Vec<nhl_api::PeriodScoring>,
}

impl ScoringTable {
    /// Create a new ScoringTable widget
    pub fn new(scoring: Vec<nhl_api::PeriodScoring>) -> Self {
        Self { scoring }
    }
}

/// Column widths for the scoring table
#[derive(Debug, Clone)]
struct ScoringColumnWidths {
    team: usize,        // Column 1: always 5 (space + 3-letter abbrev + space)
    description: usize, // Column 2: dynamic based on longest name/assists
    score: usize,       // Column 3: dynamic based on max score digits
    time: usize,        // Column 4: always 7 (space + MM:SS + space)
    shot_type: usize,   // Column 5: dynamic based on longest shot type
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
                    goal.assists
                        .iter()
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

    /// Calculate total table width
    fn total_width(&self) -> usize {
        self.team + self.description + self.score + self.time + self.shot_type + 6 // +6 for borders
    }
}

impl RenderableWidget for ScoringTable {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.scoring.is_empty() {
            return;
        }

        let widths = ScoringColumnWidths::new(&self.scoring);
        let mut y = area.y;

        for period in &self.scoring {
            // Render period header
            let period_name = match period.period_descriptor.period_type.as_str() {
                "REG" => format!("{}st Period", period.period_descriptor.number)
                    .replace("1st", "1st")
                    .replace("2st", "2nd")
                    .replace("3st", "3rd"),
                "OT" => "Overtime".to_string(),
                "SO" => "Shootout".to_string(),
                _ => format!("Period {}", period.period_descriptor.number),
            };

            // Render period name
            if y < area.bottom() {
                buf.set_string(area.x, y, &period_name, Style::default());
                y += 1;
            }

            // Blank line after period name
            y += 1;

            if period.goals.is_empty() {
                // Render "No Goals"
                if y < area.bottom() {
                    buf.set_string(area.x, y, "No Goals", Style::default());
                    y += 1;
                }
                // Blank line after "No Goals"
                y += 1;
            } else {
                // Render top border
                if y < area.bottom() {
                    let border = build_scoring_border(
                        &widths,
                        &config.box_chars.top_left,
                        &config.box_chars.top_junction,
                        &config.box_chars.top_right,
                        &config.box_chars.horizontal,
                    );
                    buf.set_string(area.x, y, &border, Style::default());
                    y += 1;
                }

                // Render each goal
                for (i, goal) in period.goals.iter().enumerate() {
                    // Render goal scorer row
                    if y < area.bottom() {
                        let goal_row = format_goal_row(goal, &widths, &config.box_chars.vertical);
                        buf.set_string(area.x, y, &goal_row, Style::default());
                        y += 1;
                    }

                    // Render assists row
                    if y < area.bottom() {
                        let assists_row = format_assists_row(goal, &widths, &config.box_chars.vertical);
                        buf.set_string(area.x, y, &assists_row, Style::default());
                        y += 1;
                    }

                    // Render separator or bottom border
                    if y < area.bottom() {
                        let border = if i < period.goals.len() - 1 {
                            // Middle separator between goals
                            build_scoring_border(
                                &widths,
                                &config.box_chars.left_junction,
                                &config.box_chars.cross,
                                &config.box_chars.right_junction,
                                &config.box_chars.horizontal,
                            )
                        } else {
                            // Bottom border after last goal
                            build_scoring_border(
                                &widths,
                                &config.box_chars.bottom_left,
                                &config.box_chars.bottom_junction,
                                &config.box_chars.bottom_right,
                                &config.box_chars.horizontal,
                            )
                        };
                        buf.set_string(area.x, y, &border, Style::default());
                        y += 1;
                    }
                }

                // Blank line after table
                y += 1;
            }
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        if self.scoring.is_empty() {
            return Some(0);
        }

        let mut height = 0;
        for period in &self.scoring {
            height += 2; // Period name + blank line
            if period.goals.is_empty() {
                height += 2; // "No Goals" + blank line
            } else {
                height += 1; // Top border
                height += period.goals.len() * 2; // Each goal = 2 rows (scorer + assists)
                height += period.goals.len(); // Separators/borders
                height += 1; // Blank line after table
            }
        }
        Some(height as u16)
    }

    fn preferred_width(&self) -> Option<u16> {
        if self.scoring.is_empty() {
            return Some(0);
        }
        let widths = ScoringColumnWidths::new(&self.scoring);
        Some(widths.total_width() as u16)
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
    line
}

/// Format the goal scorer row
fn format_goal_row(goal: &nhl_api::GoalSummary, widths: &ScoringColumnWidths, vert: &str) -> String {
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
        "{} {:3} {} {:<desc_w$} {} {:<score_w$} {} {:5} {} {:<shot_w$} {}",
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
fn format_assists_row(goal: &nhl_api::GoalSummary, widths: &ScoringColumnWidths, vert: &str) -> String {
    let assists_str = if goal.assists.is_empty() {
        "Unassisted".to_string()
    } else {
        goal.assists
            .iter()
            .map(|a| format!("{} ({})", a.name.default, a.assists_to_date))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "{} {:3} {} {:<desc_w$} {} {:<score_w$} {} {:5} {} {:shot_w$} {}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    fn create_test_goal(
        team_abbrev: &str,
        name: &str,
        goals_to_date: i32,
        assists: Vec<(&str, i32)>,
        away_score: i32,
        home_score: i32,
        time: &str,
        shot_type: &str,
    ) -> nhl_api::GoalSummary {
        nhl_api::GoalSummary {
            situation_code: "1551".to_string(),
            event_id: 100,
            strength: "EV".to_string(),
            player_id: 8000000,
            first_name: nhl_api::LocalizedString {
                default: name.split_whitespace().next().unwrap_or("").to_string(),
            },
            last_name: nhl_api::LocalizedString {
                default: name.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
            },
            name: nhl_api::LocalizedString {
                default: name.to_string(),
            },
            team_abbrev: nhl_api::LocalizedString {
                default: team_abbrev.to_string(),
            },
            headshot: "https://example.com/headshot.png".to_string(),
            highlight_clip_sharing_url: None,
            highlight_clip: None,
            discrete_clip: None,
            goals_to_date: Some(goals_to_date),
            away_score,
            home_score,
            leading_team_abbrev: None,
            time_in_period: time.to_string(),
            shot_type: shot_type.to_lowercase(),
            goal_modifier: "none".to_string(),
            assists: assists
                .into_iter()
                .enumerate()
                .map(|(i, (name, assists_to_date))| nhl_api::AssistSummary {
                    player_id: 8000001 + i as i64,
                    first_name: nhl_api::LocalizedString {
                        default: name.split_whitespace().next().unwrap_or("").to_string(),
                    },
                    last_name: nhl_api::LocalizedString {
                        default: name.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
                    },
                    name: nhl_api::LocalizedString {
                        default: name.to_string(),
                    },
                    assists_to_date,
                    sweater_number: 10 + i as i32,
                })
                .collect(),
            home_team_defending_side: "right".to_string(),
            is_home: false,
        }
    }

    #[test]
    fn test_scoring_table_empty() {
        let widget = ScoringTable::new(vec![]);
        let buf = render_widget(&widget, 80, 10);

        // Empty scoring should render nothing
        assert_buffer(&buf, &[
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
        ], 80);
    }

    #[test]
    fn test_scoring_table_single_goal() {
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

        let period = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widget = ScoringTable::new(vec![period]);
        let config = test_config();

        // Calculate required buffer size
        let width = widget.preferred_width().unwrap();
        let height = widget.preferred_height().unwrap();

        let buf = render_widget_with_config(&widget, width, height, &config);

        // Verify output matches expected format
        assert_buffer(&buf, &[
            "1st Period                                                    ",
            "                                                              ",
            "╭─────┬─────────────────────────────────┬─────┬───────┬──────╮",
            "│ OTT │ M. Amadio (4)                   │ 1-0 │ 5:42  │ Snap │",
            "│     │ S. Pinto (5), C. Giroux (7)     │ OTT │       │      │",
            "╰─────┴─────────────────────────────────┴─────┴───────┴──────╯",
            "                                                              ",
        ], 62);
    }

    #[test]
    fn test_scoring_table_no_goals() {
        let period = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![],
        };

        let widget = ScoringTable::new(vec![period]);
        let buf = render_widget(&widget, 80, 10);

        assert_buffer(&buf, &[
            "1st Period                                                                      ",
            "                                                                                ",
            "No Goals                                                                        ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
        ], 80);
    }

    #[test]
    fn test_scoring_table_multiple_periods() {
        let goal1 = create_test_goal("BOS", "M. Geekie", 10, vec![("A. Peeke", 3)], 9, 1, "01:22", "Poke");
        let goal2 = create_test_goal("MTL", "N. Suzuki", 15, vec![], 9, 2, "12:34", "Wrist");

        let period1 = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal1],
        };

        let period2 = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 2,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal2],
        };

        let widget = ScoringTable::new(vec![period1, period2]);
        let buf = render_widget(&widget, 80, 20);

        assert_buffer(&buf, &[
            "1st Period                                                                      ",
            "                                                                                ",
            "╭─────┬────────────────────┬─────┬───────┬───────╮                              ",
            "│ BOS │ M. Geekie (10)     │ 9-1 │ 01:22 │ Poke  │                              ",
            "│     │ A. Peeke (3)       │ BOS │       │       │                              ",
            "╰─────┴────────────────────┴─────┴───────┴───────╯                              ",
            "                                                                                ",
            "2nd Period                                                                      ",
            "                                                                                ",
            "╭─────┬────────────────────┬─────┬───────┬───────╮                              ",
            "│ MTL │ N. Suzuki (15)     │ 9-2 │ 12:34 │ Wrist │                              ",
            "│     │ Unassisted         │ MTL │       │       │                              ",
            "╰─────┴────────────────────┴─────┴───────┴───────╯                              ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
        ], 80);
    }

    #[test]
    fn test_scoring_table_overtime() {
        let goal = create_test_goal("TOR", "A. Matthews", 30, vec![("W. Nylander", 25)], 3, 2, "2:15", "Snap");

        let period = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 4,
                period_type: "OT".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widget = ScoringTable::new(vec![period]);
        let buf = render_widget(&widget, 80, 10);

        assert_buffer(&buf, &[
            "Overtime                                                                        ",
            "                                                                                ",
            "╭─────┬──────────────────────┬─────┬───────┬──────╮                             ",
            "│ TOR │ A. Matthews (30)     │ 3-2 │ 2:15  │ Snap │                             ",
            "│     │ W. Nylander (25)     │ TOR │       │      │                             ",
            "╰─────┴──────────────────────┴─────┴───────┴──────╯                             ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
        ], 80);
    }

    #[test]
    fn test_scoring_table_unassisted() {
        let goal = create_test_goal("MTL", "N. Suzuki", 15, vec![], 1, 0, "10:00", "Wrist");

        let period = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widget = ScoringTable::new(vec![period]);
        let buf = render_widget(&widget, 80, 10);

        assert_buffer(&buf, &[
            "1st Period                                                                      ",
            "                                                                                ",
            "╭─────┬────────────────────┬─────┬───────┬───────╮                              ",
            "│ MTL │ N. Suzuki (15)     │ 1-0 │ 10:00 │ Wrist │                              ",
            "│     │ Unassisted         │ MTL │       │       │                              ",
            "╰─────┴────────────────────┴─────┴───────┴───────╯                              ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
        ], 80);
    }

    #[test]
    fn test_preferred_dimensions() {
        let goal = create_test_goal("OTT", "M. Amadio", 4, vec![("S. Pinto", 5)], 1, 0, "5:42", "Snap");
        let period = nhl_api::PeriodScoring {
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 1,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            goals: vec![goal],
        };

        let widget = ScoringTable::new(vec![period]);

        // Should have preferred dimensions
        assert!(widget.preferred_width().is_some());
        assert!(widget.preferred_height().is_some());

        // Width should be reasonable
        let width = widget.preferred_width().unwrap();
        assert!(width > 40 && width < 150);

        // Height = period name (1) + blank (1) + top border (1) + goal rows (2) + bottom border (1) + blank (1) = 7
        assert_eq!(widget.preferred_height().unwrap(), 7);
    }
}
