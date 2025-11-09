/// GameSkaterStatsTable widget - displays skater statistics from a game
///
/// This widget renders a table showing skater statistics with columns for:
/// - # (sweater number)
/// - Name
/// - Pos (position)
/// - G (goals)
/// - A (assists)
/// - P (points)
/// - +/- (plus/minus)
/// - TOI (time on ice)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;

/// Column width constants
const NUM_COL_WIDTH: usize = 3;
const NAME_COL_WIDTH: usize = 20;
const POS_COL_WIDTH: usize = 4;
const G_COL_WIDTH: usize = 3;
const A_COL_WIDTH: usize = 3;
const P_COL_WIDTH: usize = 3;
const PLUS_MINUS_COL_WIDTH: usize = 4;
const TOI_COL_WIDTH: usize = 6;
const TABLE_WIDTH: usize = 52; // Total width including margins

/// Widget for displaying game skater statistics table
#[derive(Debug)]
pub struct GameSkaterStatsTable<'a> {
    /// Team abbreviation (e.g., "TOR")
    pub team_abbrev: &'a str,
    /// Position name ("Forwards" or "Defense")
    pub position_name: &'a str,
    /// Skaters to display in the table
    pub skaters: &'a [nhl_api::SkaterStats],
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> GameSkaterStatsTable<'a> {
    /// Create a new GameSkaterStatsTable widget
    pub fn new(
        team_abbrev: &'a str,
        position_name: &'a str,
        skaters: &'a [nhl_api::SkaterStats],
        margin: u16,
    ) -> Self {
        Self {
            team_abbrev,
            position_name,
            skaters,
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

        // Skater rows
        height += self.skaters.len() as u16;

        // Blank line after table
        height += 1;

        height
    }
}

impl<'a> RenderableWidget for GameSkaterStatsTable<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;
        let margin = self.margin;

        // Render header
        let header_text = format!("{} - {}", self.team_abbrev, self.position_name);
        y += render_section_header(&header_text, false, margin, area, y, buf, config);

        // Render table header
        if y < area.bottom() {
            let header = format!(
                "{}{:<num_width$} {:<name_width$} {:<pos_width$} {:>g_width$} {:>a_width$} {:>p_width$} {:>pm_width$} {:>toi_width$}",
                " ".repeat(margin as usize),
                "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI",
                num_width = NUM_COL_WIDTH,
                name_width = NAME_COL_WIDTH,
                pos_width = POS_COL_WIDTH,
                g_width = G_COL_WIDTH,
                a_width = A_COL_WIDTH,
                p_width = P_COL_WIDTH,
                pm_width = PLUS_MINUS_COL_WIDTH,
                toi_width = TOI_COL_WIDTH
            );
            buf.set_string(area.x, y, &header, Style::default());
            y += 1;
        }

        // Render skater rows
        for skater in self.skaters {
            if y >= area.bottom() {
                break;
            }

            let row = format!(
                "{}{:<num_width$} {:<name_width$} {:<pos_width$} {:>g_width$} {:>a_width$} {:>p_width$} {:>pm_width$} {:>toi_width$}",
                " ".repeat(margin as usize),
                skater.sweater_number,
                skater.name.default,
                skater.position,
                skater.goals,
                skater.assists,
                skater.points,
                skater.plus_minus,
                skater.toi,
                num_width = NUM_COL_WIDTH,
                name_width = NAME_COL_WIDTH,
                pos_width = POS_COL_WIDTH,
                g_width = G_COL_WIDTH,
                a_width = A_COL_WIDTH,
                p_width = P_COL_WIDTH,
                pm_width = PLUS_MINUS_COL_WIDTH,
                toi_width = TOI_COL_WIDTH
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
    use crate::tui::widgets::testing::*;
    use nhl_api::LocalizedString;

    fn create_test_skater(
        number: i32,
        name: &str,
        pos: &str,
        g: i32,
        a: i32,
        p: i32,
        pm: i32,
        toi: &str,
    ) -> nhl_api::SkaterStats {
        nhl_api::SkaterStats {
            player_id: 1,
            sweater_number: number,
            name: LocalizedString { default: name.to_string() },
            position: pos.to_string(),
            goals: g,
            assists: a,
            points: p,
            plus_minus: pm,
            pim: 0,
            hits: 0,
            power_play_goals: 0,
            sog: 0,
            faceoff_winning_pctg: 0.0,
            toi: toi.to_string(),
            blocked_shots: 0,
            shifts: 0,
            giveaways: 0,
            takeaways: 0,
        }
    }

    #[test]
    fn test_game_skater_stats_table_empty() {
        let skaters = vec![];
        let widget = GameSkaterStatsTable::new("TOR", "Forwards", &skaters, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Forwards                                              ",
            "──────────────                                              ",
            "#   Name                 Pos    G   A   P  +/-    TOI       ",
            "                                                            ",
        ]);
    }

    #[test]
    fn test_game_skater_stats_table_with_players() {
        let skaters = vec![
            create_test_skater(34, "Auston Matthews", "C", 2, 1, 3, 1, "20:15"),
            create_test_skater(16, "Mitch Marner", "RW", 0, 3, 3, 2, "19:42"),
            create_test_skater(88, "William Nylander", "RW", 1, 1, 2, 0, "18:30"),
        ];

        let widget = GameSkaterStatsTable::new("TOR", "Forwards", &skaters, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Forwards                                              ",
            "──────────────                                              ",
            "#   Name                 Pos    G   A   P  +/-    TOI       ",
            "34  Auston Matthews      C      2   1   3    1  20:15       ",
            "16  Mitch Marner         RW     0   3   3    2  19:42       ",
            "88  William Nylander     RW     1   1   2    0  18:30       ",
            "                                                            ",
        ]);
    }

    #[test]
    fn test_game_skater_stats_table_defense() {
        let skaters = vec![
            create_test_skater(44, "Morgan Rielly", "D", 0, 2, 2, -1, "22:45"),
            create_test_skater(22, "Jake McCabe", "D", 0, 0, 0, 0, "19:15"),
        ];

        let widget = GameSkaterStatsTable::new("TOR", "Defense", &skaters, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Defense                                               ",
            "─────────────                                               ",
            "#   Name                 Pos    G   A   P  +/-    TOI       ",
            "44  Morgan Rielly        D      0   2   2   -1  22:45       ",
            "22  Jake McCabe          D      0   0   0    0  19:15       ",
            "                                                            ",
        ]);
    }

    #[test]
    fn test_game_skater_stats_table_preferred_dimensions() {
        let skaters = vec![
            create_test_skater(34, "Player A", "C", 1, 1, 2, 0, "15:00"),
        ];

        let widget = GameSkaterStatsTable::new("TOR", "Forwards", &skaters, 0);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(2) + table header(1) + 1 player + blank(1) = 5
        assert_eq!(widget.preferred_height(), Some(5));
    }

    #[test]
    fn test_game_skater_stats_table_height_calculation() {
        let skaters = vec![
            create_test_skater(34, "Player A", "C", 1, 1, 2, 0, "15:00"),
            create_test_skater(16, "Player B", "RW", 0, 1, 1, 1, "14:00"),
            create_test_skater(88, "Player C", "LW", 1, 0, 1, -1, "13:00"),
        ];

        let widget = GameSkaterStatsTable::new("TOR", "Forwards", &skaters, 0);

        // Height should be: header(2) + table header(1) + 3 players + blank(1) = 7
        assert_eq!(widget.preferred_height(), Some(7));
    }

    #[test]
    fn test_game_skater_stats_table_stats_display() {
        let skaters = vec![
            create_test_skater(34, "Test Player", "C", 2, 3, 5, -2, "20:15"),
        ];

        let widget = GameSkaterStatsTable::new("TOR", "Forwards", &skaters, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "TOR - Forwards                                              ",
            "──────────────                                              ",
            "#   Name                 Pos    G   A   P  +/-    TOI       ",
            "34  Test Player          C      2   3   5   -2  20:15       ",
            "                                                            ",
        ]);
    }
}
