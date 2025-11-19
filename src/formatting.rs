use crate::config::DisplayConfig;

/// Box-drawing characters for table borders
#[derive(Debug, Clone, PartialEq)]
pub struct BoxChars {
    pub horizontal: String,
    pub double_horizontal: String,
    pub vertical: String,
    pub top_left: String,
    pub top_right: String,
    pub bottom_left: String,
    pub bottom_right: String,
    pub top_junction: String,
    pub bottom_junction: String,
    pub left_junction: String,
    pub right_junction: String,
    pub cross: String,
    pub connector2: String,
    pub connector3: String,
    pub selector: String,
}

impl BoxChars {
    pub fn unicode() -> Self {
        Self {
            horizontal: "─".to_string(),
            double_horizontal: "═".to_string(),
            vertical: "│".to_string(),
            top_left: "╭".to_string(),
            top_right: "╮".to_string(),
            bottom_left: "╰".to_string(),
            bottom_right: "╯".to_string(),
            top_junction: "┬".to_string(),
            bottom_junction: "┴".to_string(),
            left_junction: "├".to_string(),
            right_junction: "┤".to_string(),
            cross: "┼".to_string(),
            connector2: "┴".to_string(),
            connector3: "┬".to_string(),
            selector: "►".to_string(),
        }
    }

    pub fn ascii() -> Self {
        Self {
            horizontal: "-".to_string(),
            double_horizontal: "=".to_string(),
            vertical: "|".to_string(),
            top_left: "+".to_string(),
            top_right: "+".to_string(),
            bottom_left: "+".to_string(),
            bottom_right: "+".to_string(),
            top_junction: "+".to_string(),
            bottom_junction: "+".to_string(),
            left_junction: "+".to_string(),
            right_junction: "+".to_string(),
            cross: "+".to_string(),
            connector2: "-".to_string(),
            connector3: "-".to_string(),
            selector: ">".to_string(),
        }
    }

    pub fn from_use_unicode(use_unicode: bool) -> Self {
        if use_unicode {
            Self::unicode()
        } else {
            Self::ascii()
        }
    }
}

/// Format a header with text and underline
///
/// # Arguments
/// * `text` - The header text to display
/// * `double_line` - If true, uses double-line (═/=), otherwise single-line (─/-)
/// * `display` - Display configuration to determine unicode vs ASCII
///
/// # Returns
/// A formatted string with the header text and underline separator matching the text length
pub fn format_header(text: &str, double_line: bool, display: &DisplayConfig) -> String {
    let separator_char = if double_line {
        &display.box_chars.double_horizontal
    } else {
        &display.box_chars.horizontal
    };
    format!("{}\n{}\n", text, separator_char.repeat(text.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_header_single_line_unicode() {
        let display = DisplayConfig { use_unicode: true, ..Default::default() };
        let result = format_header("Test Header", false, &display);
        assert_eq!(result, "Test Header\n───────────\n");
    }

    #[test]
    fn test_format_header_double_line_unicode() {
        let display = DisplayConfig { use_unicode: true, ..Default::default() };
        let result = format_header("Test Header", true, &display);
        assert_eq!(result, "Test Header\n═══════════\n");
    }

    #[test]
    fn test_format_header_single_line_ascii() {
        let mut display = DisplayConfig { use_unicode: false, ..Default::default() };
        display.box_chars = BoxChars::ascii();
        let result = format_header("Test Header", false, &display);
        assert_eq!(result, "Test Header\n-----------\n");
    }

    #[test]
    fn test_format_header_double_line_ascii() {
        let mut display = DisplayConfig { use_unicode: false, ..Default::default() };
        display.box_chars = BoxChars::ascii();
        let result = format_header("Test Header", true, &display);
        assert_eq!(result, "Test Header\n===========\n");
    }

    #[test]
    fn test_empty_header() {
        let display = DisplayConfig { use_unicode: true, ..Default::default() };
        let result = format_header("", false, &display);
        assert_eq!(result, "\n\n");
    }

    #[test]
    fn test_long_header() {
        let display = DisplayConfig { use_unicode: true, ..Default::default() };
        let result = format_header("This is a very long header text", true, &display);
        assert_eq!(result, "This is a very long header text\n═══════════════════════════════\n");
    }
}
