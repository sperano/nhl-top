use ratatui::{
    layout::Rect,
    Frame,
};
use super::{State, build_settings_list};
use std::sync::Arc;
use crate::config::Config;
use crate::tui::widgets::settings::{SettingsPanelWidget, COLORS};

pub fn render_content(f: &mut Frame, area: Rect, state: &State, config: &Arc<Config>) {
    // Build settings list from config
    let settings = build_settings_list(config);

    // Prepare editing state
    let editing = state.editing.as_ref().map(|(key, buffer)| (key.as_str(), buffer.as_str()));

    // Prepare list modal state
    let list_modal = state.list_modal.as_ref().map(|(name, options, idx)| {
        (name.as_str(), options.as_slice(), *idx)
    });

    // Prepare color modal state
    let color_modal = state.color_modal.as_ref().map(|name| {
        // Determine current theme color based on setting name
        let current_color = match name.as_str() {
            "Selection FG" => config.display.selection_fg,
            "Division Header FG" => config.display.division_header_fg,
            "Error FG" => config.display.error_fg,
            _ => config.display.selection_fg, // fallback
        };
        (name.as_str(), state.selected_color_index, current_color)
    });

    // Build and render widget
    let widget = SettingsPanelWidget::new(&settings)
        .with_selected_index(if state.subtab_focused {
            Some(state.selected_setting_index)
        } else {
            None
        })
        .with_subtab_focused(state.subtab_focused)
        .with_editing(editing)
        .with_list_modal(list_modal)
        .with_color_modal(color_modal);

    let buf = f.buffer_mut();
    widget.render(area, buf, &config.display);
}
