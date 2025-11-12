use super::State;
use crate::tui::widgets::Container;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn build_container() -> Container {
    Container::new()
}

pub fn render_content(f: &mut Frame, area: Rect, state: &mut State) {
    if state.container.is_none() {
        state.container = Some(build_container());
    }

    let paragraph = Paragraph::new("Hello Players!")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
