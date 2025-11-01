use ratatui::{
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
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
    selection_fg: Color,
) {
    let standings_text = crate::commands::standings::format_standings_by_group(
        standings_data,
        state.view,
        western_first,
    );

    if state.team_selection_active {
        // Apply selection styling to the selected team
        let styled_text = apply_team_selection_styling(
            &standings_text,
            standings_data,
            state.view,
            state.selected_column,
            state.selected_team_index,
            western_first,
            selection_fg,
        );

        // Update scrollable dimensions
        state.scrollable.update_viewport_height(area.height);
        state.scrollable.update_content_height(styled_text.lines.len());

        // Ensure selected team is visible by auto-scrolling (use area.height directly)
        ensure_team_visible(state, standings_data, &standings_text, area.height, western_first);

        let paragraph = Paragraph::new(styled_text)
            .scroll((state.scrollable.scroll_offset, 0));
        f.render_widget(paragraph, area);
    } else {
        // No selection, render normally
        let content = standings_text
            .lines()
            .map(|line| format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), line))
            .collect::<Vec<_>>()
            .join("\n");

        state.scrollable.render_paragraph(f, area, content, None);
    }
}

/// Auto-scroll to ensure the selected team is visible in the viewport
fn ensure_team_visible(
    state: &mut State,
    standings_data: &[nhl_api::Standing],
    formatted_text: &str,
    viewport_height: u16,
    western_first: bool,
) {
    // Get the selected team
    let teams_in_column = get_teams_in_column(standings_data, state.view, state.selected_column, western_first);
    let selected_team = match teams_in_column.get(state.selected_team_index) {
        Some(team) => team,
        None => return,
    };

    // Find which line the selected team is on
    let selected_team_line = find_team_line_index(formatted_text, &selected_team.team_common_name.default);

    if let Some(line_idx) = selected_team_line {
        let scroll_offset = state.scrollable.scroll_offset as usize;
        let viewport_end = scroll_offset + viewport_height as usize;

        // If selected line is above viewport, scroll up to show it
        if line_idx < scroll_offset {
            state.scrollable.scroll_offset = line_idx as u16;
        }
        // If selected line is at or below viewport end, scroll down to show it
        // (line at viewport_end would be the first line not visible)
        else if line_idx >= viewport_end {
            // Position so the selected line is at the bottom of the viewport
            let new_offset = (line_idx + 1).saturating_sub(viewport_height as usize);
            state.scrollable.scroll_offset = new_offset as u16;
        }
    }
}

/// Find the line index of a team in the formatted standings text
fn find_team_line_index(formatted_text: &str, team_name: &str) -> Option<usize> {
    for (idx, line) in formatted_text.lines().enumerate() {
        if line.contains(team_name) && line.contains(char::is_numeric) {
            return Some(idx);
        }
    }
    None
}

/// Get teams from a specific column based on view
/// Returns teams in the same order as they appear in the visual display
fn get_teams_in_column(
    standings: &[nhl_api::Standing],
    view: GroupBy,
    column: usize,
    western_first: bool,
) -> Vec<nhl_api::Standing> {
    let mut sorted = standings.to_vec();
    sorted.sort_by(|a, b| b.points.cmp(&a.points));

    match view {
        GroupBy::League => {
            // Single column, all teams sorted by points
            sorted
        }
        GroupBy::Conference => {
            // Column 0 = Eastern (or Western if western_first), Column 1 = Western (or Eastern)
            // Teams sorted by points within conference
            let (first_conf, second_conf) = if western_first {
                ("Western", "Eastern")
            } else {
                ("Eastern", "Western")
            };

            let conf_name = if column == 0 { first_conf } else { second_conf };
            sorted.into_iter()
                .filter(|s| s.conference_name.as_deref() == Some(conf_name))
                .collect()
        }
        GroupBy::Division => {
            // Column 0 = Eastern divisions (or Western if western_first), Column 1 = Western (or Eastern)
            // Teams grouped by division, then sorted by points WITHIN each division
            let (first_divs, second_divs) = if western_first {
                (vec!["Central", "Pacific"], vec!["Atlantic", "Metropolitan"])
            } else {
                (vec!["Atlantic", "Metropolitan"], vec!["Central", "Pacific"])
            };

            let divs = if column == 0 { &first_divs } else { &second_divs };

            // Group teams by division and sort each division by points
            let mut result = Vec::new();
            for div_name in divs {
                let mut div_teams: Vec<_> = sorted.iter()
                    .filter(|s| s.division_name == *div_name)
                    .cloned()
                    .collect();
                div_teams.sort_by(|a, b| b.points.cmp(&a.points));
                result.extend(div_teams);
            }
            result
        }
    }
}

/// Apply selection styling to team lines
fn apply_team_selection_styling(
    formatted_text: &str,
    standings_data: &[nhl_api::Standing],
    view: GroupBy,
    selected_column: usize,
    selected_index: usize,
    western_first: bool,
    selection_fg: Color,
) -> Text<'static> {
    let lines: Vec<&str> = formatted_text.lines().collect();
    let mut styled_lines: Vec<Line> = Vec::new();

    // Get teams in the selected column
    let teams_in_column = get_teams_in_column(standings_data, view, selected_column, western_first);

    // Get the selected team name
    let selected_team_name = teams_in_column
        .get(selected_index)
        .map(|s| s.team_common_name.default.as_str());

    let selection_style = Style::default().fg(selection_fg);

    // Team name column width (from standings.rs TEAM_NAME_COL_WIDTH)
    const TEAM_NAME_WIDTH: usize = 25;
    // Column width for 2-column layouts (from standings.rs STANDINGS_COLUMN_WIDTH)
    const COLUMN_WIDTH: usize = 46;
    const COLUMN_SPACING: usize = 4;

    // Determine if we have 2 columns based on view
    let has_two_columns = matches!(view, GroupBy::Conference | GroupBy::Division);

    for line in lines {
        // Check if this line contains the selected team name
        let is_selected_team = if let Some(team_name) = selected_team_name {
            line.contains(team_name) && line.contains(char::is_numeric)
        } else {
            false
        };

        if is_selected_team && has_two_columns {
            // Two-column layout - need to determine which column contains the team
            let margin = " ".repeat(CONTENT_LEFT_MARGIN);

            // Check if team is in left column (first COLUMN_WIDTH chars) or right column
            let left_part = if line.len() >= COLUMN_WIDTH {
                &line[..COLUMN_WIDTH]
            } else {
                line
            };
            let is_in_left_column = left_part.contains(selected_team_name.unwrap());

            if (selected_column == 0 && is_in_left_column) || (selected_column == 1 && !is_in_left_column) {
                // Highlight the correct column
                if selected_column == 0 {
                    // Highlight left column
                    let left_team_part = if left_part.len() > TEAM_NAME_WIDTH {
                        &left_part[..TEAM_NAME_WIDTH]
                    } else {
                        left_part
                    };
                    let rest = &line[left_part.len().min(line.len())..];

                    let spans = vec![
                        Span::raw(margin),
                        Span::styled(left_team_part.to_string(), selection_style),
                        Span::raw(left_part[TEAM_NAME_WIDTH.min(left_part.len())..].to_string()),
                        Span::raw(rest.to_string()),
                    ];
                    styled_lines.push(Line::from(spans));
                } else {
                    // Highlight right column
                    let right_start = COLUMN_WIDTH + COLUMN_SPACING;
                    if line.len() > right_start {
                        let left_full = &line[..right_start];
                        let right_part = &line[right_start..];
                        let right_team_part = if right_part.len() > TEAM_NAME_WIDTH {
                            &right_part[..TEAM_NAME_WIDTH]
                        } else {
                            right_part
                        };
                        let right_rest = &right_part[TEAM_NAME_WIDTH.min(right_part.len())..];

                        let spans = vec![
                            Span::raw(margin),
                            Span::raw(left_full.to_string()),
                            Span::styled(right_team_part.to_string(), selection_style),
                            Span::raw(right_rest.to_string()),
                        ];
                        styled_lines.push(Line::from(spans));
                    } else {
                        // Right column not present on this line, render normally
                        let padded_line = format!("{}{}", margin, line);
                        styled_lines.push(Line::raw(padded_line));
                    }
                }
            } else {
                // Team name found but in wrong column, render normally
                let padded_line = format!("{}{}", margin, line);
                styled_lines.push(Line::raw(padded_line));
            }
        } else if is_selected_team && !has_two_columns {
            // Single column layout (League view)
            let margin = " ".repeat(CONTENT_LEFT_MARGIN);
            let (team_part, stats_part) = if line.len() > TEAM_NAME_WIDTH {
                (&line[..TEAM_NAME_WIDTH], &line[TEAM_NAME_WIDTH..])
            } else {
                (line, "")
            };

            let spans = vec![
                Span::raw(margin),
                Span::styled(team_part.to_string(), selection_style),
                Span::raw(stats_part.to_string()),
            ];
            styled_lines.push(Line::from(spans));
        } else {
            // Normal line with margin
            let padded_line = format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), line);
            styled_lines.push(Line::raw(padded_line));
        }
    }

    Text::from(styled_lines)
}
