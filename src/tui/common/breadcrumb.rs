//! Breadcrumb rendering utilities for navigation

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::sync::Arc;
use crate::config::ThemeConfig;

/// Render a breadcrumb trail
///
/// # Arguments
/// * `f` - Frame to render to
/// * `area` - Area to render within
/// * `trail` - Vector of breadcrumb labels
/// * `separator` - Separator between breadcrumbs (e.g., " >> ", " / ", " > ")
/// * `theme` - Theme configuration reference
/// * `base_style` - Base style for separator and line
pub fn render_breadcrumb(
    f: &mut Frame,
    area: Rect,
    trail: &[String],
    separator: &str,
    theme: &Arc<ThemeConfig>,
    base_style: Style,
) {
    if trail.is_empty() {
        return;
    }

    let mut spans = Vec::new();

    for (i, label) in trail.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(separator, base_style));
        }
        spans.push(Span::styled(label.clone(), Style::default().fg(theme.selection_fg)));
    }

    let breadcrumb_line = Line::from(spans);
    let separator_line = Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        base_style,
    )]);

    let breadcrumb_widget = Paragraph::new(vec![breadcrumb_line, separator_line]);
    f.render_widget(breadcrumb_widget, area);
}

/// Render a simple breadcrumb with default separator " ▸ "
pub fn render_breadcrumb_simple(
    f: &mut Frame,
    area: Rect,
    trail: &[String],
    theme: &Arc<ThemeConfig>,
    base_style: Style,
) {
    render_breadcrumb(f, area, trail, " ▸ ", theme, base_style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_creation() {
        let trail = vec!["Team".to_string(), "Player".to_string(), "Stats".to_string()];
        assert_eq!(trail.len(), 3);
    }
}
