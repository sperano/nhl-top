use ratatui::{
    style::Style,
    text::{Line, Span},
};
use crate::formatting::BoxChars;

// Separator Constants
/// Length of horizontal line before separator connector
const SEPARATOR_LINE_BEFORE: usize = 1;

/// Length of horizontal line after separator connector
const SEPARATOR_LINE_AFTER: usize = 1;

/// Total width of separator between tabs (line + connector + line)
const SEPARATOR_TOTAL_WIDTH: usize = 3;

/// Helper function to build a separator line with box-drawing connectors for tabs
///
/// This creates a horizontal line with "┴" connectors positioned under the gaps between tabs.
/// Uses box-drawing characters from the provided BoxChars configuration.
/// Used for rendering subtab separators in the TUI.
///
/// # Arguments
/// * `tab_names` - Iterator of tab name strings (used to calculate spacing)
/// * `area_width` - Total width available for the separator line
/// * `style` - Style to apply to the separator line
/// * `box_chars` - Box-drawing characters to use for rendering
///
/// # Returns
/// A `Line` containing the separator with box-drawing characters
pub fn build_tab_separator_line<'a, I>(tab_names: I, area_width: usize, style: Style, box_chars: &'a BoxChars) -> Line<'a>
where
    I: Iterator<Item = String>,
{
    let horizontal = &box_chars.horizontal;
    let connector = &box_chars.connector2;

    let mut separator_spans = Vec::new();
    let mut pos = 0;

    for (i, tab_name) in tab_names.enumerate() {
        if i > 0 {
            // Add horizontal line before separator
            separator_spans.push(Span::styled(horizontal.repeat(SEPARATOR_LINE_BEFORE), style));
            separator_spans.push(Span::styled(connector.as_str(), style));
            separator_spans.push(Span::styled(horizontal.repeat(SEPARATOR_LINE_AFTER), style));
            pos += SEPARATOR_TOTAL_WIDTH;
        }
        // Add horizontal line under tab
        separator_spans.push(Span::styled(horizontal.repeat(tab_name.len()), style));
        pos += tab_name.len();
    }

    // Fill rest of line
    if pos < area_width {
        separator_spans.push(Span::styled(horizontal.repeat(area_width - pos), style));
    }

    Line::from(separator_spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_separator_with_two_tabs() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["Tab1".to_string(), "Tab2".to_string()];
        let style = Style::default();

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        let text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();

        assert_eq!(text.chars().count(), 20, "Separator should fill the entire width");
        assert!(text.contains("┴"), "Should contain connector2 (┴)");

        let connector_count = text.chars().filter(|c| *c == '┴').count();
        assert_eq!(connector_count, 1, "Should have exactly 1 connector for 2 tabs");

        let dash_count = text.chars().filter(|c| *c == '─').count();
        assert!(dash_count > 10, "Should be mostly horizontal lines");
    }

    #[test]
    fn test_separator_with_three_tabs() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["AA".to_string(), "BB".to_string(), "CC".to_string()];
        let style = Style::default();

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        let text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();

        let connector_count = text.chars().filter(|c| *c == '┴').count();
        assert_eq!(connector_count, 2, "Should have 2 connectors for 3 tabs");
    }

    #[test]
    fn test_separator_ascii_mode() {
        let box_chars = BoxChars::ascii();
        let tab_names = vec!["Tab1".to_string(), "Tab2".to_string()];
        let style = Style::default();

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        let text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();

        assert!(text.contains("-"), "ASCII mode should use '-' for horizontal");
        assert!(!text.contains("─"), "ASCII mode should not contain unicode horizontal");
        assert!(!text.contains("┴"), "ASCII mode should not contain unicode connector");
    }

    #[test]
    fn test_separator_fills_width() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["X".to_string(), "Y".to_string()];
        let style = Style::default();

        for width in [10, 20, 50, 100] {
            let line = build_tab_separator_line(tab_names.clone().into_iter(), width, style, &box_chars);

            let text: String = line.spans.iter()
                .map(|span| span.content.as_ref())
                .collect();

            assert_eq!(text.chars().count(), width, "Separator should fill width {}", width);
        }
    }

    #[test]
    fn test_separator_connector_spacing() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["AAAA".to_string(), "BBBB".to_string()];
        let style = Style::default();

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        let text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();

        let connector_pos = text.chars().position(|c| c == '┴').expect("Should contain connector");

        assert_eq!(connector_pos, 5, "Connector should be after first tab (4) + 1 horizontal = position 5");
    }

    #[test]
    fn test_separator_with_style() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["A".to_string(), "B".to_string()];
        let style = Style::default().fg(Color::Blue);

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        for span in &line.spans {
            assert_eq!(span.style.fg, Some(Color::Blue), "All spans should have the specified style");
        }
    }

    #[test]
    fn test_separator_single_tab() {
        let box_chars = BoxChars::unicode();
        let tab_names = vec!["OnlyTab".to_string()];
        let style = Style::default();

        let line = build_tab_separator_line(tab_names.into_iter(), 20, style, &box_chars);

        let text: String = line.spans.iter()
            .map(|span| span.content.as_ref())
            .collect();

        let connector_count = text.chars().filter(|c| *c == '┴').count();
        assert_eq!(connector_count, 0, "Single tab should have no connectors");

        assert_eq!(text.chars().count(), 20, "Should still fill the width");
        assert!(text.chars().all(|c| c == '─'), "Should be all horizontal lines");
    }
}
