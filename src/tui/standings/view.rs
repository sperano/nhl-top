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

// Subtab Layout Constants
/// Left margin for subtab bar and content (spaces before standings view tabs)
const SUBTAB_LEFT_MARGIN: usize = 2;

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, selection_fg: Color, unfocused_selection_fg: Color) {
    let views = GroupBy::all();
    let standings_view = state.view;
    let focused = state.subtab_focused;

    let base_style = base_tab_style(focused);

    // Build subtab line with separators and left margin
    let mut subtab_spans = Vec::new();
    subtab_spans.push(Span::styled(" ".repeat(SUBTAB_LEFT_MARGIN), base_style)); // Left margin

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

    // Build separator line with connectors (adjust width for left margin)
    let tab_names = views.iter().map(|view| view.name().to_string());
    let separator_line = build_tab_separator_line(
        tab_names,
        area.width.saturating_sub(SUBTAB_LEFT_MARGIN as u16) as usize,
        base_style
    );

    // Add left margin to separator line
    let separator_with_margin = Line::from(vec![
        Span::styled(" ".repeat(SUBTAB_LEFT_MARGIN), base_style),
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
    state: &State,
    western_first: bool,
) {
    let standings_text = crate::commands::standings::format_standings_by_group(
        standings_data,
        state.view,
        western_first,
    );
    // Add left padding to each line to align with sub-tab line
    let content = standings_text
        .lines()
        .map(|line| format!("{}{}", " ".repeat(SUBTAB_LEFT_MARGIN), line))
        .collect::<Vec<_>>()
        .join("\n");

    let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
