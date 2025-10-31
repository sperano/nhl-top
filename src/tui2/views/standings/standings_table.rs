use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use crate::tui2::traits::{View, KeyResult};
use crate::tui2::components::Scrollable;

pub struct StandingsTableView {
    scrollable: Scrollable,
}

impl StandingsTableView {
    pub fn new() -> Self {
        StandingsTableView {
            scrollable: Scrollable::new(),
        }
    }
}

impl View for StandingsTableView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Standings ");

        let content = "TODO: Display standings table\n\n\
                       # Team    GP  W  L  OT PTS\n\
                       1 BOS     15 11  3   1  23\n\
                       2 TOR     15 10  4   1  21\n\
                       ...".to_string();

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
        "Standings".to_string()
    }
}
