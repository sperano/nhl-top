/// ListModal component for list selection popup
///
/// Wraps the legacy list_modal rendering function in a RenderableWidget

use ratatui::{buffer::Buffer, layout::Rect};

use crate::config::DisplayConfig;
use crate::tui::framework::component::RenderableWidget;
use crate::tui::widgets::settings::list_modal::render_list_modal;

/// Widget for rendering a list selection modal
#[derive(Clone)]
pub struct ListModalWidget {
    pub setting_name: String,
    pub options: Vec<String>,
    pub selected_index: usize,
}

impl ListModalWidget {
    pub fn new(setting_name: impl Into<String>, options: Vec<String>, selected_index: usize) -> Self {
        Self {
            setting_name: setting_name.into(),
            options,
            selected_index,
        }
    }
}

impl RenderableWidget for ListModalWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        render_list_modal(
            &self.setting_name,
            &self.options,
            self.selected_index,
            area,
            buf,
            config,
        );
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(self.clone())
    }
}
