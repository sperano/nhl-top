use crate::config::DisplayConfig;

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
    let separator_char = if display.use_unicode {
        if double_line { "═" } else { "─" }
    } else {
        if double_line { "=" } else { "-" }
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
        let display = DisplayConfig { use_unicode: false, ..Default::default() };
        let result = format_header("Test Header", false, &display);
        assert_eq!(result, "Test Header\n-----------\n");
    }

    #[test]
    fn test_format_header_double_line_ascii() {
        let display = DisplayConfig { use_unicode: false, ..Default::default() };
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
