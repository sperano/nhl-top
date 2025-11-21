/// Shared helper functions for settings management
///
/// This module provides common functionality used by the reducer, key handler,
/// and components for managing settings.
use super::types::SettingsCategory;
use crate::config::Config;

/// Get the editable setting key for a given category and index
pub fn get_editable_setting_key(category: SettingsCategory, index: usize) -> Option<String> {
    match category {
        SettingsCategory::Logging => match index {
            0 => Some("log_level".to_string()),
            1 => Some("log_file".to_string()),
            _ => None,
        },
        SettingsCategory::Display => match index {
            0 => Some("theme".to_string()),
            _ => None,
        },
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
        "theme" => "Theme".to_string(),
        "refresh_interval" => "Refresh Interval".to_string(),
        "time_format" => "Time Format".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Get the list of valid values for a setting that has a fixed set of options
pub fn get_setting_values(key: &str) -> Vec<&'static str> {
    match key {
        "log_level" => vec!["trace", "debug", "info", "warn", "error"],
        "theme" => vec![
            "none", "orange", "green", "blue", "purple", "white", "red", "yellow", "cyan",
        ],
        _ => vec![], // Empty for non-list settings
    }
}

/// Get the list of display values for a setting (for showing in modals)
pub fn get_setting_display_values(key: &str) -> Vec<String> {
    match key {
        "log_level" => vec!["trace", "debug", "info", "warn", "error"]
            .into_iter()
            .map(String::from)
            .collect(),
        "theme" => {
            use crate::config::{
                THEME_BLUE, THEME_CYAN, THEME_GREEN, THEME_ORANGE, THEME_PURPLE, THEME_RED,
                THEME_WHITE, THEME_YELLOW,
            };
            vec![
                "none".to_string(),
                THEME_ORANGE.name.to_string(),
                THEME_GREEN.name.to_string(),
                THEME_BLUE.name.to_string(),
                THEME_PURPLE.name.to_string(),
                THEME_WHITE.name.to_string(),
                THEME_RED.name.to_string(),
                THEME_YELLOW.name.to_string(),
                THEME_CYAN.name.to_string(),
            ]
        }
        _ => vec![], // Empty for non-list settings
    }
}

/// Get the current value of a setting from the config
pub fn get_current_setting_value(config: &Config, key: &str) -> String {
    match key {
        "log_level" => config.log_level.clone(),
        "theme" => config
            .display
            .theme_name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string()),
        _ => String::new(),
    }
}

/// Find the initial index for a modal based on the current setting value
pub fn find_initial_modal_index(config: &Config, key: &str) -> usize {
    let current_value = get_current_setting_value(config, key);
    let values = get_setting_values(key);

    values.iter().position(|&v| v == current_value).unwrap_or(0)
}

/// Get the setting keys and values for display in a category
pub fn get_category_settings(category: SettingsCategory, config: &Config) -> Vec<(String, String)> {
    match category {
        SettingsCategory::Logging => vec![
            ("Log Level".to_string(), config.log_level.clone()),
            ("Log File".to_string(), config.log_file.clone()),
        ],
        SettingsCategory::Display => vec![
            (
                "Theme".to_string(),
                config
                    .display
                    .theme
                    .as_ref()
                    .map(|t| t.name.to_string())
                    .unwrap_or_else(|| "none".to_string()),
            ),
            (
                "Use Unicode".to_string(),
                config.display.use_unicode.to_string(),
            ),
            ("Selection Color".to_string(), "...".to_string()), // Placeholder
            ("Division Header Color".to_string(), "...".to_string()),
            ("Error Color".to_string(), "...".to_string()),
        ],
        SettingsCategory::Data => vec![
            (
                "Refresh Interval".to_string(),
                format!("{} seconds", config.refresh_interval),
            ),
            (
                "Western Teams First".to_string(),
                config.display_standings_western_first.to_string(),
            ),
            ("Time Format".to_string(), config.time_format.clone()),
        ],
    }
}

/// Calculate the maximum key length for a category (used for alignment)
pub fn get_max_key_length(category: SettingsCategory, config: &Config) -> usize {
    get_category_settings(category, config)
        .iter()
        .map(|(key, _)| key.len())
        .max()
        .unwrap_or(0)
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
        assert_eq!(get_editable_setting_key(SettingsCategory::Logging, 2), None);
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Logging, 999),
            None
        );
    }

    #[test]
    fn test_get_editable_setting_key_display() {
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Display, 0),
            Some("theme".to_string())
        );
        assert_eq!(get_editable_setting_key(SettingsCategory::Display, 1), None);
    }

    #[test]
    fn test_get_editable_setting_key_data() {
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 0),
            Some("refresh_interval".to_string())
        );
        assert_eq!(get_editable_setting_key(SettingsCategory::Data, 1), None);
        assert_eq!(
            get_editable_setting_key(SettingsCategory::Data, 2),
            Some("time_format".to_string())
        );
        assert_eq!(get_editable_setting_key(SettingsCategory::Data, 3), None);
    }

    #[test]
    fn test_get_setting_display_name() {
        assert_eq!(get_setting_display_name("log_level"), "Log Level");
        assert_eq!(get_setting_display_name("log_file"), "Log File");
        assert_eq!(get_setting_display_name("theme"), "Theme");
        assert_eq!(
            get_setting_display_name("refresh_interval"),
            "Refresh Interval"
        );
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
    fn test_get_setting_values_theme() {
        let values = get_setting_values("theme");
        assert_eq!(
            values,
            vec!["none", "orange", "green", "blue", "purple", "white", "red", "yellow", "cyan"]
        );
    }

    #[test]
    fn test_get_setting_values_other_keys() {
        assert_eq!(get_setting_values("log_file"), Vec::<&str>::new());
        assert_eq!(get_setting_values("refresh_interval"), Vec::<&str>::new());
        assert_eq!(get_setting_values("time_format"), Vec::<&str>::new());
        assert_eq!(get_setting_values("unknown"), Vec::<&str>::new());
    }

    #[test]
    fn test_get_current_setting_value_log_level() {
        let mut config = Config::default();
        config.log_level = "debug".to_string();
        assert_eq!(get_current_setting_value(&config, "log_level"), "debug");
    }

    #[test]
    fn test_get_current_setting_value_theme_none() {
        let config = Config::default();
        assert_eq!(get_current_setting_value(&config, "theme"), "none");
    }

    #[test]
    fn test_get_current_setting_value_theme_set() {
        let mut config = Config::default();
        config.display.theme_name = Some("blue".to_string());
        assert_eq!(get_current_setting_value(&config, "theme"), "blue");
    }

    #[test]
    fn test_get_current_setting_value_unknown() {
        let config = Config::default();
        assert_eq!(get_current_setting_value(&config, "unknown"), "");
    }

    #[test]
    fn test_find_initial_modal_index_theme_none() {
        let config = Config::default();
        assert_eq!(find_initial_modal_index(&config, "theme"), 0);
    }

    #[test]
    fn test_find_initial_modal_index_theme_blue() {
        let mut config = Config::default();
        config.display.theme_name = Some("blue".to_string());
        // Theme values: ["none", "orange", "green", "blue", "purple", "white"]
        // "blue" is at index 3
        assert_eq!(find_initial_modal_index(&config, "theme"), 3);
    }

    #[test]
    fn test_find_initial_modal_index_log_level_info() {
        let mut config = Config::default();
        config.log_level = "info".to_string();
        // Log level values: ["trace", "debug", "info", "warn", "error"]
        // "info" is at index 2
        assert_eq!(find_initial_modal_index(&config, "log_level"), 2);
    }

    #[test]
    fn test_find_initial_modal_index_log_level_error() {
        let mut config = Config::default();
        config.log_level = "error".to_string();
        // Log level values: ["trace", "debug", "info", "warn", "error"]
        // "error" is at index 4
        assert_eq!(find_initial_modal_index(&config, "log_level"), 4);
    }

    #[test]
    fn test_find_initial_modal_index_unknown_value_returns_zero() {
        let mut config = Config::default();
        config.log_level = "unknown_level".to_string();
        // Should return 0 when value not found in list
        assert_eq!(find_initial_modal_index(&config, "log_level"), 0);
    }
}
