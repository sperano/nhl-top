use super::State;
use crate::tui::widgets::Container;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn build_container() -> Container {
    // For now, create an empty container - we'll add focusable widgets later
    Container::new()
}

pub fn render_content(f: &mut Frame, area: Rect, state: &mut State) {
    // Build container if not present
    if state.container.is_none() {
        state.container = Some(build_container());
    }

    // Render the placeholder content
    let paragraph = Paragraph::new("Hello Stats!")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
