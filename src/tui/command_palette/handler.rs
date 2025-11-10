use crate::tui::app::AppState;
use crate::tui::SharedDataHandle;
use crossterm::event::{KeyCode, KeyEvent};
use anyhow::Result;
use tokio::sync::mpsc;

/// Handle keyboard events when the command palette is active
pub async fn handle_key(
    app_state: &mut AppState,
    key: KeyEvent,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> Result<()> {
    let palette = match &mut app_state.command_palette {
        Some(p) if p.is_visible => p,
        _ => return Ok(()),
    };

    match key.code {
        KeyCode::Char(c) => {
            palette.input_char(c);
            super::search::update_search_results(palette, shared_data).await;
        }
        KeyCode::Backspace => {
            palette.delete_char();
            super::search::update_search_results(palette, shared_data).await;
        }
        KeyCode::Up => {
            palette.select_previous();
        }
        KeyCode::Down => {
            palette.select_next();
        }
        KeyCode::Enter => {
            if let Some(result) = palette.results.get(palette.selected_index) {
                if let Some(command) = super::search::parse_navigation_path(&result.navigation_path) {
                    app_state.execute_navigation_command(command, shared_data, refresh_tx).await?;
                }
            }
        }
        KeyCode::Esc => {
            app_state.close_command_palette();
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{AppState, CurrentTab};
    use crate::tui::widgets::{CommandPalette, SearchResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::SharedData;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    async fn create_test_state() -> (AppState, Arc<RwLock<SharedData>>, mpsc::Sender<()>) {
        let mut app_state = AppState::new();
        app_state.open_command_palette();

        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        (app_state, shared_data, tx)
    }

    #[tokio::test]
    async fn test_handle_key_char_input() {
        let (mut app_state, shared_data, tx) = create_test_state().await;

        let key = create_key_event(KeyCode::Char('t'));
        handle_key(&mut app_state, key, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.command_palette.as_ref().unwrap().input, "t");
    }

    #[tokio::test]
    async fn test_handle_key_backspace() {
        let (mut app_state, shared_data, tx) = create_test_state().await;

        if let Some(ref mut palette) = app_state.command_palette {
            palette.input_char('a');
            palette.input_char('b');
        }

        let key = create_key_event(KeyCode::Backspace);
        handle_key(&mut app_state, key, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.command_palette.as_ref().unwrap().input, "a");
    }

    #[tokio::test]
    async fn test_handle_key_up_down_navigation() {
        let (mut app_state, shared_data, tx) = create_test_state().await;

        if let Some(ref mut palette) = app_state.command_palette {
            palette.set_results(vec![
                SearchResult::new("Item 1", "Cat", vec![]),
                SearchResult::new("Item 2", "Cat", vec![]),
                SearchResult::new("Item 3", "Cat", vec![]),
            ]);
        }

        assert_eq!(app_state.command_palette.as_ref().unwrap().selected_index, 0);

        handle_key(&mut app_state, create_key_event(KeyCode::Down), &shared_data, &tx).await.unwrap();
        assert_eq!(app_state.command_palette.as_ref().unwrap().selected_index, 1);

        handle_key(&mut app_state, create_key_event(KeyCode::Down), &shared_data, &tx).await.unwrap();
        assert_eq!(app_state.command_palette.as_ref().unwrap().selected_index, 2);

        handle_key(&mut app_state, create_key_event(KeyCode::Up), &shared_data, &tx).await.unwrap();
        assert_eq!(app_state.command_palette.as_ref().unwrap().selected_index, 1);
    }

    #[tokio::test]
    async fn test_handle_key_escape() {
        let (mut app_state, shared_data, tx) = create_test_state().await;

        assert!(app_state.command_palette.as_ref().unwrap().is_visible);
        assert!(app_state.command_palette_active);

        let key = create_key_event(KeyCode::Esc);
        handle_key(&mut app_state, key, &shared_data, &tx).await.unwrap();

        assert!(!app_state.command_palette.as_ref().unwrap().is_visible);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_handle_key_enter_with_navigation() {
        let (mut app_state, shared_data, tx) = create_test_state().await;

        if let Some(ref mut palette) = app_state.command_palette {
            palette.set_results(vec![
                SearchResult::new("Standings", "Tab", vec!["tab".to_string(), "standings".to_string()]),
            ]);
        }

        app_state.current_tab = CurrentTab::Scores;

        let key = create_key_event(KeyCode::Enter);
        handle_key(&mut app_state, key, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Standings);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_handle_key_when_palette_not_visible() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        let key = create_key_event(KeyCode::Char('t'));
        handle_key(&mut app_state, key, &shared_data, &tx).await.unwrap();

        // Should not affect input since palette is not visible
        assert_eq!(app_state.command_palette.as_ref().unwrap().input, "");
    }
}
