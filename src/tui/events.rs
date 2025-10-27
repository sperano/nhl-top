use crossterm::event::{KeyCode, KeyEvent};
use crate::commands::standings::GroupBy;
use super::tabs::{AppState, Tab};
use crate::SharedDataHandle;
use tokio::sync::mpsc;

pub enum AppAction {
    Continue,
    Exit,
}

pub async fn handle_key_event(key: KeyEvent, state: &mut AppState, shared_data: &SharedDataHandle, refresh_tx: &mpsc::Sender<()>) -> AppAction {
    match key.code {
        KeyCode::Esc => AppAction::Exit,

        // Arrow key navigation
        KeyCode::Left => {
            if state.subtab_focused {
                if state.current_tab == Tab::Scores {
                    // Navigate scores dates - move selection left
                    if state.scores_selected_index > 0 {
                        // Move selection within visible window
                        state.scores_selected_index -= 1;
                        // Update game_date to reflect the newly selected date
                        {
                            let mut data = shared_data.write().await;
                            data.game_date = data.game_date.add_days(-1);
                            // Clear schedule data to show "Loading..." while fetching
                            data.schedule = None;
                            data.period_scores.clear();
                            data.game_info.clear();
                        }
                        // Trigger immediate refresh
                        let _ = refresh_tx.send(()).await;
                    } else {
                        // Already at leftmost position, shift window left
                        {
                            let mut data = shared_data.write().await;
                            data.game_date = data.game_date.add_days(-1);
                            // Clear schedule data to show "Loading..." while fetching
                            data.schedule = None;
                            data.period_scores.clear();
                            data.game_info.clear();
                        }
                        // Trigger immediate refresh
                        let _ = refresh_tx.send(()).await;
                        // Keep selection at index 0 (leftmost)
                    }
                } else if state.current_tab == Tab::Standings {
                    // Navigate standings view
                    state.standings_view = match state.standings_view {
                        GroupBy::Division => GroupBy::League,
                        GroupBy::Conference => GroupBy::Division,
                        GroupBy::League => GroupBy::Conference,
                    };
                }
            } else {
                // Navigate main tabs
                state.current_tab = match state.current_tab {
                    Tab::Scores => Tab::Settings,
                    Tab::Standings => Tab::Scores,
                    Tab::Settings => Tab::Standings,
                };
                // Reset subtab focus when leaving tabs with subtabs
                if state.current_tab != Tab::Standings && state.current_tab != Tab::Scores {
                    state.subtab_focused = false;
                }
            }
            AppAction::Continue
        }
        KeyCode::Right => {
            if state.subtab_focused {
                if state.current_tab == Tab::Scores {
                    // Navigate scores dates - move selection right
                    if state.scores_selected_index < 2 {
                        // Move selection within visible window
                        state.scores_selected_index += 1;
                        // Update game_date to reflect the newly selected date
                        {
                            let mut data = shared_data.write().await;
                            data.game_date = data.game_date.add_days(1);
                            // Clear schedule data to show "Loading..." while fetching
                            data.schedule = None;
                            data.period_scores.clear();
                            data.game_info.clear();
                        }
                        // Trigger immediate refresh
                        let _ = refresh_tx.send(()).await;
                    } else {
                        // Already at rightmost position, shift window right
                        {
                            let mut data = shared_data.write().await;
                            data.game_date = data.game_date.add_days(1);
                            // Clear schedule data to show "Loading..." while fetching
                            data.schedule = None;
                            data.period_scores.clear();
                            data.game_info.clear();
                        }
                        // Trigger immediate refresh
                        let _ = refresh_tx.send(()).await;
                        // Keep selection at index 2 (rightmost)
                    }
                } else if state.current_tab == Tab::Standings {
                    // Navigate standings view
                    state.standings_view = match state.standings_view {
                        GroupBy::Division => GroupBy::Conference,
                        GroupBy::Conference => GroupBy::League,
                        GroupBy::League => GroupBy::Division,
                    };
                }
            } else {
                // Navigate main tabs
                state.current_tab = match state.current_tab {
                    Tab::Scores => Tab::Standings,
                    Tab::Standings => Tab::Settings,
                    Tab::Settings => Tab::Scores,
                };
                // Reset subtab focus when leaving tabs with subtabs
                if state.current_tab != Tab::Standings && state.current_tab != Tab::Scores {
                    state.subtab_focused = false;
                }
            }
            AppAction::Continue
        }
        KeyCode::Down => {
            // Activate sub-tab navigation (on Scores or Standings tabs)
            if (state.current_tab == Tab::Scores || state.current_tab == Tab::Standings) && !state.subtab_focused {
                state.subtab_focused = true;
            }
            AppAction::Continue
        }
        KeyCode::Up => {
            // Deactivate sub-tab navigation
            if state.subtab_focused {
                state.subtab_focused = false;
            }
            AppAction::Continue
        }

        _ => AppAction::Continue,
    }
}
