use std::sync::Arc;

use nhl_api::{Client, GameDate, GameId};

use super::action::Action;
use super::component::Effect;
use super::state::{AppState};

/// Regular season game type identifier
const REGULAR_SEASON: i32 = 2;

/// Effect handler for data fetching operations
///
/// This handles all async data fetching from the NHL API.
/// Each method returns an Effect that will dispatch the appropriate
/// *Loaded action when complete.
pub struct DataEffects {
    client: Arc<Client>,
}

impl DataEffects {
    /// Create a new DataEffects handler with an NHL API client
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// Handle a refresh request - fetches all necessary data based on current state
    pub fn handle_refresh(&self, state: &AppState) -> Effect {
        let mut effects = vec![
            self.fetch_standings(),
            self.fetch_schedule(state.ui.scores.game_date.clone()),
        ];

        // Add game detail fetches for started games
        if let Some(schedule) = &state.data.schedule {
            for game in &schedule.games {
                // Only fetch details for games that have started
                if game.game_state != nhl_api::GameState::Future
                    && game.game_state != nhl_api::GameState::PreGame
                {
                    effects.push(self.fetch_game_details(game.id));
                }
            }
        }

        Effect::Batch(effects)
    }

    /// Fetch current league standings
    pub fn fetch_standings(&self) -> Effect {
        let client = self.client.clone();
        Effect::Async(Box::pin(async move {
            let result = client.current_league_standings().await;
            Action::StandingsLoaded(result.map_err(|e| e.to_string()))
        }))
    }

    /// Fetch daily schedule for a specific date
    pub fn fetch_schedule(&self, date: GameDate) -> Effect {
        let client = self.client.clone();
        Effect::Async(Box::pin(async move {
            let result = client.daily_schedule(Some(date)).await;
            Action::ScheduleLoaded(result.map_err(|e| e.to_string()))
        }))
    }

    /// Fetch game details for a specific game
    pub fn fetch_game_details(&self, game_id: i64) -> Effect {
        let client = self.client.clone();
        Effect::Async(Box::pin(async move {
            let result = client.landing(game_id).await;
            Action::GameDetailsLoaded(game_id, result.map_err(|e| e.to_string()))
        }))
    }

    /// Fetch team roster stats for a specific team (current season, regular season)
    ///
    /// This method dynamically determines the current season by fetching available seasons
    /// and selecting the most recent one that has regular season data.
    pub fn fetch_team_roster_stats(&self, team_abbrev: String) -> Effect {
        let client = self.client.clone();
        let abbrev = team_abbrev.clone();
        Effect::Async(Box::pin(async move {
            // First, get available seasons for this team
            let seasons_result = client.club_stats_season(&abbrev).await;

            let result = match seasons_result {
                Ok(seasons) => {
                    // Find the most recent season that has regular season data (game_type = 2)
                    let current_season = seasons
                        .iter()
                        .filter(|s| s.game_types.contains(&REGULAR_SEASON))
                        .max_by_key(|s| s.season);

                    match current_season {
                        Some(season_info) => {
                            // Fetch stats for the current season
                            client.club_stats(&abbrev, season_info.season, REGULAR_SEASON).await
                        }
                        None => {
                            Err(nhl_api::NHLApiError::ApiError {
                                message: "No regular season data available for team".to_string(),
                                status_code: 404,
                            })
                        }
                    }
                }
                Err(e) => Err(e),
            };

            Action::TeamRosterStatsLoaded(team_abbrev, result.map_err(|e| e.to_string()))
        }))
    }

    /// Fetch player stats for a specific player
    pub fn fetch_player_stats(&self, player_id: i64) -> Effect {
        Effect::Async(Box::pin(async move {
            // TODO: Implement player stats fetching when available in nhl_api
            // For now, return a placeholder
            let result = Ok(super::action::PlayerStats { player_id });
            Action::PlayerStatsLoaded(player_id, result)
        }))
    }

    /// Fetch boxscore for a specific game
    pub fn fetch_boxscore(&self, game_id: i64) -> Effect {
        let client = self.client.clone();
        Effect::Async(Box::pin(async move {
            let result = client.boxscore(GameId::new(game_id)).await;
            Action::BoxscoreLoaded(game_id, result.map_err(|e| e.to_string()))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::framework::action::Tab;
    use crate::tui::framework::state::{NavigationState, SystemState, UiState};

    fn create_test_state() -> AppState {
        AppState {
            navigation: NavigationState {
                current_tab: Tab::Scores,
                panel_stack: Vec::new(),
                content_focused: false,
            },
            data: Default::default(),
            ui: UiState::default(),
            system: SystemState::default(),
        }
    }

    #[test]
    fn test_fetch_standings_returns_async_effect() {
        let client = crate::tui::testing::create_client();
        let effects = DataEffects::new(client);

        let effect = effects.fetch_standings();

        // Verify it returns an Async effect
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_fetch_schedule_returns_async_effect() {
        let client = crate::tui::testing::create_client();
        let effects = DataEffects::new(client);

        let date = GameDate::default();
        let effect = effects.fetch_schedule(date);

        // Verify it returns an Async effect
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_fetch_game_details_returns_async_effect() {
        let client = crate::tui::testing::create_client();
        let effects = DataEffects::new(client);

        let effect = effects.fetch_game_details(2024020001);

        // Verify it returns an Async effect
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_handle_refresh_returns_batch_effect() {
        let client = crate::tui::testing::create_client();
        let effects = DataEffects::new(client);
        let state = create_test_state();

        let effect = effects.handle_refresh(&state);

        // Verify it returns a Batch effect
        assert!(matches!(effect, Effect::Batch(_)));
    }

    #[test]
    fn test_handle_refresh_includes_standings_and_schedule() {
        let client = crate::tui::testing::create_client();
        let effects = DataEffects::new(client);
        let state = create_test_state();

        let effect = effects.handle_refresh(&state);

        // Verify batch contains at least 2 effects (standings + schedule)
        if let Effect::Batch(effects) = effect {
            assert!(effects.len() >= 2);
        } else {
            panic!("Expected Batch effect");
        }
    }
}
