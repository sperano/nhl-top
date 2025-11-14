use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line},
    widgets::{Block, Borders, Paragraph},
};

use nhl_api::Standing;

use crate::commands::standings::GroupBy;
use crate::config::DisplayConfig;
use crate::tui::framework::{
    component::{Component, Element, RenderableWidget},
    state::PanelState,
};

use super::{TabbedPanel, TabbedPanelProps, TabItem};

/// Props for StandingsTab component
#[derive(Clone)]
pub struct StandingsTabProps {
    pub view: GroupBy,
    pub team_mode: bool,
    pub selected_column: usize,
    pub selected_row: usize,
    pub standings: Option<Vec<Standing>>,
    pub panel_stack: Vec<PanelState>,
    pub focused: bool,
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
            return self.render_panel(props);
        }

        // Use TabbedPanel for view selection
        self.render_view_tabs(props)
    }
}

impl StandingsTab {
    /// Render view tabs using TabbedPanel (Division/Conference/League)
    fn render_view_tabs(&self, props: &StandingsTabProps) -> Element {
        let tabs = vec![
            TabItem::new(
                "division",
                "Division",
                self.render_standings_table(props, &GroupBy::Division),
            ),
            TabItem::new(
                "conference",
                "Conference",
                self.render_standings_table(props, &GroupBy::Conference),
            ),
            TabItem::new(
                "league",
                "League",
                self.render_standings_table(props, &GroupBy::League),
            ),
        ];

        let active_key = match props.view {
            GroupBy::Division => "division",
            GroupBy::Conference => "conference",
            GroupBy::League => "league",
            GroupBy::Wildcard => "division", // Fallback
        };

        TabbedPanel.view(
            &TabbedPanelProps {
                active_key: active_key.into(),
                tabs,
                focused: props.focused,
            },
            &(),
        )
    }

    fn render_standings_table(&self, props: &StandingsTabProps, view: &GroupBy) -> Element {
        Element::Widget(Box::new(StandingsTableWidget {
            standings: props.standings.clone(),
            view: view.clone(),
            selected_column: props.selected_column,
            selected_row: props.selected_row,
            team_mode: props.team_mode,
        }))
    }

    fn render_panel(&self, _props: &StandingsTabProps) -> Element {
        // Placeholder for panel rendering
        Element::Widget(Box::new(PanelWidget))
    }
}

/// Standings table widget - shows standings grouped by view
struct StandingsTableWidget {
    standings: Option<Vec<Standing>>,
    view: GroupBy,
    selected_column: usize,
    selected_row: usize,
    team_mode: bool,
}

impl RenderableWidget for StandingsTableWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let widget = match &self.standings {
            None => Paragraph::new("Loading standings...").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Standings"),
            ),
            Some(standings) if standings.is_empty() => {
                Paragraph::new("No standings available").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Standings"),
                )
            }
            Some(standings) => {
                let mut lines: Vec<Line> = vec![Line::from(format!(
                    "{:20} {:>3} {:>3} {:>3}",
                    "Team", "W", "L", "PTS"
                ))];

                for (i, standing) in standings.iter().take(10).enumerate() {
                    let is_selected =
                        self.team_mode && i == self.selected_row && self.selected_column == 0;

                    let line = format!(
                        "{:20} {:>3} {:>3} {:>3}",
                        standing.team_abbrev.default,
                        standing.wins,
                        standing.losses,
                        standing.points
                    );

                    if is_selected {
                        lines.push(Line::styled(
                            line,
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Cyan)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                        ));
                    } else {
                        lines.push(Line::from(line));
                    }
                }

                Paragraph::new(lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Standings - {:?}", self.view)),
                )
            }
        };

        ratatui::widgets::Widget::render(widget, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(StandingsTableWidget {
            standings: self.standings.clone(),
            view: self.view.clone(),
            selected_column: self.selected_column,
            selected_row: self.selected_row,
            team_mode: self.team_mode,
        })
    }
}

/// Panel widget placeholder
struct PanelWidget;

impl RenderableWidget for PanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let widget = Paragraph::new("Panel view (not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Panel"));
        ratatui::widgets::Widget::render(widget, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(PanelWidget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standings_tab_renders_with_no_standings() {
        let standings_tab = StandingsTab;
        let props = StandingsTabProps {
            view: GroupBy::Division,
            team_mode: false,
            selected_column: 0,
            selected_row: 0,
            standings: None,
            panel_stack: Vec::new(),
            focused: false,
        };

        let element = standings_tab.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected container element"),
        }
    }
}
