/// Shared helper functions for settings management
///
/// This module provides common functionality used by the reducer, key handler,
/// and components for managing settings.

use super::state::SettingsCategory;

/// Get the editable setting key for a given category and index
pub fn get_editable_setting_key(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => match index {
            0 => Some("log_level".to_string()),
            1 => Some("log_file".to_string()),
            _ => None,
        },
        SettingsCategory::Display => None,
        SettingsCategory::Data => match index {
            0 => Some("refresh_interval".to_string()),
            2 => Some("time_format".to_string()),
            _ => None,
        },
    }
}

/// Get the display name for a setting key
pub fn get_setting_display_name(key: &str) -> String {
    match key {
        "log_level" => "Log Level".to_string(),
        "log_file" => "Log File".to_string(),
        "refresh_interval" => "Refresh Interval".to_string(),
        "time_format" => "Time Format".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Get the list of valid values for a setting that has a fixed set of options
pub fn get_setting_values(key: &str) -> Vec<&'static str> {
    match key {
        "log_level" => vec!["trace", "debug", "info", "warn", "error"],
        _ => vec![], // Empty for non-list settings
    }
}

/// Check if a setting is a list-type setting (has a fixed set of values)
pub fn is_list_setting(key: &str) -> bool {
    matches!(key, "log_level")
}
