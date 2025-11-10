use crossterm::event::{KeyCode, KeyEvent};
use nhl_api::Boxscore;

use crate::tui::scores::game_details::state::{GameDetailsState, PlayerSection};
use crate::tui::scores::game_details::players;
use crate::tui::scores::panel::ScoresPanel;
use crate::SharedDataHandle;

/// Handle key events for game details navigation
/// Returns true if the event was handled, false if parent should handle
/// Returns Some(panel) if navigation should occur to a new panel
pub async fn handle_key(
    key: KeyEvent,
    state: &mut GameDetailsState,
    boxscore: Option<&Boxscore>,
    shared_data: &SharedDataHandle,
) -> (bool, Option<ScoresPanel>) {
    match key.code {
        KeyCode::Down if !state.player_selection_active => {
            // Enter player selection mode
            state.player_selection_active = true;
            state.selected_section = PlayerSection::ScoringSummary(0);
            state.selected_index = 0;
            (true, None)
        }
        KeyCode::Down if state.player_selection_active => {
            // Navigate to next player in section
            if let Some(bs) = boxscore {
                navigate_to_next_player(state, bs);
            }
            (true, None)
        }
        KeyCode::Up if state.player_selection_active => {
            // Navigate to previous player in section
            // If at first player, exit player selection mode
            if state.selected_index == 0
                && matches!(state.selected_section, PlayerSection::ScoringSummary(0))
            {
                state.player_selection_active = false;
            } else if let Some(bs) = boxscore {
                navigate_to_previous_player(state, bs);
            }
            (true, None)
        }
        KeyCode::Tab if state.player_selection_active => {
            // Jump to next section
            jump_to_next_section(state);
            (true, None)
        }
        KeyCode::BackTab if state.player_selection_active => {
            // Jump to previous section
            jump_to_previous_section(state);
            (true, None)
        }
        KeyCode::Enter if state.player_selection_active => {
            // Select current player and navigate to player details
            if let Some(bs) = boxscore {
                if let Some(player) = players::find_player(bs, state.selected_section, state.selected_index) {
                    let mut data = shared_data.write().await;
                    data.selected_player_id = Some(player.player_id);
                    data.player_info_loading = true;

                    // Create navigation panel
                    let panel = ScoresPanel::PlayerDetail {
                        player_id: player.player_id,
                        player_name: player.name.clone(),
                        from_game_id: bs.id,
                    };

                    return (true, Some(panel));
                }
            }
            (true, None)
        }
        KeyCode::Esc => {
            // Exit player selection or game details
            if state.player_selection_active {
                state.player_selection_active = false;
                (true, None)
            } else {
                (false, None) // Let parent handle exit from game details
            }
        }
        KeyCode::PageDown | KeyCode::PageUp | KeyCode::Home | KeyCode::End => {
            // Delegate scrolling to Scrollable
            (state.scrollable.handle_key(key), None)
        }
        _ => (false, None),
    }
}

/// Navigate to the next player within the current section or move to next section
fn navigate_to_next_player(state: &mut GameDetailsState, boxscore: &Boxscore) {
    let section_count = players::section_player_count(boxscore, state.selected_section);

    if section_count == 0 {
        // Empty section, move to next section
        state.selected_section = state.selected_section.next();
        state.selected_index = 0;
        return;
    }

    if state.selected_index + 1 < section_count {
        // Move to next player in current section
        state.selected_index += 1;
    } else {
        // At end of section, wrap to next section
        state.selected_section = state.selected_section.next();
        state.selected_index = 0;
    }
}

/// Navigate to the previous player within the current section or move to previous section
fn navigate_to_previous_player(state: &mut GameDetailsState, boxscore: &Boxscore) {
    if state.selected_index > 0 {
        state.selected_index -= 1;
    } else {
        // Move to previous section's last player
        state.selected_section = state.selected_section.prev();
        let prev_section_count = players::section_player_count(boxscore, state.selected_section);
        state.selected_index = prev_section_count.saturating_sub(1);
    }
}

/// Jump to the next section in Tab order
fn jump_to_next_section(state: &mut GameDetailsState) {
    state.selected_section = state.selected_section.next();
    state.selected_index = 0;
}

/// Jump to the previous section in Shift+Tab order
fn jump_to_previous_section(state: &mut GameDetailsState) {
    state.selected_section = state.selected_section.prev();
    state.selected_index = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::SharedData;

    fn create_mock_shared_data() -> Arc<RwLock<SharedData>> {
        Arc::new(RwLock::new(SharedData::default()))
    }

    #[tokio::test]
    async fn test_enter_player_selection_mode() {
        let mut state = GameDetailsState::new();
        let shared_data = create_mock_shared_data();
        assert!(!state.player_selection_active);

        let key = KeyEvent::from(KeyCode::Down);
        let (handled, _) = handle_key(key, &mut state, None, &shared_data).await;

        assert!(handled);
        assert!(state.player_selection_active);
    }

    #[tokio::test]
    async fn test_exit_player_selection_mode_with_esc() {
        let mut state = GameDetailsState::new();
        let shared_data = create_mock_shared_data();
        state.player_selection_active = true;

        let key = KeyEvent::from(KeyCode::Esc);
        let (handled, _) = handle_key(key, &mut state, None, &shared_data).await;

        assert!(handled);
        assert!(!state.player_selection_active);
    }

    #[tokio::test]
    async fn test_exit_player_selection_mode_with_up_at_first() {
        let mut state = GameDetailsState::new();
        let shared_data = create_mock_shared_data();
        state.player_selection_active = true;
        state.selected_section = PlayerSection::ScoringSummary(0);
        state.selected_index = 0;

        let key = KeyEvent::from(KeyCode::Up);
        let (handled, _) = handle_key(key, &mut state, None, &shared_data).await;

        assert!(handled);
        assert!(!state.player_selection_active);
    }

    #[tokio::test]
    async fn test_tab_cycles_through_sections() {
        let mut state = GameDetailsState::new();
        let shared_data = create_mock_shared_data();
        state.player_selection_active = true;
        state.selected_section = PlayerSection::ScoringSummary(0);

        // Tab through all sections
        let key = KeyEvent::from(KeyCode::Tab);
        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::AwayForwards);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::AwayDefense);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::AwayGoalies);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::HomeForwards);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::HomeDefense);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::HomeGoalies);

        handle_key(key, &mut state, None, &shared_data).await;
        assert_eq!(state.selected_section, PlayerSection::ScoringSummary(0));
    }
}
