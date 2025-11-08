/// SettingRowWidget - renders a single setting row
///
/// Composes: margin + selection indicator + key label + value widget
/// Always renders as 1 line.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::settings::{Setting, SettingValue};
use super::{
    render_bool_value, render_int_value, render_string_value,
    render_list_value, render_color_value,
};

/// Renders a single setting row
///
/// Returns the height consumed (always 1)
pub fn render_setting_row(
    setting: &Setting,
    is_selected: bool,
    editing_key: Option<&str>,
    edit_buffer: Option<&str>,
    max_key_width: usize,
    margin: u16,
    area: Rect,
    y: u16,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> u16 {
    if y >= area.bottom() {
        return 0;
    }

    let mut x = area.x;

    // Render left margin
    buf.set_string(x, y, &" ".repeat(margin as usize), Style::default());
    x += margin;

    // Render selection indicator
    if is_selected {
        buf.set_string(x, y, "► ", Style::default().fg(config.selection_fg));
    } else {
        buf.set_string(x, y, "  ", Style::default());
    }
    x += 2;

    // Render padded key
    let padded_key = format!("{:<width$}", setting.key, width = max_key_width);
    buf.set_string(x, y, &padded_key, Style::default());
    x += padded_key.len() as u16;

    // Check if this specific setting is being edited
    let is_editing = editing_key == Some(&setting.key);

    // Render value based on type
    match &setting.value {
        SettingValue::Bool(value) => {
            render_bool_value(*value, config.use_unicode, config.selection_fg, x, y, buf);
        }
        SettingValue::Int(value) => {
            render_int_value(*value, is_editing, edit_buffer, x, y, buf);
        }
        SettingValue::String(value) => {
            render_string_value(value, is_editing, edit_buffer, x, y, buf);
        }
        SettingValue::List { options, current_index } => {
            render_list_value(options, *current_index, x, y, buf);
        }
        SettingValue::Color(color) => {
            render_color_value(*color, x, y, buf);
        }
    }

    1 // Always 1 line height
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_setting_row_bool_not_selected() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Use Unicode".to_string(),
            value: SettingValue::Bool(true),
        };

        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            20,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Use Unicode"));
        assert!(line.contains("[")); // Checkbox
        assert!(!line.contains("►")); // Not selected
    }

    #[test]
    fn test_setting_row_bool_selected() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Use Unicode".to_string(),
            value: SettingValue::Bool(false),
        };

        let height = render_setting_row(
            &setting,
            true,
            None,
            None,
            20,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("►")); // Selected
        assert!(line.contains("Use Unicode"));
        assert!(line.contains("[ ]")); // Unchecked
    }

    #[test]
    fn test_setting_row_int_normal() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Refresh Interval".to_string(),
            value: SettingValue::Int(60),
        };

        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            25,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Refresh Interval"));
        assert!(line.contains("60"));
    }

    #[test]
    fn test_setting_row_int_editing() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Refresh Interval".to_string(),
            value: SettingValue::Int(60),
        };

        let height = render_setting_row(
            &setting,
            true,
            Some("Refresh Interval"),
            Some("12"),
            25,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Refresh Interval"));
        assert!(line.contains("12█")); // Edit cursor
    }

    #[test]
    fn test_setting_row_string() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Log File".to_string(),
            value: SettingValue::String("/tmp/app.log".to_string()),
        };

        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            20,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Log File"));
        assert!(line.contains("/tmp/app.log"));
    }

    #[test]
    fn test_setting_row_list() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Log Level".to_string(),
            value: SettingValue::List {
                options: vec!["trace".to_string(), "debug".to_string(), "info".to_string()],
                current_index: 2,
            },
        };

        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            20,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Log Level"));
        assert!(line.contains("▼ info"));
    }

    #[test]
    fn test_setting_row_color() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting = Setting {
            key: "Selection FG".to_string(),
            value: SettingValue::Color(Color::Red),
        };

        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            20,
            2,
            area,
            0,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1);
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Selection FG"));
        assert!(line.contains("██")); // Color block
    }

    #[test]
    fn test_setting_row_key_padding() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        let area = Rect::new(0, 0, 80, 1);

        let setting1 = Setting {
            key: "Short".to_string(),
            value: SettingValue::Bool(true),
        };
        let setting2 = Setting {
            key: "Very Long Key Name".to_string(),
            value: SettingValue::Bool(false),
        };

        // Both should use same max_key_width for alignment
        let max_width = 25;

        render_setting_row(&setting1, false, None, None, max_width, 2, area, 0, &mut buf, &config);
        let line1 = buffer_to_string(&buf, 0);

        let mut buf2 = Buffer::empty(Rect::new(0, 0, 80, 1));
        render_setting_row(&setting2, false, None, None, max_width, 2, area, 0, &mut buf2, &config);
        let line2 = buffer_to_string(&buf2, 0);

        // Find position of checkbox in both lines - should be same
        let pos1 = line1.find('[').unwrap();
        let pos2 = line2.find('[').unwrap();
        assert_eq!(pos1, pos2, "Values should be aligned");
    }

    #[test]
    fn test_setting_row_at_bottom() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        let area = Rect::new(0, 0, 80, 5);

        let setting = Setting {
            key: "Test".to_string(),
            value: SettingValue::Bool(true),
        };

        // Try to render at y=5 (at bottom)
        let height = render_setting_row(
            &setting,
            false,
            None,
            None,
            20,
            2,
            area,
            5,
            &mut buf,
            &config,
        );

        assert_eq!(height, 0); // Should not render
    }
}
