/// Unified layout manager for the TUI
///
/// This module provides a flexible layout system that dynamically positions all UI components
/// based on which components are active. It orchestrates the rendering of tab bar, breadcrumb,
/// action bar, status bar, and command palette widgets.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    Frame,
};
use crate::config::DisplayConfig;
use crate::tui::widgets::{TabBar, Breadcrumb, ActionBar, StatusBar, CommandPalette, RenderableWidget};

/// Unified layout manager for the TUI
///
/// This struct manages the positioning and rendering of all top-level UI components.
pub struct Layout {
    pub tab_bar: TabBar,
    pub breadcrumb: Option<Breadcrumb>,
    pub action_bar: Option<ActionBar>,
    pub status_bar: StatusBar,
    pub command_palette: Option<CommandPalette>,
}

/// Calculated areas for each component
///
/// This struct holds the Rect for each component after layout calculation.
pub struct LayoutAreas {
    pub tab_bar: Rect,
    pub breadcrumb: Option<Rect>,
    pub content: Rect,
    pub action_bar: Option<Rect>,
    pub status_bar: Rect,
    pub command_palette: Option<Rect>,
}

impl Layout {
    /// Calculate the area rectangles for all components
    ///
    /// This method uses dynamic constraint calculation based on which components are present.
    /// The layout structure is:
    /// - Tab bar: 2 lines (always present)
    /// - Breadcrumb: 2 lines (if Some)
    /// - Content: remaining space (always present)
    /// - Action bar: 2 lines (if Some)
    /// - Status bar: 2 lines (always present)
    /// - Command palette: centered overlay (if Some and visible)
    pub fn calculate_areas(&self, terminal_area: Rect, config: &DisplayConfig) -> LayoutAreas {
        let mut constraints = vec![];

        // Track which index corresponds to which component
        let tab_bar_idx = 0;
        constraints.push(Constraint::Length(2)); // Tab bar

        let breadcrumb_idx = if self.breadcrumb.is_some() {
            constraints.push(Constraint::Length(2));
            Some(constraints.len() - 1)
        } else {
            None
        };

        // Content takes remaining space
        constraints.push(Constraint::Min(0));
        let content_idx = constraints.len() - 1;

        let action_bar_idx = if self.action_bar.is_some() {
            constraints.push(Constraint::Length(2));
            Some(constraints.len() - 1)
        } else {
            None
        };

        // Status bar always present
        constraints.push(Constraint::Length(2));
        let status_bar_idx = constraints.len() - 1;

        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(terminal_area);

        LayoutAreas {
            tab_bar: chunks[tab_bar_idx],
            breadcrumb: breadcrumb_idx.map(|idx| chunks[idx]),
            content: chunks[content_idx],
            action_bar: action_bar_idx.map(|idx| chunks[idx]),
            status_bar: chunks[status_bar_idx],
            command_palette: self.command_palette.as_ref()
                .filter(|cp| cp.is_visible)
                .map(|_| centered_rect(50, 40, terminal_area)),
        }
    }

    /// Render all layout components to the frame
    ///
    /// This method renders chrome components (tab bar, breadcrumb, action bar, status bar).
    /// Note: Command palette is NOT rendered here - it must be rendered last in the main
    /// render loop to appear on top of all content.
    pub fn render_chrome(&self, frame: &mut Frame, areas: &LayoutAreas, config: &DisplayConfig) {
        // Render tab bar
        {
            let render_area = Rect::new(0, 0, areas.tab_bar.width, areas.tab_bar.height);
            let mut tab_buf = Buffer::empty(render_area);
            self.tab_bar.render(render_area, &mut tab_buf, config);
            let buf = frame.buffer_mut();
            for y in 0..areas.tab_bar.height {
                for x in 0..areas.tab_bar.width {
                    let cell = &tab_buf[(x, y)];
                    buf[(areas.tab_bar.x + x, areas.tab_bar.y + y)]
                        .set_symbol(cell.symbol())
                        .set_style(cell.style());
                }
            }
        }

        // Render breadcrumb if present
        if let (Some(breadcrumb), Some(area)) = (&self.breadcrumb, areas.breadcrumb) {
            let render_area = Rect::new(0, 0, area.width, area.height);
            let mut buf = Buffer::empty(render_area);
            breadcrumb.render(render_area, &mut buf, config);
            let frame_buf = frame.buffer_mut();
            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &buf[(x, y)];
                    frame_buf[(area.x + x, area.y + y)]
                        .set_symbol(cell.symbol())
                        .set_style(cell.style());
                }
            }
        }

        // Render action bar if present
        if let (Some(action_bar), Some(area)) = (&self.action_bar, areas.action_bar) {
            let render_area = Rect::new(0, 0, area.width, area.height);
            let mut buf = Buffer::empty(render_area);
            action_bar.render(render_area, &mut buf, config);
            let frame_buf = frame.buffer_mut();
            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &buf[(x, y)];
                    frame_buf[(area.x + x, area.y + y)]
                        .set_symbol(cell.symbol())
                        .set_style(cell.style());
                }
            }
        }

        // Render status bar
        {
            let render_area = Rect::new(0, 0, areas.status_bar.width, areas.status_bar.height);
            let mut status_buf = Buffer::empty(render_area);
            self.status_bar.render(render_area, &mut status_buf, config);
            let buf = frame.buffer_mut();
            for y in 0..areas.status_bar.height {
                for x in 0..areas.status_bar.width {
                    let cell = &status_buf[(x, y)];
                    buf[(areas.status_bar.x + x, areas.status_bar.y + y)]
                        .set_symbol(cell.symbol())
                        .set_style(cell.style());
                }
            }
        }

        // Command palette is rendered separately in the main render loop (after all content)
        // to ensure it appears on top of everything
    }
}

/// Calculate a centered rectangle for modal overlays
///
/// Returns a Rect that is centered within the given area with the specified
/// percentage of width and height.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::CurrentTab;
    use crate::tui::widgets::{Breadcrumb, ActionBar, Action};

    #[test]
    fn test_layout_areas_minimal() {
        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 30);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Tab bar: 2 lines
        assert_eq!(areas.tab_bar.height, 2);
        // No breadcrumb
        assert!(areas.breadcrumb.is_none());
        // No action bar
        assert!(areas.action_bar.is_none());
        // Status bar: 2 lines
        assert_eq!(areas.status_bar.height, 2);
        // Content gets remaining: 30 - 2 - 2 = 26
        assert_eq!(areas.content.height, 26);
    }

    #[test]
    fn test_layout_areas_with_all_components() {
        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Standings, true),
            breadcrumb: Some(Breadcrumb {
                items: vec!["Standings".to_string()],
                separator: " ▸ ".to_string(),
                icon: Some(" ▸ ".to_string()),
                skip_items: 0,
            }),
            action_bar: Some(ActionBar {
                actions: vec![Action {
                    key: "Enter".to_string(),
                    label: "Select".to_string(),
                    enabled: true,
                }],
            }),
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 30);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Tab bar: 2 lines
        assert_eq!(areas.tab_bar.height, 2);
        // Breadcrumb: 2 lines
        assert_eq!(areas.breadcrumb.unwrap().height, 2);
        // Action bar: 2 lines
        assert_eq!(areas.action_bar.unwrap().height, 2);
        // Status bar: 2 lines
        assert_eq!(areas.status_bar.height, 2);
        // Content gets remaining: 30 - 2 - 2 - 2 - 2 = 22
        assert_eq!(areas.content.height, 22);
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(50, 40, area);

        // 50% width of 100 = 50, centered at 25
        assert_eq!(centered.width, 50);
        assert_eq!(centered.x, 25);

        // 40% height of 50 = 20, centered at 15
        assert_eq!(centered.height, 20);
        assert_eq!(centered.y, 15);
    }

    #[test]
    fn test_layout_areas_stacking() {
        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 80, 24);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Components should be stacked vertically with no gaps
        assert_eq!(areas.tab_bar.y, 0);
        assert_eq!(areas.tab_bar.height, 2);

        // Content starts right after tab bar
        assert_eq!(areas.content.y, 2);

        // Status bar is at bottom
        assert_eq!(areas.status_bar.y + areas.status_bar.height, terminal_area.height);
    }

    #[test]
    fn test_layout_areas_with_breadcrumb() {
        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Standings, true),
            breadcrumb: Some(Breadcrumb {
                items: vec!["Standings".to_string(), "Eastern".to_string()],
                separator: " > ".to_string(),
                icon: None,
                skip_items: 0,
            }),
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 30);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Tab bar: 2 lines
        assert_eq!(areas.tab_bar.height, 2);
        // Breadcrumb: 2 lines
        let breadcrumb_area = areas.breadcrumb.unwrap();
        assert_eq!(breadcrumb_area.height, 2);
        assert_eq!(breadcrumb_area.y, 2); // Right after tab bar
        // Content starts after breadcrumb
        assert_eq!(areas.content.y, 4);
        // Status bar: 2 lines at bottom
        assert_eq!(areas.status_bar.height, 2);
        // Content height: 30 - 2 - 2 - 2 = 24
        assert_eq!(areas.content.height, 24);
    }

    #[test]
    fn test_layout_areas_with_action_bar() {
        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Standings, true),
            breadcrumb: None,
            action_bar: Some(ActionBar {
                actions: vec![
                    Action {
                        key: "Enter".to_string(),
                        label: "Select".to_string(),
                        enabled: true,
                    },
                    Action {
                        key: "ESC".to_string(),
                        label: "Back".to_string(),
                        enabled: true,
                    },
                ],
            }),
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 30);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Tab bar: 2 lines
        assert_eq!(areas.tab_bar.height, 2);
        // Action bar: 2 lines
        let action_bar_area = areas.action_bar.unwrap();
        assert_eq!(action_bar_area.height, 2);
        // Action bar should be above status bar
        assert_eq!(action_bar_area.y + action_bar_area.height, areas.status_bar.y);
        // Status bar: 2 lines at bottom
        assert_eq!(areas.status_bar.height, 2);
        assert_eq!(areas.status_bar.y, 28); // 30 - 2 = 28
        // Content height: 30 - 2 (tab) - 2 (action) - 2 (status) = 24
        assert_eq!(areas.content.height, 24);
    }

    #[test]
    fn test_command_palette_overlay() {
        let mut palette = CommandPalette::new();
        palette.is_visible = true;

        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: Some(palette),
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Command palette should be present
        assert!(areas.command_palette.is_some());
        let palette_area = areas.command_palette.unwrap();

        // Should be centered (50% width, 40% height)
        assert_eq!(palette_area.width, 50);
        assert_eq!(palette_area.x, 25);
        assert_eq!(palette_area.height, 20);
        assert_eq!(palette_area.y, 15);
    }

    #[test]
    fn test_command_palette_hidden() {
        let mut palette = CommandPalette::new();
        palette.is_visible = false;

        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: Some(palette),
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate_areas(terminal_area, &config);

        // Command palette should not be rendered when not visible
        assert!(areas.command_palette.is_none());
    }

    #[test]
    fn test_centered_rect_various_sizes() {
        // Test with different percentages
        let area = Rect::new(0, 0, 80, 40);

        let centered_80_80 = centered_rect(80, 80, area);
        assert_eq!(centered_80_80.width, 64); // 80% of 80
        assert_eq!(centered_80_80.x, 8);      // (80 - 64) / 2
        assert_eq!(centered_80_80.height, 32); // 80% of 40
        assert_eq!(centered_80_80.y, 4);       // (40 - 32) / 2

        let centered_30_30 = centered_rect(30, 30, area);
        assert_eq!(centered_30_30.width, 24);  // 30% of 80
        assert_eq!(centered_30_30.x, 28);      // (80 - 24) / 2
        assert_eq!(centered_30_30.height, 12); // 30% of 40
        assert_eq!(centered_30_30.y, 14);      // (40 - 12) / 2
    }

    #[test]
    fn test_render_chrome_no_crash() {
        // Regression test for buffer indexing crash
        // This ensures render_chrome uses correct buffer indexing (not .cell())
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use crate::tui::widgets::testing::test_config;

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = test_config();

        // This should not panic with "Cell should exist"
        terminal.draw(|frame| {
            let areas = layout.calculate_areas(frame.size(), &config);
            layout.render_chrome(frame, &areas, &config);
        }).unwrap();
    }

    #[test]
    fn test_render_chrome_with_all_components() {
        // Test rendering with all optional components present
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use crate::tui::widgets::{Breadcrumb, ActionBar, Action};
        use crate::tui::widgets::testing::test_config;

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Standings, true),
            breadcrumb: Some(Breadcrumb {
                items: vec!["Standings".to_string(), "Division".to_string()],
                separator: " ▸ ".to_string(),
                icon: Some(" ▸ ".to_string()),
                skip_items: 0,
            }),
            action_bar: Some(ActionBar {
                actions: vec![Action {
                    key: "←→".to_string(),
                    label: "Change View".to_string(),
                    enabled: true,
                }],
            }),
            status_bar: StatusBar::new(),
            command_palette: None,
        };

        let config = test_config();

        // Should not panic even with all components
        terminal.draw(|frame| {
            let areas = layout.calculate_areas(frame.size(), &config);
            layout.render_chrome(frame, &areas, &config);
        }).unwrap();
    }

    #[test]
    fn test_command_palette_uses_default_size() {
        let mut palette = CommandPalette::new();
        palette.is_visible = true;

        let layout = Layout {
            tab_bar: TabBar::new(CurrentTab::Scores, true),
            breadcrumb: None,
            action_bar: None,
            status_bar: StatusBar::new(),
            command_palette: Some(palette),
        };

        let config = DisplayConfig::default();
        let terminal_area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate_areas(terminal_area, &config);
        let palette_area = areas.command_palette.unwrap();
        assert_eq!(palette_area.width, 50); // 50% width of 100 = 50
        assert_eq!(palette_area.height, 20); // 40% height of 50 = 20
    }

}
