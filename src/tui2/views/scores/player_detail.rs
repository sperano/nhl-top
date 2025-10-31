use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::components::Scrollable;

pub struct PlayerDetailView {
    player_id: i64,
    player_name: String,
    scrollable: Scrollable,
}

impl PlayerDetailView {
    pub fn new(player_id: i64, player_name: String) -> Self {
        PlayerDetailView {
            player_id,
            player_name,
            scrollable: Scrollable::new(),
        }
    }
}

impl View for PlayerDetailView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} - Player Stats ", self.player_name));

        // TODO: Get actual player stats
        let content = format!(
            "Player: {}\nID: {}\n\n\
            Tonight's Performance:\n\
            TODO: Display game stats\n\n\
            Season Stats:\n\
            TODO: Display season stats\n\n\
            This is the deepest level. Press Esc to go back.",
            self.player_name, self.player_id
        );

        self.scrollable.render_paragraph(f, area, content, Some(block));
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Esc => KeyResult::GoBack,
            KeyCode::Char('q') => KeyResult::Quit,
            _ => {
                if self.scrollable.handle_key(key) {
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
        }
    }

    fn can_drill_down(&self) -> bool {
        false // Deepest level
    }

    fn breadcrumb_label(&self) -> String {
        self.player_name.clone()
    }
}
