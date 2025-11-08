/// SettingsListWidget - renders a scrollable list of settings
///
/// Composes multiple SettingRowWidgets with:
/// - Automatic max_key_width calculation for alignment
/// - Selection highlighting
/// - Edit state handling
/// - Proper margins

use ratatui::{buffer::Buffer, layout::Rect};
use crate::config::DisplayConfig;
use crate::tui::settings::{Setting, KEY_VALUE_MARGIN};
use super::setting_row::render_setting_row;

/// Renders a list of settings with proper alignment and selection
///
/// Returns the total height consumed
pub fn render_settings_list(
    settings: &[Setting],
    selected_index: Option<usize>,
    editing_key: Option<&str>,
    edit_buffer: Option<&str>,
    margin: u16,
    area: Rect,
    buf: &mut Buffer,
    config: &DisplayConfig,
) -> u16 {
    // Calculate max key width for alignment
    let max_key_width = settings.iter()
        .map(|s| s.key.len())
        .max()
        .unwrap_or(0) + KEY_VALUE_MARGIN;

    let mut y = area.y;
    let mut total_height = 0;

    // Render each setting
    for (idx, setting) in settings.iter().enumerate() {
        let is_selected = selected_index == Some(idx);

        let height = render_setting_row(
            setting,
            is_selected,
            editing_key,
            edit_buffer,
            max_key_width,
            margin,
            area,
            y,
            buf,
            config,
        );

        y += height;
        total_height += height;

        // Stop if we've run out of vertical space
        if y >= area.bottom() {
            break;
        }
    }

    total_height
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
    fn test_settings_list_empty() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let settings: Vec<Setting> = vec![];
        let height = render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        assert_eq!(height, 0); // No settings, no height
    }

    #[test]
    fn test_settings_list_single_setting() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let settings = vec![
            Setting {
                key: "Use Unicode".to_string(),
                value: SettingValue::Bool(true),
            },
        ];

        let height = render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        assert_eq!(height, 1); // 1 setting = 1 line
        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("Use Unicode"));
    }

    #[test]
    fn test_settings_list_multiple_settings() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

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
                    options: vec!["info".to_string(), "debug".to_string()],
                    current_index: 0,
                },
            },
        ];

        let height = render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        assert_eq!(height, 3); // 3 settings = 3 lines

        // Check each line
        assert!(buffer_to_string(&buf, 0).contains("Use Unicode"));
        assert!(buffer_to_string(&buf, 1).contains("Refresh Interval"));
        assert!(buffer_to_string(&buf, 2).contains("Log Level"));
    }

    #[test]
    fn test_settings_list_with_selection() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

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

        render_settings_list(
            &settings,
            Some(1), // Select second setting
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        // First line should not have selection indicator
        let line0 = buffer_to_string(&buf, 0);
        assert!(!line0.contains("►"));

        // Second line should have selection indicator
        let line1 = buffer_to_string(&buf, 1);
        assert!(line1.contains("►"));
    }

    #[test]
    fn test_settings_list_with_editing() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let settings = vec![
            Setting {
                key: "Refresh Interval".to_string(),
                value: SettingValue::Int(60),
            },
        ];

        render_settings_list(
            &settings,
            Some(0),
            Some("Refresh Interval"),
            Some("12"),
            2,
            area,
            &mut buf,
            &config,
        );

        let line = buffer_to_string(&buf, 0);
        assert!(line.contains("12█")); // Edit cursor
    }

    #[test]
    fn test_settings_list_alignment() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let settings = vec![
            Setting {
                key: "Short".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Very Long Key Name".to_string(),
                value: SettingValue::Bool(false),
            },
        ];

        render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        // Both values should start at the same column position
        let line0 = buffer_to_string(&buf, 0);
        let line1 = buffer_to_string(&buf, 1);

        // Find position of checkbox in both lines
        let pos0 = line0.find('[').expect("Should have checkbox");
        let pos1 = line1.find('[').expect("Should have checkbox");

        assert_eq!(pos0, pos1, "Values should be aligned");
    }

    #[test]
    fn test_settings_list_stops_at_bottom() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));
        let area = Rect::new(0, 0, 80, 3);

        // Create 5 settings but only 3 lines available
        let settings = vec![
            Setting {
                key: "Setting 1".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Setting 2".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Setting 3".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Setting 4".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Setting 5".to_string(),
                value: SettingValue::Bool(true),
            },
        ];

        let height = render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        // Should only render 3 lines even though we have 5 settings
        assert_eq!(height, 3);
    }

    #[test]
    fn test_settings_list_different_types() {
        let config = test_config();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let area = Rect::new(0, 0, 80, 10);

        let settings = vec![
            Setting {
                key: "Bool Setting".to_string(),
                value: SettingValue::Bool(true),
            },
            Setting {
                key: "Int Setting".to_string(),
                value: SettingValue::Int(42),
            },
            Setting {
                key: "String Setting".to_string(),
                value: SettingValue::String("test".to_string()),
            },
            Setting {
                key: "List Setting".to_string(),
                value: SettingValue::List {
                    options: vec!["opt1".to_string(), "opt2".to_string()],
                    current_index: 0,
                },
            },
            Setting {
                key: "Color Setting".to_string(),
                value: SettingValue::Color(Color::Red),
            },
        ];

        let height = render_settings_list(
            &settings,
            None,
            None,
            None,
            2,
            area,
            &mut buf,
            &config,
        );

        assert_eq!(height, 5); // 5 settings = 5 lines

        // Verify each type renders correctly
        assert!(buffer_to_string(&buf, 0).contains("[")); // Bool checkbox
        assert!(buffer_to_string(&buf, 1).contains("42")); // Int value
        assert!(buffer_to_string(&buf, 2).contains("test")); // String value
        assert!(buffer_to_string(&buf, 3).contains("▼")); // List dropdown
        assert!(buffer_to_string(&buf, 4).contains("██")); // Color block
    }
}
