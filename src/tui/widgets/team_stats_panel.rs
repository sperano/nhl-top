/// TeamStatsPanel widget - composition widget for team game statistics
///
/// This widget combines three stat tables:
/// 1. Forwards skater stats
/// 2. Defense skater stats
/// 3. Goalie stats
///
/// The tables are rendered vertically stacked with proper spacing.

use ratatui::{buffer::Buffer, layout::Rect};
use crate::config::DisplayConfig;
use crate::tui::widgets::{RenderableWidget, GameSkaterStatsTable, GameGoalieStatsTable};

/// Widget for displaying complete team game statistics panel
#[derive(Debug)]
pub struct TeamStatsPanel<'a> {
    /// Team abbreviation (e.g., "TOR")
    pub team_abbrev: &'a str,
    /// Team player statistics containing forwards, defense, and goalies
    pub stats: &'a nhl_api::TeamPlayerStats,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> TeamStatsPanel<'a> {
    /// Create a new TeamStatsPanel widget
    pub fn new(
        team_abbrev: &'a str,
        stats: &'a nhl_api::TeamPlayerStats,
        margin: u16,
    ) -> Self {
        Self {
            team_abbrev,
            stats,
            margin,
        }
    }

    /// Calculate the total height needed for this panel
    fn calculate_height(&self) -> u16 {
        let forwards_table = GameSkaterStatsTable::new(
            self.team_abbrev,
            "Forwards",
            &self.stats.forwards,
            self.margin,
        );
        let defense_table = GameSkaterStatsTable::new(
            self.team_abbrev,
            "Defense",
            &self.stats.defense,
            self.margin,
        );
        let goalie_table = GameGoalieStatsTable::new(
            self.team_abbrev,
            &self.stats.goalies,
            self.margin,
        );

        let mut height = 0;
        height += forwards_table.preferred_height().unwrap_or(0);
        height += defense_table.preferred_height().unwrap_or(0);
        height += goalie_table.preferred_height().unwrap_or(0);
        height
    }
}

impl<'a> RenderableWidget for TeamStatsPanel<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;

        // Create and render forwards table
        let forwards_table = GameSkaterStatsTable::new(
            self.team_abbrev,
            "Forwards",
            &self.stats.forwards,
            self.margin,
        );
        let forwards_height = forwards_table.preferred_height().unwrap_or(0);
        if y < area.bottom() && forwards_height > 0 {
            let forwards_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: forwards_height.min(area.bottom().saturating_sub(y)),
            };
            forwards_table.render(forwards_area, buf, config);
            y += forwards_height;
        }

        // Create and render defense table
        let defense_table = GameSkaterStatsTable::new(
            self.team_abbrev,
            "Defense",
            &self.stats.defense,
            self.margin,
        );
        let defense_height = defense_table.preferred_height().unwrap_or(0);
        if y < area.bottom() && defense_height > 0 {
            let defense_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: defense_height.min(area.bottom().saturating_sub(y)),
            };
            defense_table.render(defense_area, buf, config);
            y += defense_height;
        }

        // Create and render goalie table
        let goalie_table = GameGoalieStatsTable::new(
            self.team_abbrev,
            &self.stats.goalies,
            self.margin,
        );
        let goalie_height = goalie_table.preferred_height().unwrap_or(0);
        if y < area.bottom() && goalie_height > 0 {
            let goalie_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: goalie_height.min(area.bottom().saturating_sub(y)),
            };
            goalie_table.render(goalie_area, buf, config);
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(self.calculate_height())
    }

    fn preferred_width(&self) -> Option<u16> {
        // Use the maximum width from the component tables
        // All tables should have the same width, so just use one
        let forwards_table = GameSkaterStatsTable::new(
            self.team_abbrev,
            "Forwards",
            &self.stats.forwards,
            self.margin,
        );
        forwards_table.preferred_width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;
    use nhl_api::LocalizedString;

    fn create_test_skater(
        number: i32,
        name: &str,
        pos: &str,
        g: i32,
        a: i32,
    ) -> nhl_api::SkaterStats {
        nhl_api::SkaterStats {
            player_id: 1,
            sweater_number: number,
            name: LocalizedString { default: name.to_string() },
            position: pos.to_string(),
            goals: g,
            assists: a,
            points: g + a,
            plus_minus: 0,
            pim: 0,
            hits: 0,
            power_play_goals: 0,
            sog: 0,
            faceoff_winning_pctg: 0.0,
            toi: "15:00".to_string(),
            blocked_shots: 0,
            shifts: 0,
            giveaways: 0,
            takeaways: 0,
        }
    }

    fn create_test_goalie(
        number: i32,
        name: &str,
        sa: i32,
        saves: i32,
    ) -> nhl_api::GoalieStats {
        nhl_api::GoalieStats {
            player_id: 1,
            sweater_number: number,
            name: LocalizedString { default: name.to_string() },
            position: "G".to_string(),
            shots_against: sa,
            saves,
            goals_against: sa - saves,
            save_pctg: Some((saves as f64) / (sa as f64)),
            pim: Some(0),
            toi: "60:00".to_string(),
            even_strength_shots_against: "0/0".to_string(),
            power_play_shots_against: "0/0".to_string(),
            shorthanded_shots_against: "0/0".to_string(),
            save_shots_against: format!("{}/{}", saves, sa),
            even_strength_goals_against: 0,
            power_play_goals_against: 0,
            shorthanded_goals_against: 0,
            starter: None,
            decision: None,
        }
    }

    #[test]
    fn test_team_stats_panel_empty() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![],
            defense: vec![],
            goalies: vec![],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 80, height, &config);

        // Should show all three section headers even with no players
        let forwards_header = (0..height).any(|y| {
            buffer_line(&buf, y).contains("TOR - Forwards")
        });
        assert!(forwards_header, "Forwards header should be present");

        let defense_header = (0..height).any(|y| {
            buffer_line(&buf, y).contains("TOR - Defense")
        });
        assert!(defense_header, "Defense header should be present");

        let goalies_header = (0..height).any(|y| {
            buffer_line(&buf, y).contains("TOR - Goalies")
        });
        assert!(goalies_header, "Goalies header should be present");
    }

    #[test]
    fn test_team_stats_panel_with_players() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![
                create_test_skater(34, "Auston Matthews", "C", 2, 1),
                create_test_skater(16, "Mitch Marner", "RW", 0, 3),
            ],
            defense: vec![
                create_test_skater(44, "Morgan Rielly", "D", 0, 2),
            ],
            goalies: vec![
                create_test_goalie(35, "Joseph Woll", 30, 28),
            ],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 80, height, &config);

        // Should show forwards
        let matthews_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Auston Matthews")
        });
        assert!(matthews_found, "Matthews should be present");

        let marner_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Mitch Marner")
        });
        assert!(marner_found, "Marner should be present");

        // Should show defense
        let rielly_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Morgan Rielly")
        });
        assert!(rielly_found, "Rielly should be present");

        // Should show goalie
        let woll_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Joseph Woll")
        });
        assert!(woll_found, "Woll should be present");
    }

    #[test]
    fn test_team_stats_panel_section_order() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![create_test_skater(34, "Forward Player", "C", 1, 1)],
            defense: vec![create_test_skater(44, "Defense Player", "D", 0, 1)],
            goalies: vec![create_test_goalie(35, "Goalie Player", 20, 18)],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 80, height, &config);

        // Find line numbers for each section
        let mut forwards_line = None;
        let mut defense_line = None;
        let mut goalies_line = None;

        for y in 0..height {
            let line = buffer_line(&buf, y);
            if line.contains("TOR - Forwards") {
                forwards_line = Some(y);
            } else if line.contains("TOR - Defense") {
                defense_line = Some(y);
            } else if line.contains("TOR - Goalies") {
                goalies_line = Some(y);
            }
        }

        // Verify order: Forwards -> Defense -> Goalies
        assert!(forwards_line.is_some(), "Forwards section should exist");
        assert!(defense_line.is_some(), "Defense section should exist");
        assert!(goalies_line.is_some(), "Goalies section should exist");

        assert!(
            forwards_line.unwrap() < defense_line.unwrap(),
            "Forwards should come before Defense"
        );
        assert!(
            defense_line.unwrap() < goalies_line.unwrap(),
            "Defense should come before Goalies"
        );
    }

    #[test]
    fn test_team_stats_panel_preferred_dimensions() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![create_test_skater(34, "Player A", "C", 1, 1)],
            defense: vec![create_test_skater(44, "Player B", "D", 0, 1)],
            goalies: vec![create_test_goalie(35, "Player C", 20, 18)],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 0);

        // Width should match the component tables (52)
        assert_eq!(widget.preferred_width(), Some(52));

        // Height should be sum of all three tables
        // Forwards: header(2) + table_header(1) + 1 player + blank(1) = 5
        // Defense: header(2) + table_header(1) + 1 player + blank(1) = 5
        // Goalies: header(2) + table_header(1) + 1 goalie + blank(1) = 5
        // Total = 15
        assert_eq!(widget.preferred_height(), Some(15));
    }

    #[test]
    fn test_team_stats_panel_height_calculation() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![
                create_test_skater(34, "F1", "C", 1, 1),
                create_test_skater(16, "F2", "RW", 0, 1),
                create_test_skater(88, "F3", "LW", 1, 0),
            ],
            defense: vec![
                create_test_skater(44, "D1", "D", 0, 1),
                create_test_skater(22, "D2", "D", 0, 0),
            ],
            goalies: vec![
                create_test_goalie(35, "G1", 25, 23),
            ],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 0);

        // Forwards: header(2) + table_header(1) + 3 players + blank(1) = 7
        // Defense: header(2) + table_header(1) + 2 players + blank(1) = 6
        // Goalies: header(2) + table_header(1) + 1 goalie + blank(1) = 5
        // Total = 18
        assert_eq!(widget.preferred_height(), Some(18));
    }

    #[test]
    fn test_team_stats_panel_with_margin() {
        let stats = nhl_api::TeamPlayerStats {
            forwards: vec![create_test_skater(34, "Player", "C", 1, 1)],
            defense: vec![],
            goalies: vec![],
        };

        let widget = TeamStatsPanel::new("TOR", &stats, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 80, height, &config);

        // Check that margin is applied (headers should be indented)
        let forwards_line = (0..height)
            .find(|&y| buffer_line(&buf, y).contains("TOR - Forwards"))
            .expect("Should find forwards header");

        let line = buffer_line(&buf, forwards_line);
        // Line should start with spaces due to margin
        assert!(line.starts_with("  "), "Header should be indented with margin");
    }
}
