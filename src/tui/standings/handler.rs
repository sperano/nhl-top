use super::State;
use super::layout::StandingsLayout;
use super::panel::StandingsPanel;
use super::state::{PanelState, TeamDetailState, PlayerDetailState};
use crate::types::SharedDataHandle;
use crate::commands::standings::GroupBy;
use crate::tui::navigation::Panel;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

/// Navigate to a different column, adjusting selected_team_index if needed
fn navigate_to_column(state: &mut State, new_column: usize, new_column_team_count: usize) {
    state.selected_column = new_column;
    // If the new column has fewer teams than our current index, clamp to last team
    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
        state.selected_team_index = new_column_team_count - 1;
    }
}

/// Change view with save/restore of selection state
async fn change_view<F>(state: &mut State, shared_data: &SharedDataHandle, view_fn: F) -> bool
where
    F: FnOnce(GroupBy) -> GroupBy,
{
    // Save current selection before changing views
    state.save_current_selection();

    // Change to new view
    state.view = view_fn(state.view);

    // Restore selection for new view (or default to 0,0)
    state.restore_selection_for_view();

    // Build layout for new view to validate selection bounds
    let data = shared_data.read().await;
    let layout = StandingsLayout::build(&data.standings, state.view, data.config.display_standings_western_first);
    drop(data);

    // Get team counts per column for validation
    let team_counts: Vec<usize> = layout.columns.iter().map(|col| col.team_count).collect();
    let max_columns = layout.column_count();

    // Validate and clamp selection to ensure it's within bounds
    state.validate_selection(max_columns, &team_counts);

    // Reset scroll position for new view
    state.scrollable.reset();

    true
}

/// Handle navigation within panel views (TeamDetail, PlayerDetail)
async fn handle_panel_navigation(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    let current_panel = match state.navigation.stack.current() {
        Some(panel) => panel.clone(),
        None => return false,
    };

    match &current_panel {
        StandingsPanel::TeamDetail { team_abbrev, .. } => {
            // Get or create TeamDetailState from cache
            let cache_key = current_panel.cache_key();
            let mut panel_state = state
                .navigation
                .data
                .get(&cache_key)
                .and_then(|s| match s {
                    PanelState::TeamDetail(tds) => Some(tds.clone()),
                    _ => None,
                })
                .unwrap_or_else(TeamDetailState::new);

            // Get player list from club_stats
            let data = shared_data.read().await;
            let players_count = data
                .club_stats
                .get(team_abbrev.as_str())
                .map(|stats| stats.skaters.len())
                .unwrap_or(0);
            drop(data);

            let handled = match key.code {
                KeyCode::Down => {
                    if !panel_state.selection_active && players_count > 0 {
                        // Enter player selection mode
                        panel_state.selection_active = true;
                        panel_state.selected_player_index = 0;
                        true
                    } else if panel_state.selection_active {
                        // Navigate down in player list
                        if panel_state.selected_player_index + 1 < players_count {
                            panel_state.selected_player_index += 1;
                        }
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Up => {
                    if panel_state.selection_active {
                        if panel_state.selected_player_index == 0 {
                            // Exit player selection mode
                            panel_state.selection_active = false;
                        } else {
                            // Navigate up in player list
                            panel_state.selected_player_index -= 1;
                        }
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Enter => {
                    if panel_state.selection_active {
                        // Navigate to player detail
                        let data = shared_data.read().await;
                        if let Some(stats) = data.club_stats.get(team_abbrev.as_str()) {
                            if let Some(skater) = stats.skaters.get(panel_state.selected_player_index) {
                                let player_id = skater.player_id;
                                let player_panel = StandingsPanel::PlayerDetail {
                                    player_id,
                                    player_name: format!(
                                        "{} {}",
                                        skater.first_name.default, skater.last_name.default
                                    ),
                                    from_team_name: team_abbrev.clone(),
                                };
                                drop(data);

                                // Store current panel state before navigating
                                state
                                    .navigation
                                    .data
                                    .insert(cache_key.clone(), PanelState::TeamDetail(panel_state));

                                // Navigate to player detail with new state
                                let player_state = PlayerDetailState::new();
                                let player_key = player_panel.cache_key();
                                state
                                    .navigation
                                    .data
                                    .insert(player_key, PanelState::PlayerDetail(player_state));
                                state.navigation.navigate_to(player_panel);

                                // Set selected_player_id to trigger player info fetch
                                let mut data = shared_data.write().await;
                                data.selected_player_id = Some(player_id);
                                drop(data);

                                // Trigger refresh to fetch player info
                                let _ = refresh_tx.send(()).await;

                                return true;
                            }
                        }
                        drop(data);
                    }
                    false
                }
                KeyCode::Esc => {
                    if panel_state.selection_active {
                        // Exit player selection mode
                        panel_state.selection_active = false;
                        true
                    } else {
                        // Go back from panel
                        state.navigation.go_back();
                        true
                    }
                }
                KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                    panel_state.scrollable.handle_key(key)
                }
                _ => false,
            };

            // Update cache with modified state
            if handled {
                state
                    .navigation
                    .data
                    .insert(cache_key, PanelState::TeamDetail(panel_state));
            }

            handled
        }
        StandingsPanel::PlayerDetail { player_id, .. } => {
            // Get or create PlayerDetailState from cache
            let cache_key = current_panel.cache_key();
            let mut player_state = state
                .navigation
                .data
                .get(&cache_key)
                .and_then(|s| match s {
                    PanelState::PlayerDetail(pds) => Some(pds.clone()),
                    _ => None,
                })
                .unwrap_or_else(PlayerDetailState::new);

            // Get season count from player data
            let seasons_count = {
                let data = shared_data.read().await;
                data.player_info
                    .get(player_id)
                    .and_then(|p| p.season_totals.as_ref())
                    .map(|seasons| {
                        seasons
                            .iter()
                            .filter(|s| s.league_abbrev == "NHL")
                            .count()
                    })
                    .unwrap_or(0)
            };

            let handled = match key.code {
                KeyCode::Down => {
                    if !player_state.selection_active && seasons_count > 0 {
                        // Enter season selection mode
                        player_state.selection_active = true;
                        player_state.selected_season_index = 0;
                        true
                    } else if player_state.selection_active {
                        // Navigate down in season list
                        if player_state.selected_season_index + 1 < seasons_count {
                            player_state.selected_season_index += 1;
                        }
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Up => {
                    if player_state.selection_active {
                        if player_state.selected_season_index == 0 {
                            // Exit season selection mode
                            player_state.selection_active = false;
                        } else {
                            // Navigate up in season list
                            player_state.selected_season_index -= 1;
                        }
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Enter => {
                    if player_state.selection_active {
                        // Navigate to team from selected season
                        let team_info = {
                            let data = shared_data.read().await;
                            if let Some(player) = data.player_info.get(player_id) {
                                if let Some(season_totals) = &player.season_totals {
                                    // Filter and reverse to match display order
                                    let nhl_seasons: Vec<_> = season_totals
                                        .iter()
                                        .filter(|s| s.league_abbrev == "NHL")
                                        .rev()
                                        .collect();

                                    if player_state.selected_season_index < nhl_seasons.len() {
                                        let season = nhl_seasons[player_state.selected_season_index];
                                        let team_name = season.team_name.default.clone();

                                        // Look up team abbreviation from standings
                                        let team_abbrev = data
                                            .standings
                                            .iter()
                                            .find(|s| s.team_name.default == team_name)
                                            .map(|s| s.team_abbrev.default.clone())
                                            .unwrap_or_else(|| team_name.clone());

                                        Some((team_name, team_abbrev))
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

                        if let Some((team_name, team_abbrev)) = team_info {
                            // Store current player state before navigating
                            state
                                .navigation
                                .data
                                .insert(cache_key, PanelState::PlayerDetail(player_state));

                            // Get team stats from standings
                            let team_data = {
                                let data = shared_data.read().await;
                                data.standings
                                    .iter()
                                    .find(|s| s.team_abbrev.default == team_abbrev)
                                    .map(|team| {
                                        (
                                            team.wins,
                                            team.losses,
                                            team.ot_losses,
                                            team.points,
                                            team.division_name.clone(),
                                            team.conference_name.clone(),
                                        )
                                    })
                            };

                            if let Some((wins, losses, ot_losses, points, division, conference)) =
                                team_data
                            {
                                let team_panel = StandingsPanel::TeamDetail {
                                    team_name,
                                    team_abbrev: team_abbrev.clone(),
                                    wins,
                                    losses,
                                    ot_losses,
                                    points,
                                    division_name: division,
                                    conference_name: conference,
                                };

                                // Create initial TeamDetailState and store in cache
                                let team_state = TeamDetailState::new();
                                let team_key = team_panel.cache_key();
                                state
                                    .navigation
                                    .data
                                    .insert(team_key, PanelState::TeamDetail(team_state));

                                state.navigation.navigate_to(team_panel);

                                // Set selected_team_abbrev to trigger club stats fetch
                                let mut data = shared_data.write().await;
                                data.selected_team_abbrev = Some(team_abbrev);
                                drop(data);

                                // Trigger refresh to fetch club stats
                                let _ = refresh_tx.send(()).await;
                            }

                            return true;
                        }
                    }
                    false
                }
                KeyCode::Esc => {
                    if player_state.selection_active {
                        // Exit season selection mode
                        player_state.selection_active = false;
                        true
                    } else {
                        // Go back from player panel
                        state.navigation.go_back();
                        true
                    }
                }
                KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                    player_state.scrollable.handle_key(key)
                }
                _ => false,
            };

            // Update cache with modified state
            if handled {
                state
                    .navigation
                    .data
                    .insert(cache_key, PanelState::PlayerDetail(player_state));
            }

            handled
        }
    }
}

pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // If we're in a panel view (navigation stack not at root), handle panel navigation
    if !state.navigation.is_at_root() {
        return handle_panel_navigation(key, state, shared_data, refresh_tx).await;
    } else {
        // Build layout to get team counts for navigation
        let data = shared_data.read().await;
        let layout = StandingsLayout::build(&data.standings, state.view, data.config.display_standings_western_first);
        drop(data);

        let max_columns = layout.column_count();
        let team_count_in_current_column = layout.columns
            .get(state.selected_column)
            .map(|col| col.team_count)
            .unwrap_or(0);

        if state.team_selection_active {
        // Team selection mode - navigate within the standings
        match key.code {
            KeyCode::Up => {
                if state.selected_team_index == 0 {
                    // At first team, exit team selection mode
                    state.team_selection_active = false;
                    true
                } else {
                    // Move up to previous team
                    state.selected_team_index = state.selected_team_index.saturating_sub(1);
                    true
                }
            }
            KeyCode::Down => {
                // Move down to next team
                if state.selected_team_index + 1 < team_count_in_current_column {
                    state.selected_team_index += 1;
                }
                true
            }
            KeyCode::Left => {
                // Switch to left column (in two-column views)
                if state.selected_column > 0 {
                    let new_column = state.selected_column - 1;
                    let new_column_team_count = layout.columns
                        .get(new_column)
                        .map(|col| col.team_count)
                        .unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::Right => {
                // Switch to right column (in two-column views)
                if state.selected_column + 1 < max_columns {
                    let new_column = state.selected_column + 1;
                    let new_column_team_count = layout.columns
                        .get(new_column)
                        .map(|col| col.team_count)
                        .unwrap_or(0);
                    navigate_to_column(state, new_column, new_column_team_count);
                }
                true
            }
            KeyCode::Enter => {
                // Navigate to team detail panel
                if let Some(team) = layout.get_team(state.selected_column, state.selected_team_index) {
                    let team_abbrev = team.team_abbrev.default.clone();
                    let panel = StandingsPanel::TeamDetail {
                        team_name: team.team_common_name.default.clone(),
                        team_abbrev: team_abbrev.clone(),
                        wins: team.wins,
                        losses: team.losses,
                        ot_losses: team.ot_losses,
                        points: team.points,
                        division_name: team.division_name.clone(),
                        conference_name: team.conference_name.clone(),
                    };

                    // Create initial TeamDetailState and store in cache
                    let panel_state = TeamDetailState::new();
                    let cache_key = panel.cache_key();
                    state
                        .navigation
                        .data
                        .insert(cache_key, PanelState::TeamDetail(panel_state));

                    state.navigation.navigate_to(panel);

                    // Set selected_team_abbrev to trigger club stats fetch
                    let mut data = shared_data.write().await;
                    data.selected_team_abbrev = Some(team_abbrev);
                    drop(data);

                    // Trigger refresh to fetch club stats
                    let _ = refresh_tx.send(()).await;
                }
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Scrolling keys
                state.scrollable.handle_key(key)
            }
            KeyCode::Esc => {
                // Exit team selection mode
                state.team_selection_active = false;
                true
            }
            _ => false,
        }
    } else {
        // View selection mode - switch between Division/Conference/League/Wildcard
        match key.code {
            KeyCode::Left => change_view(state, shared_data, |view| view.prev()).await,
            KeyCode::Right => change_view(state, shared_data, |view| view.next()).await,
            KeyCode::Down => {
                // Enter team selection mode (if there are teams to select)
                if team_count_in_current_column > 0 {
                    state.team_selection_active = true;
                }
                true
            }
            KeyCode::Up => {
                // Allow scrolling up even when not in team selection mode
                if state.scrollable.scroll_offset == 0 {
                    false // Let parent handler deal with it
                } else {
                    state.scrollable.handle_key(key)
                }
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Manual scrolling
                state.scrollable.handle_key(key)
            }
            _ => false,
        }
    }
    }
}

// === OLD IMPLEMENTATION - KEPT FOR REFERENCE ===
// This code represents the old state-based standings handler implementation
// Includes panel navigation (team details, player details) which we'll add later
//
// [... 426 lines of old handler code with panel navigation, etc. ...]
