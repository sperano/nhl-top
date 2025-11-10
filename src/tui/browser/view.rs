use super::state::State;
use super::link::Link;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Modifier},
};
use crate::config::DisplayConfig;
use unicode_width::UnicodeWidthStr;

/// Render the browser content with link highlighting
pub fn render_content(f: &mut Frame, area: Rect, state: &State, config: &DisplayConfig) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let selected_link = state.get_selected_link();
    let mut rendered_lines = Vec::new();

    for (line_idx, line_text) in state.content.lines.iter().enumerate() {
        let links_in_line = state.content.links_on_line(line_idx);
        let rendered_line = render_line_with_links(
            line_text,
            &links_in_line,
            selected_link,
            config,
        );
        rendered_lines.push(rendered_line);
    }

    let paragraph = Paragraph::new(rendered_lines)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, area);
}

/// Render a single line with link highlighting
fn render_line_with_links(
    line: &str,
    links_in_line: &[&Link],
    selected_link: Option<&Link>,
    config: &DisplayConfig,
) -> Line<'static> {
    if links_in_line.is_empty() {
        return Line::from(line.to_string());
    }

    // Calculate the start position of this line in the overall content
    let mut spans = Vec::new();
    let mut last_pos = 0;
    let line_chars: Vec<char> = line.chars().collect();

    for link in links_in_line {
        // Find the link's position within this line
        // We need to search for the link text in the line
        let link_text = &link.display;

        if let Some(link_start_in_line) = find_substring_position(line, link_text, last_pos) {
            let link_end_in_line = link_start_in_line + link_text.chars().count();

            // Add text before the link
            if link_start_in_line > last_pos {
                let before_text: String = line_chars[last_pos..link_start_in_line].iter().collect();
                spans.push(Span::raw(before_text));
            }

            // Add the link with appropriate styling
            let is_selected = selected_link.map_or(false, |sel| {
                sel.display == link.display && sel.start == link.start
            });

            let link_style = if is_selected {
                Style::default()
                    .fg(config.selection_fg)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default()
                    .add_modifier(Modifier::UNDERLINED)
            };

            spans.push(Span::styled(link_text.to_string(), link_style));
            last_pos = link_end_in_line;
        }
    }

    // Add any remaining text after the last link
    if last_pos < line_chars.len() {
        let remaining_text: String = line_chars[last_pos..].iter().collect();
        spans.push(Span::raw(remaining_text));
    }

    Line::from(spans)
}

/// Find the position of a substring in a string, starting from a given offset
fn find_substring_position(haystack: &str, needle: &str, start_char_offset: usize) -> Option<usize> {
    let haystack_chars: Vec<char> = haystack.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();

    if start_char_offset >= haystack_chars.len() {
        return None;
    }

    for i in start_char_offset..=(haystack_chars.len().saturating_sub(needle_chars.len())) {
        if haystack_chars[i..].starts_with(&needle_chars) {
            return Some(i);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::browser::{BrowserContent, Target};
    use crate::tui::widgets::testing::*;
    use ratatui::buffer::Buffer;

    fn create_test_state() -> State {
        State::new()
    }

    #[test]
    fn test_render_content_basic() {
        let state = create_test_state();
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_content(f, f.area(), &state, &config);
        }).unwrap();

        // Verify content was rendered by checking buffer
        let buffer = terminal.backend().buffer();
        let line = buffer_line(buffer, 0);
        assert!(line.contains("Nick Suzuki"));
        assert!(line.contains("Canadiens"));
        assert!(line.contains("Golden Knights"));
    }

    #[test]
    fn test_render_content_empty_area() {
        let state = create_test_state();
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 0, 0);
            render_content(f, area, &state, &config);
        }).unwrap();
    }

    #[test]
    fn test_render_content_zero_width() {
        let state = create_test_state();
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(5, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 0, 5);
            render_content(f, area, &state, &config);
        }).unwrap();
    }

    #[test]
    fn test_render_content_zero_height() {
        let state = create_test_state();
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 0);
            render_content(f, area, &state, &config);
        }).unwrap();
    }

    #[test]
    fn test_render_line_no_links() {
        let config = test_config();
        let line = render_line_with_links("Just plain text", &[], None, &config);

        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Just plain text");
    }

    #[test]
    fn test_render_line_with_selected_link() {
        let config = test_config();
        let target = Target::Player { id: 8480018 };
        let link = super::super::Link::new("TestLink", target, 0, 8);
        let line_text = "TestLink here";

        let line = render_line_with_links(
            line_text,
            &[&link],
            Some(&link),
            &config,
        );

        // Should have at least 2 spans: the link and the rest
        assert!(line.spans.len() >= 2);

        // First span should be the link with selection color
        assert_eq!(line.spans[0].content, "TestLink");
        assert_eq!(line.spans[0].style.fg, Some(config.selection_fg));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_render_line_with_unselected_link() {
        let config = test_config();
        let target = Target::Player { id: 8480018 };
        let link = super::super::Link::new("TestLink", target, 0, 8);
        let line_text = "TestLink here";

        let line = render_line_with_links(
            line_text,
            &[&link],
            None,
            &config,
        );

        // Should have at least 2 spans
        assert!(line.spans.len() >= 2);

        // First span should be underlined but not with selection color
        assert_eq!(line.spans[0].content, "TestLink");
        assert!(line.spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_render_line_multiple_links() {
        let config = test_config();
        let target1 = Target::Player { id: 8480018 };
        let target2 = Target::Team { id: "MTL".to_string() };
        let link1 = super::super::Link::new("Link1", target1, 0, 5);
        let link2 = super::super::Link::new("Link2", target2, 10, 15);
        let line_text = "Link1 and Link2";

        let line = render_line_with_links(
            line_text,
            &[&link1, &link2],
            Some(&link1),
            &config,
        );

        // Should have multiple spans
        assert!(line.spans.len() >= 3);
    }

    #[test]
    fn test_find_substring_position() {
        assert_eq!(find_substring_position("Hello World", "World", 0), Some(6));
        assert_eq!(find_substring_position("Hello World", "Hello", 0), Some(0));
        assert_eq!(find_substring_position("Hello World", "o", 0), Some(4));
        assert_eq!(find_substring_position("Hello World", "o", 5), Some(7));
        assert_eq!(find_substring_position("Hello World", "NotThere", 0), None);
        assert_eq!(find_substring_position("Hello World", "World", 20), None);
    }

    #[test]
    fn test_find_substring_position_unicode() {
        let text = "Café résumé";
        assert_eq!(find_substring_position(text, "Café", 0), Some(0));
        assert_eq!(find_substring_position(text, "résumé", 0), Some(5));
    }

    #[test]
    fn test_render_empty_content() {
        let state = State {
            content: BrowserContent::builder().build(),
            selected_link_index: None,
            scroll_offset: 0,
            subtab_focused: false,
        };
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_content(f, f.area(), &state, &config);
        }).unwrap();
    }

    #[test]
    fn test_render_content_exact_output() {
        let state = create_test_state();
        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 3);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_content(f, f.area(), &state, &config);
        }).unwrap();

        let buffer = terminal.backend().buffer();
        let expected = vec![
            "Nick Suzuki plays for the Canadiens, and was drafted by the Golden Knights.     ",
            "                                                                                ",
            "                                                                                ",
        ];

        assert_buffer(buffer, &expected);
    }

    #[test]
    fn test_render_with_second_link_selected() {
        let mut state = create_test_state();
        state.select_next_link(); // Move to second link (Canadiens)

        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 3);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_content(f, f.area(), &state, &config);
        }).unwrap();

        // Verify the second link (Canadiens) is styled
        let selected_link = state.get_selected_link().unwrap();
        assert_eq!(selected_link.display, "Canadiens");
    }

    #[test]
    fn test_render_with_last_link_selected() {
        let mut state = create_test_state();
        state.select_next_link(); // Move to Canadiens
        state.select_next_link(); // Move to Golden Knights

        let config = test_config();

        let backend = ratatui::backend::TestBackend::new(80, 3);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_content(f, f.area(), &state, &config);
        }).unwrap();

        // Verify the third link (Golden Knights) is styled
        let selected_link = state.get_selected_link().unwrap();
        assert_eq!(selected_link.display, "Golden Knights");
    }
}
