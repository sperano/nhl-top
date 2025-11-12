use super::State;
use super::layout::StandingsLayout;
use super::panel::StandingsPanel;
use super::state::{PanelState, TeamDetailState, PlayerDetailState};
use crate::tui::widgets::{
    Container, RenderableWidget, StandingsTable,
    TeamDetail, PlayerDetail, render_scrollable_widget,
};
use crate::commands::standings::GroupBy;
use crate::tui::common::subtab::render_subtabs_with_breadcrumb;
use crate::tui::navigation::Panel;
use ratatui::{layout::Rect, widgets::{Block, Borders, Paragraph}, Frame, style::Style};
use std::sync::Arc;
use std::collections::HashMap;
use crate::config::DisplayConfig;

// Layout Constants
const CONTENT_LEFT_MARGIN: usize = 2;
const STANDINGS_COLUMN_WIDTH: usize = 48;
const COLUMN_SPACING: usize = 4;

fn build_container() -> Container {
    Container::new()
}

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, theme: &Arc<DisplayConfig>) {
    use crate::tui::context::BreadcrumbProvider;

    let focused = state.subtab_focused;

    // Get view labels
    let views = GroupBy::all();
    let tab_labels: Vec<String> = views.iter().map(|v| v.name().to_string()).collect();

    // Find selected index
    let selected_index = views.iter().position(|v| *v == state.view).unwrap_or(0);

    // Get breadcrumb items if focused
    // Only pass them if we have items; visibility will be determined by render function
    let breadcrumb_items = if state.subtab_focused {
        Some(state.get_breadcrumb_items())
    } else {
        None
    };

    // Use shared rendering function
    // Skip the first BREADCRUMB_MIN_DEPTH items (e.g., skip "Standings" and "Division")
    render_subtabs_with_breadcrumb(
        f,
        area,
        tab_labels,
        selected_index,
        focused,
        breadcrumb_items,
        crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH,
        theme,
    );
}

/// Render a panel view (team details, player details, etc.)
fn render_panel(
    f: &mut Frame,
    area: Rect,
    panel: &StandingsPanel,
    state: &mut State,
    club_stats: &Arc<HashMap<String, nhl_api::ClubStats>>,
    player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
    theme: &Arc<DisplayConfig>,
) {
    match panel {
        StandingsPanel::TeamDetail {
            team_name,
            team_abbrev,
            wins,
            losses,
            ot_losses,
            points,
            division_name,
            conference_name,
        } => {
            // Get panel state from cache
            let cache_key = panel.cache_key();
            let mut panel_state = state
                .navigation
                .data
                .get(&cache_key)
                .and_then(|s| match s {
                    PanelState::TeamDetail(tds) => Some(tds.clone()),
                    _ => None,
                })
                .unwrap_or_else(TeamDetailState::new);

            let conference_str = conference_name.as_deref().unwrap_or("Unknown");

            // Create widget
            let selection = if panel_state.selection_active {
                Some(panel_state.selected_player_index)
            } else {
                None
            };

            let widget = TeamDetail::new(
                team_name,
                team_abbrev,
                conference_str,
                division_name,
                *wins,
                *losses,
                *ot_losses,
                *points,
                club_stats,
            )
            .with_selection(selection)
            .with_instructions(true);

            // Render with scrolling
            render_scrollable_widget(
                &widget,
                f,
                area,
                &mut panel_state.scrollable,
                theme,
                true, // blank line at top (after breadcrumb)
            );

            // Save updated state back to cache
            state.navigation.data.insert(cache_key, PanelState::TeamDetail(panel_state));
        }
        StandingsPanel::PlayerDetail { player_id, player_name, .. } => {
            // Get panel state from cache
            let cache_key = panel.cache_key();
            let mut panel_state = state
                .navigation
                .data
                .get(&cache_key)
                .and_then(|s| match s {
                    PanelState::PlayerDetail(pds) => Some(pds.clone()),
                    _ => None,
                })
                .unwrap_or_else(PlayerDetailState::new);

            if let Some(player) = player_info.get(player_id) {
                // Determine which season is selected (if any)
                let season_selection = if panel_state.selection_active {
                    Some(panel_state.selected_season_index)
                } else {
                    None
                };

                // Create widget
                let widget = PlayerDetail::new(player, player_name)
                    .with_selection(season_selection)
                    .with_instructions(true);

                // Render with scrolling
                render_scrollable_widget(
                    &widget,
                    f,
                    area,
                    &mut panel_state.scrollable,
                    theme,
                    true, // blank line at top (after breadcrumb)
                );
            } else {
                // Loading state - render simple message
                let buf = f.buffer_mut();
                let y = area.y + 1; // blank line after breadcrumb
                buf.set_string(area.x, y, "", Style::default());
                buf.set_string(area.x, y + 1, "  Loading player information...", Style::default());
                buf.set_string(area.x, y + 2, "", Style::default());
            }

            // Save updated state back to cache
            state.navigation.data.insert(cache_key, PanelState::PlayerDetail(panel_state));
        }
    }
}

pub fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &mut State,
    theme: &Arc<DisplayConfig>,
    standings: &[nhl_api::Standing],
    western_first: bool,
    club_stats: &Arc<HashMap<String, nhl_api::ClubStats>>,
    _selected_team_abbrev: &Option<String>,
    _player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
) {
    if state.container.is_none() {
        state.container = Some(build_container());
    }

    // Check if we're in a panel view (navigation stack not at root)
    if let Some(panel) = state.navigation.stack.current().cloned() {
        render_panel(f, area, &panel, state, club_stats, _player_info, theme);
        return;
    }

    // Build the layout from standings data
    if standings.is_empty() {
        let paragraph = Paragraph::new("Loading standings...")
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
        return;
    }

    let layout = StandingsLayout::build(standings, state.view, western_first);

    // Auto-scroll to keep selected team visible
    if state.team_selection_active {
        ensure_team_visible(state, &layout, theme);
    }

    render_standings_layout(f, area, &layout, state, theme);
}

// Helper function to render standings when we have the data
pub fn render_standings_layout(
    f: &mut Frame,
    area: Rect,
    layout: &StandingsLayout,
    state: &mut State,
    theme: &Arc<DisplayConfig>,
) {
    let buf = f.buffer_mut();
    let mut y_offset = area.y + 1; // Start with 1 line top margin

    match layout.view {
        GroupBy::League => {
            render_single_column(buf, area, &mut y_offset, layout, state, theme);
        }
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard => {
            render_two_columns(buf, area, &mut y_offset, layout, state, theme);
        }
    }

    // Update scrollable with content dimensions
    let content_height = (y_offset - area.y) as usize;
    state.scrollable.update_viewport_height(area.height);
    state.scrollable.update_content_height(content_height);
}

/// Render a single-column layout (League view)
fn render_single_column(
    buf: &mut ratatui::buffer::Buffer,
    area: Rect,
    y_offset: &mut u16,
    layout: &StandingsLayout,
    state: &State,
    display: &DisplayConfig,
) {
    let column = &layout.columns[0];
    let mut team_idx = 0;
    let scroll_offset = state.scrollable.scroll_offset;

    for group in &column.groups {
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

        let widget_height = widget.preferred_height().unwrap_or(0);

        // Apply scroll offset
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

/// Render a two-column layout (Conference/Division view)
fn render_two_columns(
    buf: &mut ratatui::buffer::Buffer,
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

    let scroll_offset = state.scrollable.scroll_offset;

    // Render left column
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

/// Auto-scroll to ensure the selected team is visible in the viewport
fn ensure_team_visible(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::standings::GroupBy;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_subtabs_without_breadcrumb() {
        let mut state = State::new();
        state.view = GroupBy::Division;
        state.subtab_focused = false; // Not focused = no breadcrumb

        let theme = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 80);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2); // 2 lines (no breadcrumb)
            render_subtabs(f, area, &state, &theme);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        use crate::tui::widgets::testing::assert_buffer;
        assert_buffer(buffer, &[
            "Wildcard │ Division │ Conference │ League                                       ",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────",
        ], 80);
    }

    #[test]
    fn test_render_subtabs_with_breadcrumb() {
        use crate::tui::standings::panel::StandingsPanel;
        use crate::tui::navigation::Panel;

        let mut state = State::new();
        state.view = GroupBy::Conference;
        state.subtab_focused = true; // Focused = show breadcrumb

        // Push a panel onto the navigation stack to simulate being in TeamDetail
        // This makes breadcrumb depth > 2 (Standings > Conference > Team Name)
        let team_panel = StandingsPanel::TeamDetail {
            team_name: "Toronto Maple Leafs".to_string(),
            team_abbrev: "TOR".to_string(),
            wins: 50,
            losses: 20,
            ot_losses: 12,
            points: 112,
            division_name: "Atlantic".to_string(),
            conference_name: Some("Eastern".to_string()),
        };
        state.navigation.stack.push(team_panel);

        let theme = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 80);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 3); // 3 lines (with breadcrumb)
            render_subtabs(f, area, &state, &theme);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // With skip=BREADCRUMB_MIN_DEPTH (2), breadcrumb should only show "Toronto Maple Leafs"
        // (skipping "Standings" and "Conference")
        use crate::tui::widgets::testing::assert_buffer;
        assert_buffer(buffer, &[
            "Wildcard │ Division │ Conference │ League                                       ",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────",
            "▸ Toronto Maple Leafs                                                           ",
        ], 80);
    }
}

// === OLD IMPLEMENTATION - KEPT FOR REFERENCE ===
// [... 748 lines of old code commented out earlier ...]
