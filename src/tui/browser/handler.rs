use super::state::State;
use super::target::Target;
use crate::SharedDataHandle;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard events for the browser tab
pub async fn handle_key(
    key: KeyEvent,
    state: &mut State,
    shared_data: &SharedDataHandle,
) -> bool {
    match key.code {
        KeyCode::Down => {
            state.select_next_link();
            true
        }
        KeyCode::Up => {
            state.select_previous_link();
            true
        }
        KeyCode::Enter => {
            if let Some(link) = state.get_selected_link() {
                let message = format_link_activation(link.display.as_str(), &link.target);
                update_status_message(shared_data, message, false).await;
            }
            true
        }
        _ => false,
    }
}

/// Format a message for link activation
fn format_link_activation(display: &str, target: &Target) -> String {
    match target {
        Target::Team { id } => format!("Team: {} ({})", display, id),
        Target::Player { id } => format!("Player: {} (ID: {})", display, id),
    }
}

/// Update the status message in shared data
async fn update_status_message(
    shared_data: &SharedDataHandle,
    message: String,
    is_error: bool,
) {
    let mut data = shared_data.write().await;
    data.status_message = Some(message);
    data.status_is_error = is_error;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::SharedData;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[tokio::test]
    async fn test_handle_down_key() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        assert_eq!(state.selected_link_index, Some(0));

        let handled = handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;

        assert!(handled);
        assert_eq!(state.selected_link_index, Some(1));
    }

    #[tokio::test]
    async fn test_handle_up_key() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        assert_eq!(state.selected_link_index, Some(0));

        let handled = handle_key(create_key_event(KeyCode::Up), &mut state, &shared_data).await;

        assert!(handled);
        assert_eq!(state.selected_link_index, Some(2)); // Wraps to last link
    }

    #[tokio::test]
    async fn test_handle_enter_key_player() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // First link is Nick Suzuki (player)
        assert_eq!(state.selected_link_index, Some(0));

        let handled = handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        assert!(handled);

        let data = shared_data.read().await;
        assert!(data.status_message.is_some());
        let message = data.status_message.as_ref().unwrap();
        assert!(message.contains("Player:"));
        assert!(message.contains("Nick Suzuki"));
        assert!(message.contains("8480018"));
        assert!(!data.status_is_error);
    }

    #[tokio::test]
    async fn test_handle_enter_key_team() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // Move to second link (Canadiens - team)
        state.select_next_link();
        assert_eq!(state.selected_link_index, Some(1));

        let handled = handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        assert!(handled);

        let data = shared_data.read().await;
        assert!(data.status_message.is_some());
        let message = data.status_message.as_ref().unwrap();
        assert!(message.contains("Team:"));
        assert!(message.contains("Canadiens"));
        assert!(message.contains("MTL"));
        assert!(!data.status_is_error);
    }

    #[tokio::test]
    async fn test_handle_enter_key_third_link() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // Move to third link (Golden Knights - team)
        state.select_next_link();
        state.select_next_link();
        assert_eq!(state.selected_link_index, Some(2));

        let handled = handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        assert!(handled);

        let data = shared_data.read().await;
        assert!(data.status_message.is_some());
        let message = data.status_message.as_ref().unwrap();
        assert!(message.contains("Team:"));
        assert!(message.contains("Golden Knights"));
        assert!(message.contains("VGK"));
    }

    #[tokio::test]
    async fn test_handle_unhandled_key() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        let handled = handle_key(create_key_event(KeyCode::Char('x')), &mut state, &shared_data).await;

        assert!(!handled);
    }

    #[tokio::test]
    async fn test_handle_multiple_keys_sequence() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // Start at first link
        assert_eq!(state.selected_link_index, Some(0));

        // Press Down twice
        handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;
        handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;
        assert_eq!(state.selected_link_index, Some(2)); // At last link

        // Press Up once
        handle_key(create_key_event(KeyCode::Up), &mut state, &shared_data).await;
        assert_eq!(state.selected_link_index, Some(1)); // Back to second link

        // Press Enter
        handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        let data = shared_data.read().await;
        let message = data.status_message.as_ref().unwrap();
        assert!(message.contains("Canadiens"));
    }

    #[tokio::test]
    async fn test_wrap_around_down() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // Press Down three times to wrap around (3 links total)
        handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;
        handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;
        handle_key(create_key_event(KeyCode::Down), &mut state, &shared_data).await;

        // Should wrap back to first link
        assert_eq!(state.selected_link_index, Some(0));
    }

    #[tokio::test]
    async fn test_wrap_around_up() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        // Start at first link, press Up to wrap to last
        handle_key(create_key_event(KeyCode::Up), &mut state, &shared_data).await;

        // Should wrap to last link (index 2)
        assert_eq!(state.selected_link_index, Some(2));
    }

    #[tokio::test]
    async fn test_format_link_activation_player() {
        let target = Target::Player { id: 8480018 };
        let message = format_link_activation("Nick Suzuki", &target);

        assert_eq!(message, "Player: Nick Suzuki (ID: 8480018)");
    }

    #[tokio::test]
    async fn test_format_link_activation_team() {
        let target = Target::Team { id: "MTL".to_string() };
        let message = format_link_activation("Canadiens", &target);

        assert_eq!(message, "Team: Canadiens (MTL)");
    }

    #[tokio::test]
    async fn test_empty_state_enter_key() {
        let mut state = State {
            content: crate::tui::browser::BrowserContent::builder().build(),
            selected_link_index: None,
            scroll_offset: 0,
            subtab_focused: false,
        };
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        let handled = handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        assert!(handled);

        // No link selected, so status message should remain None
        let data = shared_data.read().await;
        assert!(data.status_message.is_none());
    }

    #[tokio::test]
    async fn test_status_message_not_error() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        handle_key(create_key_event(KeyCode::Enter), &mut state, &shared_data).await;

        let data = shared_data.read().await;
        assert!(!data.status_is_error);
    }

    #[tokio::test]
    async fn test_ignored_keys() {
        let mut state = State::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        let ignored_keys = vec![
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Char('a'),
            KeyCode::Char('1'),
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Tab,
            KeyCode::BackTab,
        ];

        for key_code in ignored_keys {
            let handled = handle_key(create_key_event(key_code), &mut state, &shared_data).await;
            assert!(!handled, "Key {:?} should not be handled", key_code);
        }
    }
}
