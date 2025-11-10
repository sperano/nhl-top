/// TabBar widget - displays main navigation tabs
///
/// This widget renders a horizontal bar showing the main application tabs with box-drawing
/// characters. Tabs are displayed with separators and include keyboard shortcuts.
/// Selected tabs are highlighted based on focus state.

use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::{Line, Span}};
use unicode_width::UnicodeWidthStr;
use crate::config::DisplayConfig;
use crate::tui::app::CurrentTab;
use crate::tui::widgets::RenderableWidget;

/// Represents a single tab in the tab bar
#[derive(Debug, Clone)]
pub struct Tab {
    /// The label for the tab (e.g., "Scores", "Standings")
    pub label: String,
    /// Optional keyboard shortcut (e.g., Some('1') for shortcut 1)
    pub shortcut: Option<char>,
}

impl Tab {
    /// Create a new tab with the given label and shortcut
    pub fn new(label: impl Into<String>, shortcut: Option<char>) -> Self {
        Self {
            label: label.into(),
            shortcut,
        }
    }

    /// Format the tab label for display
    fn display_label(&self) -> String {
        self.label.clone()
    }

    /// Calculate the width of this tab when rendered
    fn width(&self) -> usize {
        self.display_label().len()
    }
}

/// Widget for displaying navigation tabs as a horizontal bar
#[derive(Debug)]
pub struct TabBar {
    /// List of tabs to display
    pub tabs: Vec<Tab>,
    /// Index of the currently selected tab
    pub current_tab: usize,
    /// Whether the tab bar is focused (affects styling)
    pub focused: bool,
}

impl TabBar {
    /// Create a new TabBar from the current tab state
    ///
    /// This creates tabs for: Scores(1), Standings(2), Stats(3), Players(4), Settings(5), Browser(6)
    pub fn new(current_tab: CurrentTab, focused: bool) -> Self {
        let tabs = vec![
            Tab::new("Scores", Some('1')),
            Tab::new("Standings", Some('2')),
            Tab::new("Stats", Some('3')),
            Tab::new("Players", Some('4')),
            Tab::new("Settings", Some('5')),
            Tab::new("Browser", Some('6')),
        ];

        Self {
            tabs,
            current_tab: current_tab.index(),
            focused,
        }
    }

    /// Get the base style based on focus state
    fn base_style(&self) -> Style {
        if self.focused {
            Style::default()
        } else {
            Style::default().fg(ratatui::style::Color::DarkGray)
        }
    }

    /// Get the style for a tab based on selection and focus state
    fn tab_style(&self, index: usize, config: &DisplayConfig) -> Style {
        let base_style = self.base_style();
        let is_selected = index == self.current_tab;

        if is_selected {
            if self.focused {
                base_style.fg(config.selection_fg)
            } else {
                base_style.fg(config.unfocused_selection_fg())
            }
        } else {
            base_style
        }
    }

    /// Build the tab line with separators
    fn build_tab_line(&self, config: &DisplayConfig) -> Vec<(String, Style)> {
        let base_style = self.base_style();
        let separator = format!(" {} ", config.box_chars.vertical);
        let mut segments = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            if i > 0 {
                segments.push((separator.clone(), base_style));
            }

            let style = self.tab_style(i, config);
            segments.push((tab.display_label(), style));
        }

        segments
    }

    /// Build the separator line with connectors under tab gaps
    fn build_separator_line(&self, area_width: usize, config: &DisplayConfig) -> Vec<(String, Style)> {
        let base_style = self.base_style();
        let horizontal = &config.box_chars.horizontal;
        let connector = &config.box_chars.connector2;

        let mut segments = Vec::new();
        let mut pos = 0;

        for (i, tab) in self.tabs.iter().enumerate() {
            if i > 0 {
                // Add horizontal line before separator (1 char)
                segments.push((horizontal.repeat(1), base_style));
                segments.push((connector.to_string(), base_style));
                segments.push((horizontal.repeat(1), base_style));
                pos += 3; // separator width: 1 + 1 + 1
            }
            // Add horizontal line under tab
            let tab_width = tab.width();
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

impl RenderableWidget for TabBar {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.tabs.is_empty() || area.width == 0 || area.height < 2 {
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

    fn preferred_width(&self) -> Option<u16> {
        None // Adapts to available width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_tab_bar_basic_rendering() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let buf = render_widget(&widget, 80, 2);

        assert_buffer(&buf, &[
            "Scores │ Standings │ Stats │ Players │ Settings │ Browser                       ",
            "───────┴───────────┴───────┴─────────┴──────────┴───────────────────────────────",
        ]);
    }

    #[test]
    fn test_tab_bar_focused() {
        let widget = TabBar::new(CurrentTab::Standings, true);
        let buf = render_widget(&widget, 80, 2);

        let line1 = buffer_line(&buf, 0);
        // Line 1 should be: "Scores │ Standings │ Stats │ Players │ Settings"
        // Standings starts at position 9 (after "Scores │ ")
        let standings_start = line1.find("Standings").expect("Should contain Standings");

        let config = test_config();
        let standings_cell = &buf[(standings_start as u16, 0)];
        assert_eq!(standings_cell.fg, config.selection_fg);
    }

    #[test]
    fn test_tab_bar_unfocused() {
        let widget = TabBar::new(CurrentTab::Scores, false);
        let buf = render_widget(&widget, 80, 2);

        // When unfocused, the selected tab (Scores) should use unfocused_selection_fg
        let config = test_config();
        let scores_cell = &buf[(0, 0)]; // 'S' in Scores
        assert_eq!(scores_cell.fg, config.unfocused_selection_fg());
    }
    
    #[test]
    fn test_tab_bar_empty() {
        let widget = TabBar {
            tabs: vec![],
            current_tab: 0,
            focused: true,
        };
        let buf = render_widget(&widget, 80, 2);

        let line1 = buffer_line(&buf, 0);
        assert_eq!(line1.trim(), "");
    }

    #[test]
    fn test_tab_bar_zero_height() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let buf = render_widget(&widget, 80, 0);

        // Should not panic with zero height
        assert_eq!(buf.area.height, 0);
    }

    #[test]
    fn test_tab_bar_small_area() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let buf = render_widget(&widget, 10, 2);

        // Should render without panicking
        // Note: buffer_line returns the full buffer width including padding
        assert_eq!(buf.area.width, 10);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_tab_new() {
        let tab = Tab::new("Test", Some('1'));
        assert_eq!(tab.label, "Test");
        assert_eq!(tab.shortcut, Some('1'));
    }

    #[test]
    fn test_tab_display_label() {
        let tab = Tab::new("Scores", Some('1'));
        assert_eq!(tab.display_label(), "Scores");
    }

    #[test]
    fn test_tab_width() {
        let tab = Tab::new("Standings", None);
        assert_eq!(tab.width(), 9); // "Standings".len()
    }

    #[test]
    fn test_tab_bar_preferred_dimensions() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        assert_eq!(widget.preferred_height(), Some(2));
        assert_eq!(widget.preferred_width(), None);
    }

    #[test]
    fn test_tab_bar_ascii_mode() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let config = test_config_ascii();
        let buf = render_widget_with_config(&widget, 80, 2, &config);

        assert_buffer(&buf, &[
            "Scores | Standings | Stats | Players | Settings | Browser                       ",
            "--------------------------------------------------------------------------------",
        ]);
    }

    #[test]
    fn test_separator_line_alignment_unicode() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let config = test_config();  // test_config() returns unicode config
        let buf = render_widget_with_config(&widget, 100, 2, &config);

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
    fn test_separator_line_alignment_ascii() {
        let widget = TabBar::new(CurrentTab::Standings, true);
        let config = test_config_ascii();
        let buf = render_widget_with_config(&widget, 100, 2, &config);

        let line0 = buffer_line(&buf, 0);
        let line1 = buffer_line(&buf, 1);

        // Find positions of vertical separators (|) in first line
        let vertical_positions: Vec<usize> = line0
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '|')
            .map(|(i, _)| i)
            .collect();

        // In ASCII mode, connector2 is "-", so separators align with "-"
        for pos in vertical_positions {
            let char_at_pos = line1.chars().nth(pos).unwrap_or(' ');
            assert_eq!(
                char_at_pos, '-',
                "Expected connector '-' at position {} (below vertical separator '|'), but found '{}'. Line 0: {}\nLine 1: {}",
                pos, char_at_pos, line0, line1
            );
        }
    }

    #[test]
    fn test_separator_line_continuous() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let config = test_config();  // test_config() returns unicode config
        let buf = render_widget_with_config(&widget, 100, 2, &config);

        let line1 = buffer_line(&buf, 1);

        // Separator line should only contain box-drawing characters, not underscores
        // Valid characters: ─ (horizontal) and ┴ (connector)
        assert!(
            !line1.contains('_'),
            "Separator line should not contain underscores (indicates rendering bug). Line: {}",
            line1
        );

        // Should contain unicode box characters
        assert!(
            line1.contains('─') || line1.contains('┴'),
            "Separator line should contain unicode box characters. Line: {}",
            line1
        );
    }

    #[test]
    fn test_no_mixed_unicode_ascii() {
        let widget = TabBar::new(CurrentTab::Scores, true);
        let config = test_config();  // test_config() returns unicode config
        let buf = render_widget_with_config(&widget, 100, 2, &config);

        let line1 = buffer_line(&buf, 1);

        // In unicode mode, should not have ASCII box chars mixed in
        let has_unicode = line1.contains('─') || line1.contains('┴');
        let has_ascii = line1.contains('-') || line1.contains('+');

        if has_unicode {
            assert!(
                !has_ascii,
                "Separator line has mixed unicode and ASCII characters: {}",
                line1
            );
        }
    }

    #[test]
    fn test_all_tabs_separator_alignment() {
        // Test all possible tab selections to ensure separator aligns in each case
        for tab in [
            CurrentTab::Scores,
            CurrentTab::Standings,
            CurrentTab::Stats,
            CurrentTab::Players,
            CurrentTab::Settings,
            CurrentTab::Browser,
        ] {
            let widget = TabBar::new(tab, true);
            let config = test_config();  // test_config() returns unicode config
            let buf = render_widget_with_config(&widget, 100, 2, &config);

            let line0 = buffer_line(&buf, 0);
            let line1 = buffer_line(&buf, 1);

            // Verify no underscores in separator line
            assert!(
                !line1.contains('_'),
                "Tab {:?}: Separator line contains underscores. Line: {}",
                tab, line1
            );

            // Find vertical separators and verify connectors align
            let vertical_positions: Vec<usize> = line0
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == '│')
                .map(|(i, _)| i)
                .collect();

            for pos in vertical_positions {
                let char_at_pos = line1.chars().nth(pos).unwrap_or(' ');
                assert_eq!(
                    char_at_pos, '┴',
                    "Tab {:?}: Expected '┴' at position {}, found '{}'. Line: {}",
                    tab, pos, char_at_pos, line1
                );
            }
        }
    }
}
