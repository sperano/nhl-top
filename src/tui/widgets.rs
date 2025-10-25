use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use crate::commands::standings::GroupBy;
use super::tabs::Tab;

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

pub fn render_tab_bar(f: &mut Frame, area: Rect, current_tab: Tab, focused: bool) {
    let tabs_vec = Tab::all();
    let selected_index = tabs_vec.iter().position(|&t| t == current_tab).unwrap_or(0);

    // Determine base style based on focus
    let base_style = if focused {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Build tab line with separators
    let mut tab_spans = Vec::new();
    for (i, tab) in tabs_vec.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(" │ ", base_style));
        }

        let tab_text = format!("{}", tab.name());
        let style = if i == selected_index {
            base_style.add_modifier(Modifier::REVERSED)
        } else {
            base_style
        };
        tab_spans.push(Span::styled(tab_text, style));
    }
    let tab_line = Line::from(tab_spans);

    // Build separator line with connectors
    let tab_names = tabs_vec.iter().map(|tab| tab.name().to_string());
    let separator_line = build_tab_separator_line(tab_names, area.width as usize, base_style);

    // Render custom tabs
    let tabs_widget = Paragraph::new(vec![tab_line, separator_line])
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(tabs_widget, area);
}

pub fn render_standings_subtabs(f: &mut Frame, area: Rect, standings_view: GroupBy, focused: bool) {
    let views = GroupBy::all();

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

pub fn render_status_bar(f: &mut Frame, area: Rect, last_refresh: Option<SystemTime>, time_format: &str) {
    let status_text = if let Some(refresh_time) = last_refresh {
        let datetime: DateTime<Local> = refresh_time.into();
        let formatted_time = datetime.format(time_format).to_string();
        format!("last refresh: {}", formatted_time)
    } else {
        "last refresh: never".to_string()
    };

    // Create a line that fills the entire width with spaces (for reverse video background)
    let status_line = format!("{:>width$}", status_text, width = area.width as usize);
    let status_bar = Paragraph::new(status_line)
        .style(Style::default().bg(Color::White).fg(Color::Black));

    f.render_widget(status_bar, area);
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    current_tab: Tab,
    standings_data: &[nhl_api::Standing],
    schedule_data: &Option<nhl_api::DailySchedule>,
    period_scores: &std::collections::HashMap<i64, crate::commands::scores_format::PeriodScores>,
    standings_view: GroupBy,
    western_first: bool,
) {
    let content = match current_tab {
        Tab::Scores => {
            if let Some(schedule) = schedule_data {
                crate::commands::scores_format::format_scores_for_tui(schedule, period_scores)
            } else {
                "Loading scores...".to_string()
            }
        }
        Tab::Standings => {
            let standings_text = crate::commands::standings::format_standings_by_group(
                standings_data,
                standings_view,
                western_first,
            );
            // Add 2-space left padding to each line to align with sub-tab line
            standings_text
                .lines()
                .map(|line| format!("  {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => format!("Hello {}!", current_tab.name()),
    };

    let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, area);
}
