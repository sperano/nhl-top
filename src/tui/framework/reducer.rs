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
                selected_index: Some(0),
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

        Action::SelectPlayer(player_id) => {
            debug!("PLAYER: Opening player detail panel for player_id={}", player_id);
            let mut new_state = state.clone();

            // Push PlayerDetail panel onto stack
            new_state.navigation.panel_stack.push(PanelState {
                panel: super::action::Panel::PlayerDetail { player_id },
                scroll_offset: 0,
                selected_index: Some(0), // Start with first season selected
            });

            // Note: data fetch is triggered by Runtime::check_for_player_detail_fetch()
            // which detects when a PlayerDetail panel is pushed and triggers the fetch effect

            (new_state, Effect::None)
        }

        Action::SelectTeam(team_abbrev) => {
            debug!("TEAM: Opening team detail panel for team={}", team_abbrev);
            let mut new_state = state.clone();

            // Push TeamDetail panel onto stack
            new_state.navigation.panel_stack.push(PanelState {
                panel: super::action::Panel::TeamDetail { abbrev: team_abbrev.clone() },
                scroll_offset: 0,
                selected_index: Some(0),
            });

            // Mark as loading if we don't have data yet
            if !new_state.data.team_roster_stats.contains_key(&team_abbrev) {
                new_state
                    .data
                    .loading
                    .insert(LoadingKey::TeamRosterStats(team_abbrev));
                // TODO: Trigger Effect to fetch team stats
            }

            (new_state, Effect::None)
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

        Action::TeamRosterStatsLoaded(team_abbrev, Ok(stats)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .team_roster_stats
                .insert(team_abbrev.clone(), stats);
            new_state
                .data
                .loading
                .remove(&LoadingKey::TeamRosterStats(team_abbrev));
            (new_state, Effect::None)
        }

        Action::TeamRosterStatsLoaded(team_abbrev, Err(e)) => {
            let mut new_state = state.clone();
            new_state
                .data
                .loading
                .remove(&LoadingKey::TeamRosterStats(team_abbrev.clone()));
            new_state
                .data
                .errors
                .insert(format!("team_roster_stats_{}", team_abbrev), e);
            (new_state, Effect::None)
        }

        Action::PlayerStatsLoaded(player_id, Ok(stats)) => {
            let mut new_state = state.clone();
            new_state.data.player_data.insert(player_id, stats);
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

        Action::PanelSelectNext => {
            let mut new_state = state.clone();

            // Clone panel info before getting mutable reference
            let panel_type = new_state.navigation.panel_stack.last().map(|p| p.panel.clone());

            if let Some(panel) = panel_type {
                // Get total item count based on panel type
                let total_items = match &panel {
                    super::action::Panel::TeamDetail { abbrev } => {
                        new_state.data.team_roster_stats.get(abbrev)
                            .map(|stats| stats.skaters.len() + stats.goalies.len())
                            .unwrap_or(0)
                    }
                    super::action::Panel::Boxscore { game_id } => {
                        // Count all players from both teams
                        new_state.data.boxscores.get(game_id)
                            .map(|boxscore| {
                                let away = &boxscore.player_by_game_stats.away_team;
                                let home = &boxscore.player_by_game_stats.home_team;
                                away.forwards.len() + away.defense.len() + away.goalies.len()
                                    + home.forwards.len() + home.defense.len() + home.goalies.len()
                            })
                            .unwrap_or(0)
                    }
                    super::action::Panel::PlayerDetail { player_id } => {
                        // Count NHL regular season games
                        new_state.data.player_data.get(player_id)
                            .and_then(|player| player.season_totals.as_ref())
                            .map(|seasons| {
                                seasons.iter()
                                    .filter(|s| s.game_type_id == 2 && s.league_abbrev == "NHL")
                                    .count()
                            })
                            .unwrap_or(0)
                    }
                };

                if total_items > 0 {
                    if let Some(panel_state) = new_state.navigation.panel_stack.last_mut() {
                        if let Some(current_index) = panel_state.selected_index {
                            // Wrap around to 0 if at the end
                            let next_index = if current_index + 1 >= total_items {
                                0
                            } else {
                                current_index + 1
                            };
                            panel_state.selected_index = Some(next_index);

                            // Log navigation for Boxscore panels
                            if matches!(&panel, super::action::Panel::Boxscore { .. }) {
                                tracing::debug!("BOXSCORE NAV: PanelSelectNext {} -> {}, scroll_offset={}",
                                    current_index, next_index, panel_state.scroll_offset);
                            }

                            // Auto-scroll to keep selection visible
                            // For Boxscore panels, account for section chrome when calculating Y position
                            // IMPORTANT: Use MAX counts (not actual counts) to match rendering logic
                            let estimated_y = match &panel {
                                super::action::Panel::Boxscore { game_id } => {
                                    new_state.data.boxscores.get(game_id)
                                        .map(|boxscore| {
                                            let away = &boxscore.player_by_game_stats.away_team;
                                            let home = &boxscore.player_by_game_stats.home_team;

                                            let away_forwards = away.forwards.len();
                                            let away_defense = away.defense.len();
                                            let away_goalies = away.goalies.len();
                                            let away_total = away_forwards + away_defense + away_goalies;

                                            let home_forwards = home.forwards.len();
                                            let home_defense = home.defense.len();
                                            let home_goalies = home.goalies.len();

                                            // Use MAX counts to match rendering (prevents scroll mismatch)
                                            let max_forwards = away_forwards.max(home_forwards);
                                            let max_defense = away_defense.max(home_defense);
                                            let max_goalies = away_goalies.max(home_goalies);

                                            const CHROME: usize = 4;

                                            // Calculate Y position using MAX counts (matching rendering)
                                            if next_index < away_forwards {
                                                // Away forwards: chrome + row
                                                CHROME + next_index
                                            } else if next_index < away_forwards + away_defense {
                                                // Away defense: max forwards section + defense chrome + row
                                                (max_forwards + CHROME) + CHROME + (next_index - away_forwards)
                                            } else if next_index < away_total {
                                                // Away goalies: max forwards + max defense sections + goalies chrome + row
                                                (max_forwards + CHROME) + (max_defense + CHROME) + CHROME + (next_index - away_forwards - away_defense)
                                            } else {
                                                let home_idx = next_index - away_total;

                                                // Base offset: all away sections using MAX counts
                                                let away_offset = (max_forwards + CHROME) + (max_defense + CHROME) + (max_goalies + CHROME);

                                                if home_idx < home_forwards {
                                                    // Home forwards
                                                    away_offset + CHROME + home_idx
                                                } else if home_idx < home_forwards + home_defense {
                                                    // Home defense
                                                    away_offset + (max_forwards + CHROME) + CHROME + (home_idx - home_forwards)
                                                } else {
                                                    // Home goalies
                                                    away_offset + (max_forwards + CHROME) + (max_defense + CHROME) + CHROME + (home_idx - home_forwards - home_defense)
                                                }
                                            }
                                        })
                                        .unwrap_or(next_index)
                                }
                                _ => next_index // For other panels, use index as Y
                            };

                            // Only auto-scroll for Boxscore if content exceeds reasonable terminal size
                            let old_scroll_offset = panel_state.scroll_offset;

                            if let super::action::Panel::Boxscore { game_id } = &panel {
                                // Calculate total content height to determine if scrolling is needed
                                if let Some(boxscore) = new_state.data.boxscores.get(game_id) {
                                    let away = &boxscore.player_by_game_stats.away_team;
                                    let home = &boxscore.player_by_game_stats.home_team;

                                    let max_forwards = away.forwards.len().max(home.forwards.len());
                                    let max_defense = away.defense.len().max(home.defense.len());
                                    let max_goalies = away.goalies.len().max(home.goalies.len());

                                    const CHROME: usize = 4;
                                    let total_content_height = (max_forwards + CHROME) + (max_defense + CHROME) + (max_goalies + CHROME);

                                    // Assume a reasonable terminal size of 40 lines
                                    // Only scroll if content doesn't fit
                                    const REASONABLE_TERMINAL_HEIGHT: usize = 40;

                                    if total_content_height > REASONABLE_TERMINAL_HEIGHT {
                                        // Content doesn't fit, use auto-scroll with 20-line buffer
                                        if estimated_y > panel_state.scroll_offset + 20 {
                                            panel_state.scroll_offset = estimated_y.saturating_sub(20);
                                        } else if estimated_y < panel_state.scroll_offset {
                                            panel_state.scroll_offset = estimated_y;
                                        }
                                    }
                                    // else: content fits entirely, don't scroll (keep scroll_offset at 0)
                                }
                            } else {
                                // For other panel types, use simple auto-scroll
                                if estimated_y > panel_state.scroll_offset + 20 {
                                    panel_state.scroll_offset = estimated_y.saturating_sub(20);
                                } else if estimated_y < panel_state.scroll_offset {
                                    panel_state.scroll_offset = estimated_y;
                                }
                            }

                            // Log scroll changes for Boxscore panels
                            if matches!(&panel, super::action::Panel::Boxscore { .. }) && old_scroll_offset != panel_state.scroll_offset {
                                tracing::debug!("BOXSCORE NAV: estimated_y={}, scroll_offset: {} -> {}",
                                    estimated_y, old_scroll_offset, panel_state.scroll_offset);
                            }
                        }
                    }
                }
            }
            (new_state, Effect::None)
        }

        Action::PanelSelectPrevious => {
            let mut new_state = state.clone();

            // Clone panel info before getting mutable reference
            let panel_type = new_state.navigation.panel_stack.last().map(|p| p.panel.clone());

            if let Some(panel) = panel_type {
                // Get total item count based on panel type
                let total_items = match &panel {
                    super::action::Panel::TeamDetail { abbrev } => {
                        new_state.data.team_roster_stats.get(abbrev)
                            .map(|stats| stats.skaters.len() + stats.goalies.len())
                            .unwrap_or(0)
                    }
                    super::action::Panel::Boxscore { game_id } => {
                        // Count all players from both teams
                        new_state.data.boxscores.get(game_id)
                            .map(|boxscore| {
                                let away = &boxscore.player_by_game_stats.away_team;
                                let home = &boxscore.player_by_game_stats.home_team;
                                away.forwards.len() + away.defense.len() + away.goalies.len()
                                    + home.forwards.len() + home.defense.len() + home.goalies.len()
                            })
                            .unwrap_or(0)
                    }
                    super::action::Panel::PlayerDetail { player_id } => {
                        // Count NHL regular season games
                        new_state.data.player_data.get(player_id)
                            .and_then(|player| player.season_totals.as_ref())
                            .map(|seasons| {
                                seasons.iter()
                                    .filter(|s| s.game_type_id == 2 && s.league_abbrev == "NHL")
                                    .count()
                            })
                            .unwrap_or(0)
                    }
                };

                if total_items > 0 {
                    if let Some(panel_state) = new_state.navigation.panel_stack.last_mut() {
                        if let Some(current_index) = panel_state.selected_index {
                            // Wrap around to end if at the beginning
                            let prev_index = if current_index == 0 {
                                total_items - 1
                            } else {
                                current_index - 1
                            };
                            panel_state.selected_index = Some(prev_index);

                            // Log navigation for Boxscore panels
                            if matches!(&panel, super::action::Panel::Boxscore { .. }) {
                                tracing::debug!("BOXSCORE NAV: PanelSelectPrevious {} -> {}, scroll_offset={}",
                                    current_index, prev_index, panel_state.scroll_offset);
                            }

                            // Auto-scroll to keep selection visible
                            // For Boxscore panels, account for section chrome when calculating Y position
                            // IMPORTANT: Use MAX counts (not actual counts) to match rendering logic
                            let estimated_y = match &panel {
                                super::action::Panel::Boxscore { game_id } => {
                                    new_state.data.boxscores.get(game_id)
                                        .map(|boxscore| {
                                            let away = &boxscore.player_by_game_stats.away_team;
                                            let home = &boxscore.player_by_game_stats.home_team;

                                            let away_forwards = away.forwards.len();
                                            let away_defense = away.defense.len();
                                            let away_goalies = away.goalies.len();
                                            let away_total = away_forwards + away_defense + away_goalies;

                                            let home_forwards = home.forwards.len();
                                            let home_defense = home.defense.len();
                                            let home_goalies = home.goalies.len();

                                            // Use MAX counts to match rendering (prevents scroll mismatch)
                                            let max_forwards = away_forwards.max(home_forwards);
                                            let max_defense = away_defense.max(home_defense);
                                            let max_goalies = away_goalies.max(home_goalies);

                                            const CHROME: usize = 4;

                                            // Calculate Y position using MAX counts (matching rendering)
                                            if prev_index < away_forwards {
                                                // Away forwards: chrome + row
                                                CHROME + prev_index
                                            } else if prev_index < away_forwards + away_defense {
                                                // Away defense: max forwards section + defense chrome + row
                                                (max_forwards + CHROME) + CHROME + (prev_index - away_forwards)
                                            } else if prev_index < away_total {
                                                // Away goalies: max forwards + max defense sections + goalies chrome + row
                                                (max_forwards + CHROME) + (max_defense + CHROME) + CHROME + (prev_index - away_forwards - away_defense)
                                            } else {
                                                let home_idx = prev_index - away_total;

                                                // Base offset: all away sections using MAX counts
                                                let away_offset = (max_forwards + CHROME) + (max_defense + CHROME) + (max_goalies + CHROME);

                                                if home_idx < home_forwards {
                                                    // Home forwards
                                                    away_offset + CHROME + home_idx
                                                } else if home_idx < home_forwards + home_defense {
                                                    // Home defense
                                                    away_offset + (max_forwards + CHROME) + CHROME + (home_idx - home_forwards)
                                                } else {
                                                    // Home goalies
                                                    away_offset + (max_forwards + CHROME) + (max_defense + CHROME) + CHROME + (home_idx - home_forwards - home_defense)
                                                }
                                            }
                                        })
                                        .unwrap_or(prev_index)
                                }
                                _ => prev_index // For other panels, use index as Y
                            };

                            // Only auto-scroll for Boxscore if content exceeds reasonable terminal size
                            let old_scroll_offset = panel_state.scroll_offset;

                            if let super::action::Panel::Boxscore { game_id } = &panel {
                                // Calculate total content height to determine if scrolling is needed
                                if let Some(boxscore) = new_state.data.boxscores.get(game_id) {
                                    let away = &boxscore.player_by_game_stats.away_team;
                                    let home = &boxscore.player_by_game_stats.home_team;

                                    let max_forwards = away.forwards.len().max(home.forwards.len());
                                    let max_defense = away.defense.len().max(home.defense.len());
                                    let max_goalies = away.goalies.len().max(home.goalies.len());

                                    const CHROME: usize = 4;
                                    let total_content_height = (max_forwards + CHROME) + (max_defense + CHROME) + (max_goalies + CHROME);

                                    // Assume a reasonable terminal size of 40 lines
                                    // Only scroll if content doesn't fit
                                    const REASONABLE_TERMINAL_HEIGHT: usize = 40;

                                    if total_content_height > REASONABLE_TERMINAL_HEIGHT {
                                        // Content doesn't fit, use auto-scroll with 20-line buffer
                                        if estimated_y < panel_state.scroll_offset {
                                            panel_state.scroll_offset = estimated_y;
                                        } else if estimated_y > panel_state.scroll_offset + 20 {
                                            panel_state.scroll_offset = estimated_y.saturating_sub(20);
                                        }
                                    }
                                    // else: content fits entirely, don't scroll (keep scroll_offset at 0)
                                }
                            } else {
                                // For other panel types, use simple auto-scroll
                                if estimated_y < panel_state.scroll_offset {
                                    panel_state.scroll_offset = estimated_y;
                                } else if estimated_y > panel_state.scroll_offset + 20 {
                                    panel_state.scroll_offset = estimated_y.saturating_sub(20);
                                }
                            }

                            // Log scroll changes for Boxscore panels
                            if matches!(&panel, super::action::Panel::Boxscore { .. }) && old_scroll_offset != panel_state.scroll_offset {
                                tracing::debug!("BOXSCORE NAV: estimated_y={}, scroll_offset: {} -> {}",
                                    estimated_y, old_scroll_offset, panel_state.scroll_offset);
                            }
                        }
                    }
                }
            }
            (new_state, Effect::None)
        }

        Action::PanelSelectItem => {
            // Handle selection based on panel type
            if let Some(panel_state) = state.navigation.panel_stack.last() {
                match &panel_state.panel {
                    super::action::Panel::TeamDetail { abbrev } => {
                        // Get the selected player and navigate to player detail
                        if let Some(selected_index) = panel_state.selected_index {
                            if let Some(stats) = state.data.team_roster_stats.get(abbrev) {
                                // TODO: This sorting should be done in nhl_api's club_stats() call
                                // to ensure consistent ordering across all consumers

                                // Sort skaters by points descending (same as display order)
                                let mut sorted_skaters = stats.skaters.clone();
                                sorted_skaters.sort_by(|a, b| b.points.cmp(&a.points));

                                // Sort goalies by games played descending (same as display order)
                                let mut sorted_goalies = stats.goalies.clone();
                                sorted_goalies.sort_by(|a, b| b.games_played.cmp(&a.games_played));

                                let total_skaters = sorted_skaters.len();

                                let player_id = if selected_index < total_skaters {
                                    // Selected a skater
                                    sorted_skaters.get(selected_index).map(|s| s.player_id)
                                } else {
                                    // Selected a goalie
                                    let goalie_index = selected_index - total_skaters;
                                    sorted_goalies.get(goalie_index).map(|g| g.player_id)
                                };

                                if let Some(player_id) = player_id {
                                    let mut new_state = state.clone();
                                    new_state.navigation.panel_stack.push(PanelState {
                                        panel: super::action::Panel::PlayerDetail { player_id },
                                        scroll_offset: 0,
                                        selected_index: Some(0), // Start with first season selected
                                    });
                                    return (new_state, Effect::None);
                                }
                            }
                        }
                    }
                    super::action::Panel::PlayerDetail { player_id } => {
                        // Get the selected season and navigate to team detail
                        if let Some(selected_index) = panel_state.selected_index {
                            if let Some(player) = state.data.player_data.get(player_id) {
                                if let Some(seasons) = &player.season_totals {
                                    // TODO: This sorting should be done in nhl_api's player_landing() call
                                    // to ensure consistent ordering across all consumers

                                    // Filter to NHL regular season only and sort by season descending (latest first)
                                    let mut nhl_seasons: Vec<_> = seasons.iter()
                                        .filter(|s| s.game_type_id == 2 && s.league_abbrev == "NHL")
                                        .collect();
                                    nhl_seasons.sort_by(|a, b| b.season.cmp(&a.season));

                                    if let Some(season) = nhl_seasons.get(selected_index) {
                                        // Extract team abbreviation from common name
                                        if let Some(ref common_name) = season.team_common_name {
                                            if let Some(abbrev) = crate::team_abbrev::common_name_to_abbrev(&common_name.default) {
                                                let mut new_state = state.clone();
                                                new_state.navigation.panel_stack.push(PanelState {
                                                    panel: super::action::Panel::TeamDetail {
                                                        abbrev: abbrev.to_string(),
                                                    },
                                                    scroll_offset: 0,
                                                    selected_index: Some(0),
                                                });
                                                return (new_state, Effect::None);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    super::action::Panel::Boxscore { game_id } => {
                        // Get the selected player and navigate to player detail
                        if let Some(selected_index) = panel_state.selected_index {
                            if let Some(boxscore) = state.data.boxscores.get(game_id) {
                                // Build player list in display order:
                                // Away: forwards, defense, goalies
                                // Home: forwards, defense, goalies
                                let away = &boxscore.player_by_game_stats.away_team;
                                let home = &boxscore.player_by_game_stats.home_team;

                                let mut all_players = Vec::new();

                                // Away team
                                all_players.extend(away.forwards.iter().map(|p| p.player_id));
                                all_players.extend(away.defense.iter().map(|p| p.player_id));
                                all_players.extend(away.goalies.iter().map(|p| p.player_id));

                                // Home team
                                all_players.extend(home.forwards.iter().map(|p| p.player_id));
                                all_players.extend(home.defense.iter().map(|p| p.player_id));
                                all_players.extend(home.goalies.iter().map(|p| p.player_id));

                                if let Some(&player_id) = all_players.get(selected_index) {
                                    debug!("BOXSCORE: Selected player_id={} at index={}", player_id, selected_index);
                                    let mut new_state = state.clone();
                                    new_state.navigation.panel_stack.push(PanelState {
                                        panel: super::action::Panel::PlayerDetail { player_id },
                                        scroll_offset: 0,
                                        selected_index: Some(0), // Start with first season selected
                                    });

                                    // Note: data fetch is triggered by Runtime::check_for_player_detail_fetch()
                                    // which detects when a PlayerDetail panel is pushed and triggers the fetch effect

                                    return (new_state, Effect::None);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            (state, Effect::None)
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
                            selected_index: Some(0), // Start with first player selected
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
                selected_index: Some(0), // Start with first player selected
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
            // Build layout map from standings data (same logic as renderer)
            if let Some(ref standings) = state.data.standings {
                let layout = build_standings_layout(
                    standings,
                    state.ui.standings.view,
                    state.system.config.display_standings_western_first,
                );

                // Lookup team at selected position
                if let Some(col) = layout.get(state.ui.standings.selected_column) {
                    if let Some(team_abbrev) = col.get(state.ui.standings.selected_row) {
                        debug!(
                            "STANDINGS: Selected team: {} (row={}, col={})",
                            team_abbrev,
                            state.ui.standings.selected_row,
                            state.ui.standings.selected_column
                        );

                        // Push TeamDetail panel onto navigation stack
                        let panel = super::action::Panel::TeamDetail {
                            abbrev: team_abbrev.clone(),
                        };

                        let mut new_state = state.clone();
                        new_state.navigation.panel_stack.push(PanelState {
                            panel,
                            scroll_offset: 0,
                            selected_index: Some(0), // Start with first player selected
                        });

                        debug!("STANDINGS: Pushed TeamDetail panel for {}", team_abbrev);

                        return (new_state, Effect::None);
                    } else {
                        debug!(
                            "STANDINGS: No team at position (row={}, col={})",
                            state.ui.standings.selected_row,
                            state.ui.standings.selected_column
                        );
                    }
                }
            }

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

/// Get the list of valid values for a setting that has a fixed set of options
fn get_setting_values(key: &str) -> Vec<&'static str> {
    match key {
        "log_level" => vec!["trace", "debug", "info", "warn", "error"],
        _ => vec![], // Empty for non-list settings
    }
}

/// Check if a setting is a list-type setting (has a fixed set of values)
fn is_list_setting_reducer(key: &str) -> bool {
    matches!(key, "log_level")
}

/// Get editable setting key for a given category and index (same as in keys.rs)
fn get_editable_setting_key_for_index(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => {
            match index {
                0 => Some("log_level".to_string()),
                1 => Some("log_file".to_string()),
                _ => None,
            }
        }
        SettingsCategory::Display => None,
        SettingsCategory::Data => {
            match index {
                0 => Some("refresh_interval".to_string()),
                2 => Some("time_format".to_string()),
                _ => None,
            }
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

        SettingsAction::StartEditing(key) => {
            debug!("SETTINGS: Starting edit for key: {}", key);
            let mut new_state = state.clone();

            // Check if this is a list setting - if so, open modal instead of text editing
            if is_list_setting_reducer(&key) {
                // Get the list of values and find current value's index
                let values = get_setting_values(&key);
                let current_value = match key.as_str() {
                    "log_level" => &new_state.system.config.log_level,
                    _ => "",
                };
                let selected_index = values.iter()
                    .position(|v| v == &current_value)
                    .unwrap_or(0);

                new_state.ui.settings.modal_open = true;
                new_state.ui.settings.modal_selected_index = selected_index;
                new_state.ui.settings.editing = true; // Set editing flag for modal
                return (new_state, Effect::None);
            }

            // For non-list settings, initialize edit buffer with current value
            let current_value = match key.as_str() {
                "log_file" => new_state.system.config.log_file.clone(),
                "time_format" => new_state.system.config.time_format.clone(),
                "refresh_interval" => new_state.system.config.refresh_interval.to_string(),
                _ => {
                    debug!("SETTINGS: Unknown editable setting: {}", key);
                    return (new_state, Effect::None);
                }
            };

            new_state.ui.settings.editing = true;
            new_state.ui.settings.edit_buffer = current_value;
            (new_state, Effect::None)
        }

        SettingsAction::CancelEditing => {
            debug!("SETTINGS: Cancelling edit");
            let mut new_state = state.clone();
            new_state.ui.settings.editing = false;
            new_state.ui.settings.edit_buffer.clear();
            new_state.ui.settings.modal_open = false;
            (new_state, Effect::None)
        }

        SettingsAction::AppendChar(ch) => {
            let mut new_state = state.clone();
            new_state.ui.settings.edit_buffer.push(ch);
            (new_state, Effect::None)
        }

        SettingsAction::DeleteChar => {
            let mut new_state = state.clone();
            new_state.ui.settings.edit_buffer.pop();
            (new_state, Effect::None)
        }

        SettingsAction::ModalMoveUp => {
            debug!("SETTINGS: Moving modal selection up");
            let mut new_state = state.clone();

            if new_state.ui.settings.modal_selected_index > 0 {
                new_state.ui.settings.modal_selected_index -= 1;
            }

            (new_state, Effect::None)
        }

        SettingsAction::ModalMoveDown => {
            debug!("SETTINGS: Moving modal selection down");
            let mut new_state = state.clone();

            // Get current setting key to determine max index
            let setting_key = get_editable_setting_key_for_index(
                new_state.ui.settings.selected_category,
                new_state.ui.settings.selected_setting_index,
            );

            if let Some(key) = setting_key {
                let values = get_setting_values(&key);
                let max_index = values.len().saturating_sub(1);

                if new_state.ui.settings.modal_selected_index < max_index {
                    new_state.ui.settings.modal_selected_index += 1;
                }
            }

            (new_state, Effect::None)
        }

        SettingsAction::ModalConfirm => {
            debug!("SETTINGS: Confirming modal selection");
            let mut new_state = state.clone();

            // Get the selected value from the modal
            let setting_key = get_editable_setting_key_for_index(
                new_state.ui.settings.selected_category,
                new_state.ui.settings.selected_setting_index,
            );

            if let Some(key) = setting_key {
                let values = get_setting_values(&key);
                if new_state.ui.settings.modal_selected_index < values.len() {
                    let selected_value = values[new_state.ui.settings.modal_selected_index];

                    // Update the config with the selected value
                    match key.as_str() {
                        "log_level" => {
                            new_state.system.config.log_level = selected_value.to_string();
                        }
                        _ => {}
                    }

                    // Close the modal and exit editing
                    new_state.ui.settings.modal_open = false;
                    new_state.ui.settings.editing = false;

                    // TODO: Trigger config save effect when available
                    return (new_state, Effect::None);
                }
            }

            new_state.ui.settings.modal_open = false;
            new_state.ui.settings.editing = false;
            (new_state, Effect::None)
        }

        SettingsAction::ModalCancel => {
            debug!("SETTINGS: Cancelling modal");
            let mut new_state = state.clone();
            new_state.ui.settings.modal_open = false;
            new_state.ui.settings.editing = false;
            (new_state, Effect::None)
        }

        SettingsAction::CommitEdit(key) => {
            debug!("SETTINGS: Committing edit for key: {}", key);
            let mut new_state = state.clone();
            let mut config = new_state.system.config.clone();
            let edit_value = new_state.ui.settings.edit_buffer.clone();

            // Update the config with the edited value
            let result = match key.as_str() {
                "log_file" => {
                    config.log_file = edit_value;
                    Ok(())
                }
                "log_level" => {
                    config.log_level = edit_value;
                    Ok(())
                }
                "time_format" => {
                    config.time_format = edit_value;
                    Ok(())
                }
                "refresh_interval" => {
                    match edit_value.parse::<u32>() {
                        Ok(value) => {
                            config.refresh_interval = value;
                            Ok(())
                        }
                        Err(_) => Err("Invalid number".to_string()),
                    }
                }
                _ => Err(format!("Unknown setting: {}", key)),
            };

            match result {
                Ok(()) => {
                    // Clear editing state
                    new_state.ui.settings.editing = false;
                    new_state.ui.settings.edit_buffer.clear();
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
                Err(e) => {
                    debug!("SETTINGS: Failed to commit edit: {}", e);
                    (new_state, Effect::None)
                }
            }
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
    fn test_settings_start_editing_log_file() {
        let state = AppState::default();

        let (new_state, _) = reduce_settings(state, SettingsAction::StartEditing("log_file".to_string()));

        assert_eq!(new_state.ui.settings.editing, true);
        assert_eq!(new_state.ui.settings.edit_buffer, "/dev/null"); // Default log_file
    }

    #[test]
    fn test_settings_start_editing_log_level() {
        let state = AppState::default();

        let (new_state, _) = reduce_settings(state, SettingsAction::StartEditing("log_level".to_string()));

        assert_eq!(new_state.ui.settings.editing, true);
        assert_eq!(new_state.ui.settings.edit_buffer, "info"); // Default log_level
    }

    #[test]
    fn test_settings_cancel_editing() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "some text".to_string();

        let (new_state, _) = reduce_settings(state, SettingsAction::CancelEditing);

        assert_eq!(new_state.ui.settings.editing, false);
        assert_eq!(new_state.ui.settings.edit_buffer, "");
    }

    #[test]
    fn test_settings_append_char() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "/tmp".to_string();

        let (new_state, _) = reduce_settings(state, SettingsAction::AppendChar('/'));

        assert_eq!(new_state.ui.settings.edit_buffer, "/tmp/");
    }

    #[test]
    fn test_settings_delete_char() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "/tmp/".to_string();

        let (new_state, _) = reduce_settings(state, SettingsAction::DeleteChar);

        assert_eq!(new_state.ui.settings.edit_buffer, "/tmp");
    }

    #[test]
    fn test_settings_delete_char_empty_buffer() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "".to_string();

        let (new_state, _) = reduce_settings(state, SettingsAction::DeleteChar);

        assert_eq!(new_state.ui.settings.edit_buffer, "");
    }

    #[test]
    fn test_settings_commit_edit_string() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "/tmp/test.log".to_string();

        let (new_state, effect) = reduce_settings(state, SettingsAction::CommitEdit("log_file".to_string()));

        assert_eq!(new_state.ui.settings.editing, false);
        assert_eq!(new_state.ui.settings.edit_buffer, "");
        assert_eq!(new_state.system.config.log_file, "/tmp/test.log");
        // Should return async effect to save config
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_settings_commit_edit_int_valid() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "120".to_string();

        let (new_state, effect) = reduce_settings(state, SettingsAction::CommitEdit("refresh_interval".to_string()));

        assert_eq!(new_state.ui.settings.editing, false);
        assert_eq!(new_state.ui.settings.edit_buffer, "");
        assert_eq!(new_state.system.config.refresh_interval, 120);
        // Should return async effect to save config
        assert!(matches!(effect, Effect::Async(_)));
    }

    #[test]
    fn test_settings_commit_edit_int_invalid() {
        let mut state = AppState::default();
        state.ui.settings.editing = true;
        state.ui.settings.edit_buffer = "not a number".to_string();

        let (new_state, effect) = reduce_settings(state, SettingsAction::CommitEdit("refresh_interval".to_string()));

        // Should remain in editing mode since parse failed
        assert_eq!(new_state.ui.settings.editing, true);
        assert_eq!(new_state.ui.settings.edit_buffer, "not a number");
        assert_eq!(new_state.system.config.refresh_interval, 60); // Unchanged
        assert!(matches!(effect, Effect::None));
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

/// Build standings layout: layout[column][row] = team_abbrev
///
/// This mirrors the rendering logic in StandingsTab component.
/// The layout represents what is actually displayed on screen,
/// making selection lookup a simple array access.
fn build_standings_layout(
    standings: &[nhl_api::Standing],
    view: GroupBy,
    western_first: bool,
) -> Vec<Vec<String>> {
    use std::collections::BTreeMap;

    match view {
        GroupBy::League => {
            // Single column, sorted by points
            let mut sorted = standings.to_vec();
            sorted.sort_by(|a, b| b.points.cmp(&a.points));
            vec![sorted.iter().map(|s| s.team_abbrev.default.clone()).collect()]
        }

        GroupBy::Conference => {
            // Two columns: Eastern, Western
            let mut grouped: BTreeMap<String, Vec<nhl_api::Standing>> = BTreeMap::new();
            for standing in standings {
                let conf = standing.conference_name.clone().unwrap_or_else(|| "Unknown".to_string());
                grouped.entry(conf).or_default().push(standing.clone());
            }

            for teams in grouped.values_mut() {
                teams.sort_by(|a, b| b.points.cmp(&a.points));
            }

            let groups: Vec<_> = grouped.into_iter().collect();
            if groups.len() != 2 {
                return Vec::new();
            }

            let eastern: Vec<String> = groups[0].1.iter().map(|s| s.team_abbrev.default.clone()).collect();
            let western: Vec<String> = groups[1].1.iter().map(|s| s.team_abbrev.default.clone()).collect();

            if western_first {
                vec![western, eastern]
            } else {
                vec![eastern, western]
            }
        }

        GroupBy::Division => {
            // Two columns: Eastern divisions, Western divisions
            let mut grouped: BTreeMap<String, Vec<nhl_api::Standing>> = BTreeMap::new();
            for standing in standings {
                grouped.entry(standing.division_name.clone()).or_default().push(standing.clone());
            }

            for teams in grouped.values_mut() {
                teams.sort_by(|a, b| b.points.cmp(&a.points));
            }

            let mut eastern_divs = Vec::new();
            let mut western_divs = Vec::new();

            for (div_name, teams) in grouped {
                if div_name == "Atlantic" || div_name == "Metropolitan" {
                    eastern_divs.push((div_name, teams));
                } else if div_name == "Central" || div_name == "Pacific" {
                    western_divs.push((div_name, teams));
                }
            }

            eastern_divs.sort_by(|a, b| a.0.cmp(&b.0));
            western_divs.sort_by(|a, b| a.0.cmp(&b.0));

            let eastern: Vec<String> = eastern_divs
                .into_iter()
                .flat_map(|(_, teams)| teams)
                .map(|s| s.team_abbrev.default.clone())
                .collect();

            let western: Vec<String> = western_divs
                .into_iter()
                .flat_map(|(_, teams)| teams)
                .map(|s| s.team_abbrev.default.clone())
                .collect();

            if western_first {
                vec![western, eastern]
            } else {
                vec![eastern, western]
            }
        }

        GroupBy::Wildcard => {
            // Two columns: Eastern (top 3 + wildcards), Western (top 3 + wildcards)
            let mut grouped: BTreeMap<String, Vec<nhl_api::Standing>> = BTreeMap::new();
            for standing in standings {
                grouped.entry(standing.division_name.clone()).or_default().push(standing.clone());
            }

            for teams in grouped.values_mut() {
                teams.sort_by(|a, b| b.points.cmp(&a.points));
            }

            let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
            let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
            let central = grouped.get("Central").cloned().unwrap_or_default();
            let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

            let eastern: Vec<String> = {
                let mut teams = Vec::new();
                teams.extend(atlantic.iter().take(3).cloned());
                teams.extend(metropolitan.iter().take(3).cloned());
                let mut wildcards: Vec<_> = atlantic.iter().skip(3).chain(metropolitan.iter().skip(3)).cloned().collect();
                wildcards.sort_by(|a, b| b.points.cmp(&a.points));
                teams.extend(wildcards);
                teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
            };

            let western: Vec<String> = {
                let mut teams = Vec::new();
                teams.extend(central.iter().take(3).cloned());
                teams.extend(pacific.iter().take(3).cloned());
                let mut wildcards: Vec<_> = central.iter().skip(3).chain(pacific.iter().skip(3)).cloned().collect();
                wildcards.sort_by(|a, b| b.points.cmp(&a.points));
                teams.extend(wildcards);
                teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
            };

            if western_first {
                vec![western, eastern]
            } else {
                vec![eastern, western]
            }
        }
    }
}

#[cfg(test)]
mod standings_layout_tests {
    use super::*;
    use nhl_api::LocalizedString;

    fn make_standing(abbrev: &str, division: &str, conference: &str, points: i32) -> nhl_api::Standing {
        nhl_api::Standing {
            conference_abbrev: Some(conference.to_string()),
            conference_name: Some(conference.to_string()),
            division_abbrev: division.to_string(),
            division_name: division.to_string(),
            team_name: LocalizedString { default: format!("Team {}", abbrev) },
            team_common_name: LocalizedString { default: abbrev.to_string() },
            team_abbrev: LocalizedString { default: abbrev.to_string() },
            team_logo: String::new(),
            wins: 0,
            losses: 0,
            ot_losses: 0,
            points,
        }
    }

    #[test]
    fn test_league_layout_single_column_sorted_by_points() {
        let standings = vec![
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("COL", "Central", "Western", 98),
        ];

        let layout = build_standings_layout(&standings, GroupBy::League, false);

        assert_eq!(layout.len(), 1, "League view should have 1 column");
        assert_eq!(layout[0], vec!["BOS", "COL", "TOR"], "Teams should be sorted by points descending");
    }

    #[test]
    fn test_conference_layout_eastern_first() {
        let standings = vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("COL", "Central", "Western", 98),
            make_standing("DAL", "Central", "Western", 97),
        ];

        let layout = build_standings_layout(&standings, GroupBy::Conference, false);

        assert_eq!(layout.len(), 2, "Conference view should have 2 columns");
        assert_eq!(layout[0], vec!["BOS", "TOR"], "Column 0 should be Eastern (sorted by points)");
        assert_eq!(layout[1], vec!["COL", "DAL"], "Column 1 should be Western (sorted by points)");
    }

    #[test]
    fn test_conference_layout_western_first() {
        let standings = vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("COL", "Central", "Western", 98),
            make_standing("DAL", "Central", "Western", 97),
        ];

        let layout = build_standings_layout(&standings, GroupBy::Conference, true);

        assert_eq!(layout.len(), 2, "Conference view should have 2 columns");
        assert_eq!(layout[0], vec!["COL", "DAL"], "Column 0 should be Western when western_first=true");
        assert_eq!(layout[1], vec!["BOS", "TOR"], "Column 1 should be Eastern when western_first=true");
    }

    #[test]
    fn test_division_layout_eastern_first() {
        let standings = vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("NYR", "Metropolitan", "Eastern", 93),
            make_standing("COL", "Central", "Western", 98),
            make_standing("DAL", "Central", "Western", 97),
            make_standing("VGK", "Pacific", "Western", 96),
        ];

        let layout = build_standings_layout(&standings, GroupBy::Division, false);

        assert_eq!(layout.len(), 2, "Division view should have 2 columns");
        // Eastern divisions: Atlantic, then Metropolitan (alphabetical)
        assert_eq!(layout[0], vec!["BOS", "TOR", "NYR"], "Column 0 should have Eastern divisions");
        // Western divisions: Central, then Pacific (alphabetical)
        assert_eq!(layout[1], vec!["COL", "DAL", "VGK"], "Column 1 should have Western divisions");
    }

    #[test]
    fn test_wildcard_layout_structure() {
        // Create a realistic wildcard scenario
        let standings = vec![
            // Atlantic division
            make_standing("BOS", "Atlantic", "Eastern", 110),
            make_standing("TOR", "Atlantic", "Eastern", 105),
            make_standing("TBL", "Atlantic", "Eastern", 100),
            make_standing("BUF", "Atlantic", "Eastern", 85), // Wildcard
            make_standing("OTT", "Atlantic", "Eastern", 80), // Wildcard
            // Metropolitan division
            make_standing("NYR", "Metropolitan", "Eastern", 108),
            make_standing("CAR", "Metropolitan", "Eastern", 103),
            make_standing("NJD", "Metropolitan", "Eastern", 98),
            make_standing("NYI", "Metropolitan", "Eastern", 90), // Wildcard
            make_standing("PHI", "Metropolitan", "Eastern", 82), // Wildcard
            // Central division
            make_standing("COL", "Central", "Western", 112),
            make_standing("DAL", "Central", "Western", 107),
            make_standing("WPG", "Central", "Western", 102),
            make_standing("NSH", "Central", "Western", 88), // Wildcard
            make_standing("MIN", "Central", "Western", 84), // Wildcard
            // Pacific division
            make_standing("VGK", "Pacific", "Western", 109),
            make_standing("EDM", "Pacific", "Western", 104),
            make_standing("LAK", "Pacific", "Western", 99),
            make_standing("SEA", "Pacific", "Western", 87), // Wildcard
            make_standing("CGY", "Pacific", "Western", 81), // Wildcard
        ];

        let layout = build_standings_layout(&standings, GroupBy::Wildcard, false);

        assert_eq!(layout.len(), 2, "Wildcard view should have 2 columns");

        // Eastern column: Atlantic top 3, Metro top 3, then wildcards sorted by points
        assert_eq!(layout[0][0], "BOS", "First should be Atlantic #1");
        assert_eq!(layout[0][1], "TOR", "Second should be Atlantic #2");
        assert_eq!(layout[0][2], "TBL", "Third should be Atlantic #3");
        assert_eq!(layout[0][3], "NYR", "Fourth should be Metro #1");
        assert_eq!(layout[0][4], "CAR", "Fifth should be Metro #2");
        assert_eq!(layout[0][5], "NJD", "Sixth should be Metro #3");
        // Wildcards: NYI(90), BUF(85), PHI(82), OTT(80)
        assert_eq!(layout[0][6], "NYI", "First wildcard should be NYI (90 pts)");
        assert_eq!(layout[0][7], "BUF", "Second wildcard should be BUF (85 pts)");

        // Western column: Central top 3, Pacific top 3, then wildcards sorted by points
        assert_eq!(layout[1][0], "COL", "First should be Central #1");
        assert_eq!(layout[1][3], "VGK", "Fourth should be Pacific #1");
        // Wildcards: NSH(88), SEA(87), MIN(84), CGY(81)
        assert_eq!(layout[1][6], "NSH", "First wildcard should be NSH (88 pts)");
        assert_eq!(layout[1][7], "SEA", "Second wildcard should be SEA (87 pts)");
    }

    #[test]
    fn test_lookup_team_at_position() {
        let standings = vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("COL", "Central", "Western", 98),
        ];

        let layout = build_standings_layout(&standings, GroupBy::League, false);

        // Test successful lookup
        assert_eq!(layout.get(0).and_then(|col| col.get(0)), Some(&"BOS".to_string()));
        assert_eq!(layout.get(0).and_then(|col| col.get(1)), Some(&"COL".to_string()));
        assert_eq!(layout.get(0).and_then(|col| col.get(2)), Some(&"TOR".to_string()));

        // Test out of bounds
        assert_eq!(layout.get(0).and_then(|col| col.get(3)), None, "Row 3 should be out of bounds");
        assert_eq!(layout.get(1).and_then(|col| col.get(0)), None, "Column 1 should be out of bounds");
    }

    #[test]
    fn test_select_team_pushes_panel() {
        use super::super::action::{Panel, StandingsAction};

        // Setup state with standings data
        let mut state = AppState::default();
        state.data.standings = Some(vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
            make_standing("TOR", "Atlantic", "Eastern", 95),
            make_standing("COL", "Central", "Western", 98),
        ]);
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 1; // COL (second in sorted order)
        state.ui.standings.selected_column = 0;

        // Initially no panels
        assert_eq!(state.navigation.panel_stack.len(), 0);

        // Dispatch SelectTeam action
        let (new_state, _effect) = reduce_standings(state, StandingsAction::SelectTeam);

        // Should have pushed a TeamDetail panel
        assert_eq!(new_state.navigation.panel_stack.len(), 1);

        let panel = &new_state.navigation.panel_stack[0].panel;
        match panel {
            Panel::TeamDetail { abbrev } => {
                assert_eq!(abbrev, "COL", "Should push panel for COL (row 1 in sorted standings)");
            }
            _ => panic!("Expected TeamDetail panel, got {:?}", panel),
        }
    }

    #[test]
    fn test_select_team_with_no_standings_does_nothing() {
        use super::super::action::StandingsAction;

        let mut state = AppState::default();
        state.data.standings = None; // No standings data
        state.ui.standings.selected_row = 0;
        state.ui.standings.selected_column = 0;

        let (new_state, _effect) = reduce_standings(state, StandingsAction::SelectTeam);

        // Should not push any panel
        assert_eq!(new_state.navigation.panel_stack.len(), 0);
    }

    #[test]
    fn test_select_team_out_of_bounds_does_nothing() {
        use super::super::action::StandingsAction;

        let mut state = AppState::default();
        state.data.standings = Some(vec![
            make_standing("BOS", "Atlantic", "Eastern", 100),
        ]);
        state.ui.standings.view = GroupBy::League;
        state.ui.standings.selected_row = 5; // Out of bounds
        state.ui.standings.selected_column = 0;

        let (new_state, _effect) = reduce_standings(state, StandingsAction::SelectTeam);

        // Should not push any panel
        assert_eq!(new_state.navigation.panel_stack.len(), 0);
    }
}
