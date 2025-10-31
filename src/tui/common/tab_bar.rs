use ratatui::{
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::separator::build_tab_separator_line;
use super::styling::{base_tab_style, selection_style};

pub fn render(f: &mut Frame, area: Rect, tabs: &[&str], selected_index: usize, focused: bool, selection_fg: Color, unfocused_selection_fg: Color) {
    let base_style = base_tab_style(focused);

    // Build tab line with separators
    let mut tab_spans = Vec::new();
    for (i, tab_name) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(" │ ", base_style));
        }

        let style = selection_style(
            base_style,
            i == selected_index,
            focused,
            selection_fg,
            unfocused_selection_fg,
        );
        tab_spans.push(Span::styled(tab_name.to_string(), style));
    }
    let tab_line = Line::from(tab_spans);

    // Build separator line with connectors
    let tab_names = tabs.iter().map(|s| s.to_string());
    let separator_line = build_tab_separator_line(tab_names, area.width as usize, base_style);

    // Render custom tabs
    let tabs_widget = Paragraph::new(vec![tab_line, separator_line])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(tabs_widget, area);
}
