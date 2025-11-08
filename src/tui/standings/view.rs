use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph},
    Frame,
    buffer::Buffer,
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
use crate::tui::widgets::{RenderableWidget, StandingsTable};

// Layout Constants
const CONTENT_LEFT_MARGIN: usize = 2;
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

        let tab_text = view.name().to_string();
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

    // Render using widgets
    render_layout_with_widgets(f, area, &layout, state, theme);
}

/// Render the standings layout using widgets
fn render_layout_with_widgets(
    f: &mut Frame,
    area: Rect,
    layout: &StandingsLayout,
    state: &mut State,
    display: &Arc<DisplayConfig>,
) {
    // Get buffer for direct widget rendering
    let buf = f.buffer_mut();

    let mut y_offset = area.y + 1; // Start with 1 line top margin

    match layout.view {
        GroupBy::League => {
            render_single_column_with_widgets(buf, area, &mut y_offset, layout, state, display);
        }
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard => {
            render_two_columns_with_widgets(buf, area, &mut y_offset, layout, state, display);
        }
    }

    // Calculate total content height for scrolling
    let content_height = (y_offset - area.y) as usize;
    state.scrollable.update_viewport_height(area.height);
    state.scrollable.update_content_height(content_height);

    // Auto-scroll to ensure selected team is visible
    if state.team_selection_active {
        ensure_team_visible_with_widgets(state, layout, display);
    }
}

/// Render a single-column layout (League view) using widgets
fn render_single_column_with_widgets(
    buf: &mut Buffer,
    area: Rect,
    y_offset: &mut u16,
    layout: &StandingsLayout,
    state: &State,
    display: &DisplayConfig,
) {
    let column = &layout.columns[0];
    let mut team_idx = 0;

    for group in &column.groups {
        // Create StandingsTable widget for this group
        let header = if group.header.is_empty() {
            None
        } else {
            Some(group.header.as_str())
        };

        let selected_index = if state.team_selection_active && state.selected_column == 0 {
            let start_idx = team_idx;
            let end_idx = start_idx + group.teams.len();
            if state.selected_team_index >= start_idx && state.selected_team_index < end_idx {
                Some(state.selected_team_index - start_idx)
            } else {
                None
            }
        } else {
            None
        };

        let widget = StandingsTable::new(
            &group.teams,
            header,
            group.playoff_cutoff_after,
            selected_index,
            CONTENT_LEFT_MARGIN as u16,
        );

        // Calculate widget height
        let widget_height = widget.preferred_height().unwrap_or(0);

        // Apply scroll offset
        let scroll_offset = state.scrollable.scroll_offset;
        if *y_offset >= area.y + scroll_offset && *y_offset < area.bottom() {
            let widget_area = Rect::new(
                area.x,
                *y_offset - scroll_offset,
                area.width,
                widget_height.min(area.bottom() - (*y_offset - scroll_offset)),
            );
            widget.render(widget_area, buf, display);
        }

        *y_offset += widget_height;
        team_idx += group.teams.len();
    }
}

/// Render a two-column layout (Conference/Division view) using widgets
fn render_two_columns_with_widgets(
    buf: &mut Buffer,
    area: Rect,
    y_offset: &mut u16,
    layout: &StandingsLayout,
    state: &State,
    display: &DisplayConfig,
) {
    // Build widgets for left and right columns
    let left_widgets = create_column_widgets(&layout.columns[0], 0, state, CONTENT_LEFT_MARGIN as u16);
    let right_widgets = if layout.columns.len() > 1 {
        create_column_widgets(&layout.columns[1], 1, state, CONTENT_LEFT_MARGIN as u16)
    } else {
        vec![]
    };

    // Calculate max height needed
    let left_height: u16 = left_widgets.iter().map(|w| w.preferred_height().unwrap_or(0)).sum();
    let right_height: u16 = right_widgets.iter().map(|w| w.preferred_height().unwrap_or(0)).sum();
    let max_height = left_height.max(right_height);

    // Render left column
    let scroll_offset = state.scrollable.scroll_offset;
    let mut left_y = *y_offset;
    for widget in &left_widgets {
        let widget_height = widget.preferred_height().unwrap_or(0);

        if left_y >= area.y + scroll_offset && left_y < area.bottom() {
            let widget_area = Rect::new(
                area.x,
                left_y - scroll_offset,
                STANDINGS_COLUMN_WIDTH as u16,
                widget_height.min(area.bottom() - (left_y - scroll_offset)),
            );
            widget.render(widget_area, buf, display);
        }

        left_y += widget_height;
    }

    // Render right column
    let mut right_y = *y_offset;
    for widget in &right_widgets {
        let widget_height = widget.preferred_height().unwrap_or(0);

        if right_y >= area.y + scroll_offset && right_y < area.bottom() {
            let widget_area = Rect::new(
                area.x + STANDINGS_COLUMN_WIDTH as u16 + COLUMN_SPACING as u16,
                right_y - scroll_offset,
                STANDINGS_COLUMN_WIDTH as u16,
                widget_height.min(area.bottom() - (right_y - scroll_offset)),
            );
            widget.render(widget_area, buf, display);
        }

        right_y += widget_height;
    }

    *y_offset += max_height;
}

/// Create StandingsTable widgets for a column
fn create_column_widgets<'a>(
    column: &'a super::layout::StandingsColumn,
    col_idx: usize,
    state: &State,
    margin: u16,
) -> Vec<StandingsTable<'a>> {
    let mut widgets = Vec::new();
    let mut team_idx = 0;

    for group in &column.groups {
        let header = if group.header.is_empty() {
            None
        } else {
            Some(group.header.as_str())
        };

        let selected_index = if state.team_selection_active && state.selected_column == col_idx {
            let start_idx = team_idx;
            let end_idx = start_idx + group.teams.len();
            if state.selected_team_index >= start_idx && state.selected_team_index < end_idx {
                Some(state.selected_team_index - start_idx)
            } else {
                None
            }
        } else {
            None
        };

        let widget = StandingsTable::new(
            &group.teams,
            header,
            group.playoff_cutoff_after,
            selected_index,
            margin,
        );

        widgets.push(widget);
        team_idx += group.teams.len();
    }

    widgets
}

/// Auto-scroll to ensure the selected team is visible in the viewport (widget-based)
fn ensure_team_visible_with_widgets(
    state: &mut State,
    layout: &StandingsLayout,
    _display: &DisplayConfig,
) {
    // Check if selected team exists
    if layout.get_team(state.selected_column, state.selected_team_index).is_none() {
        return;
    }

    // Calculate which widgets come before the selected team
    let column = &layout.columns[state.selected_column];
    let mut y_position = 1u16; // Top margin
    let mut team_idx = 0;
    let mut found = false;

    for group in &column.groups {
        let header = if group.header.is_empty() { None } else { Some(group.header.as_str()) };
        let widget = StandingsTable::new(
            &group.teams,
            header,
            group.playoff_cutoff_after,
            None,
            CONTENT_LEFT_MARGIN as u16,
        );
        let widget_height = widget.preferred_height().unwrap_or(0);

        // Check if selected team is in this group
        let start_idx = team_idx;
        let end_idx = start_idx + group.teams.len();

        if state.selected_team_index >= start_idx && state.selected_team_index < end_idx {
            // Selected team is in this group
            // Add height for header and table header and teams before selected
            let teams_before_selected = state.selected_team_index - start_idx;
            let lines_before_team = if group.header.is_empty() { 2 } else { 5 }; // header lines + table header + separator
            y_position += lines_before_team + teams_before_selected as u16;
            found = true;
            break;
        }

        y_position += widget_height;
        team_idx += group.teams.len();
    }

    if found {
        let scroll_offset = state.scrollable.scroll_offset as usize;
        let viewport_height = state.scrollable.viewport_height as usize;
        let viewport_end = scroll_offset + viewport_height;

        let line_idx = y_position as usize;

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

/// Render a division/group header with color
fn render_header_lines(header_text: &str, margin: usize, display: &Arc<DisplayConfig>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let header = format_header(header_text, true, display);
    for line in header.lines() {
        if !line.is_empty() {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(margin)),
                Span::styled(line.to_string(), Style::default().fg(display.division_header_fg))
            ]));
        } else {
            lines.push(Line::raw(""));
        }
    }
    lines
}

/// Render a single-line header with color (for subsections)
fn render_header_lines_single(header_text: &str, margin: usize, display: &Arc<DisplayConfig>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let header = format_header(header_text, false, display);
    for line in header.lines() {
        if !line.is_empty() {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(margin)),
                Span::styled(line.to_string(), Style::default().fg(display.division_header_fg))
            ]));
        } else {
            lines.push(Line::raw(""));
        }
    }
    lines
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
    use crate::tui::widgets::{RenderableWidget, PlayerStatsTable, GoalieStatsTable};

    // Try to get real club stats data
    let club_stats_data = selected_team_abbrev.as_ref()
        .and_then(|abbrev| club_stats_map.get(abbrev))
        .cloned();

    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.extend(render_header_lines(team_name, 2, display));

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

    // Determine which item is selected
    let (player_selection, goalie_selection) = if state.panel_selection_active {
        let idx = state.panel_selected_index;
        if idx < players.len() {
            (Some(idx), None)
        } else {
            (None, Some(idx - players.len()))
        }
    } else {
        (None, None)
    };

    // Create widgets
    let player_table = PlayerStatsTable::new(
        &players,
        Some("Player Statistics"),
        player_selection,
        0,
    );

    let goalie_table = GoalieStatsTable::new(
        &goalies,
        Some("Goaltender Statistics"),
        goalie_selection,
        0,
    );

    // Render widgets to buffer and convert to lines
    let buf_width = area.width.max(60);
    let player_height = player_table.preferred_height().unwrap_or(10);
    let goalie_height = goalie_table.preferred_height().unwrap_or(10);

    let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, buf_width, player_height + goalie_height));

    // Render player table
    player_table.render(Rect::new(0, 0, buf_width, player_height), &mut buf, display);

    // Render goalie table below player table
    goalie_table.render(Rect::new(0, player_height, buf_width, goalie_height), &mut buf, display);

    // Convert buffer to lines
    for y in 0..buf.area.height {
        let mut line_content = String::new();
        for x in 0..buf.area.width {
            let cell = buf.cell((x, y)).unwrap();
            line_content.push_str(cell.symbol());
        }
        lines.push(Line::raw(line_content));
    }

    state.panel_scrollable.update_viewport_height(area.height);
    state.panel_scrollable.update_content_height(lines.len());

    // Calculate selected item line number for scrolling
    let selected_item_line_number = if state.panel_selection_active {
        let idx = state.panel_selected_index;
        // Account for: blank line (1) + team header (3) + player table header (3 + 2 for table header/sep)
        let base_offset = 1 + 3 + 3 + 2; // Lines before first player row
        if idx < players.len() {
            // Selected item is a player
            Some(base_offset + idx)
        } else {
            // Selected item is a goalie
            // Add player count + blank lines (1) + goalie header (3 + 2)
            Some(base_offset + players.len() + 1 + 3 + 2 + (idx - players.len()))
        }
    } else {
        None
    };

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
    use crate::tui::widgets::{RenderableWidget, PlayerBioCard, CareerStatsTable};

    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.extend(render_header_lines(player_name, 2, display));

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

    // Filter to only NHL seasons
    let nhl_seasons: Vec<nhl_api::SeasonTotal> = if let Some(season_totals) = &player.season_totals {
        season_totals.iter()
            .filter(|s| s.league_abbrev == NHL_LEAGUE_ABBREV)
            .cloned()
            .collect()
    } else {
        vec![]
    };

    // Determine which season is selected (if any)
    let season_selection = if state.panel_selection_active {
        Some(state.panel_selected_index)
    } else {
        None
    };

    // Create widgets
    let bio_card = PlayerBioCard::new(player, Some("Player Information"), 0);
    let career_table = if !nhl_seasons.is_empty() {
        Some(CareerStatsTable::new(
            &nhl_seasons,
            Some("NHL Career Statistics"),
            season_selection,
            0,
        ))
    } else {
        None
    };

    // Render widgets to buffer and convert to lines
    let buf_width = area.width.max(60);
    let bio_height = bio_card.preferred_height().unwrap_or(10);
    let career_height = career_table.as_ref().and_then(|t| t.preferred_height()).unwrap_or(0);

    let total_height = bio_height + career_height;
    let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, buf_width, total_height));

    // Render bio card
    bio_card.render(Rect::new(0, 0, buf_width, bio_height), &mut buf, display);

    // Render career table if present
    if let Some(table) = &career_table {
        table.render(Rect::new(0, bio_height, buf_width, career_height), &mut buf, display);
    }

    // Convert buffer to lines
    for y in 0..buf.area.height {
        let mut line_content = String::new();
        for x in 0..buf.area.width {
            let cell = buf.cell((x, y)).unwrap();
            line_content.push_str(cell.symbol());
        }
        lines.push(Line::raw(line_content));
    }

    state.panel_scrollable.update_viewport_height(area.height);
    state.panel_scrollable.update_content_height(lines.len());

    // Calculate selected item line number for scrolling
    let selected_item_line_number = if state.panel_selection_active && !nhl_seasons.is_empty() {
        let idx = state.panel_selected_index;
        // Account for: blank (1) + player header (3) + bio card height + career header (3) + table header/sep (2)
        let base_offset = 1 + 3 + bio_height as usize + 3 + 2;
        Some(base_offset + idx)
    } else {
        None
    };

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
