/// GameGoalieStatsTable widget - displays goalie statistics from a game
///
/// This widget renders a table showing goalie statistics with columns for:
/// - # (sweater number)
/// - Name
/// - SA (shots against)
/// - Saves
/// - GA (goals against)
/// - SV% (save percentage)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;

/// Column width constants
const NUM_COL_WIDTH: usize = 3;
const NAME_COL_WIDTH: usize = 20;
const SA_COL_WIDTH: usize = 4;
const SAVES_COL_WIDTH: usize = 6;
const GA_COL_WIDTH: usize = 6;
const SV_PCT_COL_WIDTH: usize = 6;
const TABLE_WIDTH: usize = 52; // Total width including margins

/// Widget for displaying game goalie statistics table
#[derive(Debug)]
pub struct GameGoalieStatsTable<'a> {
    /// Team abbreviation (e.g., "TOR")
    pub team_abbrev: &'a str,
    /// Goalies to display in the table
    pub goalies: &'a [nhl_api::GoalieStats],
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> GameGoalieStatsTable<'a> {
    /// Create a new GameGoalieStatsTable widget
    pub fn new(
        team_abbrev: &'a str,
        goalies: &'a [nhl_api::GoalieStats],
        margin: u16,
    ) -> Self {
        Self {
            team_abbrev,
            goalies,
            margin,
        }
    }

    /// Calculate the total height needed for this table
    fn calculate_height(&self) -> u16 {
        let mut height = 0;

        // Header: single-line header is 2 lines
        height += 2;

        // Table header
        height += 1;

        // Goalie rows
        height += self.goalies.len() as u16;

        // Blank line after table
        height += 1;

        height
    }
}

impl<'a> RenderableWidget for GameGoalieStatsTable<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;
        let margin = self.margin;

        // Render header
        let header_text = format!("{} - Goalies", self.team_abbrev);
        y += render_section_header(&header_text, false, margin, area, y, buf, config);

        // Render table header
        if y < area.bottom() {
            let header = format!(
                "{}{:<num_width$} {:<name_width$} {:>sa_width$} {:>saves_width$} {:>ga_width$} {:>sv_pct_width$}",
                " ".repeat(margin as usize),
                "#", "Name", "SA", "Saves", "GA", "SV%",
                num_width = NUM_COL_WIDTH,
                name_width = NAME_COL_WIDTH,
                sa_width = SA_COL_WIDTH,
                saves_width = SAVES_COL_WIDTH,
                ga_width = GA_COL_WIDTH,
                sv_pct_width = SV_PCT_COL_WIDTH
            );
            buf.set_string(area.x, y, &header, Style::default());
            y += 1;
        }

        // Render goalie rows
        for goalie in self.goalies {
            if y >= area.bottom() {
                break;
            }

            let sv_pct = goalie.save_pctg
                .map(|p| format!("{:.3}", p))
                .unwrap_or_else(|| "-".to_string());

            let row = format!(
                "{}{:<num_width$} {:<name_width$} {:>sa_width$} {:>saves_width$} {:>ga_width$} {:>sv_pct_width$}",
                " ".repeat(margin as usize),
                goalie.sweater_number,
                goalie.name.default,
                goalie.shots_against,
                goalie.saves,
                goalie.goals_against,
                sv_pct,
                num_width = NUM_COL_WIDTH,
                name_width = NAME_COL_WIDTH,
                sa_width = SA_COL_WIDTH,
                saves_width = SAVES_COL_WIDTH,
                ga_width = GA_COL_WIDTH,
                sv_pct_width = SV_PCT_COL_WIDTH
            );

            buf.set_string(area.x, y, &row, Style::default());
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
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;
    use nhl_api::LocalizedString;

    fn create_test_goalie(
        number: i32,
        name: &str,
        sa: i32,
        saves: i32,
        ga: i32,
        sv_pct: Option<f64>,
    ) -> nhl_api::GoalieStats {
        nhl_api::GoalieStats {
            player_id: 1,
            sweater_number: number,
            name: LocalizedString { default: name.to_string() },
            position: "G".to_string(),
            shots_against: sa,
            saves,
            goals_against: ga,
            save_pctg: sv_pct,
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
    fn test_game_goalie_stats_table_empty() {
        let goalies = vec![];
        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Goalies",
            "─────────────",
            "#   Name                   SA  Saves     GA    SV%",
            "",
        ]);
    }

    #[test]
    fn test_game_goalie_stats_table_with_goalies() {
        let goalies = vec![
            create_test_goalie(35, "Joseph Woll", 30, 28, 2, Some(0.933)),
            create_test_goalie(60, "Matt Murray", 15, 13, 2, Some(0.867)),
        ];

        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Goalies",
            "─────────────",
            "#   Name                   SA  Saves     GA    SV%",
            "35  Joseph Woll            30     28      2  0.933",
            "60  Matt Murray            15     13      2  0.867",
            "",
        ]);
    }

    #[test]
    fn test_game_goalie_stats_table_preferred_dimensions() {
        let goalies = vec![
            create_test_goalie(35, "Goalie A", 25, 23, 2, Some(0.920)),
        ];

        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(2) + table header(1) + 1 goalie + blank(1) = 5
        assert_eq!(widget.preferred_height(), Some(5));
    }

    #[test]
    fn test_game_goalie_stats_table_height_calculation() {
        let goalies = vec![
            create_test_goalie(35, "Goalie A", 25, 23, 2, Some(0.920)),
            create_test_goalie(60, "Goalie B", 10, 8, 2, Some(0.800)),
        ];

        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);

        // Height should be: header(2) + table header(1) + 2 goalies + blank(1) = 6
        assert_eq!(widget.preferred_height(), Some(6));
    }

    #[test]
    fn test_game_goalie_stats_table_stats_display() {
        let goalies = vec![
            create_test_goalie(35, "Test Goalie", 32, 29, 3, Some(0.906)),
        ];

        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Goalies",
            "─────────────",
            "#   Name                   SA  Saves     GA    SV%",
            "35  Test Goalie            32     29      3  0.906",
            "",
        ]);
    }

    #[test]
    fn test_game_goalie_stats_table_missing_save_pct() {
        let goalies = vec![
            create_test_goalie(35, "Test Goalie", 0, 0, 0, None),
        ];

        let widget = GameGoalieStatsTable::new("TOR", &goalies, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Goalies",
            "─────────────",
            "#   Name                   SA  Saves     GA    SV%",
            "35  Test Goalie             0      0      0      -",
            "",
        ]);
    }
}
