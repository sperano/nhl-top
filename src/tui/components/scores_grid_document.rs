//! Document implementation for scores grid view
//!
//! This module provides a Document implementation that displays games in a
//! Row-based grid layout, making each game box a focusable element.

use std::collections::HashMap;
use std::sync::Arc;

use nhl_api::{DailySchedule, GameDate, GameMatchup};

use crate::commands::scores_format::PeriodScores;
use crate::tui::document::{Document, DocumentBuilder, DocumentElement, FocusContext, FocusableId};

/// Constant for game box height
//const GAME_BOX_HEIGHT: u16 = 7;

/// Document that displays games in a grid layout using Row elements
pub struct ScoresGridDocument {
    pub schedule: Arc<Option<DailySchedule>>,
    pub game_info: Arc<HashMap<i64, GameMatchup>>,
    pub period_scores: Arc<HashMap<i64, PeriodScores>>,
    pub boxes_per_row: u16,
    pub game_date: GameDate,
}

impl ScoresGridDocument {
    pub fn new(
        schedule: Arc<Option<DailySchedule>>,
        game_info: Arc<HashMap<i64, GameMatchup>>,
        period_scores: Arc<HashMap<i64, PeriodScores>>,
        boxes_per_row: u16,
        game_date: GameDate,
    ) -> Self {
        Self {
            schedule,
            game_info,
            period_scores,
            boxes_per_row,
            game_date,
        }
    }

    /// Create a GameBox for a given game
    /// Format a game as a text label for now (will be replaced with actual GameBox rendering)
    fn format_game_label(&self, game: &nhl_api::ScheduleGame) -> String {
        format!("{} @ {}", game.away_team.abbrev, game.home_team.abbrev)
    }
}

impl Document for ScoresGridDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        // Return empty if no schedule
        let Some(schedule) = self.schedule.as_ref() else {
            return DocumentBuilder::new()
                .text("No games scheduled for this date")
                .build();
        };

        if schedule.games.is_empty() {
            return DocumentBuilder::new()
                .text("No games scheduled for this date")
                .build();
        }

        let mut builder = DocumentBuilder::new();

        // Group games into rows
        let games: Vec<&nhl_api::ScheduleGame> = schedule.games.iter().collect();
        let chunks: Vec<&[&nhl_api::ScheduleGame]> =
            games.chunks(self.boxes_per_row as usize).collect();

        for (_row_idx, chunk) in chunks.iter().enumerate() {
            // Create game box elements for this row
            let game_elements: Vec<DocumentElement> = chunk
                .iter()
                .enumerate()
                .map(|(_col_idx, game)| {
                    let game_id = format!("game_{}", game.id);
                    let label = self.format_game_label(game);
                    let focused = focus.focused_id == Some(FocusableId::Link(game_id.clone()));

                    // For now, use Link elements (will be replaced with Custom rendering later)
                    // Target is an Action that will trigger PushPanel
                    DocumentElement::Link {
                        display: label,
                        target: crate::tui::document::LinkTarget::Action(
                            format!("open_boxscore_{}", game.id)
                        ),
                        id: game_id,
                        focused,
                    }
                })
                .collect();

            // Add row to document
            builder = builder.row(game_elements);
        }

        builder.build()
    }

    fn title(&self) -> String {
        format!("Games for {}", self.game_date)
    }

    fn id(&self) -> String {
        format!("scores_{}", self.game_date)
    }
}

// TODO: GameBoxWidget will be implemented later when we add Custom rendering support
// For now, we're using Link elements as placeholders

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::{GameState as ApiGameState, ScheduleGame, ScheduleTeam};

    fn create_test_game(id: i64, away: &str, home: &str) -> ScheduleGame {
        ScheduleGame {
            id,
            game_type: nhl_api::GameType::RegularSeason,
            game_date: Some("2024-01-15".to_string()),
            start_time_utc: "2024-01-15T20:00:00Z".to_string(),
            game_state: ApiGameState::Final,
            away_team: ScheduleTeam {
                id: 1,
                abbrev: away.to_string(),
                score: Some(2),
                logo: String::new(),
                place_name: None,
            },
            home_team: ScheduleTeam {
                id: 2,
                abbrev: home.to_string(),
                score: Some(3),
                logo: String::new(),
                place_name: None,
            },
        }
    }

    #[test]
    fn test_empty_schedule_returns_empty_document() {
        let doc = ScoresGridDocument::new(
            Arc::new(None),
            Arc::new(HashMap::new()),
            Arc::new(HashMap::new()),
            2,
            GameDate::today(),
        );

        let focus = FocusContext { focused_id: None };
        let elements = doc.build(&focus);

        // Should return a text element saying no games
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_single_game_layout() {
        let game = create_test_game(1, "TOR", "MTL");
        let schedule = DailySchedule {
            date: "2024-01-15".to_string(),
            games: vec![game],
            next_start_date: None,
            previous_start_date: None,
            number_of_games: 1,
        };

        let doc = ScoresGridDocument::new(
            Arc::new(Some(schedule)),
            Arc::new(HashMap::new()),
            Arc::new(HashMap::new()),
            2,
            GameDate::today(),
        );

        let focus = FocusContext { focused_id: None };
        let elements = doc.build(&focus);

        // Should have 1 row with 1 game
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_two_games_in_row() {
        let game1 = create_test_game(1, "TOR", "MTL");
        let game2 = create_test_game(2, "BOS", "NYR");
        let schedule = DailySchedule {
            date: "2024-01-15".to_string(),
            games: vec![game1, game2],
            next_start_date: None,
            previous_start_date: None,
            number_of_games: 2,
        };

        let doc = ScoresGridDocument::new(
            Arc::new(Some(schedule)),
            Arc::new(HashMap::new()),
            Arc::new(HashMap::new()),
            2, // 2 boxes per row
            GameDate::today(),
        );

        let focus = FocusContext { focused_id: None };
        let elements = doc.build(&focus);

        // Should have 1 row with 2 games
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_three_games_two_rows() {
        let game1 = create_test_game(1, "TOR", "MTL");
        let game2 = create_test_game(2, "BOS", "NYR");
        let game3 = create_test_game(3, "EDM", "VAN");
        let schedule = DailySchedule {
            date: "2024-01-15".to_string(),
            games: vec![game1, game2, game3],
            next_start_date: None,
            previous_start_date: None,
            number_of_games: 3,
        };

        let doc = ScoresGridDocument::new(
            Arc::new(Some(schedule)),
            Arc::new(HashMap::new()),
            Arc::new(HashMap::new()),
            2, // 2 boxes per row
            GameDate::today(),
        );

        let focus = FocusContext { focused_id: None };
        let elements = doc.build(&focus);

        // Should have 2 rows (first with 2 games, second with 1 game)
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn test_title_and_id() {
        let doc = ScoresGridDocument::new(
            Arc::new(None),
            Arc::new(HashMap::new()),
            Arc::new(HashMap::new()),
            2,
            GameDate::today(),
        );

        assert!(doc.title().contains("Games for"));
        assert!(doc.id().starts_with("scores_"));
    }
}
