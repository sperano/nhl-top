/// EnhancedBreadcrumb widget - displays navigation breadcrumb with optional icon
///
/// This widget renders a breadcrumb trail showing the current navigation context.
/// Format: "📍 Standings ▸ Division ▸ Maple Leafs ▸ Matthews"
/// The last item is highlighted in the selection color, previous items use normal foreground.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

const DEFAULT_SEPARATOR: &str = " ▸ ";
const DEFAULT_ICON: &str = "📍";
const ELLIPSIS: &str = "...";

/// Widget for displaying navigation breadcrumb trail
#[derive(Debug)]
pub struct EnhancedBreadcrumb {
    /// Breadcrumb items (e.g., ["Standings", "Division", "Maple Leafs"])
    pub items: Vec<String>,
    /// Separator between items (default: " ▸ ")
    pub separator: String,
    /// Optional icon at the start (default: "📍")
    pub icon: Option<String>,
}

impl EnhancedBreadcrumb {
    /// Create a new EnhancedBreadcrumb with the given items
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            separator: DEFAULT_SEPARATOR.to_string(),
            icon: Some(DEFAULT_ICON.to_string()),
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

    /// Format the breadcrumb as a string
    fn format_breadcrumb(&self) -> String {
        let mut result = String::new();

        if let Some(icon) = &self.icon {
            result.push_str(icon);
            result.push(' ');
        }

        for (idx, item) in self.items.iter().enumerate() {
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
        if let Some(last) = self.items.last() {
            let icon_part = if let Some(icon) = &self.icon {
                format!("{} ", icon)
            } else {
                String::new()
            };

            let last_part = last.clone();
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

impl Default for EnhancedBreadcrumb {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            separator: DEFAULT_SEPARATOR.to_string(),
            icon: Some(DEFAULT_ICON.to_string()),
        }
    }
}

impl RenderableWidget for EnhancedBreadcrumb {
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
        let mut current_pos = 0;

        // Icon segment (if present)
        if let Some(icon) = &self.icon {
            let icon_with_space = format!("{} ", icon);
            if breadcrumb.starts_with(&icon_with_space) {
                segments.push((icon_with_space.clone(), Style::default()));
                current_pos = icon_with_space.len();
            }
        }

        // If breadcrumb was truncated, just render it with default style
        if breadcrumb.contains(ELLIPSIS) {
            let remaining = breadcrumb[current_pos..].to_string();
            segments.push((remaining, Style::default()));
        } else {
            // Render each item with appropriate styling
            for (idx, item) in self.items.iter().enumerate() {
                if idx > 0 {
                    segments.push((self.separator.clone(), Style::default()));
                }

                let style = if idx == self.items.len() - 1 {
                    // Last item: highlighted
                    Style::default().fg(config.selection_fg)
                } else {
                    // Other items: normal
                    Style::default()
                };

                segments.push((item.clone(), style));
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
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_breadcrumb_empty() {
        let widget = EnhancedBreadcrumb::new(vec![]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "                                                                                ",
        ]);
    }

    #[test]
    fn test_breadcrumb_single_item() {
        let widget = EnhancedBreadcrumb::new(vec!["Standings".to_string()]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "📍 Standings                                                                     ",
        ]);
    }

    #[test]
    fn test_breadcrumb_multiple_items() {
        let widget = EnhancedBreadcrumb::new(vec![
            "Standings".to_string(),
            "Division".to_string(),
            "Maple Leafs".to_string(),
        ]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "📍 Standings ▸ Division ▸ Maple Leafs                                            ",
        ]);
    }

    #[test]
    fn test_breadcrumb_custom_separator() {
        let widget = EnhancedBreadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
        ])
        .with_separator(" > ");

        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "📍 A > B                                                                         ",
        ]);
    }

    #[test]
    fn test_breadcrumb_no_icon() {
        let widget = EnhancedBreadcrumb::new(vec!["Standings".to_string()])
            .with_icon(None);

        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "Standings                                                                       ",
        ]);
    }

    #[test]
    fn test_breadcrumb_custom_icon() {
        let widget = EnhancedBreadcrumb::new(vec!["Home".to_string()])
            .with_icon(Some("🏠".to_string()));

        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "🏠 Home                                                                          ",
        ]);
    }

    #[test]
    fn test_breadcrumb_truncation() {
        let widget = EnhancedBreadcrumb::new(vec![
            "Very Long First Item".to_string(),
            "Second".to_string(),
            "Last".to_string(),
        ]);

        let buf = render_widget(&widget, 20, 1);

        assert_buffer(&buf, &[
            "📍 ... ▸ Last        ",
        ]);
    }

    #[test]
    fn test_breadcrumb_default() {
        let widget = EnhancedBreadcrumb::default();

        assert!(widget.items.is_empty());
        assert_eq!(widget.separator, DEFAULT_SEPARATOR);
        assert_eq!(widget.icon, Some(DEFAULT_ICON.to_string()));
    }

    #[test]
    fn test_breadcrumb_preferred_dimensions() {
        let widget = EnhancedBreadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
        ]);

        assert_eq!(widget.preferred_height(), Some(1));

        let width = widget.preferred_width().unwrap();
        assert!(width > 0);
    }

    #[test]
    fn test_breadcrumb_very_narrow_width() {
        let widget = EnhancedBreadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        let buf = render_widget(&widget, 2, 1);
        assert_buffer(&buf, &[
            "  ",
        ]);
    }

    #[test]
    fn test_breadcrumb_zero_area() {
        let widget = EnhancedBreadcrumb::new(vec!["Test".to_string()]);
        let buf = render_widget(&widget, 0, 0);

        // Should not panic
        assert_eq!(buf.area.width, 0);
    }

    #[test]
    fn test_breadcrumb_format() {
        let widget = EnhancedBreadcrumb::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        let formatted = widget.format_breadcrumb();
        assert_eq!(formatted, "📍 A ▸ B ▸ C");
    }

    #[test]
    fn test_breadcrumb_truncate_to_fit() {
        let widget = EnhancedBreadcrumb::new(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);

        // Test with plenty of space
        let full = widget.truncate_to_fit(100);
        assert_eq!(full, "📍 First ▸ Second ▸ Third");

        // Test with limited space (should show icon, ellipsis, separator, and last item)
        let truncated = widget.truncate_to_fit(20);
        assert!(truncated.contains("..."));
        assert!(truncated.contains("Third"));
    }
}
