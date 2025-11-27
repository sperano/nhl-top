use crate::config::DisplayConfig;
use crate::layout_constants::SCORE_BOX_WIDTH;
use crate::tui::widgets::{StandaloneWidget, ScoreTable};
/// GameBox widget - displays a single game's score in a compact box
///
/// This is a composition widget that combines a header line with a ScoreTable.
/// Fixed dimensions: 37 columns × 7 rows (1 for header + 6 for score table)
use ratatui::{buffer::Buffer, layout::Rect, style::Style};

/// Constants for game box layout
const GAME_BOX_HEIGHT: usize = 7;
const HEADER_CONTENT_WIDTH: usize = 36;

/// Game state determines what header to display
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    /// Game hasn't started - show start time
    Scheduled { start_time: String },
    /// Game in progress - show period and time
    Live {
        period_text: String,
        time_remaining: Option<String>,
        in_intermission: bool,
    },
    /// Game finished - show "Final Score"
    Final,
}

/// Widget for displaying a single game box with header and score table
#[derive(Debug, Clone)]
pub struct GameBox {
    /// Away team abbreviation (3 letters)
    pub away_team: String,
    /// Home team abbreviation (3 letters)
    pub home_team: String,
    /// Away team total score
    pub away_score: Option<i32>,
    /// Home team total score
    pub home_score: Option<i32>,
    /// Away team period scores
    pub away_periods: Option<Vec<i32>>,
    /// Home team period scores
    pub home_periods: Option<Vec<i32>>,
    /// Whether the game has overtime
    pub has_ot: bool,
    /// Whether the game has a shootout
    pub has_so: bool,
    /// Current period for live games
    pub current_period: Option<i32>,
    /// Game state (scheduled/live/final)
    pub state: GameState,
    /// Whether this game box is selected
    pub selected: bool,
}

impl GameBox {
    /// Create a new GameBox widget
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
        state: GameState,
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
            state,
            selected,
        }
    }

    /// Generate header text based on game state
    fn generate_header(&self) -> String {
        match &self.state {
            GameState::Final => "Final Score".to_string(),
            GameState::Live {
                period_text,
                time_remaining,
                in_intermission,
            } => {
                if *in_intermission {
                    format!("{} - Intermission", period_text)
                } else if let Some(time) = time_remaining {
                    format!("{} - {}", period_text, time)
                } else {
                    period_text.clone()
                }
            }
            GameState::Scheduled { start_time } => start_time.clone(),
        }
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

impl StandaloneWidget for GameBox {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height < GAME_BOX_HEIGHT as u16 || area.width < SCORE_BOX_WIDTH {
            return; // Not enough space
        }

        let style = self.get_style(config);
        let mut y = area.y;

        // Row 1: Header
        if y < area.bottom() {
            let header = self.generate_header();
            let header_line = format!(
                "{}{:<width$}",
                " ".to_string(),
                header,
                width = HEADER_CONTENT_WIDTH
            );
            buf.set_string(area.x, y, &header_line, style);
            y += 1;
        }

        // Rows 2-7: Score table
        if y < area.bottom() {
            let score_table = ScoreTable::new(
                self.away_team.clone(),
                self.home_team.clone(),
                self.away_score,
                self.home_score,
                self.away_periods.clone(),
                self.home_periods.clone(),
                self.has_ot,
                self.has_so,
                self.current_period,
                self.selected,
            );

            let table_area = Rect::new(area.x, y, SCORE_BOX_WIDTH, 6);
            score_table.render(table_area, buf, config);
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(GAME_BOX_HEIGHT as u16)
    }

    fn preferred_width(&self) -> Option<u16> {
        Some(SCORE_BOX_WIDTH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::assert_buffer;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_game_box_scheduled() {
        let widget = GameBox::new(
            "TOR".to_string(),
            "MTL".to_string(),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            GameState::Scheduled {
                start_time: "07:00 PM".to_string(),
            },
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " 07:00 PM",
                "╭─────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ T  │",
                "├─────┼────┼────┼────┼────┤",
                "│ TOR │ -  │ -  │ -  │ -  │",
                "│ MTL │ -  │ -  │ -  │ -  │",
                "╰─────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_live() {
        let widget = GameBox::new(
            "BOS".to_string(),
            "NYR".to_string(),
            Some(2),
            Some(1),
            Some(vec![1, 1, 0]),
            Some(vec![0, 1, 0]),
            false,
            false,
            Some(2),
            GameState::Live {
                period_text: "2nd Period".to_string(),
                time_remaining: Some("12:34".to_string()),
                in_intermission: false,
            },
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " 2nd Period - 12:34",
                "╭─────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ T  │",
                "├─────┼────┼────┼────┼────┤",
                "│ BOS │ 1  │ 1  │ -  │ 2  │",
                "│ NYR │ 0  │ 1  │ -  │ 1  │",
                "╰─────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_live_intermission() {
        let widget = GameBox::new(
            "TOR".to_string(),
            "MTL".to_string(),
            Some(1),
            Some(0),
            Some(vec![1, 0, 0]),
            Some(vec![0, 0, 0]),
            false,
            false,
            Some(1),
            GameState::Live {
                period_text: "1st Period".to_string(),
                time_remaining: None,
                in_intermission: true,
            },
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " 1st Period - Intermission",
                "╭─────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ T  │",
                "├─────┼────┼────┼────┼────┤",
                "│ TOR │ 1  │ -  │ -  │ 1  │",
                "│ MTL │ 0  │ -  │ -  │ 0  │",
                "╰─────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_final() {
        let widget = GameBox::new(
            "EDM".to_string(),
            "VAN".to_string(),
            Some(4),
            Some(3),
            Some(vec![1, 1, 1, 1]),
            Some(vec![1, 1, 1, 0]),
            true,
            false,
            None,
            GameState::Final,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " Final Score",
                "╭─────┬────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ OT │ T  │",
                "├─────┼────┼────┼────┼────┼────┤",
                "│ EDM │ 1  │ 1  │ 1  │ 1  │ 4  │",
                "│ VAN │ 1  │ 1  │ 1  │ 0  │ 3  │",
                "╰─────┴────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_with_shootout() {
        let widget = GameBox::new(
            "CAR".to_string(),
            "NJD".to_string(),
            Some(4),
            Some(3),
            Some(vec![1, 1, 1, 0, 1]),
            Some(vec![1, 1, 1, 0, 0]),
            true,
            true,
            None,
            GameState::Final,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " Final Score",
                "╭─────┬────┬────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ OT │ SO │ T  │",
                "├─────┼────┼────┼────┼────┼────┼────┤",
                "│ CAR │ 1  │ 1  │ 1  │ 0  │ 1  │ 4  │",
                "│ NJD │ 1  │ 1  │ 1  │ 0  │ 0  │ 3  │",
                "╰─────┴────┴────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_preferred_dimensions() {
        let widget = GameBox::new(
            "A".to_string(),
            "B".to_string(),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            GameState::Scheduled {
                start_time: "12:00 PM".to_string(),
            },
            false,
        );

        // Should have fixed dimensions
        assert_eq!(widget.preferred_width(), Some(37));
        assert_eq!(widget.preferred_height(), Some(7));
    }

    #[test]
    fn test_game_box_header_formatting() {
        let widget = GameBox::new(
            "A".to_string(),
            "B".to_string(),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
            GameState::Scheduled {
                start_time: "Short".to_string(),
            },
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " Short",
                "╭─────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ T  │",
                "├─────┼────┼────┼────┼────┤",
                "│  A  │ -  │ -  │ -  │ -  │",
                "│  B  │ -  │ -  │ -  │ -  │",
                "╰─────┴────┴────┴────┴────╯",
            ],
        );
    }

    #[test]
    fn test_game_box_composition() {
        let widget = GameBox::new(
            "TOR".to_string(),
            "MTL".to_string(),
            Some(3),
            Some(2),
            Some(vec![1, 1, 1]),
            Some(vec![1, 1, 0]),
            false,
            false,
            None,
            GameState::Final,
            false,
        );

        let config = test_config();
        let buf = render_widget_with_config(&widget, 37, 7, &config);

        assert_buffer(
            &buf,
            &[
                " Final Score",
                "╭─────┬────┬────┬────┬────╮",
                "│     │ 1  │ 2  │ 3  │ T  │",
                "├─────┼────┼────┼────┼────┤",
                "│ TOR │ 1  │ 1  │ 1  │ 3  │",
                "│ MTL │ 1  │ 1  │ 0  │ 2  │",
                "╰─────┴────┴────┴────┴────╯",
            ],
        );
    }
}
