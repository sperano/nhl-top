/// Settings value renderer widgets
///
/// Small, focused widgets for rendering each type of setting value:
/// - Bool: checkbox ([✔] or [ ])
/// - Int: number with optional edit cursor
/// - String: text with optional edit cursor
/// - List: dropdown indicator with current value
/// - Color: colored block

pub mod bool_value;
pub mod int_value;
pub mod string_value;
pub mod list_value;
pub mod color_value;
pub mod setting_row;
pub mod settings_list;
pub mod list_modal;
pub mod color_modal;
pub mod settings_panel;

pub use bool_value::render_bool_value;
pub use int_value::render_int_value;
pub use string_value::render_string_value;
pub use list_value::render_list_value;
pub use color_value::render_color_value;
pub use settings_list::render_settings_list;
pub use list_modal::render_list_modal;
pub use color_modal::{render_color_modal, COLORS};
pub use settings_panel::SettingsPanelWidget;
