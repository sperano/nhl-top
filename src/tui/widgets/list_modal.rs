/// ListModalWidget - renders a centered popup modal for list selection
///
/// Features:
/// - Centered modal positioning
/// - Clear background behind modal
/// - Border with selection color
/// - Selection indicator for current option

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Widget},
};
use crate::config::DisplayConfig;
use crate::tui::component::RenderableWidget;

/// Widget for rendering a list selection modal
#[derive(Clone)]
pub struct ListModalWidget {
    pub setting_name: String,
    pub options: Vec<String>,
    pub selected_index: usize,
}

impl ListModalWidget {
    pub fn new(setting_name: impl Into<String>, options: Vec<String>, selected_index: usize) -> Self {
        Self {
            setting_name: setting_name.into(),
            options,
            selected_index,
        }
    }
}

impl RenderableWidget for ListModalWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        render_list_modal(
            &self.setting_name,
            &self.options,
            self.selected_index,
            area,
            buf,
            config,
        );
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(self.clone())
    }
}

/// Renders a centered list selection modal
///
/// Returns the modal area that was rendered
pub fn render_list_modal(
    setting_name: &str,
    options: &[String],
    selected_index: usize,
    area: Rect,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> Rect {
    // Calculate modal size
    let modal_height = options.len() as u16 + 4; // +4 for borders and title
    let max_option_len = options.iter().map(|s| s.len()).max().unwrap_or(20);
    let modal_width = max_option_len.max(setting_name.len()) as u16 + 6;

    // Center the modal
    let vertical_margin = (area.height.saturating_sub(modal_height)) / 2;
    let horizontal_margin = (area.width.saturating_sub(modal_width)) / 2;

    let modal_area = Rect {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Clear the area behind the modal
    Clear.render(modal_area, buf);

    // Render border
    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.selection_fg));
    border_block.render(modal_area, buf);

    // Calculate inner area (inside borders)
    let inner = Rect {
        x: modal_area.x + 1,
        y: modal_area.y + 1,
        width: modal_area.width.saturating_sub(2),
        height: modal_area.height.saturating_sub(2),
    };

    let mut y = inner.y;

    // Render title
    if y < inner.bottom() {
        let title = format!(" {} ", setting_name);
        buf.set_string(inner.x, y, &title, Style::default().fg(Color::White));
        y += 1;
    }

    // Blank line after title
    y += 1;

    // Render options
    for (idx, option) in options.iter().enumerate() {
        if y >= inner.bottom() {
            break;
        }

        let is_selected = idx == selected_index;

        if is_selected {
            buf.set_string(inner.x, y, " ► ", Style::default().fg(config.selection_fg));
            buf.set_string(inner.x + 3, y, option, Style::default().fg(Color::White));
        } else {
            buf.set_string(inner.x, y, "   ", Style::default());
            buf.set_string(inner.x + 3, y, option, Style::default().fg(Color::Gray));
        }

        y += 1;
    }

    modal_area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::test_config;

    fn buffer_to_string(buf: &Buffer, y: u16, x_start: u16, x_end: u16) -> String {
        let mut result = String::new();
        for x in x_start..x_end {
            if let Some(cell) = buf.cell((x, y)) {
                result.push_str(cell.symbol());
            }
        }
        result.trim_end().to_string()
    }

    #[test]
    fn test_list_modal_basic_render() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec!["Option 1".to_string(), "Option 2".to_string()];

        let modal_area = render_list_modal(
            "Test Setting",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Modal should be centered
        assert!(modal_area.x > 0);
        assert!(modal_area.y > 0);
        assert!(modal_area.width < area.width);
        assert!(modal_area.height < area.height);
    }

    #[test]
    fn test_list_modal_title() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec!["Option 1".to_string()];

        let modal_area = render_list_modal(
            "Log Level",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Title should be visible (y+1 is inside border)
        let title_line = buffer_to_string(&buf, modal_area.y + 1, modal_area.x, modal_area.x + modal_area.width);
        assert!(title_line.contains("Log Level"));
    }

    #[test]
    fn test_list_modal_selection_first() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec!["First".to_string(), "Second".to_string()];

        let modal_area = render_list_modal(
            "Test",
            &options,
            0, // Select first
            area,
            &mut buf,
            &config,
        );

        // First option should have selection indicator (y+3: title, blank, first option)
        let first_line = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(first_line.contains("►"));
        assert!(first_line.contains("First"));

        // Second option should not have selection indicator
        let second_line = buffer_to_string(&buf, modal_area.y + 4, modal_area.x, modal_area.x + modal_area.width);
        assert!(!second_line.contains("►"));
        assert!(second_line.contains("Second"));
    }

    #[test]
    fn test_list_modal_selection_second() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec!["First".to_string(), "Second".to_string()];

        let modal_area = render_list_modal(
            "Test",
            &options,
            1, // Select second
            area,
            &mut buf,
            &config,
        );

        // First option should not have selection indicator
        let first_line = buffer_to_string(&buf, modal_area.y + 3, modal_area.x, modal_area.x + modal_area.width);
        assert!(!first_line.contains("►"));

        // Second option should have selection indicator
        let second_line = buffer_to_string(&buf, modal_area.y + 4, modal_area.x, modal_area.x + modal_area.width);
        assert!(second_line.contains("►"));
        assert!(second_line.contains("Second"));
    }

    #[test]
    fn test_list_modal_sizing() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec![
            "Short".to_string(),
            "Very Long Option Name Here".to_string(),
        ];

        let modal_area = render_list_modal(
            "Test",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Width should accommodate the longest option
        // +6 for borders and margins
        assert!(modal_area.width >= "Very Long Option Name Here".len() as u16 + 6);

        // Height should be: 2 options + 4 (borders + title + blank)
        assert_eq!(modal_area.height, 6);
    }

    #[test]
    fn test_list_modal_empty_options() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options: Vec<String> = vec![];

        let modal_area = render_list_modal(
            "Empty",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Should still render with minimal height (borders + title)
        assert_eq!(modal_area.height, 4); // borders + title + blank
    }

    #[test]
    fn test_list_modal_centering() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let options = vec!["Option".to_string()];

        let modal_area = render_list_modal(
            "Test",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Modal should be roughly centered
        let horizontal_margin = modal_area.x - area.x;
        let expected_horizontal_margin = (area.width - modal_area.width) / 2;
        assert_eq!(horizontal_margin, expected_horizontal_margin);

        let vertical_margin = modal_area.y - area.y;
        let expected_vertical_margin = (area.height - modal_area.height) / 2;
        assert_eq!(vertical_margin, expected_vertical_margin);
    }

    #[test]
    fn test_list_modal_widget_new() {
        let options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let widget = ListModalWidget::new("Test Setting", options.clone(), 1);

        assert_eq!(widget.setting_name, "Test Setting");
        assert_eq!(widget.options, options);
        assert_eq!(widget.selected_index, 1);
    }

    #[test]
    fn test_list_modal_widget_render() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let options = vec!["First".to_string(), "Second".to_string()];
        let widget = ListModalWidget::new("Widget Test", options, 0);

        widget.render(area, &mut buf, &config);

        // Widget should have rendered via RenderableWidget trait
        // Check that something was rendered in the buffer
        let has_content = (0..buf.area.height).any(|y| {
            (0..buf.area.width).any(|x| {
                if let Some(cell) = buf.cell((x, y)) {
                    !cell.symbol().trim().is_empty()
                } else {
                    false
                }
            })
        });

        assert!(has_content, "Widget should have rendered content");
    }

    #[test]
    fn test_list_modal_widget_clone_box() {
        let options = vec!["Option".to_string()];
        let widget = ListModalWidget::new("Test", options, 0);

        let _cloned: Box<dyn RenderableWidget> = widget.clone_box();
        // If we get here, clone_box() worked
    }

    #[test]
    fn test_list_modal_truncation_when_too_tall() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8)); // Small height
        let area = Rect::new(0, 0, 40, 8);

        // Many options that won't fit
        let options = vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
            "Option 4".to_string(),
            "Option 5".to_string(),
            "Option 6".to_string(),
            "Option 7".to_string(),
            "Option 8".to_string(),
            "Option 9".to_string(),
            "Option 10".to_string(),
        ];

        let modal_area = render_list_modal(
            "Many Options",
            &options,
            0,
            area,
            &mut buf,
            &config,
        );

        // Modal height should be limited by available area
        assert!(modal_area.height <= area.height);

        // Should render without panicking even when options are truncated
    }

    #[test]
    fn test_list_modal_widget_with_string_ref() {
        let options = vec!["Test".to_string()];
        let widget = ListModalWidget::new("String ref test", options, 0);
        assert_eq!(widget.setting_name, "String ref test");
    }
}
