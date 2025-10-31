use xdg::BaseDirectories;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use ratatui::style::Color;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub log_level: String,
    pub log_file: String,
    pub refresh_interval: u32,
    pub display_standings_western_first: bool,
    pub time_format: String,
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ThemeConfig {
    #[serde(deserialize_with = "deserialize_color")]
    pub selection_fg: Color,
    #[serde(deserialize_with = "deserialize_color_optional")]
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
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            selection_fg: Color::Rgb(255, 165, 0), // Orange
            unfocused_selection_fg: None,
        }
    }
}

impl ThemeConfig {
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
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.selection_fg, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_config_default_includes_theme() {
        let config = Config::default();
        assert_eq!(config.theme.selection_fg, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_config_from_toml_named_color() {
        let toml_str = r#"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[theme]
selection_fg = "cyan"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.selection_fg, Color::Cyan);
    }

    #[test]
    fn test_config_from_toml_hex_color() {
        let toml_str = r###"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[theme]
selection_fg = "#00FFFF"
        "###;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.selection_fg, Color::Rgb(0, 255, 255));
    }

    #[test]
    fn test_config_from_toml_rgb_tuple() {
        let toml_str = r#"
log_level = "info"
log_file = "/dev/null"
refresh_interval = 60
display_standings_western_first = false
time_format = "%H:%M:%S"

[theme]
selection_fg = "128,0,128"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.selection_fg, Color::Rgb(128, 0, 128));
    }
}
