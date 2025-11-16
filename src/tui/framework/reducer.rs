use tracing::{debug, trace};
use std::time::SystemTime;

use crate::commands::standings::GroupBy;

use super::action::{Action, ScoresAction, SettingsAction, StandingsAction};
use super::component::Effect;
use super::state::{AppState, LoadingKey, PanelState, SettingsCategory};

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
            new_state.ui.standings.browse_mode = false;
            new_state.ui.settings.settings_mode = false;

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

        Action::SettingsAction(settings_action) => reduce_settings(state, settings_action),

        Action::RefreshData => {
            let mut new_state = state.clone();
            new_state.system.last_refresh = Some(SystemTime::now());
            (new_state, Effect::None)
        }

        Action::Quit | Action::Error(_) => (state, Effect::None),

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
                GroupBy::Wildcard => GroupBy::Division,
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Wildcard,
            };

            // Reset selection when changing views
            new_state.ui.standings.selected_column = 0;
            new_state.ui.standings.selected_row = 0;
            new_state.ui.standings.scroll_offset = 0;

            (new_state, Effect::None)
        }

        StandingsAction::EnterBrowseMode => {
            let mut new_state = state.clone();
            new_state.ui.standings.browse_mode = true;
            (new_state, Effect::None)
        }

        StandingsAction::ExitBrowseMode => {
            let mut new_state = state.clone();
            new_state.ui.standings.browse_mode = false;
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
                GroupBy::Wildcard => GroupBy::League,
                GroupBy::Division => GroupBy::Wildcard,
                GroupBy::Conference => GroupBy::Division,
                GroupBy::League => GroupBy::Conference,
            };
            (new_state, Effect::None)
        }

        StandingsAction::CycleViewRight => {
            let mut new_state = state.clone();
            new_state.ui.standings.view = match new_state.ui.standings.view {
                GroupBy::Wildcard => GroupBy::Division,
                GroupBy::Division => GroupBy::Conference,
                GroupBy::Conference => GroupBy::League,
                GroupBy::League => GroupBy::Wildcard,
            };
            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionUp => {
            let mut new_state = state.clone();

            // Get team count (respects column in Conference/Division/Wildcard views)
            if let Some(ref standings) = new_state.data.standings {
                let team_count = match new_state.ui.standings.view {
                    GroupBy::Conference => {
                        count_teams_in_conference_column(standings, new_state.ui.standings.selected_column)
                    }
                    GroupBy::Division => {
                        count_teams_in_division_column(
                            standings,
                            new_state.ui.standings.selected_column,
                            new_state.system.config.display_standings_western_first,
                        )
                    }
                    GroupBy::Wildcard => {
                        count_teams_in_wildcard_column(
                            standings,
                            new_state.ui.standings.selected_column,
                            new_state.system.config.display_standings_western_first,
                        )
                    }
                    _ => standings.len(),
                };

                if team_count > 0 {
                    let max_row = team_count - 1;
                    if new_state.ui.standings.selected_row == 0 {
                        // At first team - wrap to last team
                        new_state.ui.standings.selected_row = max_row;
                    } else {
                        new_state.ui.standings.selected_row -= 1;
                    }
                }
            }

            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionDown => {
            let mut new_state = state.clone();

            // Get team count (respects column in Conference/Division/Wildcard views)
            if let Some(ref standings) = new_state.data.standings {
                let team_count = match new_state.ui.standings.view {
                    GroupBy::Conference => {
                        count_teams_in_conference_column(standings, new_state.ui.standings.selected_column)
                    }
                    GroupBy::Division => {
                        count_teams_in_division_column(
                            standings,
                            new_state.ui.standings.selected_column,
                            new_state.system.config.display_standings_western_first,
                        )
                    }
                    GroupBy::Wildcard => {
                        count_teams_in_wildcard_column(
                            standings,
                            new_state.ui.standings.selected_column,
                            new_state.system.config.display_standings_western_first,
                        )
                    }
                    _ => standings.len(),
                };

                if team_count > 0 {
                    let max_row = team_count - 1;
                    if new_state.ui.standings.selected_row >= max_row {
                        // At last team - wrap to first team
                        new_state.ui.standings.selected_row = 0;
                    } else {
                        new_state.ui.standings.selected_row += 1;
                    }
                }
            }

            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionLeft => {
            let mut new_state = state.clone();

            // Conference, Division, and Wildcard views have 2 columns for navigation
            if matches!(new_state.ui.standings.view, GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard) {
                // Wrap around: 0 -> 1
                new_state.ui.standings.selected_column = if new_state.ui.standings.selected_column == 0 {
                    1
                } else {
                    0
                };

                // Clamp row to max teams in new column if needed
                if let Some(ref standings) = new_state.data.standings {
                    let team_count = match new_state.ui.standings.view {
                        GroupBy::Conference => {
                            count_teams_in_conference_column(standings, new_state.ui.standings.selected_column)
                        }
                        GroupBy::Division => {
                            count_teams_in_division_column(
                                standings,
                                new_state.ui.standings.selected_column,
                                new_state.system.config.display_standings_western_first,
                            )
                        }
                        GroupBy::Wildcard => {
                            count_teams_in_wildcard_column(
                                standings,
                                new_state.ui.standings.selected_column,
                                new_state.system.config.display_standings_western_first,
                            )
                        }
                        _ => 0,
                    };
                    if new_state.ui.standings.selected_row >= team_count && team_count > 0 {
                        new_state.ui.standings.selected_row = team_count - 1;
                    }
                }
            }

            (new_state, Effect::None)
        }

        StandingsAction::MoveSelectionRight => {
            let mut new_state = state.clone();

            // Conference, Division, and Wildcard views have 2 columns for navigation
            if matches!(new_state.ui.standings.view, GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard) {
                // Wrap around: 1 -> 0
                new_state.ui.standings.selected_column = if new_state.ui.standings.selected_column == 1 {
                    0
                } else {
                    1
                };

                // Clamp row to max teams in new column if needed
                if let Some(ref standings) = new_state.data.standings {
                    let team_count = match new_state.ui.standings.view {
                        GroupBy::Conference => {
                            count_teams_in_conference_column(standings, new_state.ui.standings.selected_column)
                        }
                        GroupBy::Division => {
                            count_teams_in_division_column(
                                standings,
                                new_state.ui.standings.selected_column,
                                new_state.system.config.display_standings_western_first,
                            )
                        }
                        GroupBy::Wildcard => {
                            count_teams_in_wildcard_column(
                                standings,
                                new_state.ui.standings.selected_column,
                                new_state.system.config.display_standings_western_first,
                            )
                        }
                        _ => 0,
                    };
                    if new_state.ui.standings.selected_row >= team_count && team_count > 0 {
                        new_state.ui.standings.selected_row = team_count - 1;
                    }
                }
            }

            (new_state, Effect::None)
        }
    }
}

/// Sub-reducer for settings tab
fn reduce_settings(state: AppState, action: SettingsAction) -> (AppState, Effect) {
    match action {
        SettingsAction::NavigateCategoryLeft => {
            let mut new_state = state.clone();
            new_state.ui.settings.selected_category = match new_state.ui.settings.selected_category {
                SettingsCategory::Logging => SettingsCategory::Data,
                SettingsCategory::Display => SettingsCategory::Logging,
                SettingsCategory::Data => SettingsCategory::Display,
            };
            new_state.ui.settings.selected_setting_index = 0; // Reset selection
            (new_state, Effect::None)
        }

        SettingsAction::NavigateCategoryRight => {
            let mut new_state = state.clone();
            new_state.ui.settings.selected_category = match new_state.ui.settings.selected_category {
                SettingsCategory::Logging => SettingsCategory::Display,
                SettingsCategory::Display => SettingsCategory::Data,
                SettingsCategory::Data => SettingsCategory::Logging,
            };
            new_state.ui.settings.selected_setting_index = 0; // Reset selection
            (new_state, Effect::None)
        }

        SettingsAction::EnterSettingsMode => {
            debug!("SETTINGS: Entering settings mode");
            let mut new_state = state.clone();
            new_state.ui.settings.settings_mode = true;
            (new_state, Effect::None)
        }

        SettingsAction::ExitSettingsMode => {
            debug!("SETTINGS: Exiting settings mode");
            let mut new_state = state.clone();
            new_state.ui.settings.settings_mode = false;
            (new_state, Effect::None)
        }

        SettingsAction::MoveSelectionUp => {
            let mut new_state = state.clone();
            if new_state.ui.settings.selected_setting_index > 0 {
                new_state.ui.settings.selected_setting_index -= 1;
            }
            (new_state, Effect::None)
        }

        SettingsAction::MoveSelectionDown => {
            let mut new_state = state.clone();
            // We'll validate max in the UI layer
            new_state.ui.settings.selected_setting_index += 1;
            (new_state, Effect::None)
        }

        SettingsAction::ToggleBoolean(key) => {
            let mut new_state = state.clone();
            let mut config = new_state.system.config.clone();

            // Toggle the boolean setting
            match key.as_str() {
                "use_unicode" => {
                    config.display.use_unicode = !config.display.use_unicode;
                    config.display.box_chars = crate::formatting::BoxChars::from_use_unicode(config.display.use_unicode);
                }
                "show_action_bar" => {
                    config.display.show_action_bar = !config.display.show_action_bar;
                }
                "western_teams_first" => {
                    config.display_standings_western_first = !config.display_standings_western_first;
                }
                _ => {
                    debug!("SETTINGS: Unknown boolean setting: {}", key);
                    return (new_state, Effect::None);
                }
            }

            new_state.system.config = config.clone();

            // Return effect to persist config
            let effect = Effect::Async(Box::pin(async move {
                match crate::config::write(&config) {
                    Ok(_) => {
                        debug!("SETTINGS: Config saved successfully");
                        Action::SettingsAction(SettingsAction::UpdateConfig(Box::new(config)))
                    }
                    Err(e) => {
                        debug!("SETTINGS: Failed to save config: {}", e);
                        Action::Error(format!("Failed to save config: {}", e))
                    }
                }
            }));

            (new_state, effect)
        }

        SettingsAction::UpdateConfig(config) => {
            // Config update confirmation (after successful save)
            let mut new_state = state.clone();
            new_state.system.config = *config;
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
        // Default is Wildcard
        assert_eq!(state.ui.standings.view, GroupBy::Wildcard);

        let (new_state, _) = reduce_standings(state.clone(), StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::Division);

        let (new_state, _) = reduce_standings(new_state, StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::Conference);

        let (new_state, _) = reduce_standings(new_state, StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::League);

        let (new_state, _) = reduce_standings(new_state, StandingsAction::CycleView);
        assert_eq!(new_state.ui.standings.view, GroupBy::Wildcard);
    }

    // Helper function to create test standings
    fn create_test_standings(count: usize) -> Vec<nhl_api::Standing> {
        (0..count)
            .map(|i| nhl_api::Standing {
                conference_abbrev: Some("TEST".to_string()),
                conference_name: Some("Test Conference".to_string()),
                division_abbrev: "TST".to_string(),
                division_name: "Test Division".to_string(),
                team_name: nhl_api::LocalizedString {
                    default: format!("Team {} Full Name", i),
                },
                team_common_name: nhl_api::LocalizedString {
                    default: format!("Team {}", i),
                },
                team_abbrev: nhl_api::LocalizedString {
                    default: format!("T{}", i),
                },
                team_logo: "https://example.com/logo.png".to_string(),
                wins: i as i32,
                losses: 0,
                ot_losses: 0,
                points: i as i32 * 2,
            })
            .collect()
    }

    #[test]
    fn test_standings_move_up_wraps_from_first_to_last() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League; // Use League view for simple test data
        state.data.standings = Some(create_test_standings(5)); // 5 teams: rows 0-4
        state.ui.standings.selected_row = 0; // At first team

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should wrap to last team (row 4)
        assert_eq!(new_state.ui.standings.selected_row, 4);
    }

    #[test]
    fn test_standings_move_down_wraps_from_last_to_first() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League; // Use League view for simple test data
        state.data.standings = Some(create_test_standings(5)); // 5 teams: rows 0-4
        state.ui.standings.selected_row = 4; // At last team

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should wrap to first team (row 0)
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_standings_move_up_normal_navigation() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League; // Use League view for simple test data
        state.data.standings = Some(create_test_standings(5));
        state.ui.standings.selected_row = 2; // Middle team

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should move to previous team
        assert_eq!(new_state.ui.standings.selected_row, 1);
    }

    #[test]
    fn test_standings_move_down_normal_navigation() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League; // Use League view for simple test data
        state.data.standings = Some(create_test_standings(5));
        state.ui.standings.selected_row = 2; // Middle team

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should move to next team
        assert_eq!(new_state.ui.standings.selected_row, 3);
    }

    #[test]
    fn test_standings_move_up_with_no_data() {
        let mut state = AppState::default();
        state.data.standings = None; // No standings data
        state.ui.standings.selected_row = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should not change
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_standings_move_down_with_no_data() {
        let mut state = AppState::default();
        state.data.standings = None; // No standings data
        state.ui.standings.selected_row = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should not change
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_standings_move_up_with_empty_data() {
        let mut state = AppState::default();
        state.data.standings = Some(vec![]); // Empty standings
        state.ui.standings.selected_row = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);

        // Should not change
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_standings_move_down_with_empty_data() {
        let mut state = AppState::default();
        state.data.standings = Some(vec![]); // Empty standings
        state.ui.standings.selected_row = 0;

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionDown);

        // Should not change
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_conference_view_left_wraps_from_0_to_1() {
        use nhl_api::LocalizedString;

        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 0;
        state.ui.standings.selected_row = 5;

        // Create test standings with 2 conferences (16 teams each)
        let mut standings = vec![];
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("E".to_string()),
                conference_name: Some("Eastern".to_string()),
                division_abbrev: "A".to_string(),
                division_name: "Atlantic".to_string(),
                team_name: LocalizedString { default: format!("Eastern Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team E{}", i) },
                team_abbrev: LocalizedString { default: format!("E{}", i) },
                team_logo: "".to_string(),
                wins: 10 - i as i32,
                losses: i as i32,
                ot_losses: 0,
                points: 20 - i as i32,
            });
        }
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("W".to_string()),
                conference_name: Some("Western".to_string()),
                division_abbrev: "C".to_string(),
                division_name: "Central".to_string(),
                team_name: LocalizedString { default: format!("Western Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team W{}", i) },
                team_abbrev: LocalizedString { default: format!("W{}", i) },
                team_logo: "".to_string(),
                wins: 10 - i as i32,
                losses: i as i32,
                ot_losses: 0,
                points: 20 - i as i32,
            });
        }
        state.data.standings = Some(standings);

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionLeft);

        // Should wrap to column 1
        assert_eq!(new_state.ui.standings.selected_column, 1);
        // Row should be preserved
        assert_eq!(new_state.ui.standings.selected_row, 5);
    }

    #[test]
    fn test_conference_view_right_wraps_from_1_to_0() {
        use nhl_api::LocalizedString;

        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 1;
        state.ui.standings.selected_row = 3;

        // Create test standings with 2 conferences
        let mut standings = vec![];
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("E".to_string()),
                conference_name: Some("Eastern".to_string()),
                division_abbrev: "A".to_string(),
                division_name: "Atlantic".to_string(),
                team_name: LocalizedString { default: format!("Eastern Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team E{}", i) },
                team_abbrev: LocalizedString { default: format!("E{}", i) },
                team_logo: "".to_string(),
                wins: 10,
                losses: 0,
                ot_losses: 0,
                points: 20,
            });
        }
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("W".to_string()),
                conference_name: Some("Western".to_string()),
                division_abbrev: "C".to_string(),
                division_name: "Central".to_string(),
                team_name: LocalizedString { default: format!("Western Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team W{}", i) },
                team_abbrev: LocalizedString { default: format!("W{}", i) },
                team_logo: "".to_string(),
                wins: 10,
                losses: 0,
                ot_losses: 0,
                points: 20,
            });
        }
        state.data.standings = Some(standings);

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionRight);

        // Should wrap to column 0
        assert_eq!(new_state.ui.standings.selected_column, 0);
        // Row should be preserved
        assert_eq!(new_state.ui.standings.selected_row, 3);
    }

    #[test]
    fn test_conference_view_up_down_respects_column() {
        use nhl_api::LocalizedString;

        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::Conference;
        state.ui.standings.selected_column = 0; // Eastern
        state.ui.standings.selected_row = 0;

        // Create test standings with 2 conferences (16 teams each)
        let mut standings = vec![];
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("E".to_string()),
                conference_name: Some("Eastern".to_string()),
                division_abbrev: "A".to_string(),
                division_name: "Atlantic".to_string(),
                team_name: LocalizedString { default: format!("Eastern Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team E{}", i) },
                team_abbrev: LocalizedString { default: format!("E{}", i) },
                team_logo: "".to_string(),
                wins: 10,
                losses: 0,
                ot_losses: 0,
                points: 20,
            });
        }
        for i in 0..16 {
            standings.push(nhl_api::Standing {
                conference_abbrev: Some("W".to_string()),
                conference_name: Some("Western".to_string()),
                division_abbrev: "C".to_string(),
                division_name: "Central".to_string(),
                team_name: LocalizedString { default: format!("Western Team {}", i) },
                team_common_name: LocalizedString { default: format!("Team W{}", i) },
                team_abbrev: LocalizedString { default: format!("W{}", i) },
                team_logo: "".to_string(),
                wins: 10,
                losses: 0,
                ot_losses: 0,
                points: 20,
            });
        }
        state.data.standings = Some(standings);

        // Move up from first team - should wrap to last team in Eastern conference (15)
        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionUp);
        assert_eq!(new_state.ui.standings.selected_row, 15);

        // Move down from last team - should wrap to first team in Eastern conference (0)
        let (new_state, _) = reduce_standings(new_state, StandingsAction::MoveSelectionDown);
        assert_eq!(new_state.ui.standings.selected_row, 0);
    }

    #[test]
    fn test_conference_view_left_right_does_nothing_in_league_view() {
        let mut state = AppState::default();
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_column = 0;

        let (new_state, _) = reduce_standings(state.clone(), StandingsAction::MoveSelectionLeft);
        assert_eq!(new_state.ui.standings.selected_column, 0); // Should not change

        let (new_state, _) = reduce_standings(state, StandingsAction::MoveSelectionRight);
        assert_eq!(new_state.ui.standings.selected_column, 0); // Should not change
    }

    #[test]
    fn test_settings_navigate_category_right() {
        let state = AppState::default(); // Starts with Logging

        let (new_state, _) = reduce_settings(state.clone(), SettingsAction::NavigateCategoryRight);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Display);

        let (new_state, _) = reduce_settings(new_state, SettingsAction::NavigateCategoryRight);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Data);

        let (new_state, _) = reduce_settings(new_state, SettingsAction::NavigateCategoryRight);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Logging);
    }

    #[test]
    fn test_settings_navigate_category_left() {
        let state = AppState::default(); // Starts with Logging

        let (new_state, _) = reduce_settings(state.clone(), SettingsAction::NavigateCategoryLeft);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Data);

        let (new_state, _) = reduce_settings(new_state, SettingsAction::NavigateCategoryLeft);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Display);

        let (new_state, _) = reduce_settings(new_state, SettingsAction::NavigateCategoryLeft);
        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Logging);
    }

    #[test]
    fn test_settings_action_dispatches_to_reducer() {
        let state = AppState::default();

        let (new_state, _) = reduce(
            state.clone(),
            Action::SettingsAction(SettingsAction::NavigateCategoryRight),
        );

        assert_eq!(new_state.ui.settings.selected_category, SettingsCategory::Display);
    }

    #[test]
    fn test_refresh_data_updates_last_refresh_timestamp() {
        let state = AppState::default();
        assert!(state.system.last_refresh.is_none());

        let (new_state, _) = reduce(state, Action::RefreshData);

        // Verify last_refresh was set
        assert!(new_state.system.last_refresh.is_some());
    }

    #[test]
    fn test_data_loaded_does_not_update_last_refresh() {
        use std::time::Duration;

        let mut state = AppState::default();
        let initial_refresh_time = SystemTime::now() - Duration::from_secs(30);
        state.system.last_refresh = Some(initial_refresh_time);

        // Load standings
        let standings = vec![];
        let (new_state, _) = reduce(state.clone(), Action::StandingsLoaded(Ok(standings)));

        // last_refresh should NOT change when data loads
        assert_eq!(new_state.system.last_refresh, Some(initial_refresh_time));

        // Load schedule
        let schedule = nhl_api::DailySchedule {
            next_start_date: None,
            previous_start_date: None,
            date: "2024-01-15".to_string(),
            games: vec![],
            number_of_games: 0,
        };
        let (new_state, _) = reduce(new_state, Action::ScheduleLoaded(Ok(schedule)));

        // last_refresh should still NOT change
        assert_eq!(new_state.system.last_refresh, Some(initial_refresh_time));
    }

    #[test]
    fn test_multiple_data_loads_preserve_single_refresh_timestamp() {
        use std::time::Duration;

        let state = AppState::default();

        // Trigger refresh - this sets last_refresh
        let (state_after_refresh, _) = reduce(state, Action::RefreshData);
        let refresh_time = state_after_refresh.system.last_refresh;
        assert!(refresh_time.is_some());

        // Simulate some time passing
        std::thread::sleep(Duration::from_millis(10));

        // Load standings
        let standings = vec![];
        let (state_after_standings, _) = reduce(
            state_after_refresh.clone(),
            Action::StandingsLoaded(Ok(standings)),
        );

        // last_refresh should be unchanged
        assert_eq!(state_after_standings.system.last_refresh, refresh_time);

        // Simulate more time passing
        std::thread::sleep(Duration::from_millis(10));

        // Load schedule
        let schedule = nhl_api::DailySchedule {
            next_start_date: None,
            previous_start_date: None,
            date: "2024-01-15".to_string(),
            games: vec![],
            number_of_games: 0,
        };
        let (state_after_schedule, _) = reduce(
            state_after_standings,
            Action::ScheduleLoaded(Ok(schedule)),
        );

        // last_refresh should STILL be the original refresh time
        assert_eq!(state_after_schedule.system.last_refresh, refresh_time);
    }
}

/// Helper function to count teams in a conference column
/// Column 0 = Eastern Conference (or Western if western_first config is true)
/// Column 1 = Western Conference (or Eastern if western_first config is true)
fn count_teams_in_conference_column(standings: &[nhl_api::Standing], column: usize) -> usize {
    use std::collections::BTreeMap;

    // Group standings by conference
    let mut grouped: BTreeMap<String, Vec<&nhl_api::Standing>> = BTreeMap::new();
    for standing in standings {
        let conference = standing.conference_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        grouped
            .entry(conference)
            .or_default()
            .push(standing);
    }

    // Convert to vec - BTreeMap gives us Eastern, Western alphabetically
    let groups: Vec<_> = grouped.into_iter().collect();

    if groups.len() != 2 {
        return 0;
    }

    // Column 0 = first conference (Eastern), Column 1 = second conference (Western)
    // Note: We're ignoring western_first config for now since we don't have access to it here
    // The proper fix would be to pass the config through, but for now this matches the rendering
    if column < groups.len() {
        groups[column].1.len()
    } else {
        0
    }
}

/// Count teams in a division column (0 = Eastern divisions, 1 = Western divisions)
/// Respects display_standings_western_first config
fn count_teams_in_division_column(
    standings: &[nhl_api::Standing],
    column: usize,
    western_first: bool,
) -> usize {
    use std::collections::BTreeMap;

    // Group standings by division
    let mut grouped: BTreeMap<String, Vec<&nhl_api::Standing>> = BTreeMap::new();
    for standing in standings {
        grouped
            .entry(standing.division_name.clone())
            .or_default()
            .push(standing);
    }

    // Separate Eastern and Western divisions
    let mut eastern_divs = Vec::new();
    let mut western_divs = Vec::new();

    for (div_name, teams) in grouped {
        if div_name == "Atlantic" || div_name == "Metropolitan" {
            eastern_divs.push((div_name, teams));
        } else if div_name == "Central" || div_name == "Pacific" {
            western_divs.push((div_name, teams));
        }
    }

    // Determine which divisions go in which column based on western_first
    let (col0_divs, col1_divs) = if western_first {
        (western_divs, eastern_divs)
    } else {
        (eastern_divs, western_divs)
    };

    // Count total teams in the requested column
    let divs = if column == 0 { col0_divs } else { col1_divs };
    divs.iter().map(|(_, teams)| teams.len()).sum()
}

/// Count teams in a wildcard column (same structure as division view)
/// Each column has: Division1 top-3 + Division2 top-3 + Wildcards (remaining teams sorted by points)
fn count_teams_in_wildcard_column(
    standings: &[nhl_api::Standing],
    column: usize,
    western_first: bool,
) -> usize {
    use std::collections::BTreeMap;

    // Group teams by division and sort by points
    let mut grouped: BTreeMap<String, Vec<&nhl_api::Standing>> = BTreeMap::new();
    for standing in standings {
        grouped
            .entry(standing.division_name.clone())
            .or_default()
            .push(standing);
    }

    // Sort teams within each division by points
    for teams in grouped.values_mut() {
        teams.sort_by(|a, b| b.points.cmp(&a.points));
    }

    // Extract divisions
    let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
    let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
    let central = grouped.get("Central").cloned().unwrap_or_default();
    let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

    // Count teams per conference
    // Eastern: Atlantic top 3 + Metropolitan top 3 + (remaining teams from both)
    let eastern_count = {
        let top_count = atlantic.len().min(3) + metropolitan.len().min(3);
        let wildcard_count = atlantic.len().saturating_sub(3) + metropolitan.len().saturating_sub(3);
        top_count + wildcard_count
    };

    // Western: Central top 3 + Pacific top 3 + (remaining teams from both)
    let western_count = {
        let top_count = central.len().min(3) + pacific.len().min(3);
        let wildcard_count = central.len().saturating_sub(3) + pacific.len().saturating_sub(3);
        top_count + wildcard_count
    };

    // Determine which conference is in which column based on western_first
    if western_first {
        if column == 0 { western_count } else { eastern_count }
    } else {
        if column == 0 { eastern_count } else { western_count }
    }
}
