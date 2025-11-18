/// Breadcrumb component for showing navigation path
///
/// Displays a breadcrumb trail showing the user's current location in the panel stack.
/// Example: "Standings > Team: TOR > Player: Sidney Crosby"

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::config::DisplayConfig;
use crate::tui::{
    component::RenderableWidget,
    state::PanelState,
    Panel,
    Tab,
};

/// Breadcrumb widget that renders a navigation path
#[derive(Clone)]
pub struct BreadcrumbWidget {
    pub current_tab: Tab,
    pub panel_stack: Vec<PanelState>,
}

impl BreadcrumbWidget {
    pub fn new(current_tab: Tab, panel_stack: Vec<PanelState>) -> Self {
        Self {
            current_tab,
            panel_stack,
        }
    }

    /// Build breadcrumb text from tab and panel stack
    fn build_breadcrumb_text(&self) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Start with the current tab name
        let tab_name = match self.current_tab {
            Tab::Scores => "Scores",
            Tab::Standings => "Standings",
            Tab::Stats => "Stats",
            Tab::Players => "Players",
            Tab::Settings => "Settings",
            Tab::Browser => "Browser",
        };

        spans.push(Span::styled(
            tab_name.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ));

        // Add each panel in the stack
        for panel_state in &self.panel_stack {
            spans.push(Span::raw(" > "));

            let panel_text = match &panel_state.panel {
                Panel::Boxscore { game_id } => format!("Boxscore: Game {}", game_id),
                Panel::TeamDetail { abbrev } => format!("Team: {}", abbrev),
                Panel::PlayerDetail { player_id } => format!("Player: {}", player_id),
            };

            spans.push(Span::raw(panel_text));
        }

        spans
    }
}

impl RenderableWidget for BreadcrumbWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let spans = self.build_breadcrumb_text();
        let line = Line::from(spans);

        // Render the breadcrumb line
        buf.set_line(area.x, area.y, &line, area.width);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(self.clone())
    }

    fn preferred_height(&self) -> Option<u16> {
        if self.panel_stack.is_empty() {
            Some(0) // No breadcrumb if no panels are open
        } else {
            Some(1) // 1 line for breadcrumb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::assert_buffer;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn test_breadcrumb_no_panels() {
        let widget = BreadcrumbWidget::new(Tab::Scores, Vec::new());
        let config = DisplayConfig::default();

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        widget.render(buf.area, &mut buf, &config);

        // With no panels, should just show the tab name
        assert_buffer(&buf, &["Scores"]);
    }

    #[test]
    fn test_breadcrumb_with_team_detail() {
        let panel_stack = vec![PanelState {
            panel: Panel::TeamDetail {
                abbrev: "TOR".to_string(),
            },
            scroll_offset: 0,
            selected_index: None,
        }];

        let widget = BreadcrumbWidget::new(Tab::Standings, panel_stack);
        let config = DisplayConfig::default();

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        widget.render(buf.area, &mut buf, &config);

        assert_buffer(&buf, &["Standings > Team: TOR"]);
    }

    #[test]
    fn test_breadcrumb_with_boxscore() {
        let panel_stack = vec![PanelState {
            panel: Panel::Boxscore { game_id: 2024020001 },
            scroll_offset: 0,
            selected_index: None,
        }];

        let widget = BreadcrumbWidget::new(Tab::Scores, panel_stack);
        let config = DisplayConfig::default();

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        widget.render(buf.area, &mut buf, &config);

        assert_buffer(&buf, &["Scores > Boxscore: Game 2024020001"]);
    }

    #[test]
    fn test_breadcrumb_with_nested_panels() {
        let panel_stack = vec![
            PanelState {
                panel: Panel::Boxscore { game_id: 2024020001 },
                scroll_offset: 0,
                selected_index: None,
            },
            PanelState {
                panel: Panel::PlayerDetail { player_id: 8471675 },
                scroll_offset: 0,
                selected_index: None,
            },
        ];

        let widget = BreadcrumbWidget::new(Tab::Scores, panel_stack);
        let config = DisplayConfig::default();

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        widget.render(buf.area, &mut buf, &config);

        assert_buffer(&buf, &["Scores > Boxscore: Game 2024020001 > Player: 8471675"]);
    }
}
