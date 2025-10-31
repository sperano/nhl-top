use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render the breadcrumb navigation trail
pub fn render_breadcrumb(f: &mut Frame, area: Rect, breadcrumb: &[String]) {
    if breadcrumb.is_empty() {
        return;
    }

    let mut spans = Vec::new();

    for (i, crumb) in breadcrumb.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
        }

        let style = if i == breadcrumb.len() - 1 {
            // Last item (current location) is brighter
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(crumb.clone(), style));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(paragraph, area);
}
