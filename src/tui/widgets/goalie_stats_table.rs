/// GoalieStatsTable widget - displays roster goaltender statistics
///
/// This widget renders a table showing goalie statistics with columns for:
/// - Goaltender name
/// - Games Played (GP)
/// - Goals Against Average (GAA)
/// - Save Percentage (SV%)
/// - Shutouts (SO)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;
use crate::tui::widgets::horizontal_separator::render_horizontal_separator;
use crate::tui::common::panels::GoalieStat;

/// Column width constants
const GOALIE_NAME_COL_WIDTH: usize = 25;
const GP_COL_WIDTH: usize = 4;
const GAA_COL_WIDTH: usize = 6;
const SV_PCT_COL_WIDTH: usize = 6;
const SO_COL_WIDTH: usize = 6;
const TABLE_WIDTH: usize = 52; // Total width including margins

/// Widget for displaying goaltender statistics table
#[derive(Debug)]
pub struct GoalieStatsTable<'a> {
    /// Goalies to display in the table
    pub goalies: &'a [GoalieStat],
    /// Optional header text (e.g., "Goaltender Statistics")
    pub header: Option<&'a str>,
    /// Index of the selected goalie (for highlighting)
    pub selected_index: Option<usize>,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> GoalieStatsTable<'a> {
    /// Create a new GoalieStatsTable widget
    pub fn new(
        goalies: &'a [GoalieStat],
        header: Option<&'a str>,
        selected_index: Option<usize>,
        margin: u16,
    ) -> Self {
        Self {
            goalies,
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

        // Goalie rows
        height += self.goalies.len() as u16;

        // Blank line after table
        height += 1;

        height
    }

    /// Get the appropriate style based on whether a goalie is selected
    fn get_goalie_style(&self, goalie_index: usize, config: &DisplayConfig) -> Style {
        if Some(goalie_index) == self.selected_index {
            Style::default().fg(config.selection_fg)
        } else {
            Style::default()
        }
    }
}

impl<'a> RenderableWidget for GoalieStatsTable<'a> {
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
                "{}{:<goalie_width$} {:>gp_width$} {:>gaa_width$} {:>sv_pct_width$} {:>so_width$}",
                " ".repeat(margin as usize),
                "Goaltender", "GP", "GAA", "SV%", "SO",
                goalie_width = GOALIE_NAME_COL_WIDTH,
                gp_width = GP_COL_WIDTH,
                gaa_width = GAA_COL_WIDTH,
                sv_pct_width = SV_PCT_COL_WIDTH,
                so_width = SO_COL_WIDTH
            );
            buf.set_string(area.x, y, &header, Style::default());
            y += 1;
        }

        // Render separator line
        y += render_horizontal_separator(TABLE_WIDTH, margin, area, y, buf, config);

        // Render goalie rows
        for (idx, goalie) in self.goalies.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            let style = self.get_goalie_style(idx, config);

            // Format the entire row
            let row = format!(
                "{}{:<goalie_width$} {:>gp_width$} {:>gaa_width$} {:>sv_pct_width$} {:>so_width$}",
                " ".repeat(margin as usize),
                goalie.name,
                goalie.gp,
                goalie.gaa,
                goalie.sv_pct,
                goalie.so,
                goalie_width = GOALIE_NAME_COL_WIDTH,
                gp_width = GP_COL_WIDTH,
                gaa_width = GAA_COL_WIDTH,
                sv_pct_width = SV_PCT_COL_WIDTH,
                so_width = SO_COL_WIDTH
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
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;

    fn create_test_goalie(name: &str, gp: i32, gaa: &str, sv_pct: &str, so: i32) -> GoalieStat {
        GoalieStat {
            name: name.to_string(),
            gp,
            gaa: gaa.to_string(),
            sv_pct: sv_pct.to_string(),
            so,
        }
    }

    #[test]
    fn test_goalie_stats_table_empty() {
        let goalies = vec![];
        let widget = GoalieStatsTable::new(&goalies, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 52, height, &config);

        assert_buffer(&buf, &[
            "  Goaltender                  GP    GAA    SV%     S",
            "  ──────────────────────────────────────────────────",
            "",
        ]);
    }

    #[test]
    fn test_goalie_stats_table_with_goalies() {
        let goalies = vec![
            create_test_goalie("Ilya Samsonov", 35, "2.89", ".903", 2),
            create_test_goalie("Joseph Woll", 23, "2.52", ".915", 1),
            create_test_goalie("Martin Jones", 10, "3.45", ".881", 0),
        ];

        let widget = GoalieStatsTable::new(&goalies, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "  Goaltender                  GP    GAA    SV%     SO",
            "  ──────────────────────────────────────────────────",
            "  Ilya Samsonov               35   2.89   .903      2",
            "  Joseph Woll                 23   2.52   .915      1",
            "  Martin Jones                10   3.45   .881      0",
            "",
        ]);
    }

    #[test]
    fn test_goalie_stats_table_with_header() {
        let goalies = vec![
            create_test_goalie("Ilya Samsonov", 35, "2.89", ".903", 2),
        ];
        let header = "Goaltender Statistics";

        let widget = GoalieStatsTable::new(
            &goalies,
            Some(header),
            None,
            2,
        );
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "  Goaltender Statistics",
            "  ═════════════════════",
            "  Goaltender                  GP    GAA    SV%     SO",
            "  ──────────────────────────────────────────────────",
            "  Ilya Samsonov               35   2.89   .903      2",
            "",
            "",
        ]);
    }

    #[test]
    fn test_goalie_stats_table_with_selection() {
        let goalies = vec![
            create_test_goalie("Ilya Samsonov", 35, "2.89", ".903", 2),
            create_test_goalie("Joseph Woll", 23, "2.52", ".915", 1),
            create_test_goalie("Martin Jones", 10, "3.45", ".881", 0),
        ];

        let widget = GoalieStatsTable::new(&goalies, None, Some(1), 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "  Goaltender                  GP    GAA    SV%     SO",
            "  ──────────────────────────────────────────────────",
            "  Ilya Samsonov               35   2.89   .903      2",
            "  Joseph Woll                 23   2.52   .915      1",
            "  Martin Jones                10   3.45   .881      0",
            "",
        ]);
    }

    #[test]
    fn test_goalie_stats_table_preferred_dimensions() {
        let goalies = vec![
            create_test_goalie("Goalie A", 20, "2.50", ".910", 3),
            create_test_goalie("Goalie B", 15, "3.00", ".900", 1),
        ];

        let widget = GoalieStatsTable::new(&goalies, None, None, 2);

        // Width should be fixed
        assert_eq!(widget.preferred_width(), Some(TABLE_WIDTH as u16));

        // Height should be: header(1) + separator(1) + 2 goalies + blank(1) = 5
        assert_eq!(widget.preferred_height(), Some(5));
    }

    #[test]
    fn test_goalie_stats_table_height_with_header() {
        let goalies = vec![
            create_test_goalie("Goalie A", 20, "2.50", ".910", 3),
            create_test_goalie("Goalie B", 15, "3.00", ".900", 1),
        ];
        let header = "Goaltender Statistics";

        let widget = GoalieStatsTable::new(
            &goalies,
            Some(header),
            None,
            2,
        );

        // Height should be: section header(3) + table header(1) + separator(1) + 2 goalies + blank(1) = 8
        assert_eq!(widget.preferred_height(), Some(8));
    }

    #[test]
    fn test_goalie_stats_table_column_alignment() {
        let goalies = vec![
            create_test_goalie("A", 1, "2.00", ".900", 5),
        ];

        let widget = GoalieStatsTable::new(&goalies, None, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, 60, height, &config);

        assert_buffer(&buf, &[
            "  Goaltender                  GP    GAA    SV%     SO",
            "  ──────────────────────────────────────────────────",
            "  A                            1   2.00   .900      5",
            "",
        ]);
    }
}
