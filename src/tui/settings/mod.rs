mod state;
mod view;
mod handler;

pub use state::State;
pub use view::render_content;
pub use handler::handle_key;

use ratatui::style::Color;
use crate::config::Config;

/// Margin between setting key and value (in spaces)
pub const KEY_VALUE_MARGIN: usize = 3;

/// Represents the different types of setting values
#[derive(Debug, Clone)]
pub enum SettingValue {
    String(String),
    Bool(bool),
    Int(u32),
    List { options: Vec<String>, current_index: usize },
    Color(Color),
}

/// Represents a single setting with a key and value
#[derive(Debug, Clone)]
pub struct Setting {
    pub key: String,
    pub value: SettingValue,
}

/// Build the list of settings from a Config
/// Settings are grouped by category: Logging, Display, Data
pub fn build_settings_list(config: &Config) -> Vec<Setting> {
    let log_levels = vec![
        "trace".to_string(),
        "debug".to_string(),
        "info".to_string(),
        "warn".to_string(),
        "error".to_string(),
    ];

    let current_log_level_index = log_levels
        .iter()
        .position(|level| level == &config.log_level)
        .unwrap_or(2); // Default to "info" if not found

    vec![
        // Logging category
        Setting {
            key: "Log Level".to_string(),
            value: SettingValue::List {
                options: log_levels,
                current_index: current_log_level_index,
            },
        },
        Setting {
            key: "Log File".to_string(),
            value: SettingValue::String(config.log_file.clone()),
        },
        // Display category
        Setting {
            key: "Use Unicode".to_string(),
            value: SettingValue::Bool(config.display.use_unicode),
        },
        Setting {
            key: "Selection FG".to_string(),
            value: SettingValue::Color(config.display.selection_fg),
        },
        Setting {
            key: "Division Header FG".to_string(),
            value: SettingValue::Color(config.display.division_header_fg),
        },
        Setting {
            key: "Error FG".to_string(),
            value: SettingValue::Color(config.display.error_fg),
        },
        Setting {
            key: "Western Teams First".to_string(),
            value: SettingValue::Bool(config.display_standings_western_first),
        },
        Setting {
            key: "Time Format".to_string(),
            value: SettingValue::String(config.time_format.clone()),
        },
        // Data category
        Setting {
            key: "Refresh Interval (seconds)".to_string(),
            value: SettingValue::Int(config.refresh_interval),
        },
    ]
}
