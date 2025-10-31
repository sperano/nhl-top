use ratatui::{
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::commands::standings::GroupBy;
use super::State;
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};

// Content Layout Constants
/// Left margin for standings content
const CONTENT_LEFT_MARGIN: usize = 2;

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, selection_fg: Color, unfocused_selection_fg: Color) {
    let views = GroupBy::all();
    let standings_view = state.view;
    let focused = state.subtab_focused;

    let base_style = base_tab_style(focused);

    // Build subtab line with separators (no left margin)
    let mut subtab_spans = Vec::new();

    for (i, view) in views.iter().enumerate() {
        if i > 0 {
            subtab_spans.push(Span::styled(" │ ", base_style));
        }

        let tab_text = format!("{}", view.name());
        let style = selection_style(
            base_style,
            *view == standings_view,
            focused,
            selection_fg,
            unfocused_selection_fg,
        );
        subtab_spans.push(Span::styled(tab_text, style));
    }
    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors
    let tab_names = views.iter().map(|view| view.name().to_string());
    let separator_line = build_tab_separator_line(
        tab_names,
        area.width as usize,
        base_style
    );

    let separator_with_margin = Line::from(vec![
        Span::styled(separator_line.to_string(), base_style),
    ]);

    // Render subtabs with separator line
    let subtab_widget = Paragraph::new(vec![subtab_line, separator_with_margin])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(subtab_widget, area);
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    standings_data: &[nhl_api::Standing],
    state: &mut State,
    western_first: bool,
) {
    let standings_text = crate::commands::standings::format_standings_by_group(
        standings_data,
        state.view,
        western_first,
    );
    // Add left padding to each line
    let content = standings_text
        .lines()
        .map(|line| format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), line))
        .collect::<Vec<_>>()
        .join("\n");

    state.scrollable.render_paragraph(f, area, content, None);
}
