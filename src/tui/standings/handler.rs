use crossterm::event::{KeyCode, KeyEvent};
use super::State;
use super::panel::StandingsPanel;
use crate::tui::navigation::NavigationContext;
use crate::types::SharedDataHandle;
use crate::types::NHL_LEAGUE_ABBREV;
use tokio::sync::mpsc;

fn navigate_to_column(state: &mut State, new_column: usize, new_column_team_count: usize) {
    state.selected_column = new_column;
    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
        state.selected_team_index = new_column_team_count - 1;
    }
}

async fn handle_panel_navigation(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle, refresh_tx: &mpsc::Sender<()>) -> bool {
    let nav_ctx = match &mut state.navigation {
        Some(ctx) => ctx,
        None => return false,
    };

    let panel = match nav_ctx.stack.current() {
        Some(p) => p.clone(),
        None => return false,
    };

    if state.panel_selection_active {
        match key.code {
            KeyCode::Up => {
                if state.panel_selected_index == 0 {
                    state.panel_selection_active = false;
                } else {
                    state.panel_selected_index = state.panel_selected_index.saturating_sub(1);
                }
                true
            }
            KeyCode::Down => {
                let max_items = get_panel_item_count(&panel, shared_data).await;
                if state.panel_selected_index + 1 < max_items {
                    state.panel_selected_index += 1;
                }
                true
            }
            KeyCode::Enter => {
                navigate_to_selected_item(state, &panel, shared_data, refresh_tx).await;
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.panel_scrollable.handle_key(key)
            }
            KeyCode::Esc | KeyCode::Left => {
                state.panel_selection_active = false;
                true
            }
            _ => false,
        }
    } else {
        match key.code {
            KeyCode::Esc | KeyCode::Left => {
                if let Some(nav_ctx) = &mut state.navigation {
                    if nav_ctx.stack.depth() <= 1 {
                        // Clear selected team and player when exiting team view
                        let mut shared = shared_data.write().await;
                        shared.selected_team_abbrev = None;
                        shared.selected_player_id = None;
                        state.navigation = None;
                    } else {
                        // Going back from player to team - clear selected player
                        let mut shared = shared_data.write().await;
                        shared.selected_player_id = None;
                        nav_ctx.go_back();
                    }
                    state.panel_scrollable.reset();
                    state.panel_selection_active = false;
                    state.panel_selected_index = 0;
                }
                true
            }
            KeyCode::Down => {
                state.panel_selection_active = true;
                state.panel_selected_index = 0;
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.panel_scrollable.handle_key(key)
            }
            _ => true,
        }
    }
}

async fn get_panel_item_count(panel: &StandingsPanel, shared_data: &SharedDataHandle) -> usize {
    match panel {
        StandingsPanel::TeamDetail { .. } => {
            // Get the real count from club stats in SharedData
            let data = shared_data.read().await;
            if let Some(team_abbrev) = &data.selected_team_abbrev {
                if let Some(stats) = data.club_stats.get(team_abbrev) {
                    return stats.skaters.len() + stats.goalies.len();
                }
            }
            // No data available yet
            0
        }
        StandingsPanel::PlayerDetail { player_id, .. } => {
            // Get real player data from SharedData
            let data = shared_data.read().await;
            if let Some(player) = data.player_info.get(player_id) {
                if let Some(season_totals) = &player.season_totals {
                    // Only count NHL seasons
                    return season_totals.iter()
                        .filter(|s| s.league_abbrev == NHL_LEAGUE_ABBREV)
                        .count();
                }
            }
            // Fallback
            0
        }
    }
}

async fn navigate_to_selected_item(state: &mut State, panel: &StandingsPanel, shared_data: &SharedDataHandle, refresh_tx: &mpsc::Sender<()>) {
    match panel {
        StandingsPanel::TeamDetail { team_name, .. } => {
            // Get real player data from SharedData
            let (player_id, player_name) = {
                let data = shared_data.read().await;
                if let Some(team_abbrev) = &data.selected_team_abbrev {
                    if let Some(stats) = data.club_stats.get(team_abbrev) {
                        let idx = state.panel_selected_index;

                        // Sort skaters by points (matching render order)
                        let mut sorted_skaters = stats.skaters.clone();
                        sorted_skaters.sort_by(|a, b| b.points.cmp(&a.points));

                        // Sort goalies by games played (matching render order)
                        let mut sorted_goalies = stats.goalies.clone();
                        sorted_goalies.sort_by(|a, b| b.games_played.cmp(&a.games_played));

                        // Check if it's a skater or goalie
                        if idx < sorted_skaters.len() {
                            let skater = &sorted_skaters[idx];
                            let name = format!("{} {}", skater.first_name.default, skater.last_name.default);
                            (skater.player_id, name)
                        } else {
                            let goalie_idx = idx - sorted_skaters.len();
                            if goalie_idx < sorted_goalies.len() {
                                let goalie = &sorted_goalies[goalie_idx];
                                let name = format!("{} {}", goalie.first_name.default, goalie.last_name.default);
                                (goalie.player_id, name)
                            } else {
                                return; // Invalid index
                            }
                        }
                    } else {
                        return; // No stats available
                    }
                } else {
                    return; // No team selected
                }
            };

            tracing::info!("Navigating to player: {} (ID: {})", player_name, player_id);

            // Set selected player ID in SharedData
            {
                let mut data = shared_data.write().await;
                data.selected_player_id = Some(player_id);
            }

            // Trigger refresh to fetch player data
            let _ = refresh_tx.send(()).await;

            let nav_ctx = state.navigation.as_mut().unwrap();
            nav_ctx.navigate_to(StandingsPanel::PlayerDetail {
                player_id,
                player_name,
                from_team_name: team_name.clone(),
            });
            state.panel_scrollable.reset();
            state.panel_selection_active = false;
            state.panel_selected_index = 0;
        }
        StandingsPanel::PlayerDetail { player_id, player_name, .. } => {
            // Get real player data and look up team abbreviation from standings
            let team_info = {
                let data = shared_data.read().await;
                if let Some(player) = data.player_info.get(player_id) {
                    if let Some(season_totals) = &player.season_totals {
                        // Filter to NHL-only seasons and reverse to match rendering order
                        let nhl_seasons: Vec<_> = season_totals.iter()
                            .filter(|s| s.league_abbrev == NHL_LEAGUE_ABBREV)
                            .collect();

                        // Reverse to match the display order (.rev() in render function)
                        let nhl_seasons_rev: Vec<_> = nhl_seasons.into_iter().rev().collect();

                        let idx = state.panel_selected_index;
                        if idx < nhl_seasons_rev.len() {
                            let season = nhl_seasons_rev[idx];
                            let team_name = season.team_name.default.clone();

                            // Look up team abbreviation from standings
                            let team_abbrev = data.standings.iter()
                                .find(|s| s.team_name.default == team_name)
                                .map(|s| s.team_abbrev.default.clone())
                                .unwrap_or_else(|| {
                                    tracing::warn!("Could not find team abbreviation for '{}', using team name", team_name);
                                    team_name.clone()
                                });

                            Some((team_name, team_abbrev, season.season))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some((team_name, team_abbrev, season_id)) = team_info {
                tracing::info!("Navigating to team: {} ({}) from season {}", team_name, team_abbrev, season_id);

                // Update selected team abbreviation and trigger refresh
                {
                    let mut data = shared_data.write().await;
                    data.selected_team_abbrev = Some(team_abbrev.clone());
                }
                let _ = refresh_tx.send(()).await;

                let nav_ctx = state.navigation.as_mut().unwrap();
                nav_ctx.navigate_to(StandingsPanel::TeamDetail {
                    team_id: 1, // TODO: We don't have team_id from season data, using placeholder
                    team_name,
                    team_abbrev,
                });
                state.panel_scrollable.reset();
                state.panel_selection_active = false;
                state.panel_selected_index = 0;
            }
        }
    }
}

pub async fn handle_key(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle, refresh_tx: &mpsc::Sender<()>) -> bool {
    if state.navigation.is_some() {
        return handle_panel_navigation(key, state, shared_data, refresh_tx).await;
    }

    let layout = match &state.layout_cache {
        Some(layout) => layout,
        None => return false,
    };

    let max_columns = layout.column_count();
    let team_count_in_current_column = layout.columns
        .get(state.selected_column)
        .map(|col| col.team_count)
        .unwrap_or(0);

    if state.team_selection_active {
        match key.code {
            KeyCode::Up => {
                if state.selected_team_index == 0 {
                    state.team_selection_active = false;
                    true
                } else {
                    state.selected_team_index = state.selected_team_index.saturating_sub(1);
                    true
                }
            }
            KeyCode::Down => {
                if state.selected_team_index + 1 < team_count_in_current_column {
                    state.selected_team_index += 1;
                }
                true
            }
            KeyCode::Left => {
                if state.selected_column > 0 {
                    let new_column = state.selected_column - 1;
                    let new_column_team_count = layout.columns.get(new_column).map(|col| col.team_count).unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::Right => {
                if state.selected_column + 1 < max_columns {
                    let new_column = state.selected_column + 1;
                    let new_column_team_count = layout.columns.get(new_column).map(|col| col.team_count).unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.scrollable.handle_key(key)
            }
            KeyCode::Enter => {
                if let Some(team) = layout.get_team(state.selected_column, state.selected_team_index) {
                    let team_abbrev = team.team_abbrev.default.clone();
                    let team_name = team.team_common_name.default.clone();

                    // Set the selected team abbreviation in shared data
                    {
                        let mut shared = shared_data.write().await;
                        shared.selected_team_abbrev = Some(team_abbrev.clone());
                    }

                    // Trigger a refresh to fetch club stats immediately
                    let _ = refresh_tx.send(()).await;

                    let mut nav_ctx: NavigationContext<StandingsPanel, String, _> = NavigationContext::new();
                    nav_ctx.navigate_to(StandingsPanel::TeamDetail {
                        team_id: 1,
                        team_name,
                        team_abbrev,
                    });
                    state.navigation = Some(nav_ctx);
                    state.panel_scrollable.reset();
                    state.panel_selection_active = false;
                    state.panel_selected_index = 0;
                }
                true
            }
            KeyCode::Esc => {
                state.team_selection_active = false;
                true
            }
            _ => false,
        }
    } else {
        match key.code {
            KeyCode::Left => {
                state.view = state.view.prev();
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                state.layout_cache = None;
                true
            }
            KeyCode::Right => {
                state.view = state.view.next();
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                state.layout_cache = None;
                true
            }
            KeyCode::Down => {
                if team_count_in_current_column > 0 {
                    state.team_selection_active = true;
                }
                true
            }
            KeyCode::Up => {
                if state.scrollable.scroll_offset == 0 {
                    false
                } else {
                    state.scrollable.handle_key(key)
                }
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.scrollable.handle_key(key)
            }
            _ => false,
        }
    }
}
