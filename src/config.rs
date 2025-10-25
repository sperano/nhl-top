use xdg::BaseDirectories;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub debug: bool,
    pub refresh_interval: u32,
    pub display_standings_western_first: bool,
    pub time_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            debug: false,
            refresh_interval: 60,
            display_standings_western_first: false,
            time_format: "%H:%M:%S".to_string(),
        }
    }
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

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(_) => Config::default(),
    }
}
