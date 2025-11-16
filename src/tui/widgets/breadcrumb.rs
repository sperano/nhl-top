/// Breadcrumb widget - displays navigation breadcrumb trail
///
/// This widget renders a breadcrumb trail showing the current navigation context.
/// Format: "▸ Standings▸ Division▸ Maple Leafs"
/// Uses the separator character as the icon for consistency.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

const DEFAULT_SEPARATOR: &str = " ▸ ";
const ELLIPSIS: &str = "...";

/// Widget for displaying navigation breadcrumb trail
#[derive(Debug)]
pub struct Breadcrumb {
    /// Breadcrumb items (e.g., ["Standings", "Division", "Maple Leafs"])
    pub items: Vec<String>,
    /// Separator between items (default: " ▸ ")
    pub separator: String,
    /// Optional icon at the start (default: same as separator)
    pub icon: Option<String>,
    /// Number of items to skip from the start (default: 0)
    pub skip_items: usize,
}

impl Breadcrumb {
    /// Create a new Breadcrumb with the given items
    pub fn new(items: Vec<String>) -> Self {
        let separator = DEFAULT_SEPARATOR.to_string();
        Self {
            items,
            icon: Some(separator.trim_start().to_string()),
            separator,
            skip_items: 0,
        }
    }

    /// Set a custom separator
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Set a custom icon (or None to disable)
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = icon;
        self
    }

    /// Set the number of items to skip from the start
    pub fn with_skip(mut self, skip_items: usize) -> Self {
        self.skip_items = skip_items;
        self
    }

    /// Format the breadcrumb as a string
    fn format_breadcrumb(&self) -> String {
        let mut result = String::new();

        if let Some(icon) = &self.icon {
            result.push_str(icon);
        }

        // Skip the first N items
        let items_to_show = self.items.iter().skip(self.skip_items);

        for (idx, item) in items_to_show.enumerate() {
            if idx > 0 {
                result.push_str(&self.separator);
            }
            result.push_str(item);
        }

        result
    }

    /// Truncate breadcrumb to fit in the given width
    fn truncate_to_fit(&self, width: usize) -> String {
        let full = self.format_breadcrumb();

        if full.len() <= width {
            return full;
        }

        // If we need to truncate, keep the icon and last item visible
        // Use items after skip_items
        let items_to_show: Vec<_> = self.items.iter().skip(self.skip_items).collect();
        if let Some(last) = items_to_show.last() {
            let icon_part = if let Some(icon) = &self.icon {
                icon.clone()
            } else {
                String::new()
            };

            let last_part = (*last).clone();
            let needed = icon_part.len() + ELLIPSIS.len() + self.separator.len() + last_part.len();

            if needed <= width {
                return format!("{}{}{}{}", icon_part, ELLIPSIS, self.separator, last_part);
            }

            // If even that doesn't fit, just show ellipsis
            if width >= ELLIPSIS.len() {
                return ELLIPSIS.to_string();
            }
        }

        String::new()
    }
}

impl Default for Breadcrumb {
    fn default() -> Self {
        let separator = DEFAULT_SEPARATOR.to_string();
        Self {
            items: Vec::new(),
            icon: Some(separator.trim_start().to_string()),
            separator,
            skip_items: 0,
        }
    }
}

impl RenderableWidget for Breadcrumb {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.items.is_empty() || area.width == 0 || area.height == 0 {
            return;
        }

        let breadcrumb = self.truncate_to_fit(area.width as usize);
        if breadcrumb.is_empty() {
            return;
        }

        // Calculate segments for styling
        let mut segments: Vec<(String, Style)> = Vec::new();
        let mut current_char_pos = 0;

        // Icon segment (if present)
        if let Some(icon) = &self.icon {
            if breadcrumb.starts_with(icon) {
                segments.push((icon.clone(), Style::default()));
                current_char_pos = icon.chars().count();
            }
        }

        // If breadcrumb was truncated, just render it with default style
        if breadcrumb.contains(ELLIPSIS) {
            let remaining: String = breadcrumb.chars().skip(current_char_pos).collect();
            segments.push((remaining, Style::default()));
        } else {
            // Render each item with appropriate styling (after skipping)
            let items_to_render: Vec<_> = self.items.iter().skip(self.skip_items).collect();
            for (idx, item) in items_to_render.iter().enumerate() {
                if idx > 0 {
                    segments.push((self.separator.clone(), Style::default()));
                }

                let style = if idx == items_to_render.len() - 1 {
                    // Last item: highlighted
                    Style::default().fg(config.selection_fg)
                } else {
                    // Other items: normal
                    Style::default()
                };

                segments.push(((*item).clone(), style));
            }
        }

        // Render segments
        let mut x = area.x;
        for (text, style) in segments {
            if x >= area.x + area.width {
                break;
            }

            buf.set_string(x, area.y, &text, style);
            x += text.chars().count() as u16;
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(1)
    }

    fn preferred_width(&self) -> Option<u16> {
        Some(self.format_breadcrumb().len() as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_breadcrumb_empty() {
        let widget = Breadcrumb::new(vec![]);
        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "",
        ]);
    }

    #[test]
    fn test_breadcrumb_single_item() {
        let widget = Breadcrumb::new(vec!["Standings".to_string()]);
        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "▸ Standings",
        ]);
    }

    #[test]
    fn test_breadcrumb_multiple_items() {
        let widget = Breadcrumb::new(vec![
            "Standings".to_string(),
            "Division".to_string(),
            "Maple Leafs".to_string(),
        ]);
        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "▸ Standings ▸ Division ▸ Maple Leafs",
        ]);
    }

    #[test]
    fn test_breadcrumb_custom_separator() {
        let widget = Breadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
        ])
        .with_separator(" > ");

        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "▸ A > B",
        ]);
    }

    #[test]
    fn test_breadcrumb_no_icon() {
        let widget = Breadcrumb::new(vec!["Standings".to_string()])
            .with_icon(None);

        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "Standings",
        ]);
    }

    #[test]
    fn test_breadcrumb_custom_icon() {
        let widget = Breadcrumb::new(vec!["Home".to_string()])
            .with_icon(Some("🏠 ".to_string()));

        let buf = render_widget(&widget, RENDER_WIDTH, 1);

        assert_buffer(&buf, &[
            "🏠 Home",
        ]);
    }

    #[test]
    fn test_breadcrumb_truncation() {
        let widget = Breadcrumb::new(vec![
            "Very Long First Item".to_string(),
            "Second".to_string(),
            "Last".to_string(),
        ]);

        let buf = render_widget(&widget, 20, 1);

        assert_buffer(&buf, &[
            "▸ ... ▸ Last",
        ]);
    }

    #[test]
    fn test_breadcrumb_default() {
        let widget = Breadcrumb::default();

        assert!(widget.items.is_empty());
        assert_eq!(widget.separator, DEFAULT_SEPARATOR);
        assert_eq!(widget.icon, Some(DEFAULT_SEPARATOR.trim_start().to_string()));
    }

    #[test]
    fn test_breadcrumb_preferred_dimensions() {
        let widget = Breadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
        ]);

        assert_eq!(widget.preferred_height(), Some(1));

        let width = widget.preferred_width().unwrap();
        assert!(width > 0);
    }

    #[test]
    fn test_breadcrumb_very_narrow_width() {
        let widget = Breadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        let buf = render_widget(&widget, 2, 1);
        assert_buffer(&buf, &[
            "",
        ]);
    }

    #[test]
    fn test_breadcrumb_zero_area() {
        let widget = Breadcrumb::new(vec!["Test".to_string()]);
        let buf = render_widget(&widget, 0, 0);

        // Should not panic
        assert_eq!(buf.area.width, 0);
    }

    #[test]
    fn test_breadcrumb_format() {
        let widget = Breadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        let formatted = widget.format_breadcrumb();
        assert_eq!(formatted, "▸ A ▸ B ▸ C");
    }

    #[test]
    fn test_breadcrumb_truncate_to_fit() {
        let widget = Breadcrumb::new(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);

        // Test with plenty of space
        let full = widget.truncate_to_fit(100);
        assert_eq!(full, "▸ First ▸ Second ▸ Third");

        // Test with limited space (should show icon, ellipsis, separator, and last item)
        let truncated = widget.truncate_to_fit(20);
        assert!(truncated.contains("..."));
        assert!(truncated.contains("Third"));
    }
}
