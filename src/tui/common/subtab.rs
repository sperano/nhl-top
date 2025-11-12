/// Shared subtab rendering with optional breadcrumb
///
/// This module provides a common implementation for rendering subtabs with breadcrumbs,
/// used by Scores and Standings tabs to avoid code duplication.

use crate::config::DisplayConfig;
use crate::tui::common::styling::{base_tab_style, selection_style};
use crate::tui::common::separator::build_tab_separator_line;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::Arc;

/// Render subtabs with optional breadcrumb
///
/// # Arguments
/// * `f` - Ratatui frame
/// * `area` - Area to render into (should be 2 or 3 lines tall)
/// * `tab_labels` - Labels for each tab
/// * `selected_index` - Index of currently selected tab
/// * `focused` - Whether the subtab area is focused
/// * `breadcrumb_text` - Optional breadcrumb text (only shown if focused)
/// * `display` - Display configuration
pub fn render_subtabs_with_breadcrumb(
    f: &mut Frame,
    area: Rect,
    tab_labels: Vec<String>,
    selected_index: usize,
    focused: bool,
    breadcrumb_text: Option<String>,
    display: &Arc<DisplayConfig>,
) {
    let base_style = base_tab_style(focused);

    // Build tab line with separators
    let separator = format!(" {} ", display.box_chars.vertical);
    let mut tab_spans = Vec::new();

    for (i, label) in tab_labels.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(separator.clone(), base_style));
        }

        let style = selection_style(
            base_style,
            i == selected_index,
            focused,
            display.selection_fg,
            display.unfocused_selection_fg(),
        );
        tab_spans.push(Span::styled(label.clone(), style));
    }
    let tab_line = Line::from(tab_spans);

    // Build separator line
    let separator_line = build_tab_separator_line(
        tab_labels.into_iter(),
        area.width as usize,
        base_style,
        &display.box_chars,
    );

    // Build lines (tabs + separator + optional breadcrumb)
    let mut lines = vec![tab_line, separator_line];
    if focused {
        if let Some(breadcrumb) = breadcrumb_text {
            let breadcrumb_line = Line::from(vec![Span::styled(breadcrumb, base_style)]);
            lines.push(breadcrumb_line);
        }
    }

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_subtabs_without_breadcrumb() {
        let display = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 2);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render_subtabs_with_breadcrumb(
                f,
                area,
                vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()],
                1,
                false, // not focused = no breadcrumb
                Some("Breadcrumb text".to_string()),
                &display,
            );
        }).unwrap();

        let buffer = terminal.backend().buffer();
        let first_line: String = (0..80)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol())
            .collect();

        assert!(first_line.contains("Tab1"));
        assert!(first_line.contains("Tab2"));
        assert!(first_line.contains("Tab3"));
    }

    #[test]
    fn test_render_subtabs_with_breadcrumb() {
        let display = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 3);
            render_subtabs_with_breadcrumb(
                f,
                area,
                vec!["Tab1".to_string(), "Tab2".to_string()],
                0,
                true, // focused = show breadcrumb
                Some("Test ▸ Breadcrumb".to_string()),
                &display,
            );
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // First line: tabs
        let first_line: String = (0..80)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol())
            .collect();
        assert!(first_line.contains("Tab1"));
        assert!(first_line.contains("Tab2"));

        // Third line: breadcrumb
        let third_line: String = (0..80)
            .map(|x| buffer.cell((x, 2)).unwrap().symbol())
            .collect();
        assert!(third_line.contains("Test"));
        assert!(third_line.contains("Breadcrumb"));
        assert!(third_line.contains("▸"));
    }
}
