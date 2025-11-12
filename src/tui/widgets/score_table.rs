/// ScoreTable widget - displays period-by-period score breakdown
///
/// This widget renders a table showing scores for each period of a hockey game,
/// with support for regular periods (1st, 2nd, 3rd), overtime, and shootout.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Constants for score table layout
const TEAM_ABBREV_COL_WIDTH: usize = 5;
const PERIOD_COL_WIDTH: usize = 4;
const TABLE_WIDTH: usize = 37; // Fixed width to accommodate all 5 periods

/// Widget for displaying period-by-period scores
#[derive(Debug, Clone)]
pub struct ScoreTable {
    /// Away team abbreviation (3 letters)
    pub away_team: String,
    /// Home team abbreviation (3 letters)
    pub home_team: String,
    /// Away team total score
    pub away_score: Option<i32>,
    /// Home team total score
    pub home_score: Option<i32>,
    /// Away team period scores (indices: 0=P1, 1=P2, 2=P3, 3=OT, 4=SO)
    pub away_periods: Option<Vec<i32>>,
    /// Home team period scores (indices: 0=P1, 1=P2, 2=P3, 3=OT, 4=SO)
    pub home_periods: Option<Vec<i32>>,
    /// Whether the game has overtime
    pub has_ot: bool,
    /// Whether the game has a shootout
    pub has_so: bool,
    /// Current period number for live games (1-3=regular, 4=OT, 5=SO)
    pub current_period: Option<i32>,
    /// Whether this table is selected
    pub selected: bool,
}

impl ScoreTable {
    /// Create a new ScoreTable widget
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        away_team: String,
        home_team: String,
        away_score: Option<i32>,
        home_score: Option<i32>,
        away_periods: Option<Vec<i32>>,
        home_periods: Option<Vec<i32>>,
        has_ot: bool,
        has_so: bool,
        current_period: Option<i32>,
        selected: bool,
    ) -> Self {
        Self {
            away_team,
            home_team,
            away_score,
            home_score,
            away_periods,
            home_periods,
            has_ot,
            has_so,
            current_period,
            selected,
        }
    }

    /// Calculate total number of columns (team + periods + total)
    fn total_columns(&self) -> usize {
        let mut cols = 5; // empty, 1, 2, 3, T
        if self.has_ot {
            cols += 1;
        }
        if self.has_so {
            cols += 1;
        }
        cols
    }

    /// Calculate padding needed to reach fixed width of 37
    fn calculate_padding(&self) -> usize {
        let total_cols = self.total_columns();
        // Current width = 1 (left border) + 5 (team) + (total_cols-1) * (1 sep + 4 data) + 1 (right border)
        let current_width = 1 + TEAM_ABBREV_COL_WIDTH + (total_cols - 1) * (1 + PERIOD_COL_WIDTH) + 1;
        if current_width < TABLE_WIDTH {
            TABLE_WIDTH - current_width
        } else {
            0
        }
    }

    /// Check if a period should show score or dash (for live games)
    fn should_show_period(&self, period: i32) -> bool {
        self.current_period.map_or(true, |current| period <= current)
    }

    /// Get period score at given index
    fn get_period_score(&self, periods: Option<&Vec<i32>>, index: usize) -> String {
        periods
            .and_then(|p| p.get(index))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    /// Get the appropriate style based on selection state
    fn get_style(&self, config: &DisplayConfig) -> Style {
        if self.selected {
            Style::default().fg(config.selection_fg)
        } else {
            Style::default()
        }
    }
}

impl RenderableWidget for ScoreTable {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height < 6 || area.width < TABLE_WIDTH as u16 {
            return; // Not enough space
        }

        let total_cols = self.total_columns();
        let padding = self.calculate_padding();
        let style = self.get_style(config);
        let mut y = area.y;

        // Row 1: Top border
        if y < area.bottom() {
            self.render_top_border(buf, area.x, y, total_cols, padding, style, config);
            y += 1;
        }

        // Row 2: Header row
        if y < area.bottom() {
            self.render_header_row(buf, area.x, y, total_cols, padding, style, config);
            y += 1;
        }

        // Row 3: Middle border
        if y < area.bottom() {
            self.render_middle_border(buf, area.x, y, total_cols, padding, style, config);
            y += 1;
        }

        // Row 4: Away team row
        if y < area.bottom() {
            self.render_team_row(
                buf,
                area.x,
                y,
                &self.away_team,
                self.away_score,
                self.away_periods.as_ref(),
                total_cols,
                padding,
                style,
                config,
            );
            y += 1;
        }

        // Row 5: Home team row
        if y < area.bottom() {
            self.render_team_row(
                buf,
                area.x,
                y,
                &self.home_team,
                self.home_score,
                self.home_periods.as_ref(),
                total_cols,
                padding,
                style,
                config,
            );
            y += 1;
        }

        // Row 6: Bottom border
        if y < area.bottom() {
            self.render_bottom_border(buf, area.x, y, total_cols, padding, style, config);
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(6) // Fixed height: top, header, middle, away, home, bottom
    }

    fn preferred_width(&self) -> Option<u16> {
        Some(TABLE_WIDTH as u16) // Fixed width
    }
}

impl ScoreTable {
    fn render_top_border(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        total_cols: usize,
        padding: usize,
        style: Style,
        config: &DisplayConfig,
    ) {
        let mut line = String::new();
        line.push_str(&config.box_chars.top_left);
        line.push_str(&config.box_chars.horizontal.repeat(TEAM_ABBREV_COL_WIDTH));
        for _ in 1..total_cols {
            line.push_str(&config.box_chars.top_junction);
            line.push_str(&config.box_chars.horizontal.repeat(PERIOD_COL_WIDTH));
        }
        line.push_str(&config.box_chars.top_right);
        if padding > 0 {
            line.push_str(&" ".repeat(padding));
        }
        buf.set_string(x, y, &line, style);
    }

    fn render_middle_border(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        total_cols: usize,
        padding: usize,
        style: Style,
        config: &DisplayConfig,
    ) {
        let mut line = String::new();
        line.push_str(&config.box_chars.left_junction);
        line.push_str(&config.box_chars.horizontal.repeat(TEAM_ABBREV_COL_WIDTH));
        for _ in 1..total_cols {
            line.push_str(&config.box_chars.cross);
            line.push_str(&config.box_chars.horizontal.repeat(PERIOD_COL_WIDTH));
        }
        line.push_str(&config.box_chars.right_junction);
        if padding > 0 {
            line.push_str(&" ".repeat(padding));
        }
        buf.set_string(x, y, &line, style);
    }

    fn render_bottom_border(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        total_cols: usize,
        padding: usize,
        style: Style,
        config: &DisplayConfig,
    ) {
        let mut line = String::new();
        line.push_str(&config.box_chars.bottom_left);
        line.push_str(&config.box_chars.horizontal.repeat(TEAM_ABBREV_COL_WIDTH));
        for _ in 1..total_cols {
            line.push_str(&config.box_chars.bottom_junction);
            line.push_str(&config.box_chars.horizontal.repeat(PERIOD_COL_WIDTH));
        }
        line.push_str(&config.box_chars.bottom_right);
        if padding > 0 {
            line.push_str(&" ".repeat(padding));
        }
        buf.set_string(x, y, &line, style);
    }

    fn render_header_row(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        total_cols: usize,
        padding: usize,
        style: Style,
        config: &DisplayConfig,
    ) {
        let mut line = String::new();
        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^5}", ""));
        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^4}", "1"));
        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^4}", "2"));
        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^4}", "3"));

        if self.has_ot {
            line.push_str(&config.box_chars.vertical);
            line.push_str(&format!("{:^4}", "OT"));
        }

        if self.has_so {
            line.push_str(&config.box_chars.vertical);
            line.push_str(&format!("{:^4}", "SO"));
        }

        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^4}", "T"));
        line.push_str(&config.box_chars.vertical);

        if padding > 0 {
            line.push_str(&" ".repeat(padding));
        }
        buf.set_string(x, y, &line, style);
    }

    fn render_team_row(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        team_abbrev: &str,
        team_score: Option<i32>,
        team_periods: Option<&Vec<i32>>,
        total_cols: usize,
        padding: usize,
        style: Style,
        config: &DisplayConfig,
    ) {
        let mut line = String::new();
        line.push_str(&config.box_chars.vertical);
        line.push_str(&format!("{:^5}", team_abbrev));
        line.push_str(&config.box_chars.vertical);

        // Period 1
        let p1 = if self.should_show_period(1) {
            self.get_period_score(team_periods, 0)
        } else {
            "-".to_string()
        };
        line.push_str(&format!("{:^4}", p1));
        line.push_str(&config.box_chars.vertical);

        // Period 2
        let p2 = if self.should_show_period(2) {
            self.get_period_score(team_periods, 1)
        } else {
            "-".to_string()
        };
        line.push_str(&format!("{:^4}", p2));
        line.push_str(&config.box_chars.vertical);

        // Period 3
        let p3 = if self.should_show_period(3) {
            self.get_period_score(team_periods, 2)
        } else {
            "-".to_string()
        };
        line.push_str(&format!("{:^4}", p3));

        // Overtime
        if self.has_ot {
            line.push_str(&config.box_chars.vertical);
            let ot = if self.should_show_period(4) {
                self.get_period_score(team_periods, 3)
            } else {
                "-".to_string()
            };
            line.push_str(&format!("{:^4}", ot));
        }

        // Shootout
        if self.has_so {
            line.push_str(&config.box_chars.vertical);
            let so = if self.should_show_period(5) {
                self.get_period_score(team_periods, 4)
            } else {
                "-".to_string()
            };
            line.push_str(&format!("{:^4}", so));
        }

        // Total
        line.push_str(&config.box_chars.vertical);
        let total = team_score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        line.push_str(&format!("{:^4}", total));
        line.push_str(&config.box_chars.vertical);

        if padding > 0 {
            line.push_str(&" ".repeat(padding));
        }
        buf.set_string(x, y, &line, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_score_table_scheduled_game() {
        let widget = ScoreTable::new(
            "TOR".to_string(),
            "MTL".to_string(),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│ TOR │ -  │ -  │ -  │ -  │          ",
            "│ MTL │ -  │ -  │ -  │ -  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

    #[test]
    fn test_score_table_final_game_regular_periods() {
        let away_periods = vec![1, 2, 0];
        let home_periods = vec![0, 1, 2];

        let widget = ScoreTable::new(
            "BOS".to_string(),
            "NYR".to_string(),
            Some(3),
            Some(3),
            Some(away_periods),
            Some(home_periods),
            false,
            false,
            None,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│ BOS │ 1  │ 2  │ 0  │ 3  │          ",
            "│ NYR │ 0  │ 1  │ 2  │ 3  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

    #[test]
    fn test_score_table_with_overtime() {
        let away_periods = vec![1, 1, 1, 1];
        let home_periods = vec![1, 1, 1, 0];

        let widget = ScoreTable::new(
            "EDM".to_string(),
            "VAN".to_string(),
            Some(4),
            Some(3),
            Some(away_periods),
            Some(home_periods),
            true,
            false,
            None,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────┬────╮     ",
            "│     │ 1  │ 2  │ 3  │ OT │ T  │     ",
            "├─────┼────┼────┼────┼────┼────┤     ",
            "│ EDM │ 1  │ 1  │ 1  │ 1  │ 4  │     ",
            "│ VAN │ 1  │ 1  │ 1  │ 0  │ 3  │     ",
            "╰─────┴────┴────┴────┴────┴────╯     ",
        ], 37);
    }

    #[test]
    fn test_score_table_with_shootout() {
        let away_periods = vec![1, 1, 1, 0, 1];
        let home_periods = vec![1, 1, 1, 0, 0];

        let widget = ScoreTable::new(
            "CAR".to_string(),
            "NJD".to_string(),
            Some(4),
            Some(3),
            Some(away_periods),
            Some(home_periods),
            true,
            true,
            None,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────┬────┬────╮",
            "│     │ 1  │ 2  │ 3  │ OT │ SO │ T  │",
            "├─────┼────┼────┼────┼────┼────┼────┤",
            "│ CAR │ 1  │ 1  │ 1  │ 0  │ 1  │ 4  │",
            "│ NJD │ 1  │ 1  │ 1  │ 0  │ 0  │ 3  │",
            "╰─────┴────┴────┴────┴────┴────┴────╯",
        ], 37);
    }

    #[test]
    fn test_score_table_live_game_with_current_period() {
        let away_periods = vec![1, 1, 0];
        let home_periods = vec![0, 1, 0];

        let widget = ScoreTable::new(
            "BOS".to_string(),
            "NYR".to_string(),
            Some(2),
            Some(1),
            Some(away_periods),
            Some(home_periods),
            false,
            false,
            Some(2),
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│ BOS │ 1  │ 1  │ -  │ 2  │          ",
            "│ NYR │ 0  │ 1  │ -  │ 1  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

    #[test]
    fn test_score_table_live_game_first_period() {
        // Live game in first period
        let away_periods = vec![1, 0, 0];
        let home_periods = vec![0, 0, 0];

        let widget = ScoreTable::new(
            "TOR".to_string(),
            "MTL".to_string(),
            Some(1),
            Some(0),
            Some(away_periods),
            Some(home_periods),
            false,
            false,
            Some(1), // Currently in period 1
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│ TOR │ 1  │ -  │ -  │ 1  │          ",
            "│ MTL │ 0  │ -  │ -  │ 0  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

    #[test]
    fn test_score_table_live_game_second_period() {
        // Live game in second period
        let away_periods = vec![1, 1, 0];
        let home_periods = vec![0, 1, 0];

        let widget = ScoreTable::new(
            "BOS".to_string(),
            "NYR".to_string(),
            Some(2),
            Some(1),
            Some(away_periods),
            Some(home_periods),
            false,
            false,
            Some(2), // Currently in period 2
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│ BOS │ 1  │ 1  │ -  │ 2  │          ",
            "│ NYR │ 0  │ 1  │ -  │ 1  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

    #[test]
    fn test_score_table_preferred_dimensions() {
        let widget = ScoreTable::new(
            "TOR".to_string(),
            "MTL".to_string(),
            Some(3),
            Some(2),
            Some(vec![1, 1, 1]),
            Some(vec![1, 1, 0]),
            false,
            false,
            None,
            false,
        );

        // Should have fixed dimensions
        assert_eq!(widget.preferred_width(), Some(37));
        assert_eq!(widget.preferred_height(), Some(6));
    }

    #[test]
    fn test_score_table_header_columns() {
        // Test that header shows correct columns for all scenarios
        let widget_no_ot_so = ScoreTable::new(
            "A".to_string(),
            "B".to_string(),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            false,
        );

        let widget_with_ot = ScoreTable::new(
            "A".to_string(),
            "B".to_string(),
            None,
            None,
            None,
            None,
            true,
            false,
            None,
            false,
        );

        let widget_with_both = ScoreTable::new(
            "A".to_string(),
            "B".to_string(),
            None,
            None,
            None,
            None,
            true,
            true,
            None,
            false,
        );

        let config = test_config();

        // Regular game: 5 columns (empty, 1, 2, 3, T)
        let buf1 = render_widget_with_config(&widget_no_ot_so, 37, 6, &config);
        let actual1 = buffer_lines(&buf1);
        let expected1 = vec![
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│  A  │ -  │ -  │ -  │ -  │          ",
            "│  B  │ -  │ -  │ -  │ -  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ];
        assert_eq!(actual1, expected1);

        // With OT: 6 columns
        let buf2 = render_widget_with_config(&widget_with_ot, 37, 6, &config);
        let actual2 = buffer_lines(&buf2);
        let expected2 = vec![
            "╭─────┬────┬────┬────┬────┬────╮     ",
            "│     │ 1  │ 2  │ 3  │ OT │ T  │     ",
            "├─────┼────┼────┼────┼────┼────┤     ",
            "│  A  │ -  │ -  │ -  │ -  │ -  │     ",
            "│  B  │ -  │ -  │ -  │ -  │ -  │     ",
            "╰─────┴────┴────┴────┴────┴────╯     ",
        ];
        assert_eq!(actual2, expected2);

        // With both: 7 columns
        let buf3 = render_widget_with_config(&widget_with_both, 37, 6, &config);
        let actual3 = buffer_lines(&buf3);
        let expected3 = vec![
            "╭─────┬────┬────┬────┬────┬────┬────╮",
            "│     │ 1  │ 2  │ 3  │ OT │ SO │ T  │",
            "├─────┼────┼────┼────┼────┼────┼────┤",
            "│  A  │ -  │ -  │ -  │ -  │ -  │ -  │",
            "│  B  │ -  │ -  │ -  │ -  │ -  │ -  │",
            "╰─────┴────┴────┴────┴────┴────┴────╯",
        ];
        assert_eq!(actual3, expected3);
    }

    #[test]
    fn test_score_table_fixed_width() {
        // All score tables should render to a fixed 37-column buffer
        let widget = ScoreTable::new(
            "A".to_string(),
            "B".to_string(),
            Some(1),
            Some(2),
            Some(vec![1, 0, 0]),
            Some(vec![0, 1, 1]),
            false,
            false,
            None,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 6, &config);

        // Verify the buffer is the right size
        assert_eq!(buf.area.width, 37);
        assert_eq!(buf.area.height, 6);

        assert_buffer(&buf, &[
            "╭─────┬────┬────┬────┬────╮          ",
            "│     │ 1  │ 2  │ 3  │ T  │          ",
            "├─────┼────┼────┼────┼────┤          ",
            "│  A  │ 1  │ 0  │ 0  │ 1  │          ",
            "│  B  │ 0  │ 1  │ 1  │ 2  │          ",
            "╰─────┴────┴────┴────┴────╯          ",
        ], 37);
    }

}

