use xdg::BaseDirectories;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use ratatui::style::Color;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub log_level: String,
    pub log_file: String,
    pub refresh_interval: u32,
    pub display_standings_western_first: bool,
    pub time_format: String,
    pub display: DisplayConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DisplayConfig {
    pub use_unicode: bool,
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(serialize_with = "serialize_color")]
    pub selection_fg: Color,
    #[serde(deserialize_with = "deserialize_color_optional")]
    #[serde(serialize_with = "serialize_color_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unfocused_selection_fg: Option<Color>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: "info".to_string(),
            log_file: "/dev/null".to_string(),
            refresh_interval: 60,
            display_standings_western_first: false,
            time_format: "%H:%M:%S".to_string(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            use_unicode: true,
            selection_fg: Color::Rgb(255, 165, 0), // Orange
            unfocused_selection_fg: None,
        }
    }
}

impl DisplayConfig {
    /// Get the unfocused selection color, calculating 50% darker if not explicitly set
    pub fn unfocused_selection_fg(&self) -> Color {
        self.unfocused_selection_fg.unwrap_or_else(|| darken_color(self.selection_fg, 0.5))
    }
}

/// Darken a color by a given factor (0.0 = black, 1.0 = original)
fn darken_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let r = (r as f32 * factor) as u8;
            let g = (g as f32 * factor) as u8;
            let b = (b as f32 * factor) as u8;
            Color::Rgb(r, g, b)
        }
        // For named colors, return them as-is (could convert to RGB if needed)
        other => other,
    }
}

/// Deserialize a color from a string (supports named colors, RGB hex, or RGB tuple)
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_color(&s).ok_or_else(|| serde::de::Error::custom(format!("Invalid color: {}", s)))
}

/// Deserialize an optional color from a string
fn deserialize_color_optional<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(color_str) => {
            let color = parse_color(&color_str)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid color: {}", color_str)))?;
            Ok(Some(color))
        }
        None => Ok(None),
    }
}

/// Serialize a color to a string
fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format_color(color))
}

/// Serialize an optional color to a string
fn serialize_color_optional<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match color {
        Some(c) => serializer.serialize_str(&format_color(c)),
        None => serializer.serialize_none(),
    }
}

/// Format a color as a string (RGB format for serialization)
fn format_color(color: &Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("{},{},{}", r, g, b),
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
        _ => "white".to_string(), // fallback for indexed colors
    }
}

/// Parse a color string into a ratatui Color
/// Supports:
/// - Named colors: "red", "blue", "cyan", "orange", etc.
/// - Hex colors: "#FF6600", "#f60"
/// - RGB tuples: "255,165,0"
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Named colors
    match s.as_str() {
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "gray" | "grey" => return Some(Color::Gray),
        "darkgray" | "darkgrey" => return Some(Color::DarkGray),
        "lightred" => return Some(Color::LightRed),
        "lightgreen" => return Some(Color::LightGreen),
        "lightyellow" => return Some(Color::LightYellow),
        "lightblue" => return Some(Color::LightBlue),
        "lightmagenta" => return Some(Color::LightMagenta),
        "lightcyan" => return Some(Color::LightCyan),
        "white" => return Some(Color::White),
        "orange" => return Some(Color::Rgb(255, 165, 0)),
        _ => {}
    }

    // Hex colors (#FF6600 or #f60)
    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }

    // RGB tuples "255,165,0"
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }

    None
}

pub fn get_config_path() -> Option<PathBuf> {
    let pgm = env!("CARGO_PKG_NAME");
    let xdg_dirs = BaseDirectories::with_prefix(pgm);
    let config_home = xdg_dirs.get_config_home()?;
    Some(config_home.join("config.toml"))
}

pub fn read() -> Config {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => return Config::default(),
    };

    // Check if file exists
    if !config_path.exists() {
        return Config::default();
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return Config::default(),
    };

    toml::from_str(&content).unwrap_or_else(|_| Config::default())
}

/// Write a config to the config file
pub fn write(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()
        .ok_or("Failed to get config path")?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize config to TOML
    let toml_string = toml::to_string_pretty(config)?;

    // Write to file
    fs::write(&config_path, toml_string)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("blue"), Some(Color::Blue));
        assert_eq!(parse_color("orange"), Some(Color::Rgb(255, 165, 0)));
        assert_eq!(parse_color("cyan"), Some(Color::Cyan));
        assert_eq!(parse_color("white"), Some(Color::White));
    }

    #[test]
    fn test_parse_color_case_insensitive() {
        assert_eq!(parse_color("RED"), Some(Color::Red));
        assert_eq!(parse_color("Blue"), Some(Color::Blue));
        assert_eq!(parse_color("ORANGE"), Some(Color::Rgb(255, 165, 0)));
    }

    #[test]
    fn test_parse_color_hex_6_digit() {
        assert_eq!(parse_color("#FF6600"), Some(Color::Rgb(255, 102, 0)));
        assert_eq!(parse_color("#ff6600"), Some(Color::Rgb(255, 102, 0)));
        assert_eq!(parse_color("#00FF00"), Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_color_hex_3_digit() {
        assert_eq!(parse_color("#F60"), Some(Color::Rgb(255, 102, 0)));
        assert_eq!(parse_color("#f60"), Some(Color::Rgb(255, 102, 0)));
        assert_eq!(parse_color("#0F0"), Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_color_rgb_tuple() {
        assert_eq!(parse_color("255,165,0"), Some(Color::Rgb(255, 165, 0)));
        assert_eq!(parse_color("0,255,0"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("255, 102, 0"), Some(Color::Rgb(255, 102, 0))); // with spaces
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color("invalid"), None);
        assert_eq!(parse_color("#ZZZ"), None);
        assert_eq!(parse_color("256,0,0"), None); // RGB values too high
        assert_eq!(parse_color("#GGGGGG"), None);
    }

    #[test]
    fn test_display_config_default() {
        let display = DisplayConfig::default();
        assert_eq!(display.selection_fg, Color::Rgb(255, 165, 0));
        assert_eq!(display.use_unicode, true);
    }

    #[test]
    fn test_config_default_includes_display() {
        let config = Config::default();
        assert_eq!(config.display.selection_fg, Color::Rgb(255, 165, 0));
        assert_eq!(config.display.use_unicode, true);
    }

    #[test]
    fn test_config_from_toml_named_color() {
        let toml_str = r#"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[display]
selection_fg = "cyan"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.display.selection_fg, Color::Cyan);
    }

    #[test]
    fn test_config_from_toml_hex_color() {
        let toml_str = r###"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[display]
selection_fg = "#00FFFF"
        "###;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.display.selection_fg, Color::Rgb(0, 255, 255));
    }

    #[test]
    fn test_config_from_toml_rgb_tuple() {
        let toml_str = r#"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[display]
selection_fg = "128,0,128"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.display.selection_fg, Color::Rgb(128, 0, 128));
    }

    #[test]
    fn test_serialize_color_rgb() {
        let color = Color::Rgb(255, 165, 0);
        assert_eq!(format_color(&color), "255,165,0");
    }

    #[test]
    fn test_serialize_color_named() {
        assert_eq!(format_color(&Color::Red), "red");
        assert_eq!(format_color(&Color::Blue), "blue");
        assert_eq!(format_color(&Color::Cyan), "cyan");
    }

    #[test]
    fn test_config_to_toml() {
        let mut config = Config::default();
        config.display.selection_fg = Color::Rgb(128, 0, 128);
        config.refresh_interval = 30;

        let toml_str = toml::to_string_pretty(&config).unwrap();

        // Verify it contains expected values
        assert!(toml_str.contains("refresh_interval = 30"));
        assert!(toml_str.contains("selection_fg = \"128,0,128\""));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut config = Config::default();
        config.display.selection_fg = Color::Rgb(255, 100, 50);
        config.display.unfocused_selection_fg = Some(Color::Cyan);
        config.display.use_unicode = false;
        config.refresh_interval = 45;
        config.display_standings_western_first = true;

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&config).unwrap();

        // Deserialize back
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.display.selection_fg, Color::Rgb(255, 100, 50));
        assert_eq!(deserialized.display.unfocused_selection_fg, Some(Color::Cyan));
        assert_eq!(deserialized.display.use_unicode, false);
        assert_eq!(deserialized.refresh_interval, 45);
        assert_eq!(deserialized.display_standings_western_first, true);
    }
}
