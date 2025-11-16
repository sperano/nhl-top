use super::State;
use super::state::{PanelState, TeamDetailState, PlayerDetailState};
use crate::tui::common::CommonPanel;
use crate::types::SharedDataHandle;
use crate::tui::navigation::Panel;
use crate::tui::widgets::focus::{InputResult, NavigationAction};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

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
        CommonPanel::TeamDetail { team_abbrev, .. } => {
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
                                let player_panel = CommonPanel::PlayerDetail {
                                    player_id,
                                    player_name: format!(
                                        "{} {}",
                                        skater.first_name.default, skater.last_name.default
                                    ),
                                    from_context: format!("from team {}", team_abbrev),
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
        CommonPanel::PlayerDetail { player_id, .. } => {
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
                                let team_panel = CommonPanel::TeamDetail {
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

/// Handle input when using FocusableTable widgets for team navigation
async fn handle_team_table_input(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    let focused_idx = match state.focused_table_index {
        Some(idx) => idx,
        None => return false, // Not in table mode
    };

    // Get mutable reference to the focused table
    if focused_idx >= state.team_tables.len() {
        return false;
    }

    // Handle input through the table
    let result = {
        let table = &mut state.team_tables[focused_idx];
        table.handle_input(key)
    };

    match result {
        InputResult::Handled => true,
        InputResult::NotHandled => {
            // Table didn't handle it - check for Left/Right to switch tables
            match key.code {
                KeyCode::Left => {
                    // Move to left table if it exists
                    if focused_idx > 0 {
                        state.team_tables[focused_idx].set_focused(false);
                        state.focused_table_index = Some(focused_idx - 1);
                        state.team_tables[focused_idx - 1].set_focused(true);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Right => {
                    // Move to right table if it exists
                    if focused_idx + 1 < state.team_tables.len() {
                        state.team_tables[focused_idx].set_focused(false);
                        state.focused_table_index = Some(focused_idx + 1);
                        state.team_tables[focused_idx + 1].set_focused(true);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Up => {
                    // If we're at top of table, exit table mode
                    if focused_idx < state.team_tables.len() {
                        state.team_tables[focused_idx].set_focused(false);
                        state.focused_table_index = None;
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Esc => {
                    // Exit table mode
                    if focused_idx < state.team_tables.len() {
                        state.team_tables[focused_idx].set_focused(false);
                        state.focused_table_index = None;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        InputResult::Navigate(action) => {
            // Handle navigation action (e.g., NavigateToTeam)
            match action {
                NavigationAction::NavigateToTeam(team_abbrev) => {
                    // Look up team details from standings
                    let team_data = {
                        let data = shared_data.read().await;
                        data.standings
                            .iter()
                            .find(|s| s.team_abbrev.default == team_abbrev)
                            .map(|team| {
                                (
                                    team.team_common_name.default.clone(),
                                    team.team_abbrev.default.clone(),
                                    team.wins,
                                    team.losses,
                                    team.ot_losses,
                                    team.points,
                                    team.division_name.clone(),
                                    team.conference_name.clone(),
                                )
                            })
                    };

                    if let Some((name, abbrev, wins, losses, ot_losses, points, division, conference)) = team_data {
                        let panel = CommonPanel::TeamDetail {
                            team_name: name,
                            team_abbrev: abbrev.clone(),
                            wins,
                            losses,
                            ot_losses,
                            points,
                            division_name: division,
                            conference_name: conference,
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
                        data.selected_team_abbrev = Some(abbrev);
                        drop(data);

                        // Trigger refresh to fetch club stats
                        let _ = refresh_tx.send(()).await;
                    }

                    true
                }
                _ => false,
            }
        }
        _ => false,
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
    }

    // Check if we're using FocusableTable widgets for team navigation
    if state.focused_table_index.is_some() {
        return handle_team_table_input(key, state, shared_data, refresh_tx).await;
    }

    // View selection mode - switch between Division/Conference/League/Wildcard
    match key.code {
        KeyCode::Left => {
            // Cycle to previous view
            state.view = state.view.prev();
            // Clear tables to force rebuild on next render
            state.team_tables.clear();
            state.focused_table_index = None;
            true
        }
        KeyCode::Right => {
            // Cycle to next view
            state.view = state.view.next();
            // Clear tables to force rebuild on next render
            state.team_tables.clear();
            state.focused_table_index = None;
            true
        }
        KeyCode::Down => {
            // Enter table mode if tables are available
            if !state.team_tables.is_empty() {
                state.focused_table_index = Some(0);
                state.team_tables[0].set_focused(true);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::standings::GroupBy;
    use crate::types::SharedData;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use nhl_api::{Standing, LocalizedString};

    /// Create a minimal test standing entry
    fn create_test_standing(abbrev: &str, name: &str, wins: i32, points: i32) -> Standing {
        Standing {
            conference_abbrev: Some("E".to_string()),
            conference_name: Some("Eastern".to_string()),
            division_abbrev: "ATL".to_string(),
            division_name: "Atlantic".to_string(),
            team_name: LocalizedString { default: name.to_string() },
            team_common_name: LocalizedString { default: name.to_string() },
            team_abbrev: LocalizedString { default: abbrev.to_string() },
            team_logo: String::new(),
            wins,
            losses: 0,
            ot_losses: 0,
            points,
        }
    }

    /// Helper to create shared data with test standings
    fn create_test_shared_data() -> SharedDataHandle {
        let mut standings = vec![
            create_test_standing("TOR", "Toronto", 30, 60),
            create_test_standing("MTL", "Montreal", 25, 50),
            create_test_standing("OTT", "Ottawa", 20, 40),
        ];

        // Add Western conference teams for multi-column tests
        standings.push(Standing {
            conference_abbrev: Some("W".to_string()),
            conference_name: Some("Western".to_string()),
            division_abbrev: "CEN".to_string(),
            division_name: "Central".to_string(),
            team_name: LocalizedString { default: "Chicago".to_string() },
            team_common_name: LocalizedString { default: "Chicago".to_string() },
            team_abbrev: LocalizedString { default: "CHI".to_string() },
            team_logo: String::new(),
            wins: 28,
            losses: 20,
            ot_losses: 5,
            points: 61,
        });

        Arc::new(RwLock::new(SharedData {
            standings: Arc::new(standings),
            ..Default::default()
        }))
    }


    #[tokio::test]
    async fn test_esc_from_panel_player_selection_exits_selection_not_panel() {
        let mut state = State::new();
        let shared_data = create_test_shared_data();
        let (tx, _rx) = mpsc::channel::<()>(10);

        // Simulate being in a TeamDetail panel with player selection active
        state.view = GroupBy::Wildcard;
        state.subtab_focused = true;

        let panel = CommonPanel::TeamDetail {
            team_name: "Toronto".to_string(),
            team_abbrev: "TOR".to_string(),
            wins: 30,
            losses: 20,
            ot_losses: 5,
            points: 65,
            division_name: "Atlantic".to_string(),
            conference_name: Some("Eastern".to_string()),
        };
        let cache_key = panel.cache_key();
        state.navigation.navigate_to(panel);

        // Create panel state with selection active
        let mut panel_state = TeamDetailState::new();
        panel_state.selection_active = true;
        panel_state.selected_player_index = 2;
        state.navigation.data.insert(cache_key.clone(), PanelState::TeamDetail(panel_state));

        // Press ESC
        let key = KeyEvent::from(KeyCode::Esc);
        let handled = handle_key(key, &mut state, &shared_data, &tx).await;

        // Should handle the key
        assert!(handled, "ESC should be handled");

        // Should still be in the panel (not popped)
        assert!(!state.navigation.is_at_root(), "Should still be in the panel");

        // Should have exited player selection mode
        let updated_state = state.navigation.data.get(&cache_key);
        assert!(updated_state.is_some(), "Panel state should still exist");
        if let Some(PanelState::TeamDetail(tds)) = updated_state {
            assert!(!tds.selection_active, "Should exit player selection mode");
        } else {
            panic!("Panel state should be TeamDetail");
        }
    }

    #[tokio::test]
    async fn test_view_navigation_left_right() {
        let mut state = State::new();
        let shared_data = create_test_shared_data();
        let (tx, _rx) = mpsc::channel::<()>(10);

        // Start in Wildcard view
        state.view = GroupBy::Wildcard;
        state.subtab_focused = true;

        // Press Right - should go to Division
        let key = KeyEvent::from(KeyCode::Right);
        let handled = handle_key(key, &mut state, &shared_data, &tx).await;
        assert!(handled, "Right should be handled");
        assert_eq!(state.view, GroupBy::Division);

        // Press Right - should go to Conference
        let key = KeyEvent::from(KeyCode::Right);
        let handled = handle_key(key, &mut state, &shared_data, &tx).await;
        assert!(handled, "Right should be handled");
        assert_eq!(state.view, GroupBy::Conference);

        // Press Left - should go back to Division
        let key = KeyEvent::from(KeyCode::Left);
        let handled = handle_key(key, &mut state, &shared_data, &tx).await;
        assert!(handled, "Left should be handled");
        assert_eq!(state.view, GroupBy::Division);
    }


    #[tokio::test]
    async fn test_down_when_no_tables_built_does_nothing() {
        let mut state = State::new();
        let shared_data = create_test_shared_data();
        let (tx, _rx) = mpsc::channel::<()>(10);

        // Start in view selection mode with NO tables
        state.view = GroupBy::League;
        state.subtab_focused = true;
        state.team_tables.clear();

        // Press Down - should not enter table mode
        let key = KeyEvent::from(KeyCode::Down);
        let handled = handle_key(key, &mut state, &shared_data, &tx).await;

        assert!(!handled, "Down should not be handled when no tables exist");
        assert!(state.focused_table_index.is_none(), "Should not enter table mode");
    }
}