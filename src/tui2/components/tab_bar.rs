use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::tui2::app::Tab;
use crate::tui2::theme;

/// Render the top tab bar with number shortcuts
pub fn render_tab_bar(f: &mut Frame, area: Rect, current_tab: Tab) {
    let tabs = [Tab::Scores, Tab::Standings, Tab::Stats, Tab::Settings];

    let mut spans = Vec::new();

    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }

        let label = format!("{}. {}", tab.number(), tab.label());

        let style = if *tab == current_tab {
            theme::tab_active_style()
        } else {
            theme::tab_inactive_style()
        };

        spans.push(Span::styled(label, style));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(paragraph, area);
}
