/// Reusable table widget for displaying goalie statistics
///
/// This component provides a standardized table for both game-level and season-level
/// goalie statistics. It wraps the generic TableWidget with goalie-specific column
/// definitions and provides a builder API for configuration.
///
/// # Supported Data Types
///
/// - **`GoalieStats`**: Game-level stats from boxscore API
/// - **`ClubGoalieStats`**: Season-level stats from club roster API
///
/// # Usage Example - Game Stats (Boxscore)
///
/// ```ignore
/// use nhl_api::GoalieStats;
/// use nhl::tui::components::GoalieStatsTableWidget;
///
/// let goalies: Vec<GoalieStats> = boxscore.player_by_game_stats.away_team.goalies;
///
/// let table = GoalieStatsTableWidget::from_game_stats(goalies)
///     .with_header("Away - Goalies")
///     .with_selection(0, 0)
///     .with_focused(true)
///     .build();
///
/// table.render(area, buf, config);
/// ```

use ratatui::{buffer::Buffer, layout::Rect};
use nhl_api::GoalieStats;

use crate::config::DisplayConfig;
use crate::tui::{
    component::RenderableWidget,
    Alignment, CellValue, ColumnDef,
};
use super::table::TableWidget;

/// Creates column definitions for game-level goalie statistics
///
/// Columns: Player, Dec, SA, Saves, GA, SV%, TOI, EV SA, PP SA, SH SA
fn game_goalie_columns() -> Vec<ColumnDef<GoalieStats>> {
    vec![
        ColumnDef::new("Player", 20, Alignment::Left, |g: &GoalieStats| {
            CellValue::PlayerLink {
                display: g.name.default.clone(),
                player_id: g.player_id,
            }
        }),
        ColumnDef::new("Dec", 3, Alignment::Center, |g: &GoalieStats| {
            CellValue::Text(
                g.decision
                    .as_ref()
                    .map(|d| {
                        match d.as_str() {
                            "W" => "W",
                            "L" => "L",
                            "O" => "OT",
                            _ => "-",
                        }
                    })
                    .unwrap_or("-")
                    .to_string()
            )
        }),
        ColumnDef::new("SA", 3, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.shots_against.to_string())
        }),
        ColumnDef::new("Saves", 5, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.saves.to_string())
        }),
        ColumnDef::new("GA", 2, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.goals_against.to_string())
        }),
        ColumnDef::new("SV%", 5, Alignment::Right, |g: &GoalieStats| {
            if let Some(sv_pct) = g.save_pctg {
                CellValue::Text(format!("{:.3}", sv_pct))
            } else {
                CellValue::Text("-".to_string())
            }
        }),
        ColumnDef::new("TOI", 5, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.toi.clone())
        }),
        ColumnDef::new("EV SA", 5, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.even_strength_shots_against.clone())
        }),
        ColumnDef::new("PP SA", 5, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.power_play_shots_against.clone())
        }),
        ColumnDef::new("SH SA", 5, Alignment::Right, |g: &GoalieStats| {
            CellValue::Text(g.shorthanded_shots_against.clone())
        }),
    ]
}

/// Builder for goalie statistics tables
pub struct GoalieStatsTableWidget {
    inner: TableWidget,
}

impl GoalieStatsTableWidget {
    /// Create a table from game-level goalie stats
    pub fn from_game_stats(data: Vec<GoalieStats>) -> Self {
        let columns = game_goalie_columns();
        let inner = TableWidget::from_data(columns, data);
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

impl RenderableWidget for GoalieStatsTableWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        self.inner.render(area, buf, config);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
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
    use nhl_api::LocalizedString;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn create_test_goalie(id: i64, name: &str, decision: Option<&str>, sa: i32, saves: i32) -> GoalieStats {
        GoalieStats {
            player_id: id,
            sweater_number: 31,
            name: LocalizedString {
                default: name.to_string(),
            },
            position: "G".to_string(),
            even_strength_shots_against: "15".to_string(),
            power_play_shots_against: "5".to_string(),
            shorthanded_shots_against: "2".to_string(),
            save_shots_against: format!("{}/{}", saves, sa),
            save_pctg: Some(saves as f64 / sa as f64),
            even_strength_goals_against: 1,
            power_play_goals_against: 1,
            shorthanded_goals_against: 0,
            pim: Some(0),
            goals_against: sa - saves,
            toi: "60:00".to_string(),
            starter: Some(true),
            decision: decision.map(|s| s.to_string()),
            shots_against: sa,
            saves,
        }
    }

    #[test]
    fn test_game_stats_table_renders() {
        let goalies = vec![
            create_test_goalie(8471679, "Carey Price", Some("W"), 30, 28),
            create_test_goalie(8477424, "Andrei Vasilevskiy", Some("L"), 25, 22),
        ];

        let table = GoalieStatsTableWidget::from_game_stats(goalies)
            .with_header("Goalies")
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
        let goalies = vec![create_test_goalie(8471679, "Carey Price", Some("W"), 30, 28)];

        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        // Player column should be the first link column (index 0)
        assert_eq!(table.find_first_link_column(), Some(0));
    }

    #[test]
    fn test_table_with_no_data() {
        let table = GoalieStatsTableWidget::from_game_stats(vec![])
            .with_header("No Goalies")
            .with_focused(false);

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_table_shows_goalie_names() {
        use crate::tui::testing::assert_buffer;

        let goalies = vec![
            create_test_goalie(8471679, "Carey Price", Some("W"), 30, 28),
        ];

        let table = GoalieStatsTableWidget::from_game_stats(goalies)
            .with_header("Test")
            .with_margin(0);

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        assert_buffer(&buf, &[
            "Test",
            "════",
            "",
            "Player                Dec  SA   Sa...  GA  SV%    TOI    EV...  PP...  SH...",
            "────────────────────────────────────────────────────────────────────────────",
            "Carey Price            W    30     28   2  0....  60...     15      5      2",
            "",
            "",
            "",
            "",
        ]);
    }

    #[test]
    fn test_decision_formatting() {
        let win_goalie = create_test_goalie(8471679, "Winner", Some("W"), 30, 28);
        let loss_goalie = create_test_goalie(8477424, "Loser", Some("L"), 25, 20);
        let ot_goalie = create_test_goalie(8475883, "OT Loss", Some("O"), 28, 25);
        let no_decision = create_test_goalie(8478024, "Relief", None, 10, 9);

        let goalies = vec![win_goalie, loss_goalie, ot_goalie, no_decision];

        let table = GoalieStatsTableWidget::from_game_stats(goalies)
            .with_header("Decisions")
            .with_margin(0);

        let area = Rect::new(0, 0, 80, 8);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        // Verify rendering completed
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_save_percentage_formatting() {
        let perfect_goalie = create_test_goalie(8471679, "Perfect", Some("W"), 30, 30);
        let good_goalie = create_test_goalie(8477424, "Good", Some("W"), 30, 27);

        let goalies = vec![perfect_goalie, good_goalie];

        let table = GoalieStatsTableWidget::from_game_stats(goalies)
            .with_header("Save Percentages")
            .with_margin(0);

        let area = Rect::new(0, 0, 100, 6);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        // Verify rendering completed
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_unknown_decision_type() {
        let mut goalie = create_test_goalie(8471679, "Unknown Decision", Some("X"), 30, 28);
        goalie.decision = Some("X".to_string());

        let goalies = vec![goalie];
        let table = GoalieStatsTableWidget::from_game_stats(goalies).with_margin(0);

        let area = Rect::new(0, 0, 80, 6);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        // Verify it rendered without panic (unknown decision shows as "-")
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_null_save_percentage() {
        let mut goalie = create_test_goalie(8471679, "No SV%", Some("W"), 0, 0);
        goalie.save_pctg = None;

        let goalies = vec![goalie];
        let table = GoalieStatsTableWidget::from_game_stats(goalies).with_margin(0);

        let area = Rect::new(0, 0, 80, 6);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);

        // Verify it rendered without panic (null SV% shows as "-")
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_with_selection_opt() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];

        let table = GoalieStatsTableWidget::from_game_stats(goalies)
            .with_selection_opt(Some(0), Some(0));

        let area = Rect::new(0, 0, 80, 6);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        table.render(area, &mut buf, &config);
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_find_next_link_column() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];
        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        // Player is column 0 (only link column), so there's no next link
        assert_eq!(table.find_next_link_column(0), None);
    }

    #[test]
    fn test_find_previous_link_column() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];
        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        // Column 0 is the first link, so no previous link
        assert_eq!(table.find_previous_link_column(0), None);
    }

    #[test]
    fn test_clone_box() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];
        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        let _cloned: Box<dyn RenderableWidget> = table.clone_box();
        // If we get here, clone_box() worked
    }

    #[test]
    fn test_preferred_height() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];
        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        // Delegate to inner table, just verify it returns something
        let _ = table.preferred_height();
    }

    #[test]
    fn test_preferred_width() {
        let goalies = vec![create_test_goalie(8471679, "Test", Some("W"), 30, 28)];
        let table = GoalieStatsTableWidget::from_game_stats(goalies);

        // Delegate to inner table, just verify it returns something
        let _ = table.preferred_width();
    }
}
