mod state;
mod view;
mod handler;

//pub use state::State;

use ratatui::style::Color;

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
