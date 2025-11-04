use ratatui::{
    style::Style,
    text::{Line, Span},
};

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
/// Used for rendering subtab separators in the TUI.
///
/// # Arguments
/// * `tab_names` - Iterator of tab name strings (used to calculate spacing)
/// * `area_width` - Total width available for the separator line
/// * `style` - Style to apply to the separator line
///
/// # Returns
/// A `Line` containing the separator with box-drawing characters
pub fn build_tab_separator_line<'a, I>(tab_names: I, area_width: usize, style: Style) -> Line<'a>
where
    I: Iterator<Item = String>,
{
    let mut separator_spans = Vec::new();
    let mut pos = 0;

    for (i, tab_name) in tab_names.enumerate() {
        if i > 0 {
            // Add horizontal line before separator
            separator_spans.push(Span::styled("─".repeat(SEPARATOR_LINE_BEFORE), style));
            separator_spans.push(Span::styled("┴", style));
            separator_spans.push(Span::styled("─".repeat(SEPARATOR_LINE_AFTER), style));
            pos += SEPARATOR_TOTAL_WIDTH;
        }
        // Add horizontal line under tab
        separator_spans.push(Span::styled("─".repeat(tab_name.len()), style));
        pos += tab_name.len();
    }

    // Fill rest of line
    if pos < area_width {
        separator_spans.push(Span::styled("─".repeat(area_width - pos), style));
    }

    Line::from(separator_spans)
}
