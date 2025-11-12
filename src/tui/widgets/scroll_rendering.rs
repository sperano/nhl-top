/// Scroll rendering utilities - adds scrolling to any widget
///
/// This provides a reusable pattern for rendering widgets with scroll support.
/// It handles:
/// - Viewport management
/// - Scroll offset calculation
/// - Content height tracking
/// - Direct rendering to frame buffer with clipping

use ratatui::{layout::Rect, Frame};
use crate::config::DisplayConfig;
use crate::tui::common::scrollable::Scrollable;
use crate::tui::widgets::RenderableWidget;

/// Renders a widget with scrolling support
///
/// This function:
/// 1. Updates the scrollable viewport height
/// 2. Calculates scroll offset
/// 3. Renders the widget at the scrolled position
/// 4. Updates content height for future scrolling
///
/// # Arguments
///
/// * `widget` - The widget to render with scrolling
/// * `f` - The frame to render into
/// * `area` - The area to render into
/// * `scrollable` - The scrollable state (maintains scroll position)
/// * `config` - Display configuration
/// * `blank_line_at_top` - Whether to add a blank line at the top (e.g., after breadcrumb)
pub fn render_scrollable_widget<W: RenderableWidget>(
    widget: &W,
    f: &mut Frame,
    area: Rect,
    scrollable: &mut Scrollable,
    config: &DisplayConfig,
    blank_line_at_top: bool,
) {
    // Update scrollable viewport
    scrollable.update_viewport_height(area.height);

    // Calculate scroll offset
    let scroll_offset = scrollable.scroll_offset as i32;

    // Start rendering position (accounting for scroll and optional blank line)
    let mut y = area.y as i32 - scroll_offset;
    if blank_line_at_top {
        y += 1;
    }

    // Get widget height
    let widget_height = widget.preferred_height().unwrap_or(area.height);

    // Only render if widget is visible in viewport
    if y + widget_height as i32 >= area.y as i32 && y < area.bottom() as i32 {
        let widget_area = Rect::new(
            area.x,
            y.max(area.y as i32) as u16,
            area.width,
            widget_height.min((area.bottom() as i32 - y.max(area.y as i32)).max(0) as u16),
        );
        widget.render(widget_area, f.buffer_mut(), config);
    }

    // Update content height for scrolling
    let content_height = if blank_line_at_top {
        (widget_height as i32 + 1 + scroll_offset) as usize
    } else {
        (widget_height as i32 + scroll_offset) as usize
    };
    scrollable.update_content_height(content_height);
}

/// Renders multiple widgets vertically with scrolling support
///
/// This is useful when you need to render several widgets in a scrollable container,
/// such as a header, content, and footer.
///
/// # Arguments
///
/// * `widgets` - Vector of widgets to render vertically
/// * `f` - The frame to render into
/// * `area` - The area to render into
/// * `scrollable` - The scrollable state (maintains scroll position)
/// * `config` - Display configuration
/// * `blank_line_at_top` - Whether to add a blank line at the top (e.g., after breadcrumb)
pub fn render_scrollable_widgets(
    widgets: Vec<&dyn RenderableWidget>,
    f: &mut Frame,
    area: Rect,
    scrollable: &mut Scrollable,
    config: &DisplayConfig,
    blank_line_at_top: bool,
) {
    // Update scrollable viewport
    scrollable.update_viewport_height(area.height);

    // Calculate scroll offset
    let scroll_offset = scrollable.scroll_offset as i32;

    // Start rendering position
    let mut y = area.y as i32 - scroll_offset;
    if blank_line_at_top {
        y += 1;
    }

    let buf = f.buffer_mut();

    // Render each widget sequentially
    for widget in widgets {
        let widget_height = widget.preferred_height().unwrap_or(10);

        // Only render if visible in viewport
        if y + widget_height as i32 >= area.y as i32 && y < area.bottom() as i32 {
            let widget_area = Rect::new(
                area.x,
                y.max(area.y as i32) as u16,
                area.width,
                widget_height.min((area.bottom() as i32 - y.max(area.y as i32)).max(0) as u16),
            );
            widget.render(widget_area, buf, config);
        }

        y += widget_height as i32;
    }

    // Update content height for scrolling
    let total_height = if blank_line_at_top {
        (y - area.y as i32 + scroll_offset + 1) as usize
    } else {
        (y - area.y as i32 + scroll_offset) as usize
    };
    scrollable.update_content_height(total_height);
}
