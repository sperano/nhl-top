use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::components::Scrollable;

pub struct TopListView {
    category: String,
    scrollable: Scrollable,
}

impl TopListView {
    pub fn new(category: String) -> Self {
        TopListView {
            category,
            scrollable: Scrollable::new(),
        }
    }
}

impl View for TopListView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Top 30 - {} ", self.category));

        let content = format!(
            "TODO: Display top 30 {}\n\n\
             #  Player           Team  GP  Stat\n\
             1  Player One       BOS   15  25\n\
             2  Player Two       TOR   14  23\n\
             ...",
            self.category
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

    fn breadcrumb_label(&self) -> String {
        format!("Top 30 {}", self.category)
    }
}
