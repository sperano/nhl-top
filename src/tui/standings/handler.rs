use crossterm::event::{KeyCode, KeyEvent};
use super::State;
use super::panel::{StandingsPanel, fake_team_data, fake_player_data};
use crate::tui::navigation::NavigationContext;
use crate::SharedDataHandle;
use tokio::sync::mpsc;

fn navigate_to_column(state: &mut State, new_column: usize, new_column_team_count: usize) {
    state.selected_column = new_column;
    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
        state.selected_team_index = new_column_team_count - 1;
    }
}

async fn handle_panel_navigation(key: KeyEvent, state: &mut State, shared_data: &SharedDataHandle) -> bool {
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
                navigate_to_selected_item(state, &panel);
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                state.panel_scrollable.handle_key(key)
            }
            KeyCode::Esc => {
                state.panel_selection_active = false;
                true
            }
            _ => false,
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                if let Some(nav_ctx) = &mut state.navigation {
                    if nav_ctx.stack.depth() <= 1 {
                        // Clear selected team when exiting team view
                        let mut shared = shared_data.write().await;
                        shared.selected_team_abbrev = None;
                        state.navigation = None;
                    } else {
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
            // Fallback to fake data count if no real data available yet
            let fake_data = fake_team_data("");
            fake_data.players.len() + fake_data.goalies.len()
        }
        StandingsPanel::PlayerDetail { player_name, .. } => {
            let data = fake_player_data(player_name);
            data.career_stats.len()
        }
    }
}

fn navigate_to_selected_item(state: &mut State, panel: &StandingsPanel) {
    match panel {
        StandingsPanel::TeamDetail { team_name, .. } => {
            let data = fake_team_data(team_name);
            let idx = state.panel_selected_index;

            if idx < data.players.len() {
                let player = &data.players[idx];
                tracing::info!("Navigating to player: {}", player.name);

                let nav_ctx = state.navigation.as_mut().unwrap();
                nav_ctx.navigate_to(StandingsPanel::PlayerDetail {
                    player_id: (idx + 1) as i64,
                    player_name: player.name.clone(),
                    from_team_name: team_name.clone(),
                });
                state.panel_scrollable.reset();
                state.panel_selection_active = false;
                state.panel_selected_index = 0;
            }
        }
        StandingsPanel::PlayerDetail { player_name, .. } => {
            let data = fake_player_data(player_name);
            let idx = state.panel_selected_index;

            if idx < data.career_stats.len() {
                let season = &data.career_stats[idx];
                tracing::info!("Navigating to team: {} from season {}", season.team, season.season);

                let nav_ctx = state.navigation.as_mut().unwrap();
                nav_ctx.navigate_to(StandingsPanel::TeamDetail {
                    team_id: season.team_id,
                    team_name: season.team.clone(),
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
        return handle_panel_navigation(key, state, shared_data).await;
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
                    // Set the selected team abbreviation in shared data
                    {
                        let mut shared = shared_data.write().await;
                        shared.selected_team_abbrev = Some(team.team_abbrev.default.clone());
                    }

                    // Trigger a refresh to fetch club stats immediately
                    let _ = refresh_tx.send(()).await;

                    let mut nav_ctx: NavigationContext<StandingsPanel, String, _> = NavigationContext::new();
                    nav_ctx.navigate_to(StandingsPanel::TeamDetail {
                        team_id: 1,
                        team_name: team.team_common_name.default.clone(),
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
