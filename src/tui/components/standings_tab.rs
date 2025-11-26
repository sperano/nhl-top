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
use crate::tui::{
    component::{Component, Element, ElementWidget},
    state::PanelState,
};

use super::{TabItem, TabbedPanel, TabbedPanelProps};

/// Props for StandingsTab component
#[derive(Clone)]
pub struct StandingsTabProps {
    pub view: GroupBy,
    pub browse_mode: bool,
    pub standings: Arc<Option<Vec<Standing>>>,
    pub panel_stack: Vec<PanelState>,
    pub focused: bool,
    pub config: Config,

    // Document system state
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
            GroupBy::League => self.render_league_view(props, standings),
        }
    }

    fn render_league_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        use super::StandingsDocumentWidget;

        Element::Widget(Box::new(StandingsDocumentWidget::league(
            Arc::new(standings.to_vec()),
            props.config.clone(),
            props.focus_index,
            props.scroll_offset,
        )))
    }

    fn render_conference_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        // Use the document system for Conference view (like League view)
        use super::StandingsDocumentWidget;

        Element::Widget(Box::new(StandingsDocumentWidget::conference(
            Arc::new(standings.to_vec()),
            props.config.clone(),
            props.focus_index,
            props.scroll_offset,
        )))
    }

    fn render_division_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        // Use the document system for Division view
        use super::StandingsDocumentWidget;

        Element::Widget(Box::new(StandingsDocumentWidget::division(
            Arc::new(standings.to_vec()),
            props.config.clone(),
            props.focus_index,
            props.scroll_offset,
        )))
    }

    fn render_wildcard_view(&self, props: &StandingsTabProps, standings: &[Standing]) -> Element {
        // Use the document system for Wildcard view
        use super::StandingsDocumentWidget;

        Element::Widget(Box::new(StandingsDocumentWidget::wildcard(
            Arc::new(standings.to_vec()),
            props.config.clone(),
            props.focus_index,
            props.scroll_offset,
        )))
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

        // Division view now uses document system with two Groups in a Row
        // Layout: Atlantic + Metropolitan on left, Central + Pacific on right
        // (when western_first = false, which is the default)
        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Atlantic                                                     Central",
            "  ════════                                                     ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30      Avalanche                     19    16    2    1     33",
            "  Bruins                        18    13    4    1     27      Stars                         20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26      Jets                          19    13    5    1     27",
            "  Lightning                     18    11    6    1     23      Wild                          19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23      Predators                     19    10    7    2     22",
            "  Senators                      18     9    7    2     20      Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18      Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14      Coyotes                       18     4   13    1      9",
            "",
            "  Metropolitan                                                 Pacific",
            "  ════════════                                                 ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Devils                        18    15    2    1     31      Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30      Oilers                        20    14    4    2     30",
            "  Rangers                       18    12    5    1     25      Kings                         19    12    6    1     25",
            "  Penguins                      19    11    6    2     24      Kraken                        19    11    6    2     24",
            "  Capitals                      18    10    7    1     21      Canucks                       19    10    7    2     22",
            "  Islanders                     18     9    7    2     20      Flames                        19     9    8    2     20",
            "  Flyers                        18     8    9    1     17      Ducks                         19     7   10    2     16",
            "  Blue Jackets                  18     5   11    2     12      Sharks                        18     5   12    1     11",
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

        // Conference view now uses document system with teams sorted by points
        assert_buffer(&buf, &[
            "Wildcard │ Division │ Conference │ League",
            "─────────┴──────────┴────────────┴──────────────────────────────────────────────────────────────────────────────────────",
            "  Eastern                                                      Western",
            "  ═══════                                                      ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Devils                        18    15    2    1     31      Avalanche                     19    16    2    1     33",
            "  Panthers                      19    14    3    2     30      Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30      Stars                         20    14    4    2     30",
            "  Bruins                        18    13    4    1     27      Oilers                        20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26      Jets                          19    13    5    1     27",
            "  Rangers                       18    12    5    1     25      Kings                         19    12    6    1     25",
            "  Penguins                      19    11    6    2     24      Wild                          19    11    6    2     24",
            "  Lightning                     18    11    6    1     23      Kraken                        19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23      Predators                     19    10    7    2     22",
            "  Capitals                      18    10    7    1     21      Canucks                       19    10    7    2     22",
            "  Senators                      18     9    7    2     20      Flames                        19     9    8    2     20",
            "  Islanders                     18     9    7    2     20      Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18      Ducks                         19     7   10    2     16",
            "  Flyers                        18     8    9    1     17      Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14      Sharks                        18     5   12    1     11",
            "  Blue Jackets                  18     5   11    2     12      Coyotes                       18     4   13    1      9",
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
            "  Atlantic                                                     Central",
            "  ════════                                                     ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Panthers                      19    14    3    2     30      Avalanche                     19    16    2    1     33",
            "  Bruins                        18    13    4    1     27      Stars                         20    14    4    2     30",
            "  Maple Leafs                   19    12    5    2     26      Jets                          19    13    5    1     27",
            "",
            "  Metropolitan                                                 Pacific",
            "  ════════════                                                 ═══════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Devils                        18    15    2    1     31      Golden Knights                19    15    3    1     31",
            "  Hurricanes                    19    14    3    2     30      Oilers                        20    14    4    2     30",
            "  Rangers                       18    12    5    1     25      Kings                         19    12    6    1     25",
            "",
            "  Wildcard                                                     Wildcard",
            "  ════════                                                     ════════",
            "",
            "  Team                        GP    W     L    OT   PTS        Team                        GP    W     L    OT   PTS",
            "  ───────────────────────────────────────────────────────      ───────────────────────────────────────────────────────",
            "  Penguins                      19    11    6    2     24      Wild                          19    11    6    2     24",
            "  Lightning                     18    11    6    1     23      Kraken                        19    11    6    2     24",
            "  Canadiens                     18    10    5    3     23      Predators                     19    10    7    2     22",
            "  Capitals                      18    10    7    1     21      Canucks                       19    10    7    2     22",
            "  Senators                      18     9    7    2     20      Flames                        19     9    8    2     20",
            "  Islanders                     18     9    7    2     20      Blues                         19     8    8    3     19",
            "  Red Wings                     18     8    8    2     18      Ducks                         19     7   10    2     16",
            "  Flyers                        18     8    9    1     17      Blackhawks                    18     7   10    1     15",
            "  Sabres                        18     6   10    2     14      Sharks                        18     5   12    1     11",
            "  Blue Jackets                  18     5   11    2     12      Coyotes                       18     4   13    1      9",
            "",
            "",
            "",
            "",
            "",
        ]);
    }
}
