/// Shared subtab rendering with optional breadcrumb
///
/// This module provides a common implementation for rendering subtabs with breadcrumbs,
/// used by Scores and Standings tabs to avoid code duplication.

use crate::config::DisplayConfig;
use crate::tui::common::styling::{base_tab_style, selection_style};
use crate::tui::common::separator::build_tab_separator_line;
use crate::tui::widgets::{Breadcrumb, RenderableWidget};
use ratatui::{
    layout::{Rect, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
    buffer::Buffer,
};
use std::sync::Arc;

/// Minimum breadcrumb depth required to show breadcrumb
/// Breadcrumb is only shown if there are more than this many items
pub const BREADCRUMB_MIN_DEPTH: usize = 2;

/// Render subtabs with optional breadcrumb
///
/// # Arguments
/// * `f` - Ratatui frame
/// * `area` - Area to render into (should be 2 or 3 lines tall)
/// * `tab_labels` - Labels for each tab
/// * `selected_index` - Index of currently selected tab
/// * `focused` - Whether the subtab area is focused
/// * `breadcrumb_items` - Optional breadcrumb items (only shown if focused and length > BREADCRUMB_MIN_DEPTH)
/// * `breadcrumb_skip` - Number of breadcrumb items to skip from the start
/// * `display` - Display configuration
pub fn render_subtabs_with_breadcrumb(
    f: &mut Frame,
    area: Rect,
    tab_labels: Vec<String>,
    selected_index: usize,
    focused: bool,
    breadcrumb_items: Option<Vec<String>>,
    breadcrumb_skip: usize,
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

    // If breadcrumb is present and focused, split area into subtabs and breadcrumb
    // Only show breadcrumb if there are more than BREADCRUMB_MIN_DEPTH items
    let should_show_breadcrumb = focused
        && breadcrumb_items.as_ref().map_or(false, |items| items.len() > BREADCRUMB_MIN_DEPTH)
        && area.height >= 3;

    if should_show_breadcrumb {
        let breadcrumb_items = breadcrumb_items.unwrap();

        // Split area: 2 lines for subtabs, 1 line for breadcrumb
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Subtabs + separator
                Constraint::Length(1), // Breadcrumb
            ])
            .split(area);

        // Render subtabs
        let subtab_widget = Paragraph::new(vec![tab_line, separator_line])
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(subtab_widget, chunks[0]);

        // Render breadcrumb using Breadcrumb widget with skip
        let breadcrumb = Breadcrumb::new(breadcrumb_items).with_skip(breadcrumb_skip);
        let breadcrumb_area = Rect::new(0, 0, chunks[1].width, chunks[1].height);
        let mut buf = Buffer::empty(breadcrumb_area);
        breadcrumb.render(breadcrumb_area, &mut buf, display);

        // Copy buffer to frame
        let frame_buf = f.buffer_mut();
        for y in 0..chunks[1].height {
            for x in 0..chunks[1].width {
                let cell = &buf[(x, y)];
                frame_buf[(chunks[1].x + x, chunks[1].y + y)]
                    .set_symbol(cell.symbol())
                    .set_style(cell.style());
            }
        }
    } else {
        // No breadcrumb: just render subtabs
        let widget = Paragraph::new(vec![tab_line, separator_line])
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(widget, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::tui::widgets::testing::assert_buffer;

    #[test]
    fn test_render_subtabs_without_breadcrumb() {
        let display = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 80);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 80);
            render_subtabs_with_breadcrumb(
                f,
                area,
                vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()],
                1,
                false, // not focused = no breadcrumb
                Some(vec!["Test".to_string(), "Breadcrumb".to_string()]),
                0, // skip 0 items
                &display,
            );
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // Line 0: tabs with Tab2 selected (index 1)
        // Line 1: separator line
        assert_buffer(buffer, &[
            "Tab1 │ Tab2 │ Tab3                                                              ",
            "─────┴──────┴───────────────────────────────────────────────────────────────────",
        ], 80);
    }

    #[test]
    fn test_render_subtabs_with_breadcrumb() {
        let display = Arc::new(DisplayConfig::default());
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 3);
            // Test with 3 items to exceed BREADCRUMB_MIN_DEPTH (2)
            render_subtabs_with_breadcrumb(
                f,
                area,
                vec!["Tab1".to_string(), "Tab2".to_string()],
                0,
                true, // focused = show breadcrumb
                Some(vec!["Standings".to_string(), "Division".to_string(), "Maple Leafs".to_string()]),
                BREADCRUMB_MIN_DEPTH, // skip first 2 items
                &display,
            );
        }).unwrap();

        let buffer = terminal.backend().buffer();

        // Line 0: tabs with Tab1 selected (index 0)
        // Line 1: separator line
        // Line 2: breadcrumb showing only "Maple Leafs" (skipped "Standings" and "Division")
        assert_buffer(buffer, &[
            "Tab1 │ Tab2                                                                     ",
            "─────┴──────────────────────────────────────────────────────────────────────────",
            "▸ Maple Leafs                                                                   ",
        ], 80);
    }
}
