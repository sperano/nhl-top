/// PlayerStatsTable widget - displays roster player statistics
///
/// This widget renders a table showing skater statistics with columns for:
/// - Player name
/// - Games Played (GP)
/// - Goals (G)
/// - Assists (A)
/// - Points (PTS)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;
use crate::tui::widgets::horizontal_separator::render_horizontal_separator;
use crate::tui::standings::panel::PlayerStat;

/// Column width constants
const PLAYER_NAME_COL_WIDTH: usize = 25;
const GP_COL_WIDTH: usize = 4;
const G_COL_WIDTH: usize = 4;
const A_COL_WIDTH: usize = 4;
const PTS_COL_WIDTH: usize = 5;
const TABLE_WIDTH: usize = 48; // Total width including margins

/// Widget for displaying player statistics table
#[derive(Debug)]
pub struct PlayerStatsTable<'a> {
    /// Players to display in the table
    pub players: &'a [PlayerStat],
    /// Optional header text (e.g., "Player Statistics")
    pub header: Option<&'a str>,
    /// Index of the selected player (for highlighting)
    pub selected_index: Option<usize>,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> PlayerStatsTable<'a> {
    /// Create a new PlayerStatsTable widget
    pub fn new(
        players: &'a [PlayerStat],
        header: Option<&'a str>,
        selected_index: Option<usize>,
        margin: u16,
    ) -> Self {
        Self {
            players,
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

        // Player rows
        height += self.players.len() as u16;

        // Blank line after table
        height += 1;

        height
    }

    /// Get the appropriate style based on whether a player is selected
    fn get_player_style(&self, player_index: usize, config: &DisplayConfig) -> Style {
        if Some(player_index) == self.selected_index {
            Style::default().fg(config.selection_fg)
        } else {
            Style::default()
        }
    }
}

impl<'a> RenderableWidget for PlayerStatsTable<'a> {
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
                "{}{:<player_width$} {:>gp_width$} {:>g_width$} {:>a_width$} {:>pts_width$}",
                " ".repeat(margin as usize),
                "Player", "GP", "G", "A", "PTS",
                player_width = PLAYER_NAME_COL_WIDTH,
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

        // Render player rows
        for (idx, player) in self.players.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let style = self.get_player_style(idx, config);

            // Format the entire row
            let row = format!(
                "{}{:<player_width$} {:>gp_width$} {:>g_width$} {:>a_width$} {:>pts_width$}",
                " ".repeat(margin as usize),
                player.name,
                player.gp,
                player.g,
                player.a,
                player.pts,
                player_width = PLAYER_NAME_COL_WIDTH,
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

    fn create_test_player(name: &str, gp: i32, g: i32, a: i32, pts: i32) -> PlayerStat {
        PlayerStat {
            name: name.to_string(),
            gp,
            g,
            a,
            pts,
        }
    }

    #[test]
    fn test_player_stats_table_empty() {
        let players = vec![];
        let widget = PlayerStatsTable::new(&players, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        assert_buffer(&buf, &[
            "  Player                      GP    G    A   PTS",
            "  ──────────────────────────────────────────────",
            "                                                ",
        ]);
    }

    #[test]
    fn test_player_stats_table_with_players() {
        let players = vec![
            create_test_player("Auston Matthews", 58, 42, 31, 73),
            create_test_player("Mitchell Marner", 58, 18, 48, 66),
            create_test_player("William Nylander", 56, 28, 35, 63),
        ];

        let widget = PlayerStatsTable::new(&players, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        assert_buffer(&buf, &[
            "  Player                      GP    G    A   PTS",
            "  ──────────────────────────────────────────────",
            "  Auston Matthews             58   42   31    73",
            "  Mitchell Marner             58   18   48    66",
            "  William Nylander            56   28   35    63",
            "                                                ",
        ]);
    }

    #[test]
    fn test_player_stats_table_with_header() {
        let players = vec![
            create_test_player("Auston Matthews", 58, 42, 31, 73),
        ];
        let header = "Player Statistics";

        let widget = PlayerStatsTable::new(
            &players,
            Some(header),
            None,
            2,
        );
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        assert_buffer(&buf, &[
            "  Player Statistics                             ",
            "  ═════════════════                             ",
            "  Player                      GP    G    A   PTS",
            "  ──────────────────────────────────────────────",
            "  Auston Matthews             58   42   31    73",
            "                                                ",
            "                                                ",
        ]);
    }

    #[test]
    fn test_player_stats_table_with_selection() {
        let players = vec![
            create_test_player("Auston Matthews", 58, 42, 31, 73),
            create_test_player("Mitchell Marner", 58, 18, 48, 66),
            create_test_player("William Nylander", 56, 28, 35, 63),
        ];

        let widget = PlayerStatsTable::new(&players, None, Some(1), 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        assert_buffer(&buf, &[
            "  Player                      GP    G    A   PTS",
            "  ──────────────────────────────────────────────",
            "  Auston Matthews             58   42   31    73",
            "  Mitchell Marner             58   18   48    66",
            "  William Nylander            56   28   35    63",
            "                                                ",
        ]);
    }

    #[test]
    fn test_player_stats_table_preferred_dimensions() {
        let players = vec![
            create_test_player("Player A", 10, 5, 5, 10),
            create_test_player("Player B", 10, 4, 6, 10),
        ];

        let widget = PlayerStatsTable::new(&players, None, None, 2);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(1) + separator(1) + 2 players + blank(1) = 5
        assert_eq!(widget.preferred_height(), Some(5));
    }

    #[test]
    fn test_player_stats_table_height_with_header() {
        let players = vec![
            create_test_player("Player A", 10, 5, 5, 10),
            create_test_player("Player B", 10, 4, 6, 10),
        ];
        let header = "Player Statistics";

        let widget = PlayerStatsTable::new(
            &players,
            Some(header),
            None,
            2,
        );

        // Height should be: section header(3) + table header(1) + separator(1) + 2 players + blank(1) = 8
        assert_eq!(widget.preferred_height(), Some(8));
    }

    #[test]
    fn test_player_stats_table_column_alignment() {
        let players = vec![
            create_test_player("A", 1, 2, 3, 5),
        ];

        let widget = PlayerStatsTable::new(&players, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 48, height, &config);

        assert_buffer(&buf, &[
            "  Player                      GP    G    A   PTS",
            "  ──────────────────────────────────────────────",
            "  A                            1    2    3     5",
            "                                                ",
        ]);
    }
}
