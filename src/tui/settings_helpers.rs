/// Shared helper functions for settings management
///
/// This module provides common functionality used by the reducer, key handler,
/// and components for managing settings.

use super::types::SettingsCategory;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_editable_setting_key_logging() {
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Logging, 0),
            Some("log_level".to_string())
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Logging, 1),
            Some("log_file".to_string())
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Logging, 2),
            None
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Logging, 999),
            None
        );
    }

    #[test]
    fn test_get_editable_setting_key_display() {
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Display, 0),
            None
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Display, 1),
            None
        );
    }

    #[test]
    fn test_get_editable_setting_key_data() {
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 0),
            Some("refresh_interval".to_string())
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 1),
            None
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 2),
            Some("time_format".to_string())
        );
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 3),
            None
        );
    }

    #[test]
    fn test_get_setting_display_name() {
        assert_eq!(get_setting_display_name("log_level"), "Log Level");
        assert_eq!(get_setting_display_name("log_file"), "Log File");
        assert_eq!(get_setting_display_name("refresh_interval"), "Refresh Interval");
        assert_eq!(get_setting_display_name("time_format"), "Time Format");
        assert_eq!(get_setting_display_name("unknown_key"), "Unknown");
        assert_eq!(get_setting_display_name(""), "Unknown");
    }

    #[test]
    fn test_get_setting_values_log_level() {
        let values = get_setting_values("log_level");
        assert_eq!(values, vec!["trace", "debug", "info", "warn", "error"]);
    }

    #[test]
    fn test_get_setting_values_other_keys() {
        assert_eq!(get_setting_values("log_file"), Vec::<&str>::new());
        assert_eq!(get_setting_values("refresh_interval"), Vec::<&str>::new());
        assert_eq!(get_setting_values("time_format"), Vec::<&str>::new());
        assert_eq!(get_setting_values("unknown"), Vec::<&str>::new());
    }
}
