use tracing::{debug, info, trace};
use std::time::SystemTime;

use crate::commands::standings::GroupBy;

use super::action::{Action, ScoresAction, StandingsAction};
use super::component::Effect;
use super::state::{AppState, LoadingKey, PanelState};

/// Pure state reducer - like Redux reducer
///
/// Takes current state and an action, returns new state and optional effect.
/// This function is PURE - no side effects, no I/O, no async.
/// All side effects are returned as `Effect` to be executed separately.
pub fn reduce(state: AppState, action: Action) -> (AppState, Effect) {
    match action {
        Action::NavigateTab(tab) => {
            trace!("Navigating to tab: {:?}", tab);
            let mut new_state = state.clone();
            new_state.navigation.current_tab = tab;
            new_state.navigation.panel_stack.clear();
            new_state.navigation.content_focused = false; // Return focus to tab bar
            trace!("  Cleared panel stack and returned focus to tab bar");
            (new_state, Effect::None)
        }

        Action::NavigateTabLeft => {
            use super::action::Tab;
            let mut new_state = state.clone();
            new_state.navigation.current_tab = match new_state.navigation.current_tab {
                Tab::Scores => Tab::Browser,
                Tab::Standings => Tab::Scores,
                Tab::Stats => Tab::Standings,
                Tab::Players => Tab::Stats,
                Tab::Settings => Tab::Players,
                Tab::Browser => Tab::Settings,
            };
            new_state.navigation.panel_stack.clear();
            new_state.navigation.content_focused = false; // Return focus to tab bar
            (new_state, Effect::None)
        }

        Action::NavigateTabRight => {
            use super::action::Tab;
            let mut new_state = state.clone();
            new_state.navigation.current_tab = match new_state.navigation.current_tab {
                Tab::Scores => Tab::Standings,
                Tab::Standings => Tab::Stats,
                Tab::Stats => Tab::Players,
                Tab::Players => Tab::Settings,
                Tab::Settings => Tab::Browser,
                Tab::Browser => Tab::Scores,
            };
            new_state.navigation.panel_stack.clear();
            new_state.navigation.content_focused = false; // Return focus to tab bar
            (new_state, Effect::None)
        }

        Action::ToggleCommandPalette => {
            // TODO: Implement command palette toggling
            (state, Effect::None)
        }

        Action::EnterContentFocus => {
            debug!("FOCUS: Entering content focus (Down key from tab bar)");
            let mut new_state = state.clone();
            new_state.navigation.content_focused = true;
            (new_state, Effect::None)
        }

        Action::ExitContentFocus => {
            debug!("FOCUS: Exiting content focus (Up key to tab bar)");
            let mut new_state = state.clone();
            new_state.navigation.content_focused = false;

            // Also exit any tab-specific modes when returning to tab bar
            new_state.ui.scores.box_selection_active = false;
            new_state.ui.standings.team_mode = false;

            (new_state, Effect::None)
        }

        // Deprecated aliases - map to new actions
        Action::EnterSubtabMode => {
            debug!("FOCUS: EnterSubtabMode (deprecated, using EnterContentFocus)");
            reduce(state, Action::EnterContentFocus)
        }

        Action::ExitSubtabMode => {
            debug!("FOCUS: ExitSubtabMode (deprecated, using ExitContentFocus)");
            reduce(state, Action::ExitContentFocus)
        }

        Action::PushPanel(panel) => {
            debug!("PANEL: Pushing panel onto stack: {:?}", panel);
            let mut new_state = state.clone();
            new_state.navigation.panel_stack.push(PanelState {
                panel,
                scroll_offset: 0,
            });
            trace!("  Panel stack depth: {}", new_state.navigation.panel_stack.len());
            (new_state, Effect::None)
        }

        Action::PopPanel => {
            let popped = state.navigation.panel_stack.last();
            debug!("PANEL: Popping panel from stack: {:?}", popped);
            let mut new_state = state.clone();
            new_state.navigation.panel_stack.pop();
            trace!("  Panel stack depth: {}", new_state.navigation.panel_stack.len());
            (new_state, Effect::None)
        }

        Action::SetGameDate(date) => {
            let mut new_state = state.clone();
            new_state.ui.scores.game_date = date.clone();

            // Clear old schedule data
            new_state.data.schedule = None;
            new_state.data.game_info.clear();
            new_state.data.period_scores.clear();

            // Mark as loading
            new_state
                .data
                .loading
                .insert(LoadingKey::Schedule(date.to_string()));

            // Return effect to refresh data
            (new_state, Effect::Action(Action::RefreshData))
        }

        Action::StandingsLoaded(Ok(standings)) => {
            let mut new_state = state.clone();
            new_state.data.standings = Some(standings);
            new_state.data.loading.remove(&LoadingKey::Standings);
            new_state.data.errors.remove("standings");
            new_state.system.last_refresh = Some(SystemTime::now());
            (new_state, Effect::None)
        }

        Action::StandingsLoaded(Err(e)) => {
            let mut new_state = state.clone();
            new_state.data.loading.remove(&LoadingKey::Standings);
            new_state.data.errors.insert("standings".into(), e);
            (new_state, Effect::None)
        }

        Action::ScheduleLoaded(Ok(schedule)) => {
            debug!("DATA: Schedule loaded successfully with {} games", schedule.games.len());
            let mut new_state = state.clone();
            let game_date = new_state.ui.scores.game_date.to_string();
            new_state.data.schedule = Some(schedule);
            new_state
                .data
                .loading
                .remove(&LoadingKey::Schedule(game_date));
            new_state.data.errors.remove("schedule");
            new_state.system.last_refresh = Some(SystemTime::now());
            (new_state, Effect::None)
        }

        Action::ScheduleLoaded(Err(e)) => {
            let mut new_state = state.clone();
            let game_date = new_state.ui.scores.game_date.to_string();
            new_state
                .data
                .loading
                .remove(&LoadingKey::Schedule(game_date));
            new_state.data.errors.insert("schedule".into(), e);
            (new_state, Effect::None)
        }

        Action::GameDetailsLoaded(game_id, Ok(game_matchup)) => {
            let mut new_state = state.clone();

            // Extract period scores from the game matchup
            if let Some(ref summary) = game_matchup.summary {
                let period_scores = crate::commands::scores_format::extract_period_scores(summary);
                new_state.data.period_scores.insert(game_id, period_scores);
            }

            // Store the full game matchup
            new_state.data.game_info.insert(game_id, game_matchup);
            new_state
                .data
                .loading
                .remove(&LoadingKey::GameDetails(game_id));
            (new_state, Effect::None)
        }

        Action::GameDetailsLoaded(game_id, Err(e)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .loading
                .remove(&LoadingKey::GameDetails(game_id));
            new_state
                .data
                .errors
                .insert(format!("game_{}", game_id), e);
            (new_state, Effect::None)
        }

        Action::BoxscoreLoaded(game_id, Ok(boxscore)) => {
            let mut new_state = state.clone();
            new_state.data.boxscores.insert(game_id, boxscore);
            new_state
                .data
                .loading
                .remove(&LoadingKey::Boxscore(game_id));
            (new_state, Effect::None)
        }

        Action::BoxscoreLoaded(game_id, Err(e)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .loading
                .remove(&LoadingKey::Boxscore(game_id));
            new_state
                .data
                .errors
                .insert(format!("boxscore_{}", game_id), e);
            (new_state, Effect::None)
        }

        Action::TeamRosterLoaded(team_abbrev, Ok(roster)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .team_roster
                .insert(team_abbrev.clone(), roster);
            new_state
                .data
                .loading
                .remove(&LoadingKey::TeamRoster(team_abbrev));
            (new_state, Effect::None)
        }

        Action::TeamRosterLoaded(team_abbrev, Err(e)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .loading
                .remove(&LoadingKey::TeamRoster(team_abbrev.clone()));
            new_state
                .data
                .errors
                .insert(format!("roster_{}", team_abbrev), e);
            (new_state, Effect::None)
        }

        Action::PlayerStatsLoaded(player_id, Ok(stats)) => {
            let mut new_state = state.clone();
            new_state.data.player_stats.insert(player_id, stats);
            new_state
                .data
                .loading
                .remove(&LoadingKey::PlayerStats(player_id));
            (new_state, Effect::None)
        }

        Action::PlayerStatsLoaded(player_id, Err(e)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .loading
                .remove(&LoadingKey::PlayerStats(player_id));
            new_state
                .data
                .errors
                .insert(format!("player_{}", player_id), e);
            (new_state, Effect::None)
        }

        Action::ScrollUp(amount) => {
            let mut new_state = state.clone();
            if let Some(panel) = new_state.navigation.panel_stack.last_mut() {
                panel.scroll_offset = panel.scroll_offset.saturating_sub(amount);
            } else {
                match new_state.navigation.current_tab {
                    super::action::Tab::Standings => {
                        new_state.ui.standings.scroll_offset =
                            new_state.ui.standings.scroll_offset.saturating_sub(amount);
                    }
                    _ => {}
                }
            }
            (new_state, Effect::None)
        }

        Action::ScrollDown(amount) => {
            let mut new_state = state.clone();
            if let Some(panel) = new_state.navigation.panel_stack.last_mut() {
                panel.scroll_offset = panel.scroll_offset.saturating_add(amount);
            } else {
                match new_state.navigation.current_tab {
                    super::action::Tab::Standings => {
                        new_state.ui.standings.scroll_offset =
                            new_state.ui.standings.scroll_offset.saturating_add(amount);
                    }
                    _ => {}
                }
            }
            (new_state, Effect::None)
        }

        // Delegate to sub-reducers
        Action::ScoresAction(scores_action) => reduce_scores(state, scores_action),

        Action::StandingsAction(standings_action) => reduce_standings(state, standings_action),

        Action::Quit | Action::Error(_) | Action::RefreshData => (state, Effect::None),

        _ => (state, Effect::None),
    }
}

/// Sub-reducer for scores tab
fn reduce_scores(state: AppState, action: ScoresAction) -> (AppState, Effect) {
    match action {
        ScoresAction::DateLeft => {
            let mut new_state = state.clone();
            let ui = &mut new_state.ui.scores;

            if ui.selected_date_index == 0 {
                // At edge - shift window left
                ui.game_date = ui.game_date.add_days(-1);
            } else {
                // Within window - move index
                ui.selected_date_index -= 1;
                let window_base = ui.game_date.add_days(-(ui.selected_date_index as i64 + 1));
                ui.game_date = window_base.add_days(ui.selected_date_index as i64);
            }

            // Clear old data
            new_state.data.schedule = None;
            new_state.data.game_info.clear();
            new_state.data.period_scores.clear();

            // Effect: fetch schedule for new date
            let effect = Effect::Action(Action::RefreshData);

            (new_state, effect)
        }

        ScoresAction::DateRight => {
            let mut new_state = state.clone();
            let ui = &mut new_state.ui.scores;

            if ui.selected_date_index == 4 {
                // At edge - shift window right
                ui.game_date = ui.game_date.add_days(1);
            } else {
                // Within window - move index
                ui.selected_date_index += 1;
                let window_base = ui.game_date.add_days(-(ui.selected_date_index as i64 - 1));
                ui.game_date = window_base.add_days(ui.selected_date_index as i64);
            }

            // Clear old data
            new_state.data.schedule = None;
            new_state.data.game_info.clear();
            new_state.data.period_scores.clear();

            // Effect: fetch schedule for new date
            let effect = Effect::Action(Action::RefreshData);

            (new_state, effect)
        }

        ScoresAction::SelectGame => {
            // Get the selected game ID and push boxscore panel
            let mut new_state = state.clone();

            if let Some(selected_index) = new_state.ui.scores.selected_game_index {
                if let Some(schedule) = &new_state.data.schedule {
                    if let Some(game) = schedule.games.get(selected_index) {
                        let game_id = game.id;

                        // Push boxscore panel onto stack
                        // The runtime will detect this and trigger the fetch automatically
                        new_state.navigation.panel_stack.push(PanelState {
                            panel: super::action::Panel::Boxscore { game_id },
                            scroll_offset: 0,
                        });

                        return (new_state, Effect::None);
                    }
                }
            }

            (new_state, Effect::None)
        }

        ScoresAction::SelectGameById(game_id) => {
            let mut new_state = state.clone();

            // Push boxscore panel onto stack
            // The runtime will detect this and trigger the fetch automatically
            new_state.navigation.panel_stack.push(PanelState {
                panel: super::action::Panel::Boxscore { game_id },
                scroll_offset: 0,
            });

            (new_state, Effect::None)
        }

        ScoresAction::EnterBoxSelection => {
            debug!("FOCUS: Entering box selection mode (Scores tab)");
            let mut new_state = state.clone();
            new_state.ui.scores.box_selection_active = true;
            // Initialize selection to first game if we have games
            if new_state.ui.scores.selected_game_index.is_none() {
                if let Some(schedule) = &new_state.data.schedule {
                    if !schedule.games.is_empty() {
                        new_state.ui.scores.selected_game_index = Some(0);
                        trace!("  Initialized game selection to index 0");
                    }
                }
            }
            trace!("  Selected game index: {:?}", new_state.ui.scores.selected_game_index);
            (new_state, Effect::None)
        }

        ScoresAction::ExitBoxSelection => {
            debug!("FOCUS: Exiting box selection mode (Scores tab)");
            let mut new_state = state.clone();
            new_state.ui.scores.box_selection_active = false;
            (new_state, Effect::None)
        }

        ScoresAction::MoveGameSelectionUp => {
            let mut new_state = state.clone();
            if !new_state.ui.scores.box_selection_active {
                return (new_state, Effect::None);
            }

            let old_index = new_state.ui.scores.selected_game_index;
            if let Some(schedule) = &new_state.data.schedule {
                if let Some(current_index) = new_state.ui.scores.selected_game_index {
                    let num_games = schedule.games.len();
                    let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
                    if num_games > 0 && current_index >= boxes_per_row {
                        new_state.ui.scores.selected_game_index = Some(current_index - boxes_per_row);
                        trace!("Game selection: moved up from {} to {}", current_index, current_index - boxes_per_row);
                    }
                }
            }
            if old_index != new_state.ui.scores.selected_game_index {
                debug!("SELECTION: Game index changed: {:?} -> {:?}", old_index, new_state.ui.scores.selected_game_index);
            }
            (new_state, Effect::None)
        }

        ScoresAction::MoveGameSelectionDown => {
            let mut new_state = state.clone();
            if !new_state.ui.scores.box_selection_active {
                return (new_state, Effect::None);
            }

            if let Some(schedule) = &new_state.data.schedule {
                if let Some(current_index) = new_state.ui.scores.selected_game_index {
                    let num_games = schedule.games.len();
                    let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
                    if num_games > 0 {
                        let new_index = current_index + boxes_per_row;
                        if new_index < num_games {
                            new_state.ui.scores.selected_game_index = Some(new_index);
                        }
                    }
                }
            }
            (new_state, Effect::None)
        }

        ScoresAction::MoveGameSelectionLeft => {
            let mut new_state = state.clone();
            if !new_state.ui.scores.box_selection_active {
                return (new_state, Effect::None);
            }

            if let Some(current_index) = new_state.ui.scores.selected_game_index {
                let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
                // Get current column position (0-indexed within row)
                let col = current_index % boxes_per_row;
                // Only move left if not already in leftmost column
                if col > 0 {
                    new_state.ui.scores.selected_game_index = Some(current_index - 1);
                }
            }
            (new_state, Effect::None)
        }

        ScoresAction::MoveGameSelectionRight => {
            let mut new_state = state.clone();
            if !new_state.ui.scores.box_selection_active {
                return (new_state, Effect::None);
            }

            if let Some(schedule) = &new_state.data.schedule {
                if let Some(current_index) = new_state.ui.scores.selected_game_index {
                    let num_games = schedule.games.len();
                    let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
                    if num_games > 0 {
                        // Get current column position (0-indexed within row)
                        let col = current_index % boxes_per_row;
                        // Move right if not at rightmost column and next game exists
                        if col < boxes_per_row - 1 && current_index + 1 < num_games {
                            new_state.ui.scores.selected_game_index = Some(current_index + 1);
                        }
                    }
                }
            }
            (new_state, Effect::None)
        }

        ScoresAction::UpdateBoxesPerRow(boxes_per_row) => {
            let mut new_state = state.clone();
            new_state.ui.scores.boxes_per_row = boxes_per_row;
            (new_state, Effect::None)
        }
    }
}

/// Sub-reducer for standings tab
fn reduce_standings(state: AppState, action: StandingsAction) -> (AppState, Effect) {
    match action {
        StandingsAction::CycleView => {
            let mut new_state = state.clone();
            new_state.ui.standings.view = match new_state.ui.standings.view {
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Division,
                GroupBy::Wildcard => GroupBy::Division, // Loop back to Division
            };

            // Reset selection when changing views
            new_state.ui.standings.selected_column = 0;
            new_state.ui.standings.selected_row = 0;
            new_state.ui.standings.scroll_offset = 0;

            (new_state, Effect::None)
        }

        StandingsAction::EnterTeamMode => {
            let mut new_state = state.clone();
            new_state.ui.standings.team_mode = true;
            (new_state, Effect::None)
        }

        StandingsAction::ExitTeamMode => {
            let mut new_state = state.clone();
            new_state.ui.standings.team_mode = false;
            (new_state, Effect::None)
        }

        StandingsAction::SelectTeam => {
            // TODO: Implement team selection (push panel?)
            (state, Effect::None)
        }

        StandingsAction::SelectTeamByPosition(column, row) => {
            let mut new_state = state.clone();
            new_state.ui.standings.selected_column = column;
            new_state.ui.standings.selected_row = row;
            (new_state, Effect::None)
        }

        StandingsAction::CycleViewLeft => {
            let mut new_state = state.clone();
            new_state.ui.standings.view = match new_state.ui.standings.view {
                GroupBy::Division => GroupBy::League,
                GroupBy::Conference => GroupBy::Division,
                GroupBy::League => GroupBy::Conference,
                GroupBy::Wildcard => GroupBy::League,
            };
            (new_state, Effect::None)
        }

        StandingsAction::CycleViewRight => {
            let mut new_state = state.clone();
            new_state.ui.standings.view = match new_state.ui.standings.view {
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Division,
                GroupBy::Wildcard => GroupBy::Division,
            };
            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionUp => {
            let mut new_state = state.clone();
            if new_state.ui.standings.selected_row > 0 {
                new_state.ui.standings.selected_row -= 1;
            }
            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionDown => {
            let mut new_state = state.clone();
            new_state.ui.standings.selected_row += 1;
            // TODO: Clamp to max teams in column
            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionLeft => {
            let mut new_state = state.clone();
            if new_state.ui.standings.selected_column > 0 {
                new_state.ui.standings.selected_column -= 1;
            }
            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionRight => {
            let mut new_state = state.clone();
            new_state.ui.standings.selected_column += 1;
            // TODO: Clamp to max columns for view
            (new_state, Effect::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::GameDate;

    #[test]
    fn test_navigate_tab() {
        let state = AppState::default();
        let (new_state, effect) =
            reduce(state.clone(), Action::NavigateTab(super::super::action::Tab::Standings));

        assert_eq!(
            new_state.navigation.current_tab,
            super::super::action::Tab::Standings
        );
        assert!(new_state.navigation.panel_stack.is_empty());
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_standings_loaded_success() {
        let state = AppState::default();
        let standings = vec![];
        let (new_state, effect) = reduce(state, Action::StandingsLoaded(Ok(standings.clone())));

        assert_eq!(new_state.data.standings, Some(standings));
        assert!(!new_state.data.loading.contains(&LoadingKey::Standings));
        assert!(!new_state.data.errors.contains_key("standings"));
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_standings_loaded_error() {
        let state = AppState::default();
        let error_msg = "Network error".to_string();
        let (new_state, effect) = reduce(state, Action::StandingsLoaded(Err(error_msg.clone())));

        assert!(new_state.data.standings.is_none());
        assert!(!new_state.data.loading.contains(&LoadingKey::Standings));
        assert_eq!(
            new_state.data.errors.get("standings"),
            Some(&error_msg)
        );
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn test_scores_date_left_within_window() {
        let mut state = AppState::default();
        state.ui.scores.selected_date_index = 2;
        state.ui.scores.game_date = GameDate::today();

        let (new_state, _effect) = reduce_scores(state.clone(), ScoresAction::DateLeft);

        assert_eq!(new_state.ui.scores.selected_date_index, 1);
        assert!(new_state.data.schedule.is_none());
    }

    #[test]
    fn test_scores_date_left_at_edge() {
        let mut state = AppState::default();
        state.ui.scores.selected_date_index = 0;
        let original_date = GameDate::today();
        state.ui.scores.game_date = original_date.clone();

        let (new_state, _effect) = reduce_scores(state.clone(), ScoresAction::DateLeft);

        assert_eq!(new_state.ui.scores.selected_date_index, 0);
        assert_eq!(
            new_state.ui.scores.game_date,
            original_date.add_days(-1)
        );
    }

    #[test]
    fn test_standings_cycle_view() {
        let state = AppState::default();

        let (new_state, _) = reduce_standings(state.clone(), StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);

        let (new_state, _) = reduce_standings(new_state, StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::League);

        let (new_state, _) = reduce_standings(new_state, StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::Division);
    }
}
