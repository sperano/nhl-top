use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::components::Scrollable;

pub struct GameDetailView {
    game_id: i64,
    scrollable: Scrollable,
}

impl GameDetailView {
    pub fn new(game_id: i64) -> Self {
        GameDetailView {
            game_id,
            scrollable: Scrollable::new(),
        }
    }
}

impl View for GameDetailView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Game {} - Boxscore ", self.game_id));

        // TODO: Get actual boxscore from shared_data
        let content = format!(
            "Boxscore for game {}\n\n\
            TODO: Display scoring summary\n\
            TODO: Display player stats tables\n\n\
            Press Enter on a player to see details.",
            self.game_id
        );

        self.scrollable.render_paragraph(f, area, content, Some(block));
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Enter => {
                // TODO: Drill down to player detail
                KeyResult::Handled
            }
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
        true
    }

    fn breadcrumb_label(&self) -> String {
        format!("Game {}", self.game_id)
    }
}
