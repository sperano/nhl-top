use nhl_api::SkaterStats;
/// Reusable table widget for displaying skater statistics
///
/// This component provides a standardized table for both game-level and season-level
/// skater statistics. It wraps the generic TableWidget with skater-specific column
/// definitions and provides a builder API for configuration.
///
/// # Supported Data Types
///
/// - **`SkaterStats`**: Game-level stats from boxscore API
/// - **`ClubSkaterStats`**: Season-level stats from club roster API
///
/// # Usage Example - Game Stats (Boxscore)
///
/// ```ignore
/// use nhl_api::SkaterStats;
/// use nhl::tui::components::SkaterStatsTableWidget;
///
/// let forwards: Vec<SkaterStats> = boxscore.player_by_game_stats.away_team.forwards;
///
/// let table = SkaterStatsTableWidget::from_game_stats(forwards)
///     .with_header("Away - Forwards")
///     .with_selection(0, 0)
///     .with_focused(true)
///     .build();
///
/// table.render(area, buf, config);
/// ```
use ratatui::{buffer::Buffer, layout::Rect};

use super::table::TableWidget;
use crate::config::DisplayConfig;
use crate::tui::{component::ElementWidget, Alignment, CellValue, ColumnDef};

/// Creates column definitions for game-level skater statistics
///
/// Columns: Player, Pos, G, A, PTS, +/-, SOG, Hits, Blk, PIM, FO%, TOI, Shft, Give, Take
fn game_skater_columns() -> Vec<ColumnDef<SkaterStats>> {
    vec![
        ColumnDef::new("Player", 20, Alignment::Left, |s: &SkaterStats| {
            CellValue::PlayerLink {
                display: s.name.default.clone(),
                player_id: s.player_id,
            }
        }),
        ColumnDef::new("Pos", 3, Alignment::Center, |s: &SkaterStats| {
            CellValue::Text(s.position.to_string())
        }),
        ColumnDef::new("G", 2, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.goals.to_string())
        }),
        ColumnDef::new("A", 2, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.assists.to_string())
        }),
        ColumnDef::new("PTS", 3, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.points.to_string())
        }),
        ColumnDef::new("+/-", 3, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(format!("{:+}", s.plus_minus))
        }),
        ColumnDef::new("SOG", 3, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.sog.to_string())
        }),
        ColumnDef::new("Hits", 4, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.hits.to_string())
        }),
        ColumnDef::new("Blk", 3, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.blocked_shots.to_string())
        }),
        ColumnDef::new("PIM", 3, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.pim.to_string())
        }),
        ColumnDef::new("FO%", 5, Alignment::Right, |s: &SkaterStats| {
            if s.faceoff_winning_pctg > 0.0 {
                CellValue::Text(format!("{:.1}", s.faceoff_winning_pctg * 100.0))
            } else {
                CellValue::Text("-".to_string())
            }
        }),
        ColumnDef::new("TOI", 5, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.toi.clone())
        }),
        ColumnDef::new("Shft", 4, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.shifts.to_string())
        }),
        ColumnDef::new("Give", 4, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.giveaways.to_string())
        }),
        ColumnDef::new("Take", 4, Alignment::Right, |s: &SkaterStats| {
            CellValue::Text(s.takeaways.to_string())
        }),
    ]
}

/// Builder for skater statistics tables
pub struct SkaterStatsTableWidget {
    inner: TableWidget,
}

impl SkaterStatsTableWidget {
    /// Create a table from game-level skater stats
    pub fn from_game_stats(data: Vec<SkaterStats>) -> Self {
        let columns = game_skater_columns();
        let inner = TableWidget::from_data(&columns, data);
        Self { inner }
    }

    /// Set the table header text
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.inner = self.inner.with_header(header);
        self
    }

    /// Set selection state (row and column indices)
    pub fn with_selection(mut self, row: usize, col: usize) -> Self {
        self.inner = self.inner.with_selection(row, col);
        self
    }

    /// Set optional selection state (None = no selection)
    pub fn with_selection_opt(mut self, row: Option<usize>, col: Option<usize>) -> Self {
        self.inner = self.inner.with_selection_opt(row, col);
        self
    }

    /// Set focused state (affects selection highlighting)
    pub fn with_focused(mut self, focused: bool) -> Self {
        self.inner = self.inner.with_focused(focused);
        self
    }

    /// Set left margin (spaces before table content)
    pub fn with_margin(mut self, margin: u16) -> Self {
        self.inner = self.inner.with_margin(margin);
        self
    }

    /// Find the first column that contains links (for keyboard navigation)
    pub fn find_first_link_column(&self) -> Option<usize> {
        self.inner.find_first_link_column()
    }

    /// Find the next link column after the given column (for keyboard navigation)
    pub fn find_next_link_column(&self, current_col: usize) -> Option<usize> {
        self.inner.find_next_link_column(current_col)
    }

    /// Find the previous link column before the given column (for keyboard navigation)
    pub fn find_previous_link_column(&self, current_col: usize) -> Option<usize> {
        self.inner.find_prev_link_column(current_col)
    }
}

impl ElementWidget for SkaterStatsTableWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        self.inner.render(area, buf, config);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        self.inner.preferred_height()
    }

    fn preferred_width(&self) -> Option<u16> {
        self.inner.preferred_width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::{LocalizedString, Position};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn create_test_skater(id: i64, name: &str, pos: Position, g: i32, a: i32) -> SkaterStats {
        SkaterStats {
            player_id: id,
            sweater_number: 34,
            name: LocalizedString {
                default: name.to_string(),
            },
            position: pos,
            goals: g,
            assists: a,
            points: g + a,
            plus_minus: 2,
            pim: 4,
            hits: 5,
            power_play_goals: 1,
            sog: 8,
            faceoff_winning_pctg: 0.55,
            toi: "18:30".to_string(),
            blocked_shots: 2,
            shifts: 22,
            giveaways: 1,
            takeaways: 3,
        }
    }

    #[test]
    fn test_game_stats_table_renders() {
        let skaters = vec![
            create_test_skater(8479318, "Auston Matthews", Position::Center, 2, 1),
            create_test_skater(8478402, "Connor McDavid", Position::Center, 1, 3),
        ];

        let table = SkaterStatsTableWidget::from_game_stats(skaters)
            .with_header("Forwards")
            .with_selection(0, 0)
            .with_focused(true);

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        // Verify the table rendered without panicking
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_player_link_column_is_first() {
        let skaters = vec![create_test_skater(
            8479318,
            "Auston Matthews",
            Position::Center,
            2,
            1,
        )];

        let table = SkaterStatsTableWidget::from_game_stats(skaters);

        // Player column should be the first link column (index 0)
        assert_eq!(table.find_first_link_column(), Some(0));
    }

    #[test]
    fn test_table_with_no_data() {
        let table = SkaterStatsTableWidget::from_game_stats(vec![])
            .with_header("No Skaters")
            .with_focused(false);

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_table_shows_player_names() {
        use crate::tui::testing::assert_buffer;

        let skaters = vec![create_test_skater(
            8479318,
            "Auston Matthews",
            Position::Center,
            2,
            1,
        )];

        let table = SkaterStatsTableWidget::from_game_stats(skaters)
            .with_header("Test")
            .with_margin(0);

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        assert_buffer(
            &buf,
            &[
                "  Test",
                "  ════",
                "",
                "  Player                Pos  G   A   PTS  +/-  SOG  H...  Blk  PIM  FO%    TOI",
                "  ──────────────────────────────────────────────────────────────────────────────",
                "  Auston Matthews        C    2   1    3   +2    8     5    2    4   55.0  18...",
                "",
                "",
                "",
                "",
            ],
        );
    }

    #[test]
    fn test_table_shows_all_stat_columns() {
        use crate::tui::testing::assert_buffer;

        let skater = create_test_skater(8479318, "A. Matthews", Position::Center, 2, 1);

        let table = SkaterStatsTableWidget::from_game_stats(vec![skater])
            .with_header("Stats")
            .with_margin(0);

        let area = Rect::new(0, 0, 100, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        assert_buffer(&buf, &[
            "  Stats",
            "  ═════",
            "",
            "  Player                Pos  G   A   PTS  +/-  SOG  H...  Blk  PIM  FO%    TOI    S...  G...  T...",
            "  ────────────────────────────────────────────────────────────────────────────────────────────────",
            "  A. Matthews            C    2   1    3   +2    8     5    2    4   55.0  18...    22     1     3",
            "",
            "",
            "",
            "",
        ]);
    }
}
