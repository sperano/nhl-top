use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::SystemTime;

use crate::config::DisplayConfig;
use crate::tui::framework::{
    component::{Component, Element, RenderableWidget},
    state::SystemState,
};

/// StatusBar component - renders status bar with refresh countdown and error messages
///
/// Left side: status/error messages
/// Right side: refresh countdown
pub struct StatusBar;

impl Component for StatusBar {
    type Props = SystemState;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(StatusBarWidget {
            last_refresh: props.last_refresh,
            refresh_interval: props.config.refresh_interval,
            error_message: None, // TODO: Get from props when we have error state
        }))
    }
}

/// Renderable widget for StatusBar
struct StatusBarWidget {
    last_refresh: Option<SystemTime>,
    refresh_interval: u32,
    error_message: Option<String>,
}

impl RenderableWidget for StatusBarWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let mut lines = Vec::new();

        // Left side: status message (if any)
        let left_text = if let Some(msg) = &self.error_message {
            format!("ERROR: {}", msg)
        } else {
            String::new()
        };

        // Right side: countdown to next refresh
        let right_text = if let Some(refresh_time) = self.last_refresh {
            if let Ok(elapsed) = SystemTime::now().duration_since(refresh_time) {
                let elapsed_secs = elapsed.as_secs();
                let remaining_secs = self.refresh_interval.saturating_sub(elapsed_secs as u32);

                if remaining_secs > 0 {
                    format!("Refresh in {}s", remaining_secs)
                } else {
                    "Refreshing...".to_string()
                }
            } else {
                "Refresh in ?s".to_string()
            }
        } else {
            "Loading...".to_string()
        };

        // Calculate where the vertical bar should be
        let right_text_with_margin = format!("{} ", right_text);
        let bar_position = area.width.saturating_sub(right_text_with_margin.len() as u16 + 1);

        // First line: horizontal separator with connector
        let left_part = "─".repeat(bar_position as usize);
        let right_part = "─".repeat((area.width.saturating_sub(bar_position + 1)) as usize);
        let line1 = format!("{}┬{}", left_part, right_part);
        lines.push(Line::raw(line1));

        // Second line: status message on left, refresh on right
        let mut line2_spans = Vec::new();

        // Left side: status message
        if !left_text.is_empty() {
            line2_spans.push(Span::raw(" "));
            line2_spans.push(Span::styled(
                &left_text,
                Style::default().fg(Color::Red),
            ));
        }

        // Middle: padding
        let left_content_len = if left_text.is_empty() {
            0
        } else {
            left_text.len() + 1
        };
        let padding_len = bar_position.saturating_sub(left_content_len as u16) as usize;
        line2_spans.push(Span::raw(" ".repeat(padding_len)));

        // Right side: vertical bar + refresh text
        line2_spans.push(Span::raw("│"));
        line2_spans.push(Span::raw(" "));
        line2_spans.push(Span::raw(&right_text));
        line2_spans.push(Span::raw(" "));

        lines.push(Line::from(line2_spans));

        let status_bar = Paragraph::new(lines);
        ratatui::widgets::Widget::render(status_bar, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(StatusBarWidget {
            last_refresh: self.last_refresh,
            refresh_interval: self.refresh_interval,
            error_message: self.error_message.clone(),
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(2) // Separator line + status line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui::buffer::Buffer;

    #[test]
    fn test_status_bar_renders_loading() {
        let status_bar = StatusBar;
        let system_state = SystemState {
            last_refresh: None,
            config: Config::default(),
        };

        let element = status_bar.view(&system_state, &());

        match element {
            Element::Widget(widget) => {
                let mut buf = Buffer::empty(Rect::new(0, 0, 80, 2));
                widget.render(Rect::new(0, 0, 80, 2), &mut buf, &DisplayConfig::default());

                let line2: String = buf
                    .content()
                    .iter()
                    .skip(80)
                    .take(80)
                    .map(|cell| cell.symbol())
                    .collect();

                assert!(line2.contains("Loading..."));
                assert!(line2.contains("│"));
            }
            _ => panic!("Expected widget element"),
        }
    }

    #[test]
    fn test_status_bar_renders_countdown() {
        let status_bar = StatusBar;
        let system_state = SystemState {
            last_refresh: Some(SystemTime::now() - std::time::Duration::from_secs(5)),
            config: Config::default(),
        };

        let element = status_bar.view(&system_state, &());

        match element {
            Element::Widget(widget) => {
                let mut buf = Buffer::empty(Rect::new(0, 0, 80, 2));
                widget.render(Rect::new(0, 0, 80, 2), &mut buf, &DisplayConfig::default());

                let line2: String = buf
                    .content()
                    .iter()
                    .skip(80)
                    .take(80)
                    .map(|cell| cell.symbol())
                    .collect();

                assert!(line2.contains("Refresh in"));
            }
            _ => panic!("Expected widget element"),
        }
    }

    /// Helper function to assert buffer contents match expected lines
    fn assert_buffer(buf: &Buffer, expected: &[&str], width: usize) {
        let area = buf.area();

        assert_eq!(
            area.width as usize, width,
            "Buffer width mismatch: expected {}, got {}",
            width, area.width
        );

        assert_eq!(
            area.height as usize, expected.len(),
            "Buffer height mismatch: expected {}, got {}",
            expected.len(), area.height
        );

        for (y, expected_line) in expected.iter().enumerate() {
            let actual_line: String = (0..area.width)
                .map(|x| buf.get(x, y as u16).symbol())
                .collect();

            assert_eq!(
                expected_line.chars().count(), width,
                "Expected line {} has wrong character count: expected {}, got {}",
                y, width, expected_line.chars().count()
            );

            assert_eq!(
                &actual_line, expected_line,
                "Line {} mismatch:\nExpected: {:?}\nActual:   {:?}",
                y, expected_line, actual_line
            );
        }
    }

    #[test]
    fn test_status_bar_renders_2_lines_with_assert_buffer() {
        let status_bar = StatusBar;
        let system_state = SystemState {
            last_refresh: Some(SystemTime::now() - std::time::Duration::from_secs(5)),
            config: Config::default(),
        };

        let element = status_bar.view(&system_state, &());

        match element {
            Element::Widget(widget) => {
                let mut buf = Buffer::empty(Rect::new(0, 0, 80, 2));
                widget.render(Rect::new(0, 0, 80, 2), &mut buf, &DisplayConfig::default());

                // Test that StatusBar renders 2 lines: separator + content
                // Line 0: horizontal separator with ┬ connector at position where vertical bar will be
                // Line 1: left content (empty) + padding + │ + right content (refresh countdown)
                assert_buffer(&buf, &[
                    "────────────────────────────────────────────────────────────────┬───────────────",
                    "                                                                │ Refresh in 55s",
                ], 80);
            }
            _ => panic!("Expected widget element"),
        }
    }
}
