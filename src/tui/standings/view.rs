use super::State;
use super::layout::StandingsLayout;
use super::state::{PanelState, TeamDetailState, PlayerDetailState};
use crate::tui::common::CommonPanel;
use crate::tui::widgets::{
    Container,
    TeamDetail, PlayerDetail, render_scrollable_widget,
    FocusableTable, ColumnDef, Alignment, TableStyle, HighlightMode,
};
use crate::tui::widgets::focus::{Focusable, NavigationAction};
use crate::commands::standings::GroupBy;
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

}

/// Render a panel view (team details, player details, etc.)
fn render_panel(
    f: &mut Frame,
    area: Rect,
    panel: &CommonPanel,
    state: &mut State,
    club_stats: &Arc<HashMap<String, nhl_api::ClubStats>>,
    player_info: &Arc<HashMap<i64, nhl_api::PlayerLanding>>,
    theme: &Arc<DisplayConfig>,
) {
    match panel {
        CommonPanel::TeamDetail {
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
        CommonPanel::PlayerDetail { player_id, player_name, .. } => {
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

/// Build FocusableTable widgets from the standings layout
///
/// Creates one table per column in the layout. Each table has:
/// - One clickable column showing team names
/// - on_activate callback that returns NavigateToTeam action
pub fn build_team_tables(layout: &StandingsLayout) -> Vec<Box<dyn Focusable>> {
    let mut tables: Vec<Box<dyn Focusable>> = Vec::new();

    for column in &layout.columns {
        // Flatten all teams from all groups in this column
        let teams: Vec<nhl_api::Standing> = column.groups
            .iter()
            .flat_map(|g| g.teams.clone())
            .collect();

        if teams.is_empty() {
            continue;
        }

        // Create column definition: just team name, clickable
        let columns = vec![
            ColumnDef::new(
                "Team",
                40,
                |team: &nhl_api::Standing| team.team_common_name.default.clone(),
                Alignment::Left,
                true, // clickable
            ),
        ];

        // Create table with on_activate callback
        let table = FocusableTable::new(teams)
            .with_columns(columns)
            .with_style(TableStyle {
                borders: false,
                row_separators: false,
                highlight_mode: HighlightMode::Row,
                margin: CONTENT_LEFT_MARGIN as u16,
            })
            .with_on_activate(|team| {
                NavigationAction::NavigateToTeam(team.team_abbrev.default.clone())
            });

        tables.push(Box::new(table));
    }

    tables
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

    // Only rebuild tables if they're empty (view changed or first render)
    if state.team_tables.is_empty() {
        state.team_tables = build_team_tables(&layout);
    }

    // Render using FocusableTable widgets
    render_standings_with_tables(f, area, &layout, state, theme);
}

/// Render standings using FocusableTable widgets
fn render_standings_with_tables(
    f: &mut Frame,
    area: Rect,
    layout: &StandingsLayout,
    state: &mut State,
    theme: &Arc<DisplayConfig>,
) {
    let buf = f.buffer_mut();
    let y_start = area.y + 1; // Start with 1 line top margin

    match layout.view {
        GroupBy::League => {
            // Single column layout
            if let Some(table) = state.team_tables.get(0) {
                let table_area = Rect::new(
                    area.x,
                    y_start,
                    area.width,
                    area.height.saturating_sub(1),
                );
                table.render(table_area, buf, theme.as_ref());
            }
        }
        GroupBy::Conference | GroupBy::Division | GroupBy::Wildcard => {
            // Two column layout
            if let Some(left_table) = state.team_tables.get(0) {
                let table_area = Rect::new(
                    area.x,
                    y_start,
                    STANDINGS_COLUMN_WIDTH as u16,
                    area.height.saturating_sub(1),
                );
                left_table.render(table_area, buf, theme.as_ref());
            }

            if let Some(right_table) = state.team_tables.get(1) {
                let table_area = Rect::new(
                    area.x + STANDINGS_COLUMN_WIDTH as u16 + COLUMN_SPACING as u16,
                    y_start,
                    STANDINGS_COLUMN_WIDTH as u16,
                    area.height.saturating_sub(1),
                );
                right_table.render(table_area, buf, theme.as_ref());
            }
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
        use crate::tui::common::CommonPanel;
        use crate::tui::navigation::Panel;

        let mut state = State::new();
        state.view = GroupBy::Conference;
        state.subtab_focused = true; // Focused = show breadcrumb

        // Push a panel onto the navigation stack to simulate being in TeamDetail
        // This makes breadcrumb depth > 2 (Standings > Conference > Team Name)
        let team_panel = CommonPanel::TeamDetail {
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
