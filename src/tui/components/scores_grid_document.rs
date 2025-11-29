//! Document implementation for scores grid view
//!
//! This module provides a Document implementation that displays games in a
//! Row-based grid layout, making each game box a focusable element.

use std::collections::HashMap;
use std::sync::Arc;

use nhl_api::{DailySchedule, GameDate, GameMatchup};

use crate::commands::scores_format::{format_period_text, PeriodScores};
use crate::tui::document::{Document, DocumentBuilder, DocumentElement, FocusContext, FocusableId};
use crate::tui::widgets::{GameBox, GameState as WidgetGameState};

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

    /// Create a GameBox widget for a given game
    fn create_game_box(&self, game: &nhl_api::ScheduleGame) -> GameBox {
        // Determine game state
        let state = if game.game_state.is_final() {
            WidgetGameState::Final
        } else if game.game_state.has_started() {
            // Get period text and time from game_info
            if let Some(info) = self.game_info.get(&game.id) {
                let period_text = format_period_text(
                    info.period_descriptor.period_type,
                    info.period_descriptor.number,
                );
                let (time_remaining, in_intermission) = if let Some(clock) = &info.clock {
                    (Some(clock.time_remaining.clone()), clock.in_intermission)
                } else {
                    (None, false)
                };
                WidgetGameState::Live {
                    period_text,
                    time_remaining,
                    in_intermission,
                }
            } else {
                WidgetGameState::Live {
                    period_text: "Live".to_string(),
                    time_remaining: None,
                    in_intermission: false,
                }
            }
        } else {
            // Scheduled game - format start time
            let start_time =
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
                    let local_time: chrono::DateTime<chrono::Local> = parsed.into();
                    local_time.format("%I:%M %p").to_string()
                } else {
                    game.start_time_utc.clone()
                };
            WidgetGameState::Scheduled { start_time }
        };

        // Get scores and period details
        let (away_score, home_score, away_periods, home_periods, has_ot, has_so) =
            if let Some(scores) = self.period_scores.get(&game.id) {
                (
                    Some(scores.away_total()),
                    Some(scores.home_total()),
                    Some(scores.away_periods.clone()),
                    Some(scores.home_periods.clone()),
                    scores.has_ot,
                    scores.has_so,
                )
            } else {
                (None, None, None, None, false, false)
            };

        // Get current period
        let current_period = self
            .game_info
            .get(&game.id)
            .map(|info| info.period_descriptor.number);

        GameBox::new(
            game.away_team.abbrev.clone(),
            game.home_team.abbrev.clone(),
            away_score,
            home_score,
            away_periods,
            home_periods,
            has_ot,
            has_so,
            current_period,
            state,
            false, // selected is set by focused state during rendering
        )
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

        for chunk in chunks.iter() {
            // Create GameBox elements for this row
            let game_elements: Vec<DocumentElement> = chunk
                .iter()
                .map(|game| {
                    // GameBoxElement uses FocusableId::GameLink(game_id)
                    let focused = focus.focused_id == Some(FocusableId::GameLink(game.id));

                    // Create the GameBox widget with all score data
                    let game_box = self.create_game_box(game);

                    // Use the new GameBoxElement variant
                    DocumentElement::game_box_element(game.id, game_box, focused)
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
