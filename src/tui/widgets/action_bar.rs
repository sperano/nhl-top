/// ActionBar widget - displays available keyboard actions at the bottom of the screen
///
/// This widget renders a horizontal bar showing available actions with their keyboard shortcuts.
/// Actions are displayed in the format: "Actions: [Enter] View Team │ [G] Game Log │ [T] Team Page"
/// Enabled actions use the selection color, disabled actions use the unfocused selection color.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// Represents a single keyboard action
#[derive(Debug, Clone)]
pub struct Action {
    /// The keyboard key (e.g., "Enter", "G", "T")
    pub key: String,
    /// The label describing the action (e.g., "View Team", "Game Log")
    pub label: String,
    /// Whether this action is currently enabled
    pub enabled: bool,
}

impl Action {
    /// Create a new enabled action
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            enabled: true,
        }
    }

    /// Create a new disabled action
    pub fn disabled(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            enabled: false,
        }
    }

    /// Set the enabled state of this action
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Widget for displaying keyboard actions as a horizontal bar
#[derive(Debug)]
pub struct ActionBar {
    /// List of actions to display
    pub actions: Vec<Action>,
}

impl ActionBar {
    /// Create a new ActionBar with the given actions
    pub fn new(actions: Vec<Action>) -> Self {
        Self { actions }
    }

    /// Format all actions as a single string for rendering
    fn format_actions(&self, config: &DisplayConfig) -> Vec<(String, Style)> {
        let mut segments = Vec::new();

        // Add "Actions: " prefix in default style
        segments.push((
            "Actions: ".to_string(),
            Style::default().fg(config.division_header_fg),
        ));

        for (idx, action) in self.actions.iter().enumerate() {
            // Add separator between actions (but not before first action)
            if idx > 0 {
                segments.push((
                    " │ ".to_string(),
                    Style::default(),
                ));
            }

            // Key part with brackets: [Enter], [G], etc.
            let key_style = if action.enabled {
                Style::default().fg(config.selection_fg)
            } else {
                Style::default().fg(config.unfocused_selection_fg())
            };

            segments.push((
                format!("[{}]", action.key),
                key_style,
            ));

            // Label part
            let label_style = Style::default().fg(config.division_header_fg);
            segments.push((
                format!(" {}", action.label),
                label_style,
            ));
        }

        segments
    }

    /// Calculate the total width of the formatted actions
    fn calculate_width(&self) -> u16 {
        let mut width = "Actions: ".len();

        for (idx, action) in self.actions.iter().enumerate() {
            if idx > 0 {
                width += " │ ".len();
            }
            width += format!("[{}] {}", action.key, action.label).len();
        }

        width as u16
    }
}

impl RenderableWidget for ActionBar {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.actions.is_empty() || area.width == 0 || area.height == 0 {
            return;
        }

        let segments = self.format_actions(config);
        let total_width = self.calculate_width();

        // Center horizontally
        let start_x = if total_width < area.width {
            area.x + (area.width - total_width) / 2
        } else {
            area.x
        };

        // Render each segment
        let mut current_x = start_x;
        for (text, style) in segments {
            if current_x >= area.x + area.width {
                break;
            }

            buf.set_string(current_x, area.y, &text, style);
            current_x += text.chars().count() as u16;
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(1)
    }

    fn preferred_width(&self) -> Option<u16> {
        Some(self.calculate_width())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_action_bar_empty() {
        let widget = ActionBar::new(vec![]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "                                                                                ",
        ]);
    }

    #[test]
    fn test_action_bar_single_action() {
        let widget = ActionBar::new(vec![
            Action::new("Enter", "View Team"),
        ]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "                           Actions: [Enter] View Team                           ",
        ]);
    }

    #[test]
    fn test_action_bar_multiple_actions() {
        let widget = ActionBar::new(vec![
            Action::new("Enter", "View Team"),
            Action::new("G", "Game Log"),
            Action::new("T", "Team Page"),
        ]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "         Actions: [Enter] View Team │ [G] Game Log │ [T] Team Page              ",
        ]);
    }

    #[test]
    fn test_action_bar_disabled_action() {
        let widget = ActionBar::new(vec![
            Action::new("Enter", "View Team"),
            Action::disabled("G", "Game Log"),
        ]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "                  Actions: [Enter] View Team │ [G] Game Log                     ",
        ]);
    }

    #[test]
    fn test_action_with_enabled() {
        let action = Action::new("Enter", "View Team")
            .with_enabled(false);

        assert_eq!(action.key, "Enter");
        assert_eq!(action.label, "View Team");
        assert!(!action.enabled);
    }

    #[test]
    fn test_action_bar_preferred_dimensions() {
        let widget = ActionBar::new(vec![
            Action::new("Enter", "View Team"),
            Action::new("G", "Game Log"),
        ]);

        // Height should always be 1
        assert_eq!(widget.preferred_height(), Some(1));

        // Width should be the total length of the formatted string
        let width = widget.preferred_width().unwrap();
        assert!(width > 0);
    }

    #[test]
    fn test_action_bar_centering() {
        let widget = ActionBar::new(vec![
            Action::new("A", "Test"),
        ]);
        let buf = render_widget(&widget, 80, 1);

        assert_buffer(&buf, &[
            "                               Actions: [A] Test                                ",
        ]);
    }

    #[test]
    fn test_action_bar_zero_area() {
        let widget = ActionBar::new(vec![
            Action::new("Enter", "View Team"),
        ]);
        let buf = render_widget(&widget, 0, 0);

        // Should not panic with zero area
        assert_eq!(buf.area.width, 0);
    }

    #[test]
    fn test_action_new_defaults_to_enabled() {
        let action = Action::new("Enter", "Test");
        assert!(action.enabled);
    }

    #[test]
    fn test_action_disabled_defaults_to_disabled() {
        let action = Action::disabled("Enter", "Test");
        assert!(!action.enabled);
    }
}
