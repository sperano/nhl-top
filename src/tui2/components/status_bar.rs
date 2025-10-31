use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::SystemTime;

/// Render the bottom status bar with contextual help and refresh time
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    at_root: bool,
    error: Option<&str>,
    last_refresh: Option<SystemTime>,
    time_format: &str,
) {
    let mut spans = Vec::new();

    if let Some(err) = error {
        // Show error message in red
        spans.push(Span::styled(
            format!("ERROR: {}", err),
            Style::default().fg(Color::Red)
        ));
    } else {
        // Show contextual help
        let help_style = Style::default().fg(Color::DarkGray);

        if at_root {
            spans.push(Span::styled("↑/↓ Navigate", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("Enter Drill-in", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("←/→ Switch tabs", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("1-4 Quick jump", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("q Quit", help_style));
        } else {
            spans.push(Span::styled("↑/↓ Navigate/Scroll", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("Enter Drill-in", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("Esc Back", help_style));
            spans.push(Span::styled(" │ ", help_style));
            spans.push(Span::styled("q Quit", help_style));
        }
    }

    // Add refresh time on the right side
    if let Some(refresh_time) = last_refresh {
        let time_str = format_refresh_time(refresh_time, time_format);

        // Calculate padding to push refresh time to the right
        let help_text_len: usize = spans.iter()
            .map(|s| s.content.len())
            .sum();

        let refresh_text = format!("Last refresh: {}", time_str);
        let padding = area.width as usize - help_text_len - refresh_text.len() - 2;

        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }

        spans.push(Span::styled(
            refresh_text,
            Style::default().fg(Color::DarkGray)
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);

    f.render_widget(paragraph, area);
}

/// Format the refresh time using the configured format
fn format_refresh_time(time: SystemTime, time_format: &str) -> String {
    use chrono::{DateTime, Local};

    let datetime: DateTime<Local> = time.into();
    datetime.format(time_format).to_string()
}
