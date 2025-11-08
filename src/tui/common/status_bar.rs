use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    style::{Color, Style},
    Frame,
};
use std::time::SystemTime;
use crate::formatting::BoxChars;

pub fn render(f: &mut Frame, area: Rect, last_refresh: Option<SystemTime>, refresh_interval: u32, status_message: Option<&str>, status_is_error: bool, error_fg: Color, box_chars: &BoxChars) {
    let mut lines = Vec::new();

    // Left side: status message (if any)
    let left_text = if let Some(msg) = status_message {
        if status_is_error {
            format!("ERROR: {}", msg)
        } else {
            msg.to_string()
        }
    } else {
        String::new()
    };

    // Right side: countdown to next refresh (always shown)
    let right_text = if let Some(refresh_time) = last_refresh {
        if let Ok(elapsed) = SystemTime::now().duration_since(refresh_time) {
            let elapsed_secs = elapsed.as_secs();
            let remaining_secs = refresh_interval.saturating_sub(elapsed_secs as u32);

            if remaining_secs > 0 {
                format!("Refresh in {}s", remaining_secs)
            } else {
                "Refreshing...".to_string()
            }
        } else {
            "Refresh in ?s".to_string()
        }
    } else {
        "Loading...".to_string()
    };

    // Calculate where the vertical bar should be (right-aligned with right text + 1 char margin)
    let right_text_with_margin = format!("{} ", right_text);
    let bar_position = area.width.saturating_sub(right_text_with_margin.len() as u16 + 1);

    // First line: horizontal line with connector at the bar position
    let left_part = box_chars.horizontal.repeat(bar_position as usize);
    let right_part = box_chars.horizontal.repeat((area.width.saturating_sub(bar_position + 1)) as usize);
    let line1 = format!("{}{}{}", left_part, box_chars.connector3, right_part);
    lines.push(Line::raw(line1));

    // Second line: left status message + right refresh countdown
    let mut line2_spans = Vec::new();

    // Left side: status message with 1 char margin
    if !left_text.is_empty() {
        line2_spans.push(Span::raw(" "));
        if status_is_error {
            line2_spans.push(Span::styled(&left_text, Style::default().fg(error_fg)));
        } else {
            line2_spans.push(Span::raw(&left_text));
        }
    }

    // Middle: padding to push right text to the right
    let left_content_len = if left_text.is_empty() { 0 } else { left_text.len() + 1 };
    let padding_len = bar_position.saturating_sub(left_content_len as u16) as usize;
    line2_spans.push(Span::raw(" ".repeat(padding_len)));

    // Right side: vertical bar + refresh countdown + margin
    line2_spans.push(Span::raw(&box_chars.vertical));
    line2_spans.push(Span::raw(" "));
    line2_spans.push(Span::raw(&right_text));
    line2_spans.push(Span::raw(" "));

    lines.push(Line::from(line2_spans));

    let status_bar = Paragraph::new(lines);
    f.render_widget(status_bar, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::time::Duration;

    #[test]
    fn test_status_bar_loading_state() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, None, 60, None, false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line1 = buffer.content().iter()
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line1.contains("┬"), "Line 1 should contain connector3 (┬)");
        assert!(line1.chars().filter(|c| *c == '─').count() > 50, "Line 1 should be mostly horizontal lines");

        assert!(line2.contains("│"), "Line 2 should contain vertical bar");
        assert!(line2.contains("Loading..."), "Line 2 should contain 'Loading...'");

        // Verify right-alignment: text should appear after the vertical bar
        let bar_pos = line2.find('│').expect("Should contain vertical bar");
        let text_after_bar = &line2[bar_pos..];
        assert!(text_after_bar.contains("Loading..."), "Text should be right-aligned after vertical bar");
    }

    #[test]
    fn test_status_bar_countdown() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        let last_refresh = SystemTime::now() - Duration::from_secs(5);

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, Some(last_refresh), 60, None, false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line2.contains("Refresh in"), "Should show countdown message");
        assert!(line2.contains("55s") || line2.contains("54s"), "Should show remaining seconds (around 55s)");
    }

    #[test]
    fn test_status_bar_refreshing_state() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        let last_refresh = SystemTime::now() - Duration::from_secs(65);

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, Some(last_refresh), 60, None, false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line2.contains("Refreshing..."), "Should show 'Refreshing...' when time expired");
    }

    #[test]
    fn test_status_bar_error_message() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, None, 60, Some("Network timeout"), true, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line1 = buffer.content().iter()
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line1.contains("┬"), "Line 1 should contain connector3");
        assert!(line2.contains("ERROR: Network timeout"), "Line 2 should contain error message on left");
        assert!(line2.contains("Loading..."), "Line 2 should contain loading text on right");
        assert!(line2.contains("│"), "Line 2 should contain vertical bar");

        // Verify error appears before the vertical bar (left side)
        let error_pos = line2.find("ERROR:").expect("Should contain ERROR:");
        let bar_pos = line2.find('│').expect("Should contain vertical bar");
        assert!(error_pos < bar_pos, "Error should appear on left side of vertical bar");
    }

    #[test]
    fn test_status_bar_with_status_message() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        let last_refresh = SystemTime::now() - Duration::from_secs(5);

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, Some(last_refresh), 60, Some("Setting saved"), false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line2.contains("Setting saved"), "Line 2 should contain status message on left");
        assert!(line2.contains("Refresh in"), "Line 2 should contain refresh countdown on right");
        assert!(!line2.contains("ERROR:"), "Non-error status should not have ERROR: prefix");

        // Verify status message appears before the vertical bar (left side)
        let status_pos = line2.find("Setting saved").expect("Should contain status message");
        let bar_pos = line2.find('│').expect("Should contain vertical bar");
        assert!(status_pos < bar_pos, "Status message should appear on left side of vertical bar");
    }

    #[test]
    fn test_status_bar_ascii_mode() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::ascii();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, None, 60, None, false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let line1 = buffer.content().iter()
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        let line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(line1.contains("-"), "ASCII mode should use '-' for horizontal");
        assert!(!line1.contains("─"), "ASCII mode should not contain unicode horizontal");
        assert!(line2.contains("|"), "ASCII mode should use '|' for vertical");
        assert!(!line2.contains("│"), "ASCII mode should not contain unicode vertical");
    }

    #[test]
    fn test_status_bar_connector_alignment() {
        let mut terminal = Terminal::new(TestBackend::new(80, 2)).unwrap();
        let box_chars = BoxChars::unicode();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 2);
            render(f, area, None, 60, None, false, Color::Red, &box_chars);
        }).unwrap();

        let buffer = terminal.backend().buffer();

        let connector_pos_line1 = buffer.content().iter()
            .take(80)
            .position(|cell| cell.symbol() == "┬");

        let bar_pos_line2 = buffer.content().iter()
            .skip(80)
            .take(80)
            .position(|cell| cell.symbol() == "│");

        assert!(connector_pos_line1.is_some(), "Should find connector in line 1");
        assert!(bar_pos_line2.is_some(), "Should find vertical bar in line 2");
        assert_eq!(connector_pos_line1, bar_pos_line2, "Connector and vertical bar should be aligned");
    }
}
