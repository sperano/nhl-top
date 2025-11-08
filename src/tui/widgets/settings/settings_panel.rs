/// SettingsPanelWidget - top-level composition widget for settings view
///
/// Composes:
/// - Instruction text at top
/// - Settings list in middle
/// - Modals overlaid when needed (list modal or color picker)

use ratatui::{buffer::Buffer, layout::Rect};
use crate::config::DisplayConfig;
use crate::tui::settings::Setting;
use super::{
    render_settings_list,
    render_list_modal,
    render_color_modal,
};

/// State for the settings panel widget
pub struct SettingsPanelWidget<'a> {
    settings: &'a [Setting],
    selected_index: Option<usize>,
    subtab_focused: bool,
    editing: Option<(&'a str, &'a str)>, // (setting_key, edit_buffer)
    list_modal: Option<(&'a str, &'a [String], usize)>, // (setting_name, options, selected_index)
    color_modal: Option<(&'a str, usize, ratatui::style::Color)>, // (setting_name, selected_color_index, current_theme_color)
}

impl<'a> SettingsPanelWidget<'a> {
    /// Create a new settings panel widget
    pub fn new(settings: &'a [Setting]) -> Self {
        Self {
            settings,
            selected_index: None,
            subtab_focused: false,
            editing: None,
            list_modal: None,
            color_modal: None,
        }
    }

    /// Set whether a setting is selected
    pub fn with_selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Set whether the subtab (settings list) is focused
    pub fn with_subtab_focused(mut self, focused: bool) -> Self {
        self.subtab_focused = focused;
        self
    }

    /// Set editing state (setting key and edit buffer)
    pub fn with_editing(mut self, editing: Option<(&'a str, &'a str)>) -> Self {
        self.editing = editing;
        self
    }

    /// Set list modal state (setting name, options, selected index)
    pub fn with_list_modal(mut self, list_modal: Option<(&'a str, &'a [String], usize)>) -> Self {
        self.list_modal = list_modal;
        self
    }

    /// Set color modal state (setting name, selected color index, current theme color)
    pub fn with_color_modal(mut self, color_modal: Option<(&'a str, usize, ratatui::style::Color)>) -> Self {
        self.color_modal = color_modal;
        self
    }

    /// Render the settings panel
    pub fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;

        // Blank line at top
        y += 1;
        if y >= area.bottom() {
            return;
        }

        // Instruction text
        let instruction = if self.subtab_focused {
            "  Use Up/Down to navigate, Enter to select, Up/Esc to exit"
        } else {
            "  Press Down/Enter to edit settings"
        };
        buf.set_string(area.x, y, instruction, ratatui::style::Style::default());
        y += 1;

        // Blank line after instruction
        y += 1;
        if y >= area.bottom() {
            return;
        }

        // Calculate remaining area for settings list
        let list_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: area.height.saturating_sub(y - area.y),
        };

        // Render settings list
        let editing_key = self.editing.map(|(key, _)| key);
        let edit_buffer = self.editing.map(|(_, buffer)| buffer);

        render_settings_list(
            self.settings,
            self.selected_index,
            editing_key,
            edit_buffer,
            2, // margin
            list_area,
            buf,
            config,
        );

        // Render modals (overlaid on top)
        if let Some((setting_name, options, selected_index)) = self.list_modal {
            render_list_modal(
                setting_name,
                options,
                selected_index,
                area,
                buf,
                config,
            );
        }

        if let Some((setting_name, selected_color_index, current_theme_color)) = self.color_modal {
            render_color_modal(
                setting_name,
                selected_color_index,
                current_theme_color,
                area,
                buf,
                config,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::settings::SettingValue;
    use crate::tui::widgets::testing::test_config;
    use ratatui::style::Color;

    fn buffer_to_string(buf: &Buffer, y: u16) -> String {
        let mut result = String::new();
        for x in 0..buf.area.width {
            if let Some(cell) = buf.cell((x, y)) {
                result.push_str(cell.symbol());
            }
        }
        result.trim_end().to_string()
    }

    #[test]
    fn test_settings_panel_basic_render() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Use Unicode".to_string(),
                value: SettingValue::Bool(true),
            },
        ];

        let widget = SettingsPanelWidget::new(&settings);
        widget.render(area, &mut buf, &config);

        // Should have instruction text at line 1
        let line1 = buffer_to_string(&buf, 1);
        assert!(line1.contains("Press Down/Enter"));

        // Should have setting at line 3 (blank, instruction, blank, setting)
        let line3 = buffer_to_string(&buf, 3);
        assert!(line3.contains("Use Unicode"));
    }

    #[test]
    fn test_settings_panel_with_focus() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Test".to_string(),
                value: SettingValue::Bool(true),
            },
        ];

        let widget = SettingsPanelWidget::new(&settings)
            .with_subtab_focused(true);
        widget.render(area, &mut buf, &config);

        // Instruction should change when focused
        let line1 = buffer_to_string(&buf, 1);
        assert!(line1.contains("Use Up/Down"));
    }

    #[test]
    fn test_settings_panel_with_selection() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Setting 1".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Setting 2".to_string(),
                value: SettingValue::Bool(false),
            },
        ];

        let widget = SettingsPanelWidget::new(&settings)
            .with_selected_index(Some(1))
            .with_subtab_focused(true);
        widget.render(area, &mut buf, &config);

        // Second setting should have selection indicator
        let line4 = buffer_to_string(&buf, 4);
        assert!(line4.contains("►"));
        assert!(line4.contains("Setting 2"));
    }

    #[test]
    fn test_settings_panel_with_editing() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Refresh Interval".to_string(),
                value: SettingValue::Int(60),
            },
        ];

        let widget = SettingsPanelWidget::new(&settings)
            .with_selected_index(Some(0))
            .with_subtab_focused(true)
            .with_editing(Some(("Refresh Interval", "12")));
        widget.render(area, &mut buf, &config);

        // Should show edit cursor
        let line3 = buffer_to_string(&buf, 3);
        assert!(line3.contains("12█"));
    }

    #[test]
    fn test_settings_panel_with_list_modal() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Log Level".to_string(),
                value: SettingValue::List {
                    options: vec!["info".to_string(), "debug".to_string()],
                    current_index: 0,
                },
            },
        ];

        let options = vec!["info".to_string(), "debug".to_string()];
        let widget = SettingsPanelWidget::new(&settings)
            .with_list_modal(Some(("Log Level", &options, 0)));
        widget.render(area, &mut buf, &config);

        // Modal should be rendered somewhere in the buffer
        // We'll just check that the modal title appears
        let mut found_title = false;
        for y in 0..buf.area.height {
            let line = buffer_to_string(&buf, y);
            if line.contains("Log Level") {
                found_title = true;
                break;
            }
        }
        assert!(found_title, "Modal title should be visible");
    }

    #[test]
    fn test_settings_panel_with_color_modal() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
        let area = Rect::new(0, 0, 100, 30);

        let settings = vec![
            Setting {
                key: "Selection FG".to_string(),
                value: SettingValue::Color(Color::Red),
            },
        ];

        let widget = SettingsPanelWidget::new(&settings)
            .with_color_modal(Some(("Selection FG", 0, Color::Red)));
        widget.render(area, &mut buf, &config);

        // Modal should be rendered
        let mut found_title = false;
        for y in 0..buf.area.height {
            let line = buffer_to_string(&buf, y);
            if line.contains("Selection FG") {
                found_title = true;
                break;
            }
        }
        assert!(found_title, "Modal title should be visible");
    }

    #[test]
    fn test_settings_panel_multiple_settings() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings = vec![
            Setting {
                key: "Use Unicode".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Refresh Interval".to_string(),
                value: SettingValue::Int(60),
            },
            Setting {
                key: "Log Level".to_string(),
                value: SettingValue::List {
                    options: vec!["info".to_string()],
                    current_index: 0,
                },
            },
        ];

        let widget = SettingsPanelWidget::new(&settings);
        widget.render(area, &mut buf, &config);

        // All three settings should be visible
        let line3 = buffer_to_string(&buf, 3);
        let line4 = buffer_to_string(&buf, 4);
        let line5 = buffer_to_string(&buf, 5);

        assert!(line3.contains("Use Unicode"));
        assert!(line4.contains("Refresh Interval"));
        assert!(line5.contains("Log Level"));
    }

    #[test]
    fn test_settings_panel_empty_settings() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        let area = Rect::new(0, 0, 80, 24);

        let settings: Vec<Setting> = vec![];

        let widget = SettingsPanelWidget::new(&settings);
        widget.render(area, &mut buf, &config);

        // Should still render instruction text
        let line1 = buffer_to_string(&buf, 1);
        assert!(line1.contains("Press Down/Enter"));
    }
}
