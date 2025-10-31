use ratatui::style::{Style, Color};

/// Get the base style for a tab bar based on focus state
///
/// When unfocused, applies DarkGray foreground to all elements
pub fn base_tab_style(focused: bool) -> Style {
    if focused {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

/// Get the appropriate tab/subtab style based on selection and focus state
///
/// # Arguments
/// * `base_style` - The base style to apply (may include dimming for unfocused tabs)
/// * `is_selected` - Whether this tab/subtab is currently selected
/// * `focused` - Whether the tab bar is currently focused
/// * `selection_fg` - Foreground color for focused selections
/// * `unfocused_selection_fg` - Foreground color for unfocused selections
pub fn selection_style(
    base_style: Style,
    is_selected: bool,
    focused: bool,
    selection_fg: Color,
    unfocused_selection_fg: Color,
) -> Style {
    if is_selected {
        if focused {
            base_style.fg(selection_fg)
        } else {
            base_style.fg(unfocused_selection_fg)
        }
    } else {
        base_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_tab_style_focused() {
        let style = base_tab_style(true);
        assert_eq!(style, Style::default());
    }

    #[test]
    fn test_base_tab_style_unfocused() {
        let style = base_tab_style(false);
        assert_eq!(style, Style::default().fg(Color::DarkGray));
    }

    #[test]
    fn test_selection_style_selected_focused() {
        let base = Style::default();
        let result = selection_style(base, true, true, Color::Red, Color::Blue);
        assert_eq!(result, base.fg(Color::Red));
    }

    #[test]
    fn test_selection_style_selected_unfocused() {
        let base = Style::default();
        let result = selection_style(base, true, false, Color::Red, Color::Blue);
        assert_eq!(result, base.fg(Color::Blue));
    }

    #[test]
    fn test_selection_style_not_selected() {
        let base = Style::default().fg(Color::DarkGray);
        let result = selection_style(base, false, true, Color::Red, Color::Blue);
        assert_eq!(result, base);
    }
}
