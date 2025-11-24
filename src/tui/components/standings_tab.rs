use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use std::sync::Arc;

use nhl_api::Standing;

use crate::commands::standings::GroupBy;
use crate::config::Config;
use crate::config::DisplayConfig;
use crate::tui::helpers::StandingsSorting;
use crate::tui::{
    component::{horizontal, vertical, Component, Constraint, Element, ElementWidget},
    state::PanelState,
};

use super::{
    create_standings_table_with_selection, standings_columns, TabItem, TabbedPanel,
    TabbedPanelProps, TableWidget,
};

/// Props for StandingsTab component
#[derive(Clone)]
pub struct StandingsTabProps {
    pub view: GroupBy,
    pub browse_mode: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub standings: Arc<Option<Vec<Standing>>>,
    pub panel_stack: Vec<PanelState>,
    pub focused: bool,
    pub config: Config,

    // Document system state (for League view)
    pub focus_index: Option<usize>,
    pub scroll_offset: u16,
}

/// StandingsTab component - renders standings with view selector
pub struct StandingsTab;

impl Component for StandingsTab {
    type Props = StandingsTabProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        // If in panel view, render the panel instead
        if !props.panel_stack.is_empty() {
            tracing::debug!(
                "RENDER: Panel stack has {} items, rendering panel",
                props.panel_stack.len()
            );
            return self.render_panel(props);
        }

        // Use TabbedPanel for view selection
        self.render_view_tabs(props)
    }
}

impl StandingsTab {

    /// Render view tabs using TabbedPanel (Wildcard/Division/Conference/League)
    fn render_view_tabs(&self, props: &StandingsTabProps) -> Element {
        // All inactive tabs get Element::None to avoid cloning issues
        let tabs = [
            GroupBy::Wildcard,
            GroupBy::Division,
            GroupBy::Conference,
            GroupBy::League,
        ];
        let tabs = tabs
            .iter()
            .map(|g| {
                TabItem::new(
                    g.name(),
                    g.name(),
                    if props.view == *g {
                        self.render_standings_table(props, g)
                    } else {
                        Element::None
                    },
                )
            })
            .collect();

        // let tabs = vec![
        //     TabItem::new(
        //         GroupBy::Wildcard.name(),
        //         GroupBy::Wildcard.name(),
        //         if props.view == GroupBy::Wildcard {
        //             self.render_standings_table(props, &GroupBy::Wildcard)
        //         } else {
        //             Element::None
        //         },
        //     ),
        //     TabItem::new(
        //         GroupBy::Division.name(),
        //         GroupBy::Division.name(),
        //         if props.view == GroupBy::Division {
        //             self.render_standings_table(props, &GroupBy::Division)
        //         } else {
        //             Element::None
        //         },
        //     ),
        //     TabItem::new(
        //         GroupBy::Conference.name(),
        //         GroupBy::Conference.name(),
        //         if props.view == GroupBy::Conference {
        //             self.render_standings_table(props, &GroupBy::Conference)
        //         } else {
        //             Element::None
        //         },
        //     ),
        //     TabItem::new(
        //         GroupBy::League.name(),
        //         GroupBy::League.name(),
        //         if props.view == GroupBy::League {
        //             self.render_standings_table(props, &GroupBy::League)
        //         } else {
        //             Element::None
        //         },
        //     ),
        // ];

        // Determine active tab key
        // let active_key = match props.view {
        //     GroupBy::Wildcard => "wildcard",
        //     GroupBy::Division => "division",
        //     GroupBy::Conference => "conference",
        //     GroupBy::League => "league",
        // };
        TabbedPanel.view(
            &TabbedPanelProps {
                active_key: props.view.name().to_string(),
                tabs,
                focused: props.focused && !props.browse_mode,
            },
            &(),
        )
    }

    fn render_standings_table(&self, props: &StandingsTabProps, view: &GroupBy) -> Element {
        // If no standings data, show loading message
        let Some(standings) = props.standings.as_ref().as_ref() else {
            return Element::Widget(Box::new(LoadingWidget {
                message: "Loading standings...".to_string(),
            }));
        };

        if standings.is_empty() {
            return Element::Widget(Box::new(LoadingWidget {
                message: "No standings available".to_string(),
            }));
        }

        match view {
            GroupBy::Conference => self.render_conference_view(props, standings),
            GroupBy::Division => self.render_division_view(props, standings),
            GroupBy::Wildcard => self.render_wildcard_view(props, standings),
            _ => self.render_single_column_view(props, standings),
        }
    }

    fn render_single_column_view(
        &self,
        props: &StandingsTabProps,
        standings: &[Standing],
    ) -> Element {
        // For League view, use the document system
        if props.view == GroupBy::League {
            use super::StandingsDocumentWidget;

            return Element::Widget(Box::new(StandingsDocumentWidget {
                standings: Arc::new(standings.to_vec()),
                config: props.config.clone(),
                focus_index: props.focus_index,
                scroll_offset: props.scroll_offset,
            }));
        }

        // For other single-column views (if any), use the old rendering
        // Convert old selection state to new focused_row
        let focused_row = if props.browse_mode {
            Some(props.selected_row)
        } else {
            None
        };
        let table = create_standings_table_with_selection(standings.to_vec(), None, focused_row);
        Element::Widget(Box::new(table))
    }

    fn render_conference_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        use std::collections::BTreeMap;

        // Group standings by conference
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in standings {
            let conference = standing
                .conference_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            grouped
                .entry(conference)
                .or_default()
                .push(standing.clone());
        }

        // Convert to vec to determine ordering
        let mut groups: Vec<_> = grouped.into_iter().collect();

        // If western_first is true, reverse to show Western first
        // BTreeMap gives us Eastern, Western alphabetically
        if groups.len() == 2 {
            let western_first = props.config.display_standings_western_first;
            if western_first {
                groups.reverse();
            }
        }

        // If we don't have exactly 2 conferences, fall back to single column
        if groups.len() != 2 {
            return self.render_single_column_view(props, standings);
        }

        // Convert selection state to focused_row for each column
        let left_focused = if props.browse_mode && props.selected_column == 0 {
            Some(props.selected_row)
        } else {
            None
        };
        let right_focused = if props.browse_mode && props.selected_column == 1 {
            Some(props.selected_row)
        } else {
            None
        };

        // Create left conference table
        let left_table = TableWidget::from_data(standings_columns(), groups[0].1.clone())
            .with_header(&groups[0].0)
            .with_focused_row(left_focused)
            .with_margin(0);

        // Create right conference table
        let right_table = TableWidget::from_data(standings_columns(), groups[1].1.clone())
            .with_header(&groups[1].0)
            .with_focused_row(right_focused)
            .with_margin(0);

        // Return horizontal layout with both tables
        // Split 50/50 between left and right conference
        horizontal(
            [Constraint::Percentage(50), Constraint::Percentage(50)],
            vec![
                Element::Widget(Box::new(left_table)),
                Element::Widget(Box::new(right_table)),
            ],
        )
    }

    /// Render a column of divisions with tables and spacing
    ///
    /// # Arguments
    /// * `divisions` - List of (division_name, teams) pairs
    /// * `focused_row` - Which row in this column is focused (None = no focus)
    fn render_division_column(
        divisions: &[(String, Vec<Standing>)],
        focused_row: Option<usize>,
    ) -> Vec<Element> {
        let mut elements = Vec::new();
        let mut team_offset = 0;

        for (idx, (div_name, teams)) in divisions.iter().enumerate() {
            let teams_count = teams.len();

            // Calculate focused row within this division
            let row_in_division = focused_row.and_then(|row| {
                if row >= team_offset && row < team_offset + teams_count {
                    Some(row - team_offset)
                } else {
                    None
                }
            });

            let table = TableWidget::from_data(standings_columns(), teams.clone())
                .with_header(div_name)
                .with_focused_row(row_in_division)
                .with_margin(0);

            elements.push(Element::Widget(Box::new(table)));
            team_offset += teams_count;

            // Add spacing between divisions (except after the last one)
            if idx < divisions.len() - 1 {
                elements.push(Element::Widget(Box::new(SpacerWidget { height: 1 })));
            }
        }

        elements
    }

    fn render_division_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        use std::collections::BTreeMap;

        // Group standings by division
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in standings {
            grouped
                .entry(standing.division_name.clone())
                .or_default()
                .push(standing.clone());
        }

        // Separate Eastern and Western divisions
        let mut eastern_divs = Vec::new();
        let mut western_divs = Vec::new();

        for (div_name, teams) in grouped {
            if div_name == "Atlantic" || div_name == "Metropolitan" {
                eastern_divs.push((div_name, teams));
            } else if div_name == "Central" || div_name == "Pacific" {
                western_divs.push((div_name, teams));
            }
        }

        // Sort divisions alphabetically within each conference
        eastern_divs.sort_by(|a, b| a.0.cmp(&b.0));
        western_divs.sort_by(|a, b| a.0.cmp(&b.0));

        // Build column 1 and column 2 based on western_first
        let (col1_divs, col2_divs) = if props.config.display_standings_western_first {
            (western_divs, eastern_divs)
        } else {
            (eastern_divs, western_divs)
        };

        // Convert selection state to focused_row for each column
        let left_focused = if props.browse_mode && props.selected_column == 0 {
            Some(props.selected_row)
        } else {
            None
        };
        let right_focused = if props.browse_mode && props.selected_column == 1 {
            Some(props.selected_row)
        } else {
            None
        };

        // Create tables for left column divisions
        let left_elements = Self::render_division_column(&col1_divs, left_focused);

        // Create tables for right column divisions
        let right_elements = Self::render_division_column(&col2_divs, right_focused);

        // Create vertical layouts for each column
        // Each column has 2 divisions + 1 spacer = 3 elements
        // Each division table: header(1) + underline(1) + blank(1) + col_headers(1) + separator(1) + 8 teams = 13 lines
        // Use Length constraints to keep content top-aligned
        const DIVISION_TABLE_HEIGHT: u16 = 13;
        const SPACER_HEIGHT: u16 = 1;

        let left_column = if left_elements.len() == 3 {
            vertical(
                [
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                ],
                left_elements,
            )
        } else if left_elements.len() == 2 {
            // No spacer (shouldn't happen with 2 divisions, but handle gracefully)
            vertical(
                [
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                ],
                left_elements,
            )
        } else {
            // Fallback for unexpected number of divisions
            vertical(
                [Constraint::Length(
                    DIVISION_TABLE_HEIGHT * 2 + SPACER_HEIGHT,
                )],
                left_elements,
            )
        };

        let right_column = if right_elements.len() == 3 {
            vertical(
                [
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                ],
                right_elements,
            )
        } else if right_elements.len() == 2 {
            // No spacer
            vertical(
                [
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                    Constraint::Length(DIVISION_TABLE_HEIGHT),
                ],
                right_elements,
            )
        } else {
            // Fallback for unexpected number of divisions
            vertical(
                [Constraint::Length(
                    DIVISION_TABLE_HEIGHT * 2 + SPACER_HEIGHT,
                )],
                right_elements,
            )
        };

        // Return horizontal layout with both columns
        horizontal(
            [Constraint::Percentage(50), Constraint::Percentage(50)],
            vec![left_column, right_column],
        )
    }

    fn render_wildcard_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        use std::collections::BTreeMap;

        // Group teams by division and sort by points
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in standings {
            grouped
                .entry(standing.division_name.clone())
                .or_default()
                .push(standing.clone());
        }

        // Sort teams within each division by points
        for teams in grouped.values_mut() {
            teams.sort_by_points_desc();
        }

        // Extract divisions
        let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
        let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
        let central = grouped.get("Central").cloned().unwrap_or_default();
        let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

        // Convert selection state to focused_row for each column
        // The column assignment depends on western_first config
        let (eastern_focused, western_focused) = if props.config.display_standings_western_first {
            // Western is column 0, Eastern is column 1
            let western = if props.browse_mode && props.selected_column == 0 {
                Some(props.selected_row)
            } else {
                None
            };
            let eastern = if props.browse_mode && props.selected_column == 1 {
                Some(props.selected_row)
            } else {
                None
            };
            (eastern, western)
        } else {
            // Eastern is column 0, Western is column 1
            let eastern = if props.browse_mode && props.selected_column == 0 {
                Some(props.selected_row)
            } else {
                None
            };
            let western = if props.browse_mode && props.selected_column == 1 {
                Some(props.selected_row)
            } else {
                None
            };
            (eastern, western)
        };

        // Build Eastern Conference column (Atlantic top 3 + Metropolitan top 3 + wildcards)
        let eastern_elements = self.build_wildcard_conference_column(
            "Atlantic",
            &atlantic,
            "Metropolitan",
            &metropolitan,
            eastern_focused,
        );

        // Build Western Conference column (Central top 3 + Pacific top 3 + wildcards)
        let western_elements =
            self.build_wildcard_conference_column("Central", &central, "Pacific", &pacific, western_focused);

        // Determine column order based on western_first config
        let (left_elements, right_elements) = if props.config.display_standings_western_first {
            (western_elements, eastern_elements)
        } else {
            (eastern_elements, western_elements)
        };

        // Create vertical layouts for each column
        let left_column = self.create_wildcard_column_layout(left_elements);
        let right_column = self.create_wildcard_column_layout(right_elements);

        // Return horizontal layout with both columns
        horizontal(
            [Constraint::Percentage(50), Constraint::Percentage(50)],
            vec![left_column, right_column],
        )
    }

    /// Build a wildcard conference column (2 division top-3s + wildcard section)
    ///
    /// # Arguments
    /// * `div1_name`, `div1_teams` - First division
    /// * `div2_name`, `div2_teams` - Second division
    /// * `focused_row` - Which row in this column is focused (None = no focus)
    fn build_wildcard_conference_column(
        &self,
        div1_name: &str,
        div1_teams: &[Standing],
        div2_name: &str,
        div2_teams: &[Standing],
        focused_row: Option<usize>,
    ) -> Vec<Element> {
        let mut elements = Vec::new();
        let mut team_offset = 0;

        // Division 1 - top 3 teams
        let div1_top3: Vec<_> = div1_teams.iter().take(3).cloned().collect();
        if !div1_top3.is_empty() {
            let table = self.create_wildcard_table(div1_name, &div1_top3, team_offset, focused_row);
            elements.push(Element::Widget(Box::new(table)));
            elements.push(Element::Widget(Box::new(SpacerWidget { height: 1 })));
            team_offset += div1_top3.len();
        }

        // Division 2 - top 3 teams
        let div2_top3: Vec<_> = div2_teams.iter().take(3).cloned().collect();
        if !div2_top3.is_empty() {
            let table = self.create_wildcard_table(div2_name, &div2_top3, team_offset, focused_row);
            elements.push(Element::Widget(Box::new(table)));
            elements.push(Element::Widget(Box::new(SpacerWidget { height: 1 })));
            team_offset += div2_top3.len();
        }

        // Wildcard section - remaining teams from both divisions, sorted by points
        let div1_remaining: Vec<_> = div1_teams.iter().skip(3).cloned().collect();
        let div2_remaining: Vec<_> = div2_teams.iter().skip(3).cloned().collect();

        let mut wildcard_teams: Vec<_> = div1_remaining.into_iter().chain(div2_remaining).collect();
        wildcard_teams.sort_by_points_desc();

        if !wildcard_teams.is_empty() {
            let table =
                self.create_wildcard_table("Wildcard", &wildcard_teams, team_offset, focused_row);
            // TODO: Add playoff cutoff line after 2nd wildcard team
            elements.push(Element::Widget(Box::new(table)));
        }

        elements
    }

    /// Create a table for wildcard view with proper selection
    ///
    /// # Arguments
    /// * `header` - Table header text
    /// * `teams` - Teams to display
    /// * `team_offset` - Offset of first team in this table within the column's total teams
    /// * `focused_row` - Which row in the column is focused (None = no focus)
    fn create_wildcard_table(
        &self,
        header: &str,
        teams: &[Standing],
        team_offset: usize,
        focused_row: Option<usize>,
    ) -> TableWidget {
        let teams_count = teams.len();

        // Calculate focused row within this table
        let row_in_table = focused_row.and_then(|row| {
            if row >= team_offset && row < team_offset + teams_count {
                Some(row - team_offset)
            } else {
                None
            }
        });

        TableWidget::from_data(standings_columns(), teams.to_vec())
            .with_header(header)
            .with_focused_row(row_in_table)
            .with_margin(0)
    }

    /// Create vertical layout for wildcard column elements
    fn create_wildcard_column_layout(&self, elements: Vec<Element>) -> Element {
        // Each division top-3 table needs exactly: header (1) + separator (1) + blank (1) + column headers (1) + separator (1) + 3 rows = 8 lines
        // Spacers are 1 line
        // Use Length for fixed-size elements to prevent ratatui from expanding them
        const TABLE_HEIGHT: u16 = 8;
        const SPACER_HEIGHT: u16 = 1;
        match elements.len() {
            0 => Element::None,
            1 => vertical([Constraint::Min(0)], elements),
            2 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                ],
                elements,
            ),
            3 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(TABLE_HEIGHT),
                ],
                elements,
            ),
            4 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                ],
                elements,
            ),
            5 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Min(1),
                ],
                elements,
            ),
            6 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Min(1),
                    Constraint::Length(SPACER_HEIGHT),
                ],
                elements,
            ),
            7 => vertical(
                [
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Length(TABLE_HEIGHT),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Min(1),
                    Constraint::Length(SPACER_HEIGHT),
                    Constraint::Min(1),
                ],
                elements,
            ),
            _ => {
                // Fallback for more elements
                vertical([Constraint::Min(0)], elements)
            }
        }
    }

    fn render_panel(&self, props: &StandingsTabProps) -> Element {
        // Get the current panel info
        let panel_info = if let Some(panel_state) = props.panel_stack.last() {
            let msg = match &panel_state.panel {
                super::super::Panel::TeamDetail { abbrev } => {
                    format!("Team Detail: {}\n\n(Panel rendering not yet implemented)\n\nPress ESC to go back", abbrev)
                }
                super::super::Panel::PlayerDetail { player_id } => {
                    format!("Player Detail: {}\n\n(Panel rendering not yet implemented)\n\nPress ESC to go back", player_id)
                }
                super::super::Panel::Boxscore { game_id } => {
                    format!("Boxscore: {}\n\n(Panel rendering not yet implemented)\n\nPress ESC to go back", game_id)
                }
            };
            tracing::debug!("RENDER: Rendering panel with message: {}", msg);
            msg
        } else {
            tracing::warn!("RENDER: render_panel called but panel_stack is empty!");
            "No panel".to_string()
        };

        Element::Widget(Box::new(PanelWidget {
            message: panel_info,
        }))
    }
}

/// Loading widget - shows a simple loading or error message
struct LoadingWidget {
    message: String,
}

impl ElementWidget for LoadingWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let widget =
            Paragraph::new(self.message.as_str()).block(Block::default().borders(Borders::NONE));
        ratatui::widgets::Widget::render(widget, area, buf);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(LoadingWidget {
            message: self.message.clone(),
        })
    }
}

/// Panel widget placeholder
struct PanelWidget {
    message: String,
}

impl ElementWidget for PanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let widget = Paragraph::new(self.message.as_str())
            .block(Block::default().borders(Borders::ALL).title("Panel View"));
        ratatui::widgets::Widget::render(widget, area, buf);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(PanelWidget {
            message: self.message.clone(),
        })
    }
}

/// Spacer widget - renders empty lines for vertical spacing
struct SpacerWidget {
    height: u16,
}

impl ElementWidget for SpacerWidget {
    fn render(&self, _area: Rect, _buf: &mut Buffer, _config: &DisplayConfig) {
        // Intentionally empty - just takes up space
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(SpacerWidget {
            height: self.height,
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::renderer::Renderer;
    use crate::tui::testing::{assert_buffer, create_test_standings};
    use ratatui::{buffer::Buffer, layout::Rect};
    const RENDER_WIDTH: u16 = 120;
    const RENDER_HEIGHT: u16 = 40;

    #[test]
    fn test_standings_tab_renders_with_no_standings() {
        let standings_tab = StandingsTab;
        let props = StandingsTabProps {
            view: GroupBy::Division,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(None),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        let element = standings_tab.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected container element"),
        }
    }

    #[test]
    fn test_standings_tab_renders_league_view() {
        let standings_tab = StandingsTab;
        let standings = create_test_standings();

        let props = StandingsTabProps {
            view: GroupBy::League,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(Some(standings)),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        // This should not panic - verifies TableWidget can be created
        let element = standings_tab.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2); // Tab bar + content
            }
            _ => panic!("Expected container element"),
        }
    }

    // === Rendering Tests ===

    /// Helper to render element to buffer
    fn render_element_to_buffer(
        element: &Element,
        width: u16,
        height: u16,
        config: &DisplayConfig,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let mut renderer = Renderer::new();
        renderer.render(element.clone(), buf.area, &mut buf, config);
        buf
    }

    #[test]
    fn test_league_view_full_render() {
        let standings_tab = StandingsTab;
        let standings = create_test_standings();

        let props = StandingsTabProps {
            view: GroupBy::League,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(Some(standings)),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        let element = standings_tab.view(&props, &());
        let config = DisplayConfig::default();
        let buf = render_element_to_buffer(&element, RENDER_WIDTH, RENDER_HEIGHT, &config);

        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30",
            "  Bruins                        18    13    4    1     27",
            "  Maple Leafs                   19    12    5    2     26",
            "  Lightning                     18    11    6    1     23",
            "  Canadiens                     18    10    5    3     23",
            "  Senators                      18     9    7    2     20",
            "  Red Wings                     18     8    8    2     18",
            "  Sabres                        18     6   10    2     14",
            "  Devils                        18    15    2    1     31",
            "  Hurricanes                    19    14    3    2     30",
            "  Rangers                       18    12    5    1     25",
            "  Penguins                      19    11    6    2     24",
            "  Capitals                      18    10    7    1     21",
            "  Islanders                     18     9    7    2     20",
            "  Flyers                        18     8    9    1     17",
            "  Blue Jackets                  18     5   11    2     12",
            "  Avalanche                     19    16    2    1     33",
            "  Stars                         20    14    4    2     30",
            "  Jets                          19    13    5    1     27",
            "  Wild                          19    11    6    2     24",
            "  Predators                     19    10    7    2     22",
            "  Blues                         19     8    8    3     19",
            "  Blackhawks                    18     7   10    1     15",
            "  Coyotes                       18     4   13    1      9",
            "  Golden Knights                19    15    3    1     31",
            "  Oilers                        20    14    4    2     30",
            "  Kings                         19    12    6    1     25",
            "  Kraken                        19    11    6    2     24",
            "  Canucks                       19    10    7    2     22",
            "  Flames                        19     9    8    2     20",
            "  Ducks                         19     7   10    2     16",
            "  Sharks                        18     5   12    1     11",
            "",
            "",
            "",
            "",
        ]);
    }

    #[test]
    fn test_division_view_full_render() {
        let standings_tab = StandingsTab;
        let standings = create_test_standings();

        let props = StandingsTabProps {
            view: GroupBy::Division,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(Some(standings)),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        let element = standings_tab.view(&props, &());
        let config = DisplayConfig::default();
        let buf = render_element_to_buffer(&element, RENDER_WIDTH, RENDER_HEIGHT, &config);

        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Atlantic                                                    Central",
            "  ════════                                                    ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30     Avalanche                     19    16    2    1     33",
            "  Bruins                        18    13    4    1     27     Stars                         20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26     Jets                          19    13    5    1     27",
            "  Lightning                     18    11    6    1     23     Wild                          19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23     Predators                     19    10    7    2     22",
            "  Senators                      18     9    7    2     20     Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18     Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14     Coyotes                       18     4   13    1      9",
            "",
            "  Metropolitan                                                Pacific",
            "  ════════════                                                ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Devils                        18    15    2    1     31     Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30     Oilers                        20    14    4    2     30",
            "  Rangers                       18    12    5    1     25     Kings                         19    12    6    1     25",
            "  Penguins                      19    11    6    2     24     Kraken                        19    11    6    2     24",
            "  Capitals                      18    10    7    1     21     Canucks                       19    10    7    2     22",
            "  Islanders                     18     9    7    2     20     Flames                        19     9    8    2     20",
            "  Flyers                        18     8    9    1     17     Ducks                         19     7   10    2     16",
            "  Blue Jackets                  18     5   11    2     12     Sharks                        18     5   12    1     11",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
    }

    #[test]
    fn test_conference_view_full_render() {
        let standings_tab = StandingsTab;
        let standings = create_test_standings();

        let props = StandingsTabProps {
            view: GroupBy::Conference,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(Some(standings)),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        let element = standings_tab.view(&props, &());
        let config = DisplayConfig::default();
        let buf = render_element_to_buffer(&element, RENDER_WIDTH, RENDER_HEIGHT, &config);

        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Eastern                                                     Western",
            "  ═══════                                                     ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30     Avalanche                     19    16    2    1     33",
            "  Bruins                        18    13    4    1     27     Stars                         20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26     Jets                          19    13    5    1     27",
            "  Lightning                     18    11    6    1     23     Wild                          19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23     Predators                     19    10    7    2     22",
            "  Senators                      18     9    7    2     20     Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18     Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14     Coyotes                       18     4   13    1      9",
            "  Devils                        18    15    2    1     31     Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30     Oilers                        20    14    4    2     30",
            "  Rangers                       18    12    5    1     25     Kings                         19    12    6    1     25",
            "  Penguins                      19    11    6    2     24     Kraken                        19    11    6    2     24",
            "  Capitals                      18    10    7    1     21     Canucks                       19    10    7    2     22",
            "  Islanders                     18     9    7    2     20     Flames                        19     9    8    2     20",
            "  Flyers                        18     8    9    1     17     Ducks                         19     7   10    2     16",
            "  Blue Jackets                  18     5   11    2     12     Sharks                        18     5   12    1     11",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
    }

    #[test]
    fn test_wildcard_view_full_render() {
        let standings_tab = StandingsTab;
        let standings = create_test_standings();
        let props = StandingsTabProps {
            view: GroupBy::Wildcard,
            browse_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: Arc::new(Some(standings)),
            panel_stack: Vec::new(),
            focused: false,
            config: Config::default(),
            focus_index: None,
            scroll_offset: 0,
        };

        let element = standings_tab.view(&props, &());
        let config = DisplayConfig::default();
        let buf = render_element_to_buffer(&element, RENDER_WIDTH, RENDER_HEIGHT, &config);

        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Atlantic                                                    Central",
            "  ════════                                                    ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30     Avalanche                     19    16    2    1     33",
            "  Bruins                        18    13    4    1     27     Stars                         20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26     Jets                          19    13    5    1     27",
            "",
            "  Metropolitan                                                Pacific",
            "  ════════════                                                ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Devils                        18    15    2    1     31     Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30     Oilers                        20    14    4    2     30",
            "  Rangers                       18    12    5    1     25     Kings                         19    12    6    1     25",
            "",
            "  Wildcard                                                    Wildcard",
            "  ════════                                                    ════════",
            "",
            "  Team                        GP    W     L    OT   PTS       Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────     ───────────────────────────────────────────────────────",
            "  Penguins                      19    11    6    2     24     Wild                          19    11    6    2     24",
            "  Lightning                     18    11    6    1     23     Kraken                        19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23     Predators                     19    10    7    2     22",
            "  Capitals                      18    10    7    1     21     Canucks                       19    10    7    2     22",
            "  Senators                      18     9    7    2     20     Flames                        19     9    8    2     20",
            "  Islanders                     18     9    7    2     20     Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18     Ducks                         19     7   10    2     16",
            "  Flyers                        18     8    9    1     17     Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14     Sharks                        18     5   12    1     11",
            "  Blue Jackets                  18     5   11    2     12     Coyotes                       18     4   13    1      9",
            "",
            "",
            "",
            "",
            "",
        ]);
    }

    // Helper function for creating test standings
    fn create_test_standing(name: &str, points: i32) -> Standing {
        use crate::tui::testing::create_division_team;
        create_division_team(
            name,
            &name.replace("Team ", "T"),
            "Division",
            "Conference",
            0, // wins
            0, // losses
            0, // ot
            points,
        )
    }
}
