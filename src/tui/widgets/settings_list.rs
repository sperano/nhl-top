/// SettingsListWidget - displays a read-only list of settings with their current values
///
/// This widget renders a simple key-value table for configuration settings.
/// Settings are displayed as "Setting Name: Current Value" pairs.

use ratatui::{buffer::Buffer, layout::Rect, text::Line};
use crate::config::{Config, DisplayConfig};
use crate::tui::framework::component::RenderableWidget;
use crate::tui::framework::state::SettingsCategory;

/// Widget for displaying settings list
#[derive(Debug, Clone)]
pub struct SettingsListWidget {
    /// Category of settings to display
    pub category: SettingsCategory,
    /// Configuration to display settings from
    pub config: Config,
    /// Left margin for indentation
    pub margin: u16,
    /// Index of selected setting (for highlighting)
    pub selected_index: Option<usize>,
    /// Whether we're in settings navigation mode
    pub settings_mode: bool,
}

impl SettingsListWidget {
    /// Create a new SettingsListWidget
    pub fn new(
        category: SettingsCategory,
        config: Config,
        margin: u16,
        selected_index: Option<usize>,
        settings_mode: bool,
    ) -> Self {
        Self {
            category,
            config,
            margin,
            selected_index,
            settings_mode,
        }
    }

    /// Get the settings to display for this category
    fn get_settings(&self) -> Vec<(String, String)> {
        match self.category {
            SettingsCategory::Logging => vec![
                ("Log Level".to_string(), self.config.log_level.clone()),
                ("Log File".to_string(), self.config.log_file.clone()),
            ],
            SettingsCategory::Display => vec![
                ("Use Unicode".to_string(), self.config.display.use_unicode.to_string()),
                ("Selection Color".to_string(), format_color(&self.config.display.selection_fg)),
                ("Division Header Color".to_string(), format_color(&self.config.display.division_header_fg)),
                ("Error Color".to_string(), format_color(&self.config.display.error_fg)),
                ("Show Action Bar".to_string(), self.config.display.show_action_bar.to_string()),
            ],
            SettingsCategory::Data => vec![
                ("Refresh Interval".to_string(), format!("{} seconds", self.config.refresh_interval)),
                ("Western Teams First".to_string(), self.config.display_standings_western_first.to_string()),
                ("Time Format".to_string(), self.config.time_format.clone()),
            ],
        }
    }
}

impl RenderableWidget for SettingsListWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let settings = self.get_settings();
        let x = area.x + self.margin;
        let mut y = area.y;

        for (index, (key, value)) in settings.iter().enumerate() {
            if y >= area.y + area.height {
                break; // Stop if we run out of space
            }

            // Format as "Key: Value"
            let line_text = format!("{}: {}", key, value);

            // Apply selection highlighting if this is the selected setting
            let style = if self.settings_mode && self.selected_index == Some(index) {
                ratatui::style::Style::default().fg(config.selection_fg)
            } else {
                ratatui::style::Style::default()
            };

            let line = Line::from(line_text).style(style);

            // Render the line
            let line_width = line.width() as u16;
            if x + line_width <= area.x + area.width {
                buf.set_line(x, y, &line, line_width);
            }

            y += 1;
        }
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(self.clone())
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(self.get_settings().len() as u16)
    }
}

/// Format a ratatui Color as a readable string
fn format_color(color: &ratatui::style::Color) -> String {
    use ratatui::style::Color;
    match color {
        Color::Rgb(r, g, b) => format!("rgb({}, {}, {})", r, g, b),
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "darkgray".to_string(),
        Color::LightRed => "lightred".to_string(),
        Color::LightGreen => "lightgreen".to_string(),
        Color::LightYellow => "lightyellow".to_string(),
        Color::LightBlue => "lightblue".to_string(),
        Color::LightMagenta => "lightmagenta".to_string(),
        Color::LightCyan => "lightcyan".to_string(),
        Color::White => "white".to_string(),
        _ => "custom".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};

    #[test]
    fn test_settings_list_logging_category() {
        let config = Config::default();
        let widget = SettingsListWidget::new(SettingsCategory::Logging, config.clone(), 2, None, false);

        let area = Rect::new(0, 0, 50, 10);
        let mut buf = Buffer::empty(area);
        let display_config = DisplayConfig::default();

        widget.render(area, &mut buf, &display_config);

        // Verify log level is rendered
        let expected = format!("Log Level: {}", config.log_level);
        let line = buf.content[buf.index_of(2, 0)..buf.index_of(2 + expected.len() as u16, 0)]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(line.contains("Log Level:"));
    }

    #[test]
    fn test_settings_list_display_category() {
        let config = Config::default();
        let widget = SettingsListWidget::new(SettingsCategory::Display, config, 2, None, false);

        let settings = widget.get_settings();
        assert_eq!(settings.len(), 5);
        assert_eq!(settings[0].0, "Use Unicode");
    }

    #[test]
    fn test_settings_list_data_category() {
        let config = Config::default();
        let widget = SettingsListWidget::new(SettingsCategory::Data, config.clone(), 2, None, false);

        let settings = widget.get_settings();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].0, "Refresh Interval");
        assert_eq!(settings[0].1, format!("{} seconds", config.refresh_interval));
    }

    #[test]
    fn test_format_color() {
        use ratatui::style::Color;

        assert_eq!(format_color(&Color::Red), "red");
        assert_eq!(format_color(&Color::Rgb(255, 165, 0)), "rgb(255, 165, 0)");
        assert_eq!(format_color(&Color::Cyan), "cyan");
    }
}
