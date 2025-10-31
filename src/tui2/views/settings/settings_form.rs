use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use crate::tui2::traits::{View, KeyResult};

pub struct SettingsFormView {
    // TODO: Track which field is selected
}

impl SettingsFormView {
    pub fn new() -> Self {
        SettingsFormView {}
    }
}

impl View for SettingsFormView {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Settings ");

        let content = "TODO: Editable settings form\n\n\
                       Refresh Interval: [60] seconds\n\
                       Western Conf First: [✓]\n\
                       Time Format: [%H:%M:%S]\n\
                       Debug Mode: [ ]\n\n\
                       [Save]  [Cancel]".to_string();

        let paragraph = Paragraph::new(content).block(block);
        f.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            KeyCode::Esc => KeyResult::GoBack,
            KeyCode::Char('q') => KeyResult::Quit,
            _ => KeyResult::NotHandled,
        }
    }

    fn breadcrumb_label(&self) -> String {
        "Settings".to_string()
    }
}
