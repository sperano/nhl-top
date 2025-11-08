/// GameGrid widget - displays multiple GameBox widgets in a multi-column grid
///
/// Automatically calculates the number of columns based on available width:
/// - 3 columns for width >= 115
/// - 2 columns for width >= 76
/// - 1 column otherwise

use ratatui::{buffer::Buffer, layout::Rect};
use crate::config::DisplayConfig;
use crate::tui::widgets::{RenderableWidget, GameBox};

/// Constants for grid layout
const GAME_BOX_WIDTH: u16 = 37;
const GAME_BOX_HEIGHT: u16 = 7;
const GAME_BOX_GAP: u16 = 2;
const THREE_COLUMN_WIDTH: u16 = 115; // 37*3 + 2*2
const TWO_COLUMN_WIDTH: u16 = 76;    // 37*2 + 2

/// Widget for displaying multiple game boxes in a grid layout
#[derive(Debug, Clone)]
pub struct GameGrid {
    /// Game boxes to display
    pub games: Vec<GameBox>,
}

impl GameGrid {
    /// Create a new GameGrid widget
    pub fn new(games: Vec<GameBox>) -> Self {
        Self { games }
    }

    /// Calculate number of columns based on available width
    fn calculate_columns(&self, width: u16) -> usize {
        if width >= THREE_COLUMN_WIDTH {
            3
        } else if width >= TWO_COLUMN_WIDTH {
            2
        } else {
            1
        }
    }
}

impl RenderableWidget for GameGrid {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.games.is_empty() {
            return;
        }

        let columns = self.calculate_columns(area.width);
        if columns == 0 {
            return;
        }

        // Group games into rows
        let mut y_offset = 0;
        for row_games in self.games.chunks(columns) {
            if area.y + y_offset >= area.bottom() {
                break; // No more vertical space
            }

            // Render each game in this row
            for (col_idx, game) in row_games.iter().enumerate() {
                let x = area.x + (GAME_BOX_WIDTH + GAME_BOX_GAP) * col_idx as u16;

                // Check if we have horizontal space for this column
                if x + GAME_BOX_WIDTH > area.right() {
                    break;
                }

                let game_area = Rect::new(
                    x,
                    area.y + y_offset,
                    GAME_BOX_WIDTH,
                    GAME_BOX_HEIGHT.min(area.bottom() - (area.y + y_offset)),
                );

                game.render(game_area, buf, config);
            }

            y_offset += GAME_BOX_HEIGHT;
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        // We don't know the width here, so we can't calculate exact height
        // Return None to indicate we can adapt to any height
        None
    }

    fn preferred_width(&self) -> Option<u16> {
        // Prefer 3 columns if we have enough games, otherwise match game count
        if self.games.is_empty() {
            Some(0)
        } else if self.games.len() >= 3 {
            Some(THREE_COLUMN_WIDTH)
        } else if self.games.len() >= 2 {
            Some(TWO_COLUMN_WIDTH)
        } else {
            Some(GAME_BOX_WIDTH)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::{GameState, testing::*};

    fn create_test_game(away: &str, home: &str) -> GameBox {
        GameBox::new(
            away.to_string(),
            home.to_string(),
            Some(3),
            Some(2),
            Some(vec![1, 1, 1]),
            Some(vec![1, 1, 0]),
            false,
            false,
            None,
            GameState::Final,
            false,
        )
    }

    #[test]
    fn test_game_grid_empty() {
        let widget = GameGrid::new(vec![]);
        let config = test_config();
        let buf = render_widget_with_config(&widget, 115, 20, &config);

        // Empty grid should render nothing
        for y in 0..20 {
            assert_eq!(buffer_line(&buf, y).trim(), "");
        }
    }

    #[test]
    fn test_game_grid_single_game() {
        let game = create_test_game("TOR", "MTL");
        let widget = GameGrid::new(vec![game]);
        let config = test_config();
        let buf = render_widget_with_config(&widget, 115, 20, &config);

        // Should render one game in the top-left
        assert!(buffer_line(&buf, 0).contains("Final Score"));
        assert!(buffer_line(&buf, 4).contains("TOR"));
        assert!(buffer_line(&buf, 5).contains("MTL"));
    }

    #[test]
    fn test_game_grid_two_games_wide_screen() {
        let game1 = create_test_game("TOR", "MTL");
        let game2 = create_test_game("BOS", "NYR");
        let widget = GameGrid::new(vec![game1, game2]);
        let config = test_config();

        // Wide screen: both games should be side-by-side
        let buf = render_widget_with_config(&widget, 115, 20, &config);

        // First game (column 0)
        let row4 = buffer_line(&buf, 4);
        assert!(row4.contains("TOR"));

        // Second game (column 1, starts at x=37+2=39)
        assert!(row4.contains("BOS"));
    }

    #[test]
    fn test_game_grid_three_games_wide_screen() {
        let game1 = create_test_game("TOR", "MTL");
        let game2 = create_test_game("BOS", "NYR");
        let game3 = create_test_game("EDM", "VAN");
        let widget = GameGrid::new(vec![game1, game2, game3]);
        let config = test_config();

        // Wide screen: all three games in one row
        let buf = render_widget_with_config(&widget, 115, 20, &config);

        let row4 = buffer_line(&buf, 4);
        assert!(row4.contains("TOR"));
        assert!(row4.contains("BOS"));
        assert!(row4.contains("EDM"));
    }

    #[test]
    fn test_game_grid_four_games_two_rows() {
        let games = vec![
            create_test_game("TOR", "MTL"),
            create_test_game("BOS", "NYR"),
            create_test_game("EDM", "VAN"),
            create_test_game("CAR", "NJD"),
        ];
        let widget = GameGrid::new(games);
        let config = test_config();

        // Wide screen: 3 games in first row, 1 in second row
        let buf = render_widget_with_config(&widget, 115, 20, &config);

        // First row (y=0-6)
        let row4 = buffer_line(&buf, 4);
        assert!(row4.contains("TOR"));
        assert!(row4.contains("BOS"));
        assert!(row4.contains("EDM"));

        // Second row (y=7-13)
        let row11 = buffer_line(&buf, 11); // 7 + 4
        assert!(row11.contains("CAR"));
    }

    #[test]
    fn test_game_grid_narrow_screen_single_column() {
        let game1 = create_test_game("TOR", "MTL");
        let game2 = create_test_game("BOS", "NYR");
        let widget = GameGrid::new(vec![game1, game2]);
        let config = test_config();

        // Narrow screen: single column layout
        let buf = render_widget_with_config(&widget, 50, 20, &config);

        // First game (row 0)
        assert!(buffer_line(&buf, 4).contains("TOR"));
        assert!(!buffer_line(&buf, 4).contains("BOS"));

        // Second game (row 1, starts at y=7)
        assert!(buffer_line(&buf, 11).contains("BOS")); // 7 + 4
    }

    #[test]
    fn test_game_grid_medium_screen_two_columns() {
        let games = vec![
            create_test_game("TOR", "MTL"),
            create_test_game("BOS", "NYR"),
            create_test_game("EDM", "VAN"),
        ];
        let widget = GameGrid::new(games);
        let config = test_config();

        // Medium screen: 2 columns
        let buf = render_widget_with_config(&widget, 80, 20, &config);

        // First row: 2 games
        let row4 = buffer_line(&buf, 4);
        assert!(row4.contains("TOR"));
        assert!(row4.contains("BOS"));
        assert!(!row4.contains("EDM"));

        // Second row: 1 game
        let row11 = buffer_line(&buf, 11);
        assert!(row11.contains("EDM"));
    }

    #[test]
    fn test_game_grid_calculate_columns() {
        let widget = GameGrid::new(vec![]);

        assert_eq!(widget.calculate_columns(120), 3);
        assert_eq!(widget.calculate_columns(115), 3);
        assert_eq!(widget.calculate_columns(100), 2);
        assert_eq!(widget.calculate_columns(76), 2);
        assert_eq!(widget.calculate_columns(75), 1);
        assert_eq!(widget.calculate_columns(50), 1);
    }

    #[test]
    fn test_game_grid_preferred_width() {
        // 3+ games: prefer 3 columns
        let widget3 = GameGrid::new(vec![
            create_test_game("A", "B"),
            create_test_game("C", "D"),
            create_test_game("E", "F"),
        ]);
        assert_eq!(widget3.preferred_width(), Some(THREE_COLUMN_WIDTH));

        // 2 games: prefer 2 columns
        let widget2 = GameGrid::new(vec![
            create_test_game("A", "B"),
            create_test_game("C", "D"),
        ]);
        assert_eq!(widget2.preferred_width(), Some(TWO_COLUMN_WIDTH));

        // 1 game: single column
        let widget1 = GameGrid::new(vec![create_test_game("A", "B")]);
        assert_eq!(widget1.preferred_width(), Some(GAME_BOX_WIDTH));

        // 0 games: 0 width
        let widget0 = GameGrid::new(vec![]);
        assert_eq!(widget0.preferred_width(), Some(0));
    }

    #[test]
    fn test_game_grid_vertical_overflow() {
        // Create more games than can fit vertically
        let games = vec![
            create_test_game("A", "B"),
            create_test_game("C", "D"),
            create_test_game("E", "F"),
            create_test_game("G", "H"),
        ];
        let widget = GameGrid::new(games);
        let config = test_config();

        // Only 10 rows of height (can fit 1 game + part of another)
        let buf = render_widget_with_config(&widget, 115, 10, &config);

        // First row should be visible
        assert!(buffer_line(&buf, 4).contains('A'));

        // Second row (y=7) should be partially cut off at y=10
        // We'll just check it doesn't panic
        let _ = buffer_line(&buf, 7);
    }
}
