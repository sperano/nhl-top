/// CareerStatsTable widget - displays player career statistics by season
///
/// This widget renders a table showing season-by-season statistics with columns for:
/// - Season (year range)
/// - Team name
/// - Games Played (GP)
/// - Goals (G)
/// - Assists (A)
/// - Points (PTS)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;
use crate::tui::widgets::horizontal_separator::render_horizontal_separator;

/// Column width constants
const SEASON_COL_WIDTH: usize = 10;
const TEAM_COL_WIDTH: usize = 20;
const GP_COL_WIDTH: usize = 4;
const G_COL_WIDTH: usize = 4;
const A_COL_WIDTH: usize = 4;
const PTS_COL_WIDTH: usize = 5;
const TABLE_WIDTH: usize = 54; // Total width including margins

/// Widget for displaying career statistics table
#[derive(Debug)]
pub struct CareerStatsTable<'a> {
    /// Season stats to display in the table
    pub seasons: &'a [nhl_api::SeasonTotal],
    /// Optional header text (e.g., "NHL Career Statistics")
    pub header: Option<&'a str>,
    /// Index of the selected season (for highlighting)
    pub selected_index: Option<usize>,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> CareerStatsTable<'a> {
    /// Create a new CareerStatsTable widget
    pub fn new(
        seasons: &'a [nhl_api::SeasonTotal],
        header: Option<&'a str>,
        selected_index: Option<usize>,
        margin: u16,
    ) -> Self {
        Self {
            seasons,
            header,
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

        // Season rows
        height += self.seasons.len() as u16;

        // Blank line after table
        height += 1;

        height
    }

    /// Get the appropriate style based on whether a season is selected
    fn get_season_style(&self, season_index: usize, config: &DisplayConfig) -> Style {
        if Some(season_index) == self.selected_index {
            Style::default().fg(config.selection_fg)
        } else {
            Style::default()
        }
    }

    /// Format season as year range (e.g., 20232024 -> 2023-2024)
    fn format_season(season: i32) -> String {
        let start_year = season / 10000;
        let end_year = season % 10000;
        format!("{}-{}", start_year, end_year)
    }
}

impl<'a> RenderableWidget for CareerStatsTable<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;
        let margin = self.margin;

        // Render header if present
        if let Some(header_text) = &self.header {
            y += render_section_header(header_text, true, margin, area, y, buf, config);
        }

        // Render table header
        if y < area.bottom() {
            let header = format!(
                "{}{:<season_width$} {:<team_width$} {:>gp_width$} {:>g_width$} {:>a_width$} {:>pts_width$}",
                " ".repeat(margin as usize),
                "Season", "Team", "GP", "G", "A", "PTS",
                season_width = SEASON_COL_WIDTH,
                team_width = TEAM_COL_WIDTH,
                gp_width = GP_COL_WIDTH,
                g_width = G_COL_WIDTH,
                a_width = A_COL_WIDTH,
                pts_width = PTS_COL_WIDTH
            );
            buf.set_string(area.x, y, &header, Style::default());
            y += 1;
        }

        // Render separator line
        y += render_horizontal_separator(TABLE_WIDTH, margin, area, y, buf, config);

        // Render season rows (in reverse order for most recent first)
        for (idx, season) in self.seasons.iter().rev().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let style = self.get_season_style(idx, config);

            // Format the entire row
            let row = format!(
                "{}{:<season_width$} {:<team_width$} {:>gp_width$} {:>g_width$} {:>a_width$} {:>pts_width$}",
                " ".repeat(margin as usize),
                Self::format_season(season.season),
                season.team_name.default,
                season.games_played,
                season.goals.unwrap_or(0),
                season.assists.unwrap_or(0),
                season.points.unwrap_or(0),
                season_width = SEASON_COL_WIDTH,
                team_width = TEAM_COL_WIDTH,
                gp_width = GP_COL_WIDTH,
                g_width = G_COL_WIDTH,
                a_width = A_COL_WIDTH,
                pts_width = PTS_COL_WIDTH
            );

            buf.set_string(area.x, y, &row, style);
            y += 1;
        }

        // Blank line after table
        if y < area.bottom() {
            buf.set_string(area.x, y, "", Style::default());
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
    use nhl_api::LocalizedString;

    fn create_test_season(
        season: i32,
        team: &str,
        gp: i32,
        g: i32,
        a: i32,
        pts: i32
    ) -> nhl_api::SeasonTotal {
        nhl_api::SeasonTotal {
            season,
            game_type_id: 2,
            league_abbrev: "NHL".to_string(),
            team_name: LocalizedString {
                default: team.to_string(),
            },
            sequence: None,
            games_played: gp,
            goals: Some(g),
            assists: Some(a),
            points: Some(pts),
            plus_minus: None,
            pim: None,
        }
    }

    #[test]
    fn test_career_stats_table_empty() {
        let seasons = vec![];
        let widget = CareerStatsTable::new(&seasons, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Should show header and separator only
        assert!(buffer_line(&buf, 0).contains("Season"));
        assert!(buffer_line(&buf, 1).contains(&config.box_chars.horizontal));
    }

    #[test]
    fn test_career_stats_table_with_seasons() {
        let seasons = vec![
            create_test_season(20222023, "Toronto Maple Leafs", 78, 40, 46, 86),
            create_test_season(20232024, "Toronto Maple Leafs", 81, 69, 38, 107),
        ];

        let widget = CareerStatsTable::new(&seasons, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Should show header
        assert!(buffer_line(&buf, 0).contains("Season"));
        assert!(buffer_line(&buf, 0).contains("Team"));
        assert!(buffer_line(&buf, 0).contains("PTS"));

        // Should show separator
        assert!(buffer_line(&buf, 1).contains(&config.box_chars.horizontal));

        // Seasons should be in reverse order (most recent first)
        assert!(buffer_line(&buf, 2).contains("2023-2024")); // Most recent
        assert!(buffer_line(&buf, 2).contains("107")); // Points
        assert!(buffer_line(&buf, 3).contains("2022-2023")); // Earlier season
        assert!(buffer_line(&buf, 3).contains("86")); // Points
    }

    #[test]
    fn test_career_stats_table_with_header() {
        let seasons = vec![
            create_test_season(20232024, "Toronto Maple Leafs", 81, 69, 38, 107),
        ];
        let header = "NHL Career Statistics";

        let widget = CareerStatsTable::new(
            &seasons,
            Some(header),
            None,
            2,
        );
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Should show section header
        let header_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("NHL Career Statistics")
        });
        assert!(header_found, "Section header should be present");

        // Should show season
        let season_found = (0..height).any(|y| {
            buffer_line(&buf, y).contains("2023-2024")
        });
        assert!(season_found, "Season should be present");
    }

    #[test]
    fn test_career_stats_table_with_selection() {
        let seasons = vec![
            create_test_season(20222023, "Toronto Maple Leafs", 78, 40, 46, 86),
            create_test_season(20232024, "Toronto Maple Leafs", 81, 69, 38, 107),
            create_test_season(20242025, "Toronto Maple Leafs", 30, 25, 20, 45),
        ];

        let widget = CareerStatsTable::new(&seasons, None, Some(1), 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Second season in reverse order (2023-2024) should be on line 3
        let line = buffer_line(&buf, 3);
        assert!(line.contains("2023-2024"));

        // Note: We can't easily test the actual color in buffer_line, but we verify the season is there
    }

    #[test]
    fn test_career_stats_table_preferred_dimensions() {
        let seasons = vec![
            create_test_season(20222023, "Team A", 50, 20, 30, 50),
            create_test_season(20232024, "Team B", 60, 25, 35, 60),
        ];

        let widget = CareerStatsTable::new(&seasons, None, None, 2);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(1) + separator(1) + 2 seasons + blank(1) = 5
        assert_eq!(widget.preferred_height(), Some(5));
    }

    #[test]
    fn test_career_stats_table_height_with_header() {
        let seasons = vec![
            create_test_season(20222023, "Team A", 50, 20, 30, 50),
            create_test_season(20232024, "Team B", 60, 25, 35, 60),
        ];
        let header = "NHL Career Statistics";

        let widget = CareerStatsTable::new(
            &seasons,
            Some(header),
            None,
            2,
        );

        // Height should be: section header(3) + table header(1) + separator(1) + 2 seasons + blank(1) = 8
        assert_eq!(widget.preferred_height(), Some(8));
    }

    #[test]
    fn test_career_stats_table_season_formatting() {
        let seasons = vec![
            create_test_season(20232024, "Team A", 82, 50, 50, 100),
        ];

        let widget = CareerStatsTable::new(&seasons, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Check that season is formatted correctly
        let line = buffer_line(&buf, 2);
        assert!(line.contains("2023-2024"));
    }

    #[test]
    fn test_career_stats_table_handles_none_values() {
        let mut season = create_test_season(20232024, "Team A", 10, 0, 0, 0);
        season.goals = None;
        season.assists = None;
        season.points = None;

        let seasons = vec![season];
        let widget = CareerStatsTable::new(&seasons, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        // Should display 0 for None values
        let line = buffer_line(&buf, 2);
        assert!(line.contains("2023-2024"));
        // The line should contain the zeros rendered from None values
        assert!(line.contains("0")); // At least one zero should be present
    }
}
