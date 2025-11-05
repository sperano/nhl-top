use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph},
    Frame,
};
use std::sync::Arc;
use std::collections::HashMap;
use crate::config::DisplayConfig;
use crate::commands::standings::GroupBy;
use crate::formatting::format_header;
use crate::NHL_LEAGUE_ABBREV;
use super::{State, layout::StandingsLayout};
use super::panel::{StandingsPanel, PlayerStat, GoalieStat};
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::common::styling::{base_tab_style, selection_style};
use crate::tui::common::breadcrumb::render_breadcrumb_simple;

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

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, theme: &Arc<DisplayConfig>) {
    let base_style = base_tab_style(state.subtab_focused);

    if let Some(nav_ctx) = &state.navigation {
        if !nav_ctx.is_at_root() {
            let trail = nav_ctx.stack.breadcrumb_trail();
            render_breadcrumb_simple(f, area, &trail, theme, base_style);
            return;
        }
    }

    // Otherwise show view selection tabs
    let views = GroupBy::all();
    let standings_view = state.view;
    let focused = state.subtab_focused;

    // Build subtab line with separators
    let separator = format!(" {} ", theme.box_chars.vertical);
    let mut subtab_spans = Vec::new();

    for (i, view) in views.iter().enumerate() {
        if i > 0 {
            subtab_spans.push(Span::styled(&separator, base_style));
        }

        let tab_text = format!("{}", view.name());
        let style = selection_style(
            base_style,
            *view == standings_view,
            focused,
            theme.selection_fg,
            theme.unfocused_selection_fg(),
        );
        subtab_spans.push(Span::styled(tab_text, style));
    }
    let subtab_line = Line::from(subtab_spans);

    // Build separator line with connectors
    let tab_names = views.iter().map(|view| view.name().to_string());
    let separator_line = build_tab_separator_line(
        tab_names,
        area.width as usize,
        base_style,
        &theme.box_chars,
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
    theme: &Arc<DisplayConfig>,
    club_stats: &Arc<HashMap<String, nhl_api::ClubStats>>,
    selected_team_abbrev: &Option<String>,
    player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
) {
    let panel_to_render = state.navigation.as_ref()
        .and_then(|nav_ctx| nav_ctx.stack.current())
        .cloned();

    if let Some(panel) = panel_to_render {
        render_panel(f, area, state, &panel, theme, club_stats, selected_team_abbrev, player_info);
        return;
    }

    // Build layout if standings data is available
    let layout = match &state.layout_cache {
        Some(layout) => layout.clone(),
        None => return, // No data to render
    };

    // Render the layout
    let lines = render_layout(&layout, state, theme);

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
fn render_layout(layout: &StandingsLayout, state: &State, display: &Arc<DisplayConfig>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add initial blank line
    lines.push(Line::raw(""));

    match layout.view {
        GroupBy::League => render_single_column(layout, state, display, &mut lines),
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard => render_two_columns(layout, state, display, &mut lines),
    }

    lines
}

/// Render a single-column layout (League view)
fn render_single_column(layout: &StandingsLayout, state: &State, display: &Arc<DisplayConfig>, lines: &mut Vec<Line<'static>>) {
    let column = &layout.columns[0];

    for group in &column.groups {
        // Render header if present
        if !group.header.is_empty() {
            let header = format_header(&group.header, true, display);
            for line in header.lines() {
                if !line.is_empty() {
                    lines.push(Line::raw(format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), line)));
                } else {
                    lines.push(Line::raw(""));
                }
            }
        }

        // Render table header
        lines.push(render_table_header());
        // Separator line should exclude the margin since we add it separately
        lines.push(Line::raw(format!("{}{}", " ".repeat(CONTENT_LEFT_MARGIN), display.box_chars.horizontal.repeat(STANDINGS_COLUMN_WIDTH - CONTENT_LEFT_MARGIN))));

        // Render teams
        let mut team_idx = 0;
        for team in &group.teams {
            let is_selected = state.team_selection_active
                && state.selected_column == 0
                && state.selected_team_index == team_idx;

            lines.push(render_team_row(team, is_selected, display.selection_fg, CONTENT_LEFT_MARGIN));
            team_idx += 1;
        }
    }
}

/// Render a two-column layout (Conference/Division view)
fn render_two_columns(layout: &StandingsLayout, state: &State, display: &Arc<DisplayConfig>, lines: &mut Vec<Line<'static>>) {
    let left_lines = render_column(&layout.columns[0], state, display, 0);
    let right_lines = if layout.columns.len() > 1 {
        render_column(&layout.columns[1], state, display, 1)
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
fn render_column(column: &super::layout::StandingsColumn, state: &State, display: &Arc<DisplayConfig>, col_idx: usize) -> Vec<Line<'static>> {
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
            let header = format_header(&group.header, true, display);
            for line in header.lines() {
                if !line.is_empty() {
                    lines.push(Line::raw(format!("{}{}", " ".repeat(margin), line)));
                } else {
                    lines.push(Line::raw(""));
                }
            }
        }

        // Render table header
        lines.push(render_table_header_with_margin(margin));
        // Separator line should exclude the margin since we add it separately
        lines.push(Line::raw(format!("{}{}", " ".repeat(margin), display.box_chars.horizontal.repeat(STANDINGS_COLUMN_WIDTH - margin))));

        // Render teams
        for (idx_in_group, team) in group.teams.iter().enumerate() {
            let is_selected = state.team_selection_active
                && state.selected_column == col_idx
                && state.selected_team_index == team_idx;

            lines.push(render_team_row(team, is_selected, display.selection_fg, margin));

            // Draw playoff cutoff line after specified team index (for wildcard view)
            if let Some(cutoff_idx) = group.playoff_cutoff_after {
                if idx_in_group == cutoff_idx {
                    lines.push(Line::raw(format!("{}{}", " ".repeat(margin), display.box_chars.horizontal.repeat(STANDINGS_COLUMN_WIDTH - margin))));
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

fn render_panel(f: &mut Frame, area: Rect, state: &mut State, panel: &StandingsPanel, display: &Arc<DisplayConfig>, club_stats: &Arc<HashMap<String, nhl_api::ClubStats>>, selected_team_abbrev: &Option<String>, player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>) {
    match panel {
        StandingsPanel::TeamDetail { team_name, .. } => {
            render_team_panel(f, area, state, team_name, display, club_stats, selected_team_abbrev);
        }
        StandingsPanel::PlayerDetail { player_id, player_name, .. } => {
            render_player_panel(f, area, state, *player_id, player_name, display, player_info);
        }
    }
}

fn render_team_panel(f: &mut Frame, area: Rect, state: &mut State, team_name: &str, display: &Arc<DisplayConfig>, club_stats_map: &Arc<HashMap<String, nhl_api::ClubStats>>, selected_team_abbrev: &Option<String>) {
    // Try to get real club stats data
    let club_stats_data = selected_team_abbrev.as_ref()
        .and_then(|abbrev| club_stats_map.get(abbrev))
        .cloned();

    let mut lines = Vec::new();
    let mut item_index = 0;
    let mut selected_item_line_number: Option<usize> = None;

    lines.push(Line::raw(""));
    let header = format_header(team_name, true, display);
    for line in header.lines() {
        if !line.is_empty() {
            lines.push(Line::raw(format!("  {}", line)));
        } else {
            lines.push(Line::raw(""));
        }
    }

    // Show loading message if we don't have data yet
    if club_stats_data.is_none() {
        lines.push(Line::raw("  Loading team statistics..."));
        lines.push(Line::raw(""));

        state.panel_scrollable.update_viewport_height(area.height);
        state.panel_scrollable.update_content_height(lines.len());

        let paragraph = Paragraph::new(lines)
            .scroll((state.panel_scrollable.scroll_offset, 0));
        f.render_widget(paragraph, area);
        return;
    }

    let stats = club_stats_data.unwrap();

    // Convert API stats to panel format
    let mut players: Vec<PlayerStat> = stats.skaters.iter().map(|s| PlayerStat {
        name: format!("{} {}", s.first_name.default, s.last_name.default),
        gp: s.games_played,
        g: s.goals,
        a: s.assists,
        pts: s.points,
    }).collect();

    // Sort by points (highest to lowest)
    players.sort_by(|a, b| b.pts.cmp(&a.pts));

    let mut goalies: Vec<GoalieStat> = stats.goalies.iter().map(|g| GoalieStat {
        name: format!("{} {}", g.first_name.default, g.last_name.default),
        gp: g.games_played,
        gaa: format!("{:.2}", g.goals_against_average),
        sv_pct: format!("{:.3}", g.save_percentage),
        so: g.shutouts,
    }).collect();

    // Sort by games played (highest to lowest)
    goalies.sort_by(|a, b| b.gp.cmp(&a.gp));

    let player_header = format_header("Player Statistics", true, display);
    for line in player_header.lines() {
        if !line.is_empty() {
            lines.push(Line::raw(format!("  {}", line)));
        } else {
            lines.push(Line::raw(""));
        }
    }

    lines.push(Line::raw(format!(
        "  {:<25} {:>4} {:>4} {:>4} {:>5}",
        "Player", "GP", "G", "A", "PTS"
    )));
    lines.push(Line::raw(format!(
        "  {}",
        display.box_chars.horizontal.repeat(46)
    )));

    for player in &players {
        let is_selected = state.panel_selection_active && state.panel_selected_index == item_index;

        if is_selected {
            selected_item_line_number = Some(lines.len());
        }

        let line_text = format!(
            "  {:<25} {:>4} {:>4} {:>4} {:>5}",
            player.name, player.gp, player.g, player.a, player.pts
        );

        if is_selected {
            lines.push(Line::from(vec![
                Span::styled(line_text, Style::default().fg(display.selection_fg))
            ]));
        } else {
            lines.push(Line::raw(line_text));
        }

        item_index += 1;
    }

    lines.push(Line::raw(""));
    lines.push(Line::raw(""));

    let goalie_header = format_header("Goaltender Statistics", true, display);
    for line in goalie_header.lines() {
        if !line.is_empty() {
            lines.push(Line::raw(format!("  {}", line)));
        } else {
            lines.push(Line::raw(""));
        }
    }

    lines.push(Line::raw(format!(
        "  {:<25} {:>4} {:>6} {:>6} {:>6}",
        "Goaltender", "GP", "GAA", "SV%", "SO"
    )));
    lines.push(Line::raw(format!(
        "  {}",
        display.box_chars.horizontal.repeat(50)
    )));

    for goalie in &goalies {
        let is_selected = state.panel_selection_active && state.panel_selected_index == item_index;

        if is_selected {
            selected_item_line_number = Some(lines.len());
        }

        let line_text = format!(
            "  {:<25} {:>4} {:>6} {:>6} {:>6}",
            goalie.name, goalie.gp, goalie.gaa, goalie.sv_pct, goalie.so
        );

        if is_selected {
            lines.push(Line::from(vec![
                Span::styled(line_text, Style::default().fg(display.selection_fg))
            ]));
        } else {
            lines.push(Line::raw(line_text));
        }

        item_index += 1;
    }

    lines.push(Line::raw(""));

    state.panel_scrollable.update_viewport_height(area.height);
    state.panel_scrollable.update_content_height(lines.len());

    if state.panel_selection_active {
        if let Some(line_idx) = selected_item_line_number {
            let scroll_offset = state.panel_scrollable.scroll_offset as usize;
            let viewport_height = state.panel_scrollable.viewport_height as usize;
            let viewport_end = scroll_offset + viewport_height;

            if line_idx < scroll_offset {
                state.panel_scrollable.scroll_offset = line_idx as u16;
            } else if line_idx >= viewport_end {
                let new_offset = (line_idx + 1).saturating_sub(viewport_height);
                state.panel_scrollable.scroll_offset = new_offset as u16;
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .scroll((state.panel_scrollable.scroll_offset, 0));
    f.render_widget(paragraph, area);
}

fn render_player_panel(f: &mut Frame, area: Rect, state: &mut State, player_id: i64, player_name: &str, display: &Arc<DisplayConfig>, player_info_map: &Arc<HashMap<i64, nhl_api::PlayerLanding>>) {
    let mut lines = Vec::new();
    let mut item_index = 0;
    let mut selected_item_line_number: Option<usize> = None;

    lines.push(Line::raw(""));
    let header = format_header(player_name, true, display);
    for line in header.lines() {
        if !line.is_empty() {
            lines.push(Line::raw(format!("  {}", line)));
        } else {
            lines.push(Line::raw(""));
        }
    }

    // Get real player data
    let player_data = player_info_map.get(&player_id);

    if player_data.is_none() {
        lines.push(Line::raw("  Loading player information..."));
        lines.push(Line::raw(""));

        state.panel_scrollable.update_viewport_height(area.height);
        state.panel_scrollable.update_content_height(lines.len());

        let paragraph = Paragraph::new(lines)
            .scroll((state.panel_scrollable.scroll_offset, 0));
        f.render_widget(paragraph, area);
        return;
    }

    let player = player_data.unwrap();

    // Player Information
    let player_info_header = format_header("Player Information", false, display);
    for line in player_info_header.lines() {
        if !line.is_empty() {
            lines.push(Line::raw(format!("  {}", line)));
        } else {
            lines.push(Line::raw(""));
        }
    }
    lines.push(Line::raw(format!("  Position:      {}", player.position)));
    if let Some(num) = player.sweater_number {
        lines.push(Line::raw(format!("  Number:        #{}", num)));
    }

    let height_ft = player.height_in_inches / 12;
    let height_in = player.height_in_inches % 12;
    lines.push(Line::raw(format!("  Height:        {}'{}\"", height_ft, height_in)));
    lines.push(Line::raw(format!("  Weight:        {} lbs", player.weight_in_pounds)));

    let mut birthplace = String::new();
    if let Some(city) = &player.birth_city {
        birthplace.push_str(&city.default);
    }
    if let Some(state_prov) = &player.birth_state_province {
        if !birthplace.is_empty() {
            birthplace.push_str(", ");
        }
        birthplace.push_str(&state_prov.default);
    }
    if let Some(country) = &player.birth_country {
        if !birthplace.is_empty() {
            birthplace.push_str(", ");
        }
        birthplace.push_str(country);
    }
    if !birthplace.is_empty() {
        lines.push(Line::raw(format!("  Birthplace:    {}", birthplace)));
    }

    lines.push(Line::raw(""));
    lines.push(Line::raw(""));

    // Career Statistics (NHL only)
    if let Some(season_totals) = &player.season_totals {
        // Filter to only NHL games
        let nhl_seasons: Vec<_> = season_totals.iter()
            .filter(|s| s.league_abbrev == NHL_LEAGUE_ABBREV)
            .collect();

        if !nhl_seasons.is_empty() {
            let career_header = format_header("NHL Career Statistics", true, display);
            for line in career_header.lines() {
                if !line.is_empty() {
                    lines.push(Line::raw(format!("  {}", line)));
                } else {
                    lines.push(Line::raw(""));
                }
            }

            lines.push(Line::raw(format!(
                "  {:<10} {:<20} {:>4} {:>4} {:>4} {:>5}",
                "Season", "Team", "GP", "G", "A", "PTS"
            )));
            lines.push(Line::raw(format!(
                "  {}",
                display.box_chars.horizontal.repeat(52)
            )));

            for season in nhl_seasons.iter().rev() {
                let is_selected = state.panel_selection_active && state.panel_selected_index == item_index;

                if is_selected {
                    selected_item_line_number = Some(lines.len());
                }

                let season_str = format!("{}-{}", season.season / 10000, season.season % 10000);
                let team_name = &season.team_name.default;

                let line_text = format!(
                    "  {:<10} {:<20} {:>4} {:>4} {:>4} {:>5}",
                    season_str,
                    team_name,
                    season.games_played,
                    season.goals.unwrap_or(0),
                    season.assists.unwrap_or(0),
                    season.points.unwrap_or(0)
                );

                if is_selected {
                    lines.push(Line::from(vec![
                        Span::styled(line_text, Style::default().fg(display.selection_fg))
                    ]));
                } else {
                    lines.push(Line::raw(line_text));
                }

                item_index += 1;
            }
        }
    }

    lines.push(Line::raw(""));

    state.panel_scrollable.update_viewport_height(area.height);
    state.panel_scrollable.update_content_height(lines.len());

    if state.panel_selection_active {
        if let Some(line_idx) = selected_item_line_number {
            let scroll_offset = state.panel_scrollable.scroll_offset as usize;
            let viewport_height = state.panel_scrollable.viewport_height as usize;
            let viewport_end = scroll_offset + viewport_height;

            if line_idx < scroll_offset {
                state.panel_scrollable.scroll_offset = line_idx as u16;
            } else if line_idx >= viewport_end {
                let new_offset = (line_idx + 1).saturating_sub(viewport_height);
                state.panel_scrollable.scroll_offset = new_offset as u16;
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .scroll((state.panel_scrollable.scroll_offset, 0));
    f.render_widget(paragraph, area);
}
