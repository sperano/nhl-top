/// StandingsTable widget - displays NHL standings with team statistics
///
/// This widget renders a table showing team standings with columns for:
/// - Team name
/// - Games Played (GP)
/// - Wins (W)
/// - Losses (L)
/// - Overtime Losses (OT)
/// - Points (PTS)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Column width constants
const TEAM_NAME_COL_WIDTH: usize = 25;
const GP_COL_WIDTH: usize = 3;
const W_COL_WIDTH: usize = 3;
const L_COL_WIDTH: usize = 3;
const OT_COL_WIDTH: usize = 3;
const PTS_COL_WIDTH: usize = 4;
const TABLE_WIDTH: usize = 48; // Total width including margins

/// Widget for displaying NHL standings table
#[derive(Debug)]
pub struct StandingsTable<'a> {
    /// Teams to display in the table
    pub teams: &'a [nhl_api::Standing],
    /// Optional header text (e.g., "Atlantic Division")
    pub header: Option<&'a str>,
    /// Optional playoff cutoff index (draws line after this team)
    pub playoff_cutoff_after: Option<usize>,
    /// Index of the selected team (for highlighting)
    pub selected_index: Option<usize>,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> StandingsTable<'a> {
    /// Create a new StandingsTable widget
    pub fn new(
        teams: &'a [nhl_api::Standing],
        header: Option<&'a str>,
        playoff_cutoff_after: Option<usize>,
        selected_index: Option<usize>,
        margin: u16,
    ) -> Self {
        Self {
            teams,
            header,
            playoff_cutoff_after,
            selected_index,
            margin,
        }
    }

    /// Calculate the total height needed for this table
    fn calculate_height(&self) -> u16 {
        let mut height = 0;

        // Header (if present): double-line header is 3 lines
        if self.header.is_some() {
            height += 3;
        }

        // Table header + separator
        height += 2;

        // Team rows
        height += self.teams.len() as u16;

        // Playoff cutoff line (if present)
        if self.playoff_cutoff_after.is_some() {
            height += 1;
        }

        height
    }

    /// Get the appropriate style based on whether a team is selected
    fn get_team_style(&self, team_index: usize, config: &DisplayConfig) -> Style {
        if Some(team_index) == self.selected_index {
            Style::default().fg(config.selection_fg)
        } else {
            Style::default()
        }
    }
}

impl<'a> RenderableWidget for StandingsTable<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;
        let margin = self.margin;

        // Render header if present
        if let Some(header_text) = &self.header {
            if y < area.bottom() {
                // Render double-line header with box characters
                let header_line = crate::formatting::format_header(header_text, true, config);
                for line in header_line.lines() {
                    if y >= area.bottom() {
                        break;
                    }
                    if !line.is_empty() {
                        let formatted = format!("{}{}", " ".repeat(margin as usize), line);
                        buf.set_string(
                            area.x,
                            y,
                            &formatted,
                            Style::default().fg(config.division_header_fg),
                        );
                    }
                    y += 1;
                }
            }
        }

        // Render table header
        if y < area.bottom() {
            let header = format!(
                "{}{:<team_width$} {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}",
                " ".repeat(margin as usize),
                "Team", "GP", "W", "L", "OT", "PTS",
                team_width = TEAM_NAME_COL_WIDTH,
                gp_width = GP_COL_WIDTH,
                w_width = W_COL_WIDTH,
                l_width = L_COL_WIDTH,
                ot_width = OT_COL_WIDTH,
                pts_width = PTS_COL_WIDTH
            );
            buf.set_string(area.x, y, &header, Style::default());
            y += 1;
        }

        // Render separator line
        if y < area.bottom() {
            let separator = format!(
                "{}{}",
                " ".repeat(margin as usize),
                config.box_chars.horizontal.repeat(TABLE_WIDTH - margin as usize)
            );
            buf.set_string(area.x, y, &separator, Style::default());
            y += 1;
        }

        // Render team rows
        for (idx, team) in self.teams.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let team_name = &team.team_common_name.default;
            let style = self.get_team_style(idx, config);

            // Format team name part (with selection styling if selected)
            let team_part = format!("{:<width$}", team_name, width = TEAM_NAME_COL_WIDTH);

            // Format stats part (always unstyled)
            let stats_part = format!(
                " {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}",
                team.games_played(),
                team.wins,
                team.losses,
                team.ot_losses,
                team.points,
                gp_width = GP_COL_WIDTH,
                w_width = W_COL_WIDTH,
                l_width = L_COL_WIDTH,
                ot_width = OT_COL_WIDTH,
                pts_width = PTS_COL_WIDTH
            );

            // Render margin
            buf.set_string(
                area.x,
                y,
                &" ".repeat(margin as usize),
                Style::default(),
            );

            // Render team name with selection style if selected
            buf.set_string(
                area.x + margin,
                y,
                &team_part,
                style,
            );

            // Render stats (always unstyled)
            buf.set_string(
                area.x + margin + team_part.len() as u16,
                y,
                &stats_part,
                Style::default(),
            );

            y += 1;

            // Draw playoff cutoff line if this is the cutoff position
            if Some(idx) == self.playoff_cutoff_after && y < area.bottom() {
                let cutoff_line = format!(
                    "{}{}",
                    " ".repeat(margin as usize),
                    config.box_chars.horizontal.repeat(TABLE_WIDTH - margin as usize)
                );
                buf.set_string(area.x, y, &cutoff_line, Style::default());
                y += 1;
            }
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(self.calculate_height())
    }

    fn preferred_width(&self) -> Option<u16> {
        Some(TABLE_WIDTH as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    fn create_test_team(name: &str, _gp: i32, w: i32, l: i32, ot: i32, pts: i32) -> nhl_api::Standing {
        nhl_api::Standing {
            team_common_name: nhl_api::LocalizedString {
                default: name.to_string(),
            },
            team_abbrev: nhl_api::LocalizedString {
                default: name[..3].to_uppercase(),
            },
            wins: w,
            losses: l,
            ot_losses: ot,
            points: pts,
            conference_abbrev: Some("".to_string()),
            conference_name: Some("".to_string()),
            division_abbrev: "".to_string(),
            division_name: "".to_string(),
            team_name: nhl_api::LocalizedString {
                default: name.to_string(),
            },
            team_logo: "".to_string(),
        }
    }

    #[test]
    fn test_standings_table_empty() {
        let teams = vec![];
        let widget = StandingsTable::new(&teams, None, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        // Should show header and separator only
        assert!(buffer_line(&buf, 0).contains("Team"));
        assert!(buffer_line(&buf, 1).contains(&config.box_chars.horizontal));
    }

    #[test]
    fn test_standings_table_with_teams() {
        let teams = vec![
            create_test_team("Toronto Maple Leafs", 10, 6, 3, 1, 13),
            create_test_team("Montreal Canadiens", 10, 5, 4, 1, 11),
            create_test_team("Boston Bruins", 10, 4, 5, 1, 9),
        ];

        let widget = StandingsTable::new(&teams, None, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        // Should show header
        assert!(buffer_line(&buf, 0).contains("Team"));
        assert!(buffer_line(&buf, 0).contains("GP"));
        assert!(buffer_line(&buf, 0).contains("PTS"));

        // Should show separator
        assert!(buffer_line(&buf, 1).contains(&config.box_chars.horizontal));

        // Should show teams
        assert!(buffer_line(&buf, 2).contains("Toronto Maple Leafs"));
        assert!(buffer_line(&buf, 2).contains("13")); // Points
        assert!(buffer_line(&buf, 3).contains("Montreal Canadiens"));
        assert!(buffer_line(&buf, 4).contains("Boston Bruins"));
    }

    #[test]
    fn test_standings_table_with_header() {
        let teams = vec![
            create_test_team("Toronto Maple Leafs", 10, 6, 3, 1, 13),
        ];
        let header = "Atlantic Division";

        let widget = StandingsTable::new(
            &teams,
            Some(header),
            None,
            None,
            2,
        );
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        // Should show division header
        let header_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Atlantic Division")
        });
        assert!(header_found, "Division header should be present");

        // Should show team
        let team_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("Toronto Maple Leafs")
        });
        assert!(team_found, "Team should be present");
    }

    #[test]
    fn test_standings_table_with_selection() {
        let teams = vec![
            create_test_team("Toronto Maple Leafs", 10, 6, 3, 1, 13),
            create_test_team("Montreal Canadiens", 10, 5, 4, 1, 11),
            create_test_team("Boston Bruins", 10, 4, 5, 1, 9),
        ];

        let widget = StandingsTable::new(&teams, None, None, Some(1), 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        // Second team (Montreal) should be on line 3 (0=header, 1=separator, 2=first team, 3=second team)
        let line = buffer_line(&buf, 3);
        assert!(line.contains("Montreal Canadiens"));

        // Note: We can't easily test the actual color in buffer_line, but we verify the team is there
    }

    #[test]
    fn test_standings_table_with_playoff_cutoff() {
        let teams = vec![
            create_test_team("Toronto Maple Leafs", 10, 6, 3, 1, 13),
            create_test_team("Montreal Canadiens", 10, 5, 4, 1, 11),
            create_test_team("Boston Bruins", 10, 4, 5, 1, 9),
        ];

        let widget = StandingsTable::new(&teams, None, Some(1), None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        // Should have cutoff line after second team
        // Line 4 should be the cutoff line (after Montreal at line 3)
        let cutoff_line = buffer_line(&buf, 4);
        assert!(cutoff_line.contains(&config.box_chars.horizontal));
    }

    #[test]
    fn test_standings_table_preferred_dimensions() {
        let teams = vec![
            create_test_team("Team A", 10, 5, 5, 0, 10),
            create_test_team("Team B", 10, 4, 6, 0, 8),
        ];

        let widget = StandingsTable::new(&teams, None, None, None, 2);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(1) + separator(1) + 2 teams = 4
        assert_eq!(widget.preferred_height(), Some(4));
    }

    #[test]
    fn test_standings_table_height_with_header_and_cutoff() {
        let teams = vec![
            create_test_team("Team A", 10, 5, 5, 0, 10),
            create_test_team("Team B", 10, 4, 6, 0, 8),
        ];
        let header = "Division";

        let widget = StandingsTable::new(
            &teams,
            Some(header),
            Some(0),
            None,
            2,
        );

        // Height should be: division header(3) + table header(1) + separator(1) + 2 teams(2) + cutoff(1) = 8
        assert_eq!(widget.preferred_height(), Some(8));
    }
}
