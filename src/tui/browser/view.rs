use super::state::State;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use crate::config::DisplayConfig;

/// Render the browser content with link highlighting
pub fn render_content(f: &mut Frame, area: Rect, _state: &State, _config: &DisplayConfig) {
    let paragraph = Paragraph::new("Hello Browser!")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
