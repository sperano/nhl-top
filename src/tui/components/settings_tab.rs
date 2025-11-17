use crate::config::Config;
use crate::tui::framework::{
    component::{Component, Element},
    state::SettingsCategory,
};
use crate::tui::widgets::SettingsListWidget;

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
}

/// SettingsTab component - displays settings with category tabs
pub struct SettingsTab;

impl Component for SettingsTab {
    type Props = SettingsTabProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        self.render_category_tabs(props)
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
                focused: props.focused,
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
}
