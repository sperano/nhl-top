use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Helper function to build a separator line with box-drawing connectors for tabs
fn build_tab_separator_line<'a, I>(tab_names: I, area_width: usize, style: Style) -> Line<'a>
where
    I: Iterator<Item = String>,
{
    let mut separator_spans = Vec::new();
    let mut pos = 0;

    for (i, tab_name) in tab_names.enumerate() {
        if i > 0 {
            // Add horizontal line before separator
            separator_spans.push(Span::raw("─".repeat(1)));
            separator_spans.push(Span::raw("┴"));
            separator_spans.push(Span::raw("─".repeat(1)));
            pos += 3;
        }
        // Add horizontal line under tab
        separator_spans.push(Span::raw("─".repeat(tab_name.len())));
        pos += tab_name.len();
    }

    // Fill rest of line
    if pos < area_width {
        separator_spans.push(Span::raw("─".repeat(area_width - pos)));
    }

    Line::from(separator_spans).style(style)
}

pub fn render(f: &mut Frame, area: Rect, tabs: &[&str], selected_index: usize, focused: bool) {
    // Determine base style based on focus
    let base_style = if focused {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Build tab line with separators
    let mut tab_spans = Vec::new();
    for (i, tab_name) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(" │ ", base_style));
        }

        let style = if i == selected_index {
            base_style.add_modifier(Modifier::REVERSED)
        } else {
            base_style
        };
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
