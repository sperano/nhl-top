use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_content(f: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new("...")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
