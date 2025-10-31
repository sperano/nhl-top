use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::components::Scrollable;
use crate::SharedDataHandle;
use nhl_api::DailySchedule;
use std::collections::HashMap;

pub struct GameListView {
    shared_data: SharedDataHandle,
    selected_index: usize,
    scrollable: Scrollable,
}

impl GameListView {
    pub fn new(shared_data: SharedDataHandle) -> Self {
        GameListView {
            shared_data,
            selected_index: 0,
            scrollable: Scrollable::new(),
        }
    }

    fn get_games_count(&self) -> usize {
        // This would need to be async or cached - for now return estimate
        // In practice, you'd store schedule in the view or get it from shared data
        0
    }
}

impl View for GameListView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Today's Games ");

        // TODO: Get actual schedule from shared_data
        // For now, render placeholder
        let content = "Loading games...\n\nPress Enter on a game to see details.".to_string();

        self.scrollable.render_paragraph(f, area, content, Some(block));
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                KeyResult::Handled
            }
            KeyCode::Down => {
                let max_index = self.get_games_count().saturating_sub(1);
                if self.selected_index < max_index {
                    self.selected_index += 1;
                }
                KeyResult::Handled
            }
            KeyCode::Enter => {
                // TODO: Drill down to game detail
                // For now, just handle the key
                KeyResult::Handled
            }
            KeyCode::Esc => KeyResult::GoBack,
            KeyCode::Char('q') => KeyResult::Quit,
            _ => {
                // Try scrollable keys
                if self.scrollable.handle_key(key) {
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
        }
    }

    fn can_drill_down(&self) -> bool {
        self.get_games_count() > 0
    }

    fn breadcrumb_label(&self) -> String {
        "Game List".to_string()
    }
}
