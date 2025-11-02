use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph},
    Frame,
};
use crate::commands::standings::GroupBy;
use super::{State, layout::StandingsLayout};
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};

// Layout Constants
const CONTENT_LEFT_MARGIN: usize = 2;
const TEAM_NAME_COL_WIDTH: usize = 25;
const GP_COL_WIDTH: usize = 3;
const W_COL_WIDTH: usize = 3;
const L_COL_WIDTH: usize = 3;
const OT_COL_WIDTH: usize = 3;
const PTS_COL_WIDTH: usize = 4;
const STANDINGS_COLUMN_WIDTH: usize = 48; // Actual table width with all columns
const COLUMN_SPACING: usize = 4;

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, selection_fg: Color, unfocused_selection_fg: Color) {
    let views = GroupBy::all();
    let standings_view = state.view;
    let focused = state.subtab_focused;

    let base_style = base_tab_style(focused);

    // Build subtab line with separators
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
    let subtab_widget = Paragraph::new(vec![subtab_line, separator_with_margin]);

    f.render_widget(subtab_widget, area);
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    selection_fg: Color,
) {
    // Build layout if standings data is available
    let layout = match &state.layout_cache {
        Some(layout) => layout.clone(),
        None => return, // No data to render
    };

    // Render the layout
    let lines = render_layout(&layout, state, selection_fg);

    // Update scrollable dimensions
    state.scrollable.update_viewport_height(area.height);
    state.scrollable.update_content_height(lines.len());

    // Auto-scroll to ensure selected team is visible
    if state.team_selection_active {
        ensure_team_visible(state, &lines);
    }

    let paragraph = Paragraph::new(lines)
        .scroll((state.scrollable.scroll_offset, 0));
    f.render_widget(paragraph, area);
}

/// Render the standings layout to a vector of lines
fn render_layout(layout: &StandingsLayout, state: &State, selection_fg: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add initial blank line
    lines.push(Line::raw(""));

    match layout.view {
        GroupBy::League => render_single_column(layout, state, selection_fg, &mut lines),
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard => render_two_columns(layout, state, selection_fg, &mut lines),
    }

    lines
}

/// Render a single-column layout (League view)
fn render_single_column(layout: &StandingsLayout, state: &State, selection_fg: Color, lines: &mut Vec<Line<'static>>) {
    let column = &layout.columns[0];

    for group in &column.groups {
        // Render header if present
        if !group.header.is_empty() {
            lines.push(Line::raw(format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), group.header)));
            lines.push(Line::raw(format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), "═".repeat(group.header.len()))));
            lines.push(Line::raw(""));
        }

        // Render table header
        lines.push(render_table_header());
        // Separator line should exclude the margin since we add it separately
        lines.push(Line::raw(format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), "─".repeat(STANDINGS_COLUMN_WIDTH - CONTENT_LEFT_MARGIN))));

        // Render teams
        let mut team_idx = 0;
        for team in &group.teams {
            let is_selected = state.team_selection_active
                && state.selected_column == 0
                && state.selected_team_index == team_idx;

            lines.push(render_team_row(team, is_selected, selection_fg, CONTENT_LEFT_MARGIN));
            team_idx += 1;
        }
    }
}

/// Render a two-column layout (Conference/Division view)
fn render_two_columns(layout: &StandingsLayout, state: &State, selection_fg: Color, lines: &mut Vec<Line<'static>>) {
    let left_lines = render_column(&layout.columns[0], state, selection_fg, 0);
    let right_lines = if layout.columns.len() > 1 {
        render_column(&layout.columns[1], state, selection_fg, 1)
    } else {
        vec![]
    };

    // Merge columns side by side
    let max_len = left_lines.len().max(right_lines.len());
    for i in 0..max_len {
        let left = left_lines.get(i).cloned().unwrap_or_else(|| Line::raw(""));
        let right = right_lines.get(i).cloned().unwrap_or_else(|| Line::raw(""));

        // Combine left and right with proper spacing
        let mut spans = Vec::new();

        // Add left content with padding - preserve spans for styling
        let left_text = line_to_string(&left);
        let left_len = left_text.chars().count(); // Count characters, not bytes (for Unicode)

        // Add all spans from left column
        for span in left.spans {
            spans.push(span);
        }

        // Add padding to reach column width
        if left_len < STANDINGS_COLUMN_WIDTH {
            spans.push(Span::raw(" ".repeat(STANDINGS_COLUMN_WIDTH - left_len)));
        }

        // Add column spacing
        spans.push(Span::raw(" ".repeat(COLUMN_SPACING)));

        // Add right content
        for span in right.spans {
            spans.push(span);
        }

        lines.push(Line::from(spans));
    }
}

/// Render a single column (for two-column layouts)
fn render_column(column: &super::layout::StandingsColumn, state: &State, selection_fg: Color, col_idx: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut team_idx = 0;

    // Both columns should have the same internal margin for consistency
    let margin = CONTENT_LEFT_MARGIN;

    for (group_idx, group) in column.groups.iter().enumerate() {
        // Add spacing between groups (except before first group)
        if group_idx > 0 {
            lines.push(Line::raw(""));
        }

        // Render header if present
        if !group.header.is_empty() {
            lines.push(Line::raw(format!("{}{}", " ".repeat(margin), group.header)));
            lines.push(Line::raw(format!("{}{}", " ".repeat(margin), "═".repeat(group.header.len()))));
            lines.push(Line::raw(""));
        }

        // Render table header
        lines.push(render_table_header_with_margin(margin));
        // Separator line should exclude the margin since we add it separately
        lines.push(Line::raw(format!("{}{}", " ".repeat(margin), "─".repeat(STANDINGS_COLUMN_WIDTH - margin))));

        // Render teams
        for (idx_in_group, team) in group.teams.iter().enumerate() {
            let is_selected = state.team_selection_active
                && state.selected_column == col_idx
                && state.selected_team_index == team_idx;

            lines.push(render_team_row(team, is_selected, selection_fg, margin));

            // Draw playoff cutoff line after specified team index (for wildcard view)
            if let Some(cutoff_idx) = group.playoff_cutoff_after {
                if idx_in_group == cutoff_idx {
                    lines.push(Line::raw(format!("{}{}", " ".repeat(margin), "─".repeat(STANDINGS_COLUMN_WIDTH - margin))));
                }
            }

            team_idx += 1;
        }
    }

    lines
}

/// Render the table header (for single-column layouts)
fn render_table_header() -> Line<'static> {
    render_table_header_with_margin(CONTENT_LEFT_MARGIN)
}

/// Render the table header with custom margin
fn render_table_header_with_margin(margin: usize) -> Line<'static> {
    let header = format!(
        "{}{:<team_width$} {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}",
        " ".repeat(margin),
        "Team", "GP", "W", "L", "OT", "PTS",
        team_width = TEAM_NAME_COL_WIDTH,
        gp_width = GP_COL_WIDTH,
        w_width = W_COL_WIDTH,
        l_width = L_COL_WIDTH,
        ot_width = OT_COL_WIDTH,
        pts_width = PTS_COL_WIDTH
    );
    Line::raw(header)
}

/// Render a single team row
fn render_team_row(team: &nhl_api::Standing, is_selected: bool, selection_fg: Color, margin: usize) -> Line<'static> {
    let team_name = &team.team_common_name.default;

    // Format the full row
    let team_part = format!("{:<width$}", team_name, width = TEAM_NAME_COL_WIDTH);
    let stats_part = format!(
        " {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}",
        team.games_played(),
        team.wins,
        team.losses,
        team.ot_losses,
        team.points,
        gp_width = GP_COL_WIDTH,
        w_width = W_COL_WIDTH,
        l_width = L_COL_WIDTH,
        ot_width = OT_COL_WIDTH,
        pts_width = PTS_COL_WIDTH
    );

    let mut spans = vec![Span::raw(" ".repeat(margin))];

    if is_selected {
        let selection_style = Style::default().fg(selection_fg);
        spans.push(Span::styled(team_part, selection_style));
        spans.push(Span::raw(stats_part));
    } else {
        spans.push(Span::raw(team_part));
        spans.push(Span::raw(stats_part));
    }

    Line::from(spans)
}

/// Convert a Line to a plain string (for padding calculations)
fn line_to_string(line: &Line) -> String {
    line.spans.iter().map(|span| span.content.as_ref()).collect()
}

/// Auto-scroll to ensure the selected team is visible in the viewport
fn ensure_team_visible(state: &mut State, lines: &[Line]) {
    if let Some(layout) = &state.layout_cache {
        // Get the selected team
        let selected_team = match layout.get_team(state.selected_column, state.selected_team_index) {
            Some(team) => team,
            None => return,
        };

        // Find which line the selected team is on
        let selected_team_line = find_team_line_index(lines, &selected_team.team_common_name.default);

        if let Some(line_idx) = selected_team_line {
            let scroll_offset = state.scrollable.scroll_offset as usize;
            let viewport_height = state.scrollable.viewport_height as usize;
            let viewport_end = scroll_offset + viewport_height;

            // If selected line is above viewport, scroll up to show it
            if line_idx < scroll_offset {
                state.scrollable.scroll_offset = line_idx as u16;
            }
            // If selected line is at or below viewport end, scroll down to show it
            else if line_idx >= viewport_end {
                let new_offset = (line_idx + 1).saturating_sub(viewport_height);
                state.scrollable.scroll_offset = new_offset as u16;
            }
        }
    }
}

/// Find the line index of a team by name
fn find_team_line_index(lines: &[Line], team_name: &str) -> Option<usize> {
    for (idx, line) in lines.iter().enumerate() {
        let line_text = line_to_string(line);
        if line_text.contains(team_name) && line_text.chars().any(|c| c.is_numeric()) {
            return Some(idx);
        }
    }
    None
}
