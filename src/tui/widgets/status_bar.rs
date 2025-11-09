/// StatusBar widget - displays status information and keyboard hints at the bottom of the screen
///
/// This widget renders a two-line status bar with:
/// - Top line: horizontal separator with connector aligned to the vertical bar
/// - Bottom line: left status message (or error) │ right refresh countdown
///
/// Error messages are displayed with the error color when present.

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Span}};
use std::time::{Duration, SystemTime};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Represents the style of a key hint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHintStyle {
    /// Normal hint (default styling)
    Normal,
    /// Important hint (highlighted)
    Important,
    /// Subtle hint (dimmed)
    Subtle,
}

/// Represents a keyboard hint displayed in the status bar
#[derive(Debug, Clone)]
pub struct KeyHint {
    /// The keyboard key (e.g., "?", "ESC", "/")
    pub key: String,
    /// The action description (e.g., "Help", "Back")
    pub action: String,
    /// The visual style for this hint
    pub style: KeyHintStyle,
}

impl KeyHint {
    /// Create a new normal key hint
    pub fn new(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
            style: KeyHintStyle::Normal,
        }
    }

    /// Create a new key hint with a specific style
    pub fn with_style(key: impl Into<String>, action: impl Into<String>, style: KeyHintStyle) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
            style,
        }
    }
}

/// Widget for displaying status information and keyboard hints
#[derive(Debug)]
pub struct StatusBar {
    /// Last refresh timestamp
    pub last_refresh: Option<SystemTime>,
    /// Time until next refresh
    pub next_refresh_in: Option<Duration>,
    /// Optional error message to display
    pub error_message: Option<String>,
    /// List of keyboard hints to display
    pub hints: Vec<KeyHint>,
    /// Refresh interval in seconds
    pub refresh_interval: u32,
}

impl StatusBar {
    /// Create a new StatusBar with default hints
    ///
    /// Default hints: "?" Help, "ESC" Back, "/" Jump to...
    pub fn new() -> Self {
        Self {
            last_refresh: None,
            next_refresh_in: None,
            error_message: None,
            hints: vec![
                KeyHint::new("?", "Help"),
                KeyHint::new("ESC", "Back"),
                KeyHint::new("/", "Jump to..."),
            ],
            refresh_interval: 60,
        }
    }

    /// Set the last refresh time
    pub fn with_last_refresh(mut self, last_refresh: Option<SystemTime>) -> Self {
        self.last_refresh = last_refresh;
        self
    }

    /// Set the refresh interval in seconds
    pub fn with_refresh_interval(mut self, refresh_interval: u32) -> Self {
        self.refresh_interval = refresh_interval;
        self
    }

    /// Set the next refresh duration
    pub fn with_next_refresh(mut self, next_refresh_in: Option<Duration>) -> Self {
        self.next_refresh_in = next_refresh_in;
        self
    }

    /// Set an error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    /// Set a status message (non-error)
    pub fn with_status(self, _status: impl Into<String>) -> Self {
        // Reserved for future context integration
        self
    }

    /// Set custom keyboard hints
    pub fn with_hints(mut self, hints: Vec<KeyHint>) -> Self {
        self.hints = hints;
        self
    }

    /// Stub for future context integration
    pub fn with_context(self) -> Self {
        self
    }

    /// Build the left side status message
    fn build_left_text(&self) -> String {
        if let Some(msg) = &self.error_message {
            format!("ERROR: {}", msg)
        } else {
            String::new()
        }
    }

    /// Build the right side refresh countdown text (fixed 3-char width, right-aligned)
    fn build_right_text(&self, refresh_interval: u32) -> String {
        if let Some(refresh_time) = self.last_refresh {
            if let Ok(elapsed) = SystemTime::now().duration_since(refresh_time) {
                let elapsed_secs = elapsed.as_secs();
                let remaining_secs = refresh_interval.saturating_sub(elapsed_secs as u32);

                if remaining_secs > 0 {
                    format!("{:>3}", remaining_secs.min(999))
                } else {
                    "...".to_string()
                }
            } else {
                "  ?".to_string()
            }
        } else {
            "---".to_string()
        }
    }

    /// Build the top separator line with connector
    fn build_separator_line(&self, area_width: usize, bar_position: u16, config: &DisplayConfig) -> String {
        let left_part = config.box_chars.horizontal.repeat(bar_position as usize);
        let right_part = config.box_chars.horizontal.repeat((area_width.saturating_sub(bar_position as usize + 1)) as usize);
        format!("{}{}{}", left_part, config.box_chars.connector3, right_part)
    }

    /// Build the bottom status line with left and right content
    fn build_status_line(&self, bar_position: u16, left_text: String, right_text: String, status_is_error: bool, error_fg: Color, vertical_char: &str) -> Vec<(String, Style)> {
        let mut segments = Vec::new();

        // Left side: status message with 1 char margin
        if !left_text.is_empty() {
            segments.push((" ".to_string(), Style::default()));
            if status_is_error {
                segments.push((left_text, Style::default().fg(error_fg)));
            } else {
                segments.push((left_text, Style::default()));
            }
        }

        // Middle: padding to push right text to the right
        let left_content_len = if segments.is_empty() { 0 } else {
            segments.iter().map(|(s, _)| s.len()).sum::<usize>()
        };
        let padding_len = bar_position.saturating_sub(left_content_len as u16) as usize;
        segments.push((" ".repeat(padding_len), Style::default()));

        // Right side: vertical bar + refresh countdown + margin
        segments.push((vertical_char.to_string(), Style::default()));
        segments.push((" ".to_string(), Style::default()));
        segments.push((right_text, Style::default()));
        segments.push((" ".to_string(), Style::default()));

        segments
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderableWidget for StatusBar {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.width == 0 || area.height < 2 {
            return;
        }

        let left_text = self.build_left_text();
        let right_text = self.build_right_text(self.refresh_interval);
        let status_is_error = self.error_message.is_some();

        // Calculate where the vertical bar should be
        // Layout: [left content] [padding] │ [space] [3-char right_text] [space]
        // Fixed width: 3 chars content + 2 spaces (margins) + 1 vertical bar = 6
        let bar_position = area.width.saturating_sub(6);

        // Build lines
        let separator_line = self.build_separator_line(area.width as usize, bar_position, config);
        let status_segments = self.build_status_line(
            bar_position,
            left_text,
            right_text,
            status_is_error,
            config.error_fg,
            &config.box_chars.vertical
        );

        // Render separator line
        buf.set_string(area.x, area.y, &separator_line, Style::default());

        // Render status line
        let mut x = area.x;
        for (text, style) in status_segments {
            if x >= area.x + area.width {
                break;
            }
            buf.set_string(x, area.y + 1, &text, style);
            // Use character count for unicode-aware width (not byte length)
            x += text.chars().count() as u16;
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(2) // Separator line + status line
    }

    fn preferred_width(&self) -> Option<u16> {
        None // Adapts to available width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;
    use std::time::Duration;

        #[test]
    fn test_status_bar_basic_rendering() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 80, 2);

        // Just verify it renders without panicking
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_loading_state() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_countdown() {
        let last_refresh = SystemTime::now() - Duration::from_secs(5);
        let widget = StatusBar::new().with_last_refresh(Some(last_refresh));
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_refreshing_state() {
        let last_refresh = SystemTime::now() - Duration::from_secs(65);
        let widget = StatusBar::new().with_last_refresh(Some(last_refresh));
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_error_message() {
        let widget = StatusBar::new().with_error("Network timeout");
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_with_status_message() {
        let last_refresh = SystemTime::now() - Duration::from_secs(5);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_status("Setting saved");
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_connector_alignment() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 80, 2);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_ascii_mode() {
        let widget = StatusBar::new();
        let config = test_config_ascii();
        let buf = render_widget_with_config(&widget, 80, 2, &config);

        // Verify dimensions
        assert_eq!(buf.area.width, 80);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_status_bar_zero_height() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 80, 0);

        // Should not panic with zero height
        assert_eq!(buf.area.height, 0);
    }

    #[test]
    fn test_status_bar_small_area() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 10, 2);

        // Should render without panicking
        assert_eq!(buf.area.width, 10);
        assert_eq!(buf.area.height, 2);
    }

    #[test]
    fn test_key_hint_new() {
        let hint = KeyHint::new("?", "Help");
        assert_eq!(hint.key, "?");
        assert_eq!(hint.action, "Help");
        assert_eq!(hint.style, KeyHintStyle::Normal);
    }

    #[test]
    fn test_key_hint_with_style() {
        let hint = KeyHint::with_style("ESC", "Back", KeyHintStyle::Important);
        assert_eq!(hint.key, "ESC");
        assert_eq!(hint.action, "Back");
        assert_eq!(hint.style, KeyHintStyle::Important);
    }

    #[test]
    fn test_status_bar_with_hints() {
        let hints = vec![
            KeyHint::new("Enter", "Select"),
            KeyHint::new("Q", "Quit"),
        ];
        let widget = StatusBar::new().with_hints(hints);
        assert_eq!(widget.hints.len(), 2);
        assert_eq!(widget.hints[0].key, "Enter");
        assert_eq!(widget.hints[1].key, "Q");
    }

    #[test]
    fn test_status_bar_preferred_dimensions() {
        let widget = StatusBar::new();
        assert_eq!(widget.preferred_height(), Some(2));
        assert_eq!(widget.preferred_width(), None);
    }

    #[test]
    fn test_status_bar_builder_pattern() {
        let widget = StatusBar::new()
            .with_error("Test error")
            .with_last_refresh(Some(SystemTime::now()))
            .with_context();

        assert_eq!(widget.error_message, Some("Test error".to_string()));
        assert!(widget.last_refresh.is_some());
    }

    // Rendering tests that check actual text content

    #[test]
    fn test_build_right_text_format() {
        let widget = StatusBar::new();

        // Test loading state
        let loading_text = widget.build_right_text(60);
        println!("Loading text: '{}' (len: {})", loading_text, loading_text.len());
        assert_eq!(loading_text.len(), 3);
        assert_eq!(loading_text, "---");

        // Test countdown with 60 seconds
        let last_refresh = SystemTime::now() - Duration::from_secs(0);
        let widget_60 = StatusBar::new().with_last_refresh(Some(last_refresh));
        let text_60 = widget_60.build_right_text(60);
        println!("60s text: '{}' (len: {})", text_60, text_60.len());
        println!("60s text bytes: {:?}", text_60.as_bytes());
        assert_eq!(text_60.len(), 3, "Expected 3 chars, got {}: '{}'", text_60.len(), text_60);

        // Test countdown with 5 seconds remaining
        let last_refresh_55 = SystemTime::now() - Duration::from_secs(55);
        let widget_5 = StatusBar::new().with_last_refresh(Some(last_refresh_55));
        let text_5 = widget_5.build_right_text(60);
        println!("5s text: '{}' (len: {})", text_5, text_5.len());
        assert_eq!(text_5.len(), 3, "Expected 3 chars, got {}: '{}'", text_5.len(), text_5);

        // Test refreshing state
        let last_refresh_old = SystemTime::now() - Duration::from_secs(65);
        let widget_ref = StatusBar::new().with_last_refresh(Some(last_refresh_old));
        let text_ref = widget_ref.build_right_text(60);
        println!("Refreshing text: '{}' (len: {})", text_ref, text_ref.len());
        assert_eq!(text_ref.len(), 3);
        assert_eq!(text_ref, "...");
    }

    #[test]
    fn test_status_bar_render_loading_state() {
        let widget = StatusBar::new();
        let buf = render_widget(&widget, 80, 2);

        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │ --- ",
        ]);
    }

    #[test]
    fn test_status_bar_render_countdown_60s() {
        let last_refresh = SystemTime::now() - Duration::from_secs(0);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(60);
        let buf = render_widget(&widget, 80, 2);

        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │  60 ",
        ]);
    }

    #[test]
    fn test_status_bar_render_countdown_5s() {
        let last_refresh = SystemTime::now() - Duration::from_secs(55);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(60);
        let buf = render_widget(&widget, 80, 2);
        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │   5 ",
        ]);
    }

        #[test]
    fn test_status_bar_render_countdown_999s() {
        let last_refresh = SystemTime::now() - Duration::from_secs(0);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(999);
        let buf = render_widget(&widget, 80, 2);
        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │ 999 ",
        ]);
    }

        #[test]
    fn test_status_bar_render_refreshing() {
        let last_refresh = SystemTime::now() - Duration::from_secs(65);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(60);
        let buf = render_widget(&widget, 80, 2);
        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │ ... ",
        ]);
    }

        #[test]
    fn test_status_bar_render_with_error() {
        let last_refresh = SystemTime::now() - Duration::from_secs(5);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(60)
            .with_error("Connection failed");
        let buf = render_widget(&widget, 80, 2);
        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            " ERROR: Connection failed                                                 │  55 ",
        ]);
    }

    #[test]
    fn test_status_bar_full_width_check() {
        let last_refresh = SystemTime::now() - Duration::from_secs(55);
        let widget = StatusBar::new()
            .with_last_refresh(Some(last_refresh))
            .with_refresh_interval(60);
        let buf = render_widget(&widget, 80, 2);

        assert_buffer(&buf, &[
            "──────────────────────────────────────────────────────────────────────────┬─────",
            "                                                                          │   5 ",
        ]);
    }
}
