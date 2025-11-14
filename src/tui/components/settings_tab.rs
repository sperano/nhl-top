use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::config::DisplayConfig;
use crate::tui::framework::component::{Component, Element, RenderableWidget};

/// SettingsTab component - placeholder for settings UI
pub struct SettingsTab;

impl Component for SettingsTab {
    type Props = ();
    type State = ();
    type Message = ();

    fn view(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(SettingsWidget))
    }
}

/// Placeholder widget for settings
struct SettingsWidget;

impl RenderableWidget for SettingsWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let widget = Paragraph::new("Settings (not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Settings"));
        ratatui::widgets::Widget::render(widget, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(SettingsWidget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_tab_renders() {
        let settings_tab = SettingsTab;
        let element = settings_tab.view(&(), &());

        match element {
            Element::Widget(_) => {
                // Widget created successfully
            }
            _ => panic!("Expected widget element"),
        }
    }
}
