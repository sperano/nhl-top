use crate::config::Config;
use crate::tui::{
    component::{Component, Element},
    SettingsCategory,
};
use crate::tui::widgets::{ListModalWidget, SettingsListWidget};

use super::{TabbedPanel, TabbedPanelProps, TabItem};

/// Props for SettingsTab component
#[derive(Clone)]
pub struct SettingsTabProps {
    pub config: Config,
    pub selected_category: SettingsCategory,
    pub selected_setting_index: usize,
    pub settings_mode: bool,
    pub focused: bool,
    pub editing: bool,
    pub edit_buffer: String,
    pub modal_open: bool,
    pub modal_selected_index: usize,
}

/// SettingsTab component - displays settings with category tabs
pub struct SettingsTab;

impl Component for SettingsTab {
    type Props = SettingsTabProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let base = self.render_category_tabs(props);

        // If modal is open, wrap with overlay
        if props.modal_open {
            self.render_with_modal(base, props)
        } else {
            base
        }
    }
}

impl SettingsTab {
    /// Render category tabs using TabbedPanel
    fn render_category_tabs(&self, props: &SettingsTabProps) -> Element {
        // Create tabs for each category
        let tabs = vec![
            TabItem::new(
                "logging",
                "Logging",
                self.render_settings_list(SettingsCategory::Logging, props),
            ),
            TabItem::new(
                "display",
                "Display",
                self.render_settings_list(SettingsCategory::Display, props),
            ),
            TabItem::new(
                "data",
                "Data",
                self.render_settings_list(SettingsCategory::Data, props),
            ),
        ];

        // Active key based on selected category
        let active_key = self.category_to_key(props.selected_category);

        TabbedPanel.view(
            &TabbedPanelProps {
                active_key,
                tabs,
                focused: props.focused && !props.settings_mode,
            },
            &(),
        )
    }

    /// Convert category to tab key
    fn category_to_key(&self, category: SettingsCategory) -> String {
        match category {
            SettingsCategory::Logging => "logging".to_string(),
            SettingsCategory::Display => "display".to_string(),
            SettingsCategory::Data => "data".to_string(),
        }
    }

    /// Render the settings list for a category
    fn render_settings_list(&self, category: SettingsCategory, props: &SettingsTabProps) -> Element {
        let selected_index = if props.settings_mode && category == props.selected_category {
            Some(props.selected_setting_index)
        } else {
            None
        };

        Element::Widget(Box::new(SettingsListWidget::new(
            category,
            props.config.clone(),
            2, // Left margin
            selected_index,
            props.settings_mode,
            props.editing,
            props.edit_buffer.clone(),
        )))
    }

    /// Render with modal overlay
    fn render_with_modal(&self, base: Element, props: &SettingsTabProps) -> Element {
        use crate::tui::settings_helpers;

        // Get the setting key to determine what we're editing
        let setting_key = settings_helpers::get_editable_setting_key(
            props.selected_category,
            props.selected_setting_index,
        );

        if let Some(key) = setting_key {
            let setting_name = settings_helpers::get_setting_display_name(&key);
            let options: Vec<String> = settings_helpers::get_setting_values(&key)
                .iter()
                .map(|s| s.to_string())
                .collect();

            let modal = Element::Widget(Box::new(ListModalWidget::new(
                setting_name,
                options,
                props.modal_selected_index,
            )));

            Element::Overlay {
                base: Box::new(base),
                overlay: Box::new(modal),
            }
        } else {
            // No valid setting key, just return base without modal
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_settings_tab_renders() {
        let settings_tab = SettingsTab;
        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Logging,
            selected_setting_index: 0,
            settings_mode: false,
            focused: false,
            editing: false,
            edit_buffer: String::new(),
            modal_open: false,
            modal_selected_index: 0,
        };

        let element = settings_tab.view(&props, &());

        // Should create a container element (from TabbedPanel's vertical layout)
        match element {
            Element::Container { children, .. } => {
                // Container created successfully with children
                assert_eq!(children.len(), 2); // Tab bar + content
            }
            _ => panic!("Expected container element"),
        }
    }

    #[test]
    fn test_settings_tab_has_three_categories() {
        let settings_tab = SettingsTab;
        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Display,
            selected_setting_index: 0,
            settings_mode: false,
            focused: false,
            editing: false,
            edit_buffer: String::new(),
            modal_open: false,
            modal_selected_index: 0,
        };

        // Render all three categories and verify each has content
        let logging_list = settings_tab.render_settings_list(SettingsCategory::Logging, &props);
        let display_list = settings_tab.render_settings_list(SettingsCategory::Display, &props);
        let data_list = settings_tab.render_settings_list(SettingsCategory::Data, &props);

        // All should be Widget elements
        match logging_list {
            Element::Widget(_) => {}
            _ => panic!("Expected Widget element for Logging category"),
        }
        match display_list {
            Element::Widget(_) => {}
            _ => panic!("Expected Widget element for Display category"),
        }
        match data_list {
            Element::Widget(_) => {}
            _ => panic!("Expected Widget element for Data category"),
        }
    }

    #[test]
    fn test_settings_mode_changes_selection() {
        let settings_tab = SettingsTab;

        // Not in settings mode
        let props_unfocused = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Logging,
            selected_setting_index: 1,
            settings_mode: false,
            focused: false,
            editing: false,
            edit_buffer: String::new(),
            modal_open: false,
            modal_selected_index: 0,
        };

        // In settings mode
        let props_focused = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Logging,
            selected_setting_index: 1,
            settings_mode: true,
            focused: true,
            editing: false,
            edit_buffer: String::new(),
            modal_open: false,
            modal_selected_index: 0,
        };

        // Both should render without panicking
        let _ = settings_tab.view(&props_unfocused, &());
        let _ = settings_tab.view(&props_focused, &());
    }

    #[test]
    fn test_category_to_key() {
        let settings_tab = SettingsTab;

        assert_eq!(
            settings_tab.category_to_key(SettingsCategory::Logging),
            "logging"
        );
        assert_eq!(
            settings_tab.category_to_key(SettingsCategory::Display),
            "display"
        );
        assert_eq!(
            settings_tab.category_to_key(SettingsCategory::Data),
            "data"
        );
    }

    #[test]
    fn test_modal_open_renders_overlay() {
        let settings_tab = SettingsTab;
        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Logging,
            selected_setting_index: 0, // log_level setting
            settings_mode: true,
            focused: true,
            editing: false,
            edit_buffer: String::new(),
            modal_open: true,
            modal_selected_index: 0,
        };

        let element = settings_tab.view(&props, &());

        // Should create an overlay element
        match element {
            Element::Overlay { .. } => {
                // Overlay created successfully
            }
            _ => panic!("Expected overlay element when modal_open=true"),
        }
    }

    #[test]
    fn test_modal_with_invalid_setting_returns_base() {
        let settings_tab = SettingsTab;
        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Display,
            selected_setting_index: 1, // Index 1 is "Use Unicode" which is not editable
            settings_mode: true,
            focused: true,
            editing: false,
            edit_buffer: String::new(),
            modal_open: true,
            modal_selected_index: 0,
        };

        let element = settings_tab.view(&props, &());

        // Should return base element (Container), not overlay, since index 1 is not editable
        match element {
            Element::Container { .. } => {
                // Base element returned as expected
            }
            _ => panic!("Expected container element when no valid setting"),
        }
    }

    #[test]
    fn test_render_with_modal_logging_category() {
        let settings_tab = SettingsTab;
        let base = Element::Widget(Box::new(crate::tui::widgets::SettingsListWidget::new(
            SettingsCategory::Logging,
            Config::default(),
            2,
            None,
            false,
            false,
            String::new(),
        )));

        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Logging,
            selected_setting_index: 0, // log_level
            settings_mode: true,
            focused: true,
            editing: false,
            edit_buffer: String::new(),
            modal_open: true,
            modal_selected_index: 2,
        };

        let result = settings_tab.render_with_modal(base, &props);

        // Should return an overlay
        match result {
            Element::Overlay { .. } => {}
            _ => panic!("Expected overlay element"),
        }
    }

    #[test]
    fn test_render_with_modal_data_category() {
        let settings_tab = SettingsTab;
        let base = Element::Widget(Box::new(crate::tui::widgets::SettingsListWidget::new(
            SettingsCategory::Data,
            Config::default(),
            2,
            None,
            false,
            false,
            String::new(),
        )));

        let props = SettingsTabProps {
            config: Config::default(),
            selected_category: SettingsCategory::Data,
            selected_setting_index: 0, // refresh_interval
            settings_mode: true,
            focused: true,
            editing: false,
            edit_buffer: String::new(),
            modal_open: true,
            modal_selected_index: 0,
        };

        let result = settings_tab.render_with_modal(base.clone(), &props);

        // Data category settings that aren't list-based should return base
        // (refresh_interval is not a list setting, only log_level is)
        // Actually, get_setting_values returns empty vec for non-list settings
        // So this should still create overlay with empty options
        match result {
            Element::Overlay { .. } => {}
            _ => panic!("Should handle refresh_interval"),
        }
    }
}
