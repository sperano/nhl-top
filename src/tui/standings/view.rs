use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::commands::standings::GroupBy;
use super::State;

/// Helper function to build a separator line with box-drawing connectors for tabs
fn build_tab_separator_line<'a, I>(tab_names: I, area_width: usize, style: Style) -> Line<'a>
where
    I: Iterator<Item = String>,
{
    let mut separator_spans = Vec::new();
    let mut pos = 0;

    for (i, tab_name) in tab_names.enumerate() {
        if i > 0 {
            separator_spans.push(Span::raw("─".repeat(1)));
            separator_spans.push(Span::raw("┴"));
            separator_spans.push(Span::raw("─".repeat(1)));
            pos += 3;
        }
        separator_spans.push(Span::raw("─".repeat(tab_name.len())));
        pos += tab_name.len();
    }

    if pos < area_width {
        separator_spans.push(Span::raw("─".repeat(area_width - pos)));
    }

    Line::from(separator_spans).style(style)
}

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State) {
    let views = GroupBy::all();
    let standings_view = state.view;
    let focused = state.subtab_focused;

    // Determine base style based on focus
    let base_style = if focused {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Build subtab line with separators and left margin
    let mut subtab_spans = Vec::new();
    subtab_spans.push(Span::styled("  ", base_style)); // 2-space left margin

    for (i, view) in views.iter().enumerate() {
        if i > 0 {
            subtab_spans.push(Span::styled(" │ ", base_style));
        }

        let tab_text = format!("{}", view.name());
        let style = if *view == standings_view {
            base_style.add_modifier(Modifier::REVERSED)
        } else {
            base_style
        };
        subtab_spans.push(Span::styled(tab_text, style));
    }
    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors (adjust width for 2-space margin)
    let tab_names = views.iter().map(|view| view.name().to_string());
    let separator_line = build_tab_separator_line(tab_names, area.width.saturating_sub(2) as usize, base_style);

    // Add left margin to separator line
    let separator_with_margin = Line::from(vec![
        Span::styled("  ", base_style),
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
    // Add 2-space left padding to each line to align with sub-tab line
    let content = standings_text
        .lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}
