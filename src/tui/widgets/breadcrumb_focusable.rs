/// Interactive breadcrumb widget with clickable segments
///
/// This widget provides a navigation breadcrumb with keyboard navigation support.
/// Each segment can be clickable and trigger navigation actions.

use super::focus::*;
use crate::config::DisplayConfig;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

/// An interactive breadcrumb widget with clickable segments
///
/// The breadcrumb shows a navigation trail with segments separated by a separator.
/// Users can navigate between segments with Left/Right arrows and activate them with Enter.
pub struct BreadcrumbWidget {
    id: WidgetId,
    /// Breadcrumb segments
    segments: Vec<BreadcrumbSegment>,
    /// Currently focused segment index
    focused_segment: usize,
    /// Whether widget has focus
    focused: bool,
    /// Separator between segments
    separator: String,
    /// Style configuration
    style: BreadcrumbStyle,
}

/// A single breadcrumb segment
pub struct BreadcrumbSegment {
    /// Display text
    pub text: String,
    /// Whether this segment is clickable
    pub clickable: bool,
    /// Optional action when clicked
    pub action: Option<Box<dyn FnMut() -> NavigationAction + Send>>,
}

/// Visual styling for breadcrumbs
#[derive(Debug, Clone)]
pub struct BreadcrumbStyle {
    /// Show separator line below breadcrumb
    pub show_separator_line: bool,
    /// Style for focused segment
    pub focused_style: Style,
    /// Style for unfocused segments
    pub normal_style: Style,
    /// Focus indicator prefix
    pub focus_indicator: String,
}

impl Default for BreadcrumbStyle {
    fn default() -> Self {
        Self {
            show_separator_line: true,
            focused_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default(),
            focus_indicator: "▶ ".to_string(),
        }
    }
}

impl BreadcrumbSegment {
    /// Create a new breadcrumb segment
    pub fn new(text: impl Into<String>, clickable: bool) -> Self {
        Self {
            text: text.into(),
            clickable,
            action: None,
        }
    }

    /// Create a clickable segment with an action
    pub fn with_action<F>(text: impl Into<String>, action: F) -> Self
    where
        F: FnMut() -> NavigationAction + Send + 'static,
    {
        Self {
            text: text.into(),
            clickable: true,
            action: Some(Box::new(action)),
        }
    }
}

impl BreadcrumbWidget {
    /// Create a new empty breadcrumb widget
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            segments: Vec::new(),
            focused_segment: 0,
            focused: false,
            separator: " ▸ ".to_string(),
            style: BreadcrumbStyle::default(),
        }
    }

    /// Set the separator string
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Set the visual style
    pub fn with_style(mut self, style: BreadcrumbStyle) -> Self {
        self.style = style;
        self
    }

    /// Add a segment to the breadcrumb
    pub fn add_segment(&mut self, text: impl Into<String>, clickable: bool) {
        self.segments.push(BreadcrumbSegment::new(text, clickable));
    }

    /// Add a segment with an action callback
    pub fn add_segment_with_action<F>(&mut self, text: impl Into<String>, action: F)
    where
        F: FnMut() -> NavigationAction + Send + 'static,
    {
        self.segments.push(BreadcrumbSegment::with_action(text, action));
    }

    /// Create a breadcrumb from a trail (for compatibility)
    ///
    /// All segments except the last are clickable and trigger PopPanel.
    pub fn from_trail(trail: &[String]) -> Self {
        let mut breadcrumb = Self::new();

        for (i, segment) in trail.iter().enumerate() {
            let is_last = i == trail.len() - 1;
            if is_last {
                // Last segment is not clickable (current page)
                breadcrumb.add_segment(segment.clone(), false);
            } else {
                // Earlier segments pop back to that level
                breadcrumb.add_segment_with_action(segment.clone(), || NavigationAction::PopPanel);
            }
        }

        breadcrumb
    }

    /// Get the number of segments
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if the breadcrumb is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Move focus to the left segment
    fn move_left(&mut self) -> bool {
        if self.focused_segment > 0 {
            self.focused_segment -= 1;
            true
        } else {
            false
        }
    }

    /// Move focus to the right segment
    fn move_right(&mut self) -> bool {
        if self.focused_segment + 1 < self.segments.len() {
            self.focused_segment += 1;
            true
        } else {
            false
        }
    }
}

impl Default for BreadcrumbWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Focusable for BreadcrumbWidget {
    fn widget_id(&self) -> WidgetId {
        self.id
    }

    fn can_focus(&self) -> bool {
        !self.segments.is_empty()
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        if !self.focused {
            return InputResult::NotHandled;
        }

        match event.code {
            KeyCode::Left => {
                if self.move_left() {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Left)
                }
            }
            KeyCode::Right => {
                if self.move_right() {
                    InputResult::Handled
                } else {
                    InputResult::MoveFocus(FocusDirection::Right)
                }
            }
            KeyCode::Enter => {
                // Activate current segment if clickable
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    if segment.clickable {
                        if let Some(ref mut action) = segment.action {
                            let nav_action = action();
                            return InputResult::Navigate(nav_action);
                        }
                    }
                }
                InputResult::Handled
            }
            KeyCode::Down => {
                // Give up focus to content below
                InputResult::MoveFocus(FocusDirection::Next)
            }
            KeyCode::Up => {
                // Give up focus to tabs above
                InputResult::MoveFocus(FocusDirection::Previous)
            }
            _ => InputResult::NotHandled,
        }
    }
}

impl super::RenderableWidget for BreadcrumbWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if area.height == 0 || area.width == 0 || self.segments.is_empty() {
            return;
        }

        let mut x = area.x;
        let y = area.y;

        // Render segments
        for (i, segment) in self.segments.iter().enumerate() {
            if x >= area.right() {
                break;
            }

            // Determine style
            let is_focused_segment = self.focused && i == self.focused_segment;
            let segment_style = if is_focused_segment {
                self.style.focused_style.patch(Style::default().fg(config.selection_fg))
            } else {
                self.style.normal_style
            };

            // Render focus indicator
            if is_focused_segment {
                let indicator_width = self.style.focus_indicator.len() as u16;
                if x + indicator_width <= area.right() {
                    buf.set_string(x, y, &self.style.focus_indicator, segment_style);
                    x += indicator_width;
                }
            }

            // Render segment text
            let text_width = segment.text.len() as u16;
            if x + text_width <= area.right() {
                buf.set_string(x, y, &segment.text, segment_style);
                x += text_width;
            }

            // Render separator (except after last segment)
            if i < self.segments.len() - 1 {
                let sep_width = self.separator.len() as u16;
                if x + sep_width <= area.right() {
                    buf.set_string(x, y, &self.separator, Style::default());
                    x += sep_width;
                }
            }
        }

        // Render separator line if enabled
        if self.style.show_separator_line && area.height > 1 {
            let sep_char = if config.use_unicode { "─" } else { "-" };
            for col in area.x..area.right() {
                buf.set_string(col, y + 1, sep_char, Style::default());
            }
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        if self.style.show_separator_line {
            Some(2) // Breadcrumb + separator line
        } else {
            Some(1) // Just breadcrumb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> DisplayConfig {
        DisplayConfig::default()
    }

    #[test]
    fn test_breadcrumb_creation() {
        let breadcrumb = BreadcrumbWidget::new();
        assert_eq!(breadcrumb.len(), 0);
        assert!(breadcrumb.is_empty());
        assert!(!breadcrumb.can_focus());
        assert!(!breadcrumb.is_focused());
    }

    #[test]
    fn test_breadcrumb_add_segment() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment("Home", true);
        breadcrumb.add_segment("Teams", true);

        assert_eq!(breadcrumb.len(), 2);
        assert!(!breadcrumb.is_empty());
        assert!(breadcrumb.can_focus());
    }

    #[test]
    fn test_breadcrumb_from_trail() {
        let trail = vec!["Home".to_string(), "Teams".to_string(), "TOR".to_string()];
        let breadcrumb = BreadcrumbWidget::from_trail(&trail);

        assert_eq!(breadcrumb.len(), 3);
        assert!(breadcrumb.segments[0].clickable); // Home is clickable
        assert!(breadcrumb.segments[1].clickable); // Teams is clickable
        assert!(!breadcrumb.segments[2].clickable); // Current page not clickable
    }

    #[test]
    fn test_breadcrumb_navigation_left_right() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment("Seg1", true);
        breadcrumb.add_segment("Seg2", true);
        breadcrumb.add_segment("Seg3", true);
        breadcrumb.set_focused(true);

        assert_eq!(breadcrumb.focused_segment, 0);

        // Move right
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(breadcrumb.focused_segment, 1);

        // Move right again
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(breadcrumb.focused_segment, 2);

        // At end - should return MoveFocus
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::MoveFocus(FocusDirection::Right));
        assert_eq!(breadcrumb.focused_segment, 2); // Stays at end

        // Move left
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(breadcrumb.focused_segment, 1);

        // Move left again
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
        assert_eq!(breadcrumb.focused_segment, 0);

        // At start - should return MoveFocus
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::MoveFocus(FocusDirection::Left));
    }

    #[test]
    fn test_breadcrumb_activation() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment_with_action("Home", || NavigationAction::PopPanel);
        breadcrumb.set_focused(true);

        // Press Enter on clickable segment
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE));
        match result {
            InputResult::Navigate(NavigationAction::PopPanel) => {
                // Success!
            }
            _ => panic!("Expected Navigate(PopPanel) action"),
        }
    }

    #[test]
    fn test_breadcrumb_activation_non_clickable() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment("Current Page", false);
        breadcrumb.set_focused(true);

        // Press Enter on non-clickable segment - should just be handled
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::Handled);
    }

    #[test]
    fn test_breadcrumb_up_down() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment("Test", true);
        breadcrumb.set_focused(true);

        // Down - give up focus
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::MoveFocus(FocusDirection::Next));

        // Up - give up focus
        let result = breadcrumb.handle_input(KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE));
        assert_eq!(result, InputResult::MoveFocus(FocusDirection::Previous));
    }

    #[test]
    fn test_breadcrumb_focus_state() {
        let mut breadcrumb = BreadcrumbWidget::new();
        breadcrumb.add_segment("Test", true);

        assert!(!breadcrumb.is_focused());

        breadcrumb.set_focused(true);
        assert!(breadcrumb.is_focused());

        breadcrumb.set_focused(false);
        assert!(!breadcrumb.is_focused());
    }

    #[test]
    fn test_breadcrumb_widget_id_unique() {
        let bc1 = BreadcrumbWidget::new();
        let bc2 = BreadcrumbWidget::new();

        assert_ne!(bc1.widget_id(), bc2.widget_id());
    }

    #[test]
    fn test_breadcrumb_custom_separator() {
        let breadcrumb = BreadcrumbWidget::new().with_separator(" > ");
        assert_eq!(breadcrumb.separator, " > ");
    }

    #[test]
    fn test_breadcrumb_preferred_height() {
        use super::super::RenderableWidget;

        let breadcrumb = BreadcrumbWidget::new();
        assert_eq!(breadcrumb.preferred_height(), Some(2)); // With separator line

        let no_sep = BreadcrumbWidget::new().with_style(BreadcrumbStyle {
            show_separator_line: false,
            ..Default::default()
        });
        assert_eq!(no_sep.preferred_height(), Some(1)); // Without separator line
    }
}
