/// CommandPalette widget - displays a searchable command modal
///
/// This widget renders as a centered modal overlay that allows users to search
/// and navigate through available commands. Similar to VS Code's command palette.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Represents a single search result in the command palette
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Display label for the result
    pub label: String,
    /// Category/grouping for the result (e.g., "Navigation", "Actions")
    pub category: String,
    /// Navigation path this result leads to
    pub navigation_path: Vec<String>,
    /// Optional icon to display
    pub icon: Option<String>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        label: impl Into<String>,
        category: impl Into<String>,
        navigation_path: Vec<String>,
    ) -> Self {
        Self {
            label: label.into(),
            category: category.into(),
            navigation_path,
            icon: None,
        }
    }

    /// Set the icon for this search result
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Widget for displaying a searchable command palette
#[derive(Debug, Clone)]
pub struct CommandPalette {
    /// Search input text
    pub input: String,
    /// Cursor position in the input
    pub cursor_position: usize,
    /// Filtered search results
    pub results: Vec<SearchResult>,
    /// Index of the selected result
    pub selected_index: usize,
    /// Whether the palette is visible
    pub is_visible: bool,
}

impl CommandPalette {
    /// Create a new CommandPalette
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            results: Vec::new(),
            selected_index: 0,
            is_visible: false,
        }
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.is_visible = true;
        self.input.clear();
        self.cursor_position = 0;
        self.results.clear();
        self.selected_index = 0;
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Add a character to the input
    pub fn input_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.results.len() {
            self.selected_index += 1;
        }
    }

    /// Set the search results
    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        self.selected_index = 0;
    }


    /// Render the border with shadow effect
    fn render_border(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let chars = &config.box_chars;

        // Top border
        buf.set_string(
            area.x,
            area.y,
            &format!(
                "{}{}{}",
                chars.top_left,
                chars.horizontal.repeat((area.width.saturating_sub(2)) as usize),
                chars.top_right
            ),
            Style::default(),
        );

        // Side borders
        for y in area.y + 1..area.bottom().saturating_sub(1) {
            buf.set_string(area.x, y, &chars.vertical, Style::default());
            buf.set_string(
                area.right().saturating_sub(1),
                y,
                &chars.vertical,
                Style::default(),
            );
        }

        // Bottom border
        buf.set_string(
            area.x,
            area.bottom().saturating_sub(1),
            &format!(
                "{}{}{}",
                chars.bottom_left,
                chars.horizontal.repeat((area.width.saturating_sub(2)) as usize),
                chars.bottom_right
            ),
            Style::default(),
        );
    }

    /// Render the search input at the top
    fn render_input(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height < 3 {
            return;
        }

        let input_y = area.y + 1;
        let input_x = area.x + 2;
        let available_width = area.width.saturating_sub(4);

        // Render prompt
        buf.set_string(input_x, input_y, "> ", Style::default());

        // Render input text
        let display_text = if self.input.len() as u16 > available_width.saturating_sub(2) {
            // Scroll input if too long
            let start = self.input.len().saturating_sub((available_width.saturating_sub(2)) as usize);
            &self.input[start..]
        } else {
            &self.input
        };

        buf.set_string(
            input_x + 2,
            input_y,
            display_text,
            Style::default(),
        );

        // Render cursor (simple block at cursor position)
        let cursor_x = input_x + 2 + self.cursor_position.min(display_text.len()) as u16;
        if cursor_x < area.right().saturating_sub(2) {
            buf.set_string(
                cursor_x,
                input_y,
                "▏",
                Style::default().fg(config.selection_fg),
            );
        }
    }

    /// Render the search results list
    fn render_results(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height < 4 {
            return;
        }

        let results_start_y = area.y + 3;
        let results_height = area.height.saturating_sub(4);

        // Render separator
        buf.set_string(
            area.x + 1,
            area.y + 2,
            &config.box_chars.horizontal.repeat((area.width.saturating_sub(2)) as usize),
            Style::default(),
        );

        // Render results
        let visible_results = self.results.iter().take(results_height as usize);
        for (idx, result) in visible_results.enumerate() {
            let y = results_start_y + idx as u16;
            if y >= area.bottom().saturating_sub(1) {
                break;
            }

            let is_selected = idx == self.selected_index;
            let style = if is_selected {
                Style::default().fg(config.selection_fg)
            } else {
                Style::default()
            };

            // Selection indicator
            let indicator = if is_selected { "▸" } else { " " };
            buf.set_string(area.x + 1, y, indicator, style);

            // Icon (if present)
            let mut x = area.x + 3;
            if let Some(icon) = &result.icon {
                buf.set_string(x, y, icon, style);
                x += icon.len() as u16 + 1;
            }

            // Label
            let available_width = area.width.saturating_sub((x - area.x) + 2);
            let label = if result.label.len() as u16 > available_width {
                format!("{}...", &result.label[..available_width.saturating_sub(3) as usize])
            } else {
                result.label.clone()
            };
            buf.set_string(x, y, &label, style);

            // Category (right-aligned if space available)
            let category_width = result.category.len() as u16;
            if category_width + 5 < area.width {
                let category_x = area.right().saturating_sub(category_width + 2);
                if category_x > x + label.len() as u16 {
                    buf.set_string(
                        category_x,
                        y,
                        &result.category,
                        Style::default().fg(config.division_header_fg),
                    );
                }
            }
        }

        // Show "no results" message if empty
        if self.results.is_empty() && !self.input.is_empty() {
            let msg = "No results found";
            let x = area.x + (area.width.saturating_sub(msg.len() as u16)) / 2;
            buf.set_string(x, results_start_y, msg, Style::default());
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderableWidget for CommandPalette {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if !self.is_visible || area.width < 10 || area.height < 5 {
            return;
        }

        // Use the area directly - it's already centered by the layout manager
        self.render_border(area, buf, config);
        self.render_input(area, buf, config);
        self.render_results(area, buf, config);
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Adaptive based on container
    }

    fn preferred_width(&self) -> Option<u16> {
        None // Adaptive based on container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_command_palette_new() {
        let palette = CommandPalette::new();
        assert_eq!(palette.input, "");
        assert_eq!(palette.cursor_position, 0);
        assert_eq!(palette.results.len(), 0);
        assert_eq!(palette.selected_index, 0);
        assert!(!palette.is_visible);
    }

    #[test]
    fn test_command_palette_show_hide() {
        let mut palette = CommandPalette::new();

        palette.show();
        assert!(palette.is_visible);
        assert_eq!(palette.input, "");

        palette.hide();
        assert!(!palette.is_visible);
    }

    #[test]
    fn test_command_palette_input_char() {
        let mut palette = CommandPalette::new();

        palette.input_char('a');
        assert_eq!(palette.input, "a");
        assert_eq!(palette.cursor_position, 1);

        palette.input_char('b');
        assert_eq!(palette.input, "ab");
        assert_eq!(palette.cursor_position, 2);
    }

    #[test]
    fn test_command_palette_delete_char() {
        let mut palette = CommandPalette::new();
        palette.input_char('a');
        palette.input_char('b');

        palette.delete_char();
        assert_eq!(palette.input, "a");
        assert_eq!(palette.cursor_position, 1);

        palette.delete_char();
        assert_eq!(palette.input, "");
        assert_eq!(palette.cursor_position, 0);

        // Should not panic when deleting from empty
        palette.delete_char();
        assert_eq!(palette.input, "");
    }

    #[test]
    fn test_command_palette_cursor_movement() {
        let mut palette = CommandPalette::new();
        palette.input = "test".to_string();
        palette.cursor_position = 4;

        palette.cursor_left();
        assert_eq!(palette.cursor_position, 3);

        palette.cursor_left();
        assert_eq!(palette.cursor_position, 2);

        palette.cursor_right();
        assert_eq!(palette.cursor_position, 3);

        // Boundary checks
        palette.cursor_position = 0;
        palette.cursor_left();
        assert_eq!(palette.cursor_position, 0);

        palette.cursor_position = 4;
        palette.cursor_right();
        assert_eq!(palette.cursor_position, 4);
    }

    #[test]
    fn test_command_palette_selection_movement() {
        let mut palette = CommandPalette::new();
        palette.results = vec![
            SearchResult::new("A", "Cat1", vec![]),
            SearchResult::new("B", "Cat2", vec![]),
            SearchResult::new("C", "Cat3", vec![]),
        ];

        assert_eq!(palette.selected_index, 0);

        palette.select_next();
        assert_eq!(palette.selected_index, 1);

        palette.select_next();
        assert_eq!(palette.selected_index, 2);

        // Boundary check
        palette.select_next();
        assert_eq!(palette.selected_index, 2);

        palette.select_previous();
        assert_eq!(palette.selected_index, 1);

        palette.select_previous();
        assert_eq!(palette.selected_index, 0);

        // Boundary check
        palette.select_previous();
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_command_palette_set_results() {
        let mut palette = CommandPalette::new();
        palette.selected_index = 5;

        let results = vec![
            SearchResult::new("Test", "Category", vec![]),
        ];

        palette.set_results(results);
        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.selected_index, 0); // Reset to 0
    }

    #[test]
    fn test_search_result_with_icon() {
        let result = SearchResult::new("Test", "Cat", vec![])
            .with_icon("🔍");

        assert_eq!(result.icon, Some("🔍".to_string()));
    }

    #[test]
    fn test_command_palette_not_visible() {
        let palette = CommandPalette::new();
        let buf = render_widget(&palette, 80, 24);

        // Should render nothing when not visible
        let line = buffer_line(&buf, 0);
        assert_eq!(line.trim(), "");
    }

    #[test]
    fn test_command_palette_visible() {
        let mut palette = CommandPalette::new();
        palette.show();

        let buf = render_widget(&palette, 80, 24);

        // Should render border when visible
        let has_border = (0..24).any(|y| {
            let line = buffer_line(&buf, y);
            line.contains("╭") || line.contains("╮") || line.contains("│")
        });
        assert!(has_border);
    }

    #[test]
    fn test_command_palette_with_results() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.set_results(vec![
            SearchResult::new("View Standings", "Navigation", vec![]),
            SearchResult::new("View Scores", "Navigation", vec![]),
        ]);

        let buf = render_widget(&palette, 80, 24);

        let has_results = (0..24).any(|y| {
            let line = buffer_line(&buf, y);
            line.contains("View Standings") || line.contains("View Scores")
        });
        assert!(has_results);
    }

    #[test]
    fn test_command_palette_small_area() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Should not panic with small area
        let buf = render_widget(&palette, 8, 4);
        assert_eq!(buf.area.width, 8);
    }

    #[test]
    fn test_command_palette_zero_area() {
        let mut palette = CommandPalette::new();
        palette.show();

        // Should not panic
        let buf = render_widget(&palette, 0, 0);
        assert_eq!(buf.area.width, 0);
    }

    #[test]
    fn test_command_palette_renders_at_given_position() {
        // Regression test: Verify widget uses the area parameter directly
        // instead of recalculating a centered position (which would cause
        // double-centering when the layout manager already provides a centered area)
        let mut palette = CommandPalette::new();
        palette.show();
        palette.input = "test".to_string();

        // Create a buffer representing a full terminal (100x30)
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));

        // Simulate what the layout manager does: calculate a centered area
        // (50% width = 50, 40% height = 12)
        let centered_area = Rect::new(25, 9, 50, 12);

        // Render the palette at this centered area
        palette.render(centered_area, &mut buf, &config);

        // Verify the border renders at the EXACT position we gave it
        // Top-left corner should be at (25, 9)
        let top_left_cell = &buf[(25, 9)];
        assert!(
            top_left_cell.symbol().contains("╭") || top_left_cell.symbol().contains("+"),
            "Expected top-left border character at position (25, 9), found '{}'",
            top_left_cell.symbol()
        );

        // Top-right corner should be at (25 + 50 - 1 = 74, 9)
        let top_right_cell = &buf[(74, 9)];
        assert!(
            top_right_cell.symbol().contains("╮") || top_right_cell.symbol().contains("+"),
            "Expected top-right border character at position (74, 9), found '{}'",
            top_right_cell.symbol()
        );

        // Bottom-left corner should be at (25, 9 + 12 - 1 = 20)
        let bottom_left_cell = &buf[(25, 20)];
        assert!(
            bottom_left_cell.symbol().contains("╰") || bottom_left_cell.symbol().contains("+"),
            "Expected bottom-left border character at position (25, 20), found '{}'",
            bottom_left_cell.symbol()
        );

        // Input prompt should be at (27, 10) - inside the border
        let prompt_cell = &buf[(27, 10)];
        assert_eq!(prompt_cell.symbol(), ">");
    }

    #[test]
    fn test_command_palette_no_double_centering() {
        // Regression test: Ensure the widget doesn't recalculate centering
        // when it receives an already-centered area from the layout manager
        let mut palette = CommandPalette::new();
        palette.show();

        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));

        // Give it an off-center area (simulating a pre-positioned area)
        let area = Rect::new(10, 5, 60, 15);

        palette.render(area, &mut buf, &config);

        // Verify content renders at the given position, NOT re-centered
        // The top-left border should be at (10, 5), exactly where we specified
        let cell = &buf[(10, 5)];
        assert!(
            cell.symbol().contains("╭") || cell.symbol().contains("+"),
            "Widget should render at the given position (10, 5) without recalculating center. Found '{}' instead of border.",
            cell.symbol()
        );

        // Verify nothing rendered at what would be a "re-centered" position
        // If it tried to center within the given area, content would shift
        // The point (25, 10) is roughly the center of the 60x15 area starting at (10,5)
        // which would be (10+7, 5+3) = (17, 8) - let's check that's NOT the border
        let potentially_recentered = &buf[(17, 8)];
        assert!(
            !potentially_recentered.symbol().contains("╭"),
            "Widget should NOT recenter within the given area"
        );
    }
}
