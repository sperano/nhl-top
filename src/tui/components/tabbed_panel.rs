use crate::config::DisplayConfig;
use crate::tui::component::{vertical, Component, Constraint, Element};

/// A single tab item containing its label and content
#[derive(Clone)]
pub struct TabItem {
    /// Unique key identifying this tab
    pub key: String,
    /// Display title for the tab
    pub title: String,
    /// Content to show when this tab is active
    pub content: Element,
    /// Whether this tab is disabled
    pub disabled: bool,
}

impl TabItem {
    /// Create a new tab item
    pub fn new(key: impl Into<String>, title: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            content,
            disabled: false,
        }
    }

    /// Create a disabled tab item
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for TabbedPanel component
#[derive(Clone)]
pub struct TabbedPanelProps {
    /// Currently active tab key
    pub active_key: String,
    /// List of tabs with their content
    pub tabs: Vec<TabItem>,
    /// Whether the tab bar is focused (affects styling)
    pub focused: bool,
}

/// TabbedPanel component - renders a tab bar with associated content
///
/// This component combines tab navigation with content display, similar to
/// React Bootstrap's Tabs component. Each tab has its own content area.
///
/// Tabs can be nested - a tab's content can itself be another TabbedPanel.
pub struct TabbedPanel;

impl Component for TabbedPanel {
    type Props = TabbedPanelProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        // Find the active tab's content
        let active_content = props
            .tabs
            .iter()
            .find(|tab| tab.key == props.active_key)
            .map(|tab| tab.content.clone())
            .unwrap_or(Element::None);

        // Build tab labels for the tab bar widget
        let tab_labels: Vec<TabLabel> = props
            .tabs
            .iter()
            .map(|tab| TabLabel {
                title: tab.title.clone(),
                key: tab.key.clone(),
                disabled: tab.disabled,
                active: tab.key == props.active_key,
            })
            .collect();

        vertical(
            [
                Constraint::Length(2), // Tab bar (2 lines: labels + separator)
                Constraint::Min(0),    // Content area
            ],
            vec![
                self.render_tab_bar(&tab_labels, props.focused),
                active_content,
            ],
        )
    }
}

impl TabbedPanel {
    fn render_tab_bar(&self, labels: &[TabLabel], focused: bool) -> Element {
        Element::Widget(Box::new(TabBarWidget {
            labels: labels.to_vec(),
            focused,
        }))
    }
}

/// Label for a single tab in the tab bar
#[derive(Clone)]
struct TabLabel {
    title: String,
    #[allow(dead_code)]
    key: String,
    disabled: bool,
    active: bool,
}

/// Widget that renders the tab bar (just the labels)
struct TabBarWidget {
    labels: Vec<TabLabel>,
    focused: bool,
}

impl TabBarWidget {
    /// Get the base style based on focus state
    fn base_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        if self.focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    /// Build segments for the tab line with separators
    fn build_tab_line(&self, config: &DisplayConfig) -> Vec<(String, ratatui::style::Style)> {
        use ratatui::style::{Color, Modifier, Style};

        let base_style = self.base_style();
        let separator = format!(" {} ", config.box_chars.vertical);
        let mut segments = Vec::new();

        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 {
                segments.push((separator.clone(), base_style));
            }

            let style = if label.active {
                if self.focused {
                    base_style
                        .fg(config.selection_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    base_style
                        .fg(config.unfocused_selection_fg())
                        .add_modifier(Modifier::BOLD)
                }
            } else if label.disabled {
                Style::default().fg(Color::DarkGray)
            } else {
                base_style
            };

            segments.push((label.title.clone(), style));
        }

        segments
    }

    /// Build the separator line with connectors under tab gaps
    fn build_separator_line(&self, area_width: usize, config: &DisplayConfig) -> Vec<(String, ratatui::style::Style)> {
        use unicode_width::UnicodeWidthStr;

        let horizontal = &config.box_chars.horizontal;
        let connector = &config.box_chars.connector2;
        let base_style = self.base_style();

        let mut segments = Vec::new();
        let mut pos = 0;

        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 {
                // Add horizontal line before separator (1 char)
                segments.push((horizontal.repeat(1), base_style));
                segments.push((connector.to_string(), base_style));
                segments.push((horizontal.repeat(1), base_style));
                pos += 3; // separator width: 1 + 1 + 1 (" │ ")
            }
            // Add horizontal line under tab
            let tab_width = label.title.width();
            segments.push((horizontal.repeat(tab_width), base_style));
            pos += tab_width;
        }

        // Fill rest of line
        if pos < area_width {
            segments.push((horizontal.repeat(area_width - pos), base_style));
        }

        segments
    }
}

impl crate::tui::component::RenderableWidget for TabBarWidget {
    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, config: &DisplayConfig) {
        use unicode_width::UnicodeWidthStr;

        if self.labels.is_empty() || area.width == 0 || area.height < 2 {
            return;
        }

        let tab_segments = self.build_tab_line(config);
        let separator_segments = self.build_separator_line(area.width as usize, config);

        // Render tab line
        let mut x = area.x;
        for (text, style) in tab_segments {
            if x >= area.x + area.width {
                break;
            }
            buf.set_string(x, area.y, &text, style);
            x += text.width() as u16;  // Use display width, not byte length
        }

        // Render separator line
        let mut x = area.x;
        for (text, style) in separator_segments {
            if x >= area.x + area.width {
                break;
            }
            buf.set_string(x, area.y + 1, &text, style);
            x += text.width() as u16;  // Use display width, not byte length
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(2) // Tab line + separator line
    }

    fn clone_box(&self) -> Box<dyn crate::tui::component::RenderableWidget> {
        Box::new(TabBarWidget {
            labels: self.labels.clone(),
            focused: self.focused,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::component::Element;
    use crate::config::DisplayConfig;
    use crate::formatting::BoxChars;
    use ratatui::{buffer::Buffer, layout::Rect, style::Color};
    use crate::tui::testing::{RENDER_WIDTH, assert_buffer};

    // Helper functions for testing framework widgets

    fn test_config() -> DisplayConfig {
        DisplayConfig {
            use_unicode: true,
            selection_fg: Color::Rgb(255, 200, 0),
            unfocused_selection_fg: None,
            division_header_fg: Color::Rgb(159, 226, 191),
            error_fg: Color::Red,
            box_chars: BoxChars::unicode(),
        }
    }

    fn test_config_ascii() -> DisplayConfig {
        DisplayConfig {
            use_unicode: false,
            selection_fg: Color::Rgb(255, 200, 0),
            unfocused_selection_fg: None,
            division_header_fg: Color::Rgb(159, 226, 191),
            error_fg: Color::Red,
            box_chars: BoxChars::ascii(),
        }
    }

    fn render_widget(
        widget: &impl crate::tui::component::RenderableWidget,
        width: u16,
        height: u16,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let config = test_config();
        widget.render(buf.area, &mut buf, &config);
        buf
    }

    fn render_widget_with_config(
        widget: &impl crate::tui::component::RenderableWidget,
        width: u16,
        height: u16,
        config: &DisplayConfig,
    ) -> Buffer {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        widget.render(buf.area, &mut buf, config);
        buf
    }

    fn buffer_line(buf: &Buffer, line: usize) -> String {
        let area = buf.area();
        let mut result = String::new();
        let y = line as u16;

        if y >= area.height {
            return result;
        }

        for x in 0..area.width {
            let cell = &buf[(x, y)];
            result.push_str(cell.symbol());
        }

        result
    }

    #[test]
    fn test_tabbed_panel_renders_container() {
        let panel = TabbedPanel;
        let props = TabbedPanelProps {
            active_key: "tab1".into(),
            tabs: vec![
                TabItem::new("tab1", "Tab 1", Element::None),
                TabItem::new("tab2", "Tab 2", Element::None),
            ],
            focused: true,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2); // Tab bar + content
            }
            _ => panic!("Expected container element"),
        }
    }

    #[test]
    fn test_tabbed_panel_shows_active_content() {
        let panel = TabbedPanel;

        // Create distinctive content for each tab
        let content1 = Element::Widget(Box::new(TestWidget { id: 1 }));
        let content2 = Element::Widget(Box::new(TestWidget { id: 2 }));

        let props = TabbedPanelProps {
            active_key: "tab2".into(),
            tabs: vec![
                TabItem::new("tab1", "Tab 1", content1),
                TabItem::new("tab2", "Tab 2", content2.clone()),
            ],
            focused: true,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                // Second child should be tab2's content
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected container element"),
        }
    }

    #[test]
    fn test_tab_item_builder() {
        let tab = TabItem::new("key", "Title", Element::None);
        assert_eq!(tab.key, "key");
        assert_eq!(tab.title, "Title");
        assert!(!tab.disabled);

        let disabled_tab = TabItem::new("key", "Title", Element::None).disabled();
        assert!(disabled_tab.disabled);
    }

    #[test]
    fn test_empty_tabs_shows_nothing() {
        let panel = TabbedPanel;
        let props = TabbedPanelProps {
            active_key: "none".into(),
            tabs: vec![],
            focused: true,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2);
                // Content should be Element::None
            }
            _ => panic!("Expected container element"),
        }
    }

    #[test]
    fn test_nonexistent_active_key_shows_none() {
        let panel = TabbedPanel;
        let props = TabbedPanelProps {
            active_key: "nonexistent".into(),
            tabs: vec![
                TabItem::new("tab1", "Tab 1", Element::None),
            ],
            focused: true,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2);
                // Content should be Element::None since key doesn't match
            }
            _ => panic!("Expected container element"),
        }
    }

    // TabBarWidget rendering tests with assert_buffer

    #[test]
    fn test_tab_bar_widget_basic_rendering() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Home".into(),
                    key: "home".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Profile".into(),
                    key: "profile".into(),
                    disabled: false,
                    active: false,
                },
                TabLabel {
                    title: "Settings".into(),
                    key: "settings".into(),
                    disabled: false,
                    active: false,
                },
            ],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);

        assert_buffer(&buf, &[
            "Home │ Profile │ Settings",
            "─────┴─────────┴────────────────────────────────────────────────────────────────",
        ]);
    }

    #[test]
    fn test_tab_bar_widget_with_disabled_tab() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Tab 1".into(),
                    key: "tab1".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Tab 2".into(),
                    key: "tab2".into(),
                    disabled: true,
                    active: false,
                },
            ],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);

        assert_buffer(&buf, &[
            "Tab 1 │ Tab 2",
            "──────┴─────────────────────────────────────────────────────────────────────────",
        ]);
    }

    #[test]
    fn test_tab_bar_widget_single_tab() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Only Tab".into(),
                    key: "only".into(),
                    disabled: false,
                    active: true,
                },
            ],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);
        assert_buffer(&buf, &[
            "Only Tab",
            "────────────────────────────────────────────────────────────────────────────────",
        ]);
    }

    #[test]
    fn test_tab_bar_widget_empty() {
        let widget = TabBarWidget {
            labels: vec![],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);

        // Empty widget should render nothing
        let line1 = buffer_line(&buf, 0);
        let line2 = buffer_line(&buf, 1);
        assert_eq!(line1.trim(), "");
        assert_eq!(line2.trim(), "");
    }

    #[test]
    fn test_tab_bar_widget_ascii_mode() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Tab A".into(),
                    key: "a".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Tab B".into(),
                    key: "b".into(),
                    disabled: false,
                    active: false,
                },
            ],
            focused: true,
        };

        let config = test_config_ascii();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, 2, &config);

        assert_buffer(&buf, &[
            "Tab A | Tab B",
            "--------------------------------------------------------------------------------",
        ]);
    }

    #[test]
    fn test_tab_bar_widget_connector_alignment() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Scores".into(),
                    key: "scores".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Standings".into(),
                    key: "standings".into(),
                    disabled: false,
                    active: false,
                },
                TabLabel {
                    title: "Stats".into(),
                    key: "stats".into(),
                    disabled: false,
                    active: false,
                },
            ],
            focused: true,
        };

        let config = test_config();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, 2, &config);

        let line0 = buffer_line(&buf, 0);
        let line1 = buffer_line(&buf, 1);

        // Find positions of vertical separators (│) in first line
        let vertical_positions: Vec<usize> = line0
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '│')
            .map(|(i, _)| i)
            .collect();

        // Verify connectors (┴) align with vertical separators
        for pos in vertical_positions {
            let char_at_pos = line1.chars().nth(pos).unwrap_or(' ');
            assert_eq!(
                char_at_pos, '┴',
                "Expected connector '┴' at position {} (below vertical separator '│'), but found '{}'. Line 0: {}\nLine 1: {}",
                pos, char_at_pos, line0, line1
            );
        }
    }

    #[test]
    fn test_tab_bar_widget_zero_height() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Test".into(),
                    key: "test".into(),
                    disabled: false,
                    active: true,
                },
            ],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 0);

        // Should not panic with zero height
        assert_eq!(buf.area.height, 0);
    }

    #[test]
    fn test_tab_bar_widget_insufficient_height() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Test".into(),
                    key: "test".into(),
                    disabled: false,
                    active: true,
                },
            ],
            focused: true,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        // Should not render anything if height < 2
        let line = buffer_line(&buf, 0);
        assert_eq!(line.trim(), "");
    }

    // Unfocused rendering tests

    #[test]
    fn test_tab_bar_widget_unfocused_basic() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Home".into(),
                    key: "home".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Profile".into(),
                    key: "profile".into(),
                    disabled: false,
                    active: false,
                },
            ],
            focused: false,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);

        // When unfocused, all text should use DarkGray
        // Check that the separators and inactive tabs use DarkGray
        let config = test_config();

        // Check inactive tab (Profile at position 9+)
        let profile_cell = &buf[(9, 0)]; // 'P' in Profile
        assert_eq!(profile_cell.fg, ratatui::style::Color::DarkGray);

        // Check separator (│ at position 5)
        let separator_cell = &buf[(5, 0)];
        assert_eq!(separator_cell.fg, ratatui::style::Color::DarkGray);

        // Check active tab uses unfocused_selection_fg (darker orange)
        let home_cell = &buf[(0, 0)]; // 'H' in Home
        assert_eq!(home_cell.fg, config.unfocused_selection_fg());
    }

    #[test]
    fn test_tab_bar_widget_unfocused_separator_line() {
        let widget = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Tab A".into(),
                    key: "a".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Tab B".into(),
                    key: "b".into(),
                    disabled: false,
                    active: false,
                },
            ],
            focused: false,
        };

        let buf = render_widget(&widget, RENDER_WIDTH, 2);

        // Check separator line uses DarkGray
        let horizontal_cell = &buf[(0, 1)]; // First horizontal line character
        assert_eq!(horizontal_cell.fg, ratatui::style::Color::DarkGray);

        let connector_cell = &buf[(6, 1)]; // Connector ┴ position
        assert_eq!(connector_cell.fg, ratatui::style::Color::DarkGray);
    }

    #[test]
    fn test_tab_bar_widget_disabled_always_dark_gray() {
        // Disabled tabs should always be DarkGray regardless of focus state
        let widget_focused = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Active".into(),
                    key: "active".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Disabled".into(),
                    key: "disabled".into(),
                    disabled: true,
                    active: false,
                },
            ],
            focused: true,
        };

        let widget_unfocused = TabBarWidget {
            labels: vec![
                TabLabel {
                    title: "Active".into(),
                    key: "active".into(),
                    disabled: false,
                    active: true,
                },
                TabLabel {
                    title: "Disabled".into(),
                    key: "disabled".into(),
                    disabled: true,
                    active: false,
                },
            ],
            focused: false,
        };

        let buf_focused = render_widget(&widget_focused, RENDER_WIDTH, 2);
        let buf_unfocused = render_widget(&widget_unfocused, RENDER_WIDTH, 2);

        // "Disabled" starts at position 10 (after "Active │ ")
        let disabled_cell_focused = &buf_focused[(10, 0)];
        let disabled_cell_unfocused = &buf_unfocused[(10, 0)];

        // Both should be DarkGray
        assert_eq!(disabled_cell_focused.fg, ratatui::style::Color::DarkGray);
        assert_eq!(disabled_cell_unfocused.fg, ratatui::style::Color::DarkGray);
    }

    // Helper test widget
    struct TestWidget {
        id: u32,
    }

    impl crate::tui::component::RenderableWidget for TestWidget {
        fn render(&self, _area: ratatui::layout::Rect, _buf: &mut ratatui::buffer::Buffer, _config: &DisplayConfig) {}
        fn clone_box(&self) -> Box<dyn crate::tui::component::RenderableWidget> {
            Box::new(TestWidget { id: self.id })
        }
    }
}
