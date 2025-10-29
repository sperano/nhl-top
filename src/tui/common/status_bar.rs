use ratatui::{
    layout::Rect,
    style::{Style, Color},
    widgets::Paragraph,
    Frame,
};
use std::time::SystemTime;
use chrono::{DateTime, Local};

pub fn render(f: &mut Frame, area: Rect, last_refresh: Option<SystemTime>, time_format: &str, error_message: Option<&str>) {
    if let Some(error) = error_message {
        // Display error message in red if present
        let error_line = format!("ERROR: {}", error);
        let status_line = format!("{:width$}", error_line, width = area.width as usize);
        let status_bar = Paragraph::new(status_line)
            .style(Style::default().bg(Color::Red).fg(Color::White));
        f.render_widget(status_bar, area);
        return;
    }

    // Normal status display
    let status_text = if let Some(refresh_time) = last_refresh {
        let datetime: DateTime<Local> = refresh_time.into();
        let formatted_time = datetime.format(time_format).to_string();
        format!("last refresh: {}", formatted_time)
    } else {
        "last refresh: never".to_string()
    };

    // Create a line that fills the entire width with spaces (for reverse video background)
    let status_line = format!("{:>width$}", status_text, width = area.width as usize);
    let status_bar = Paragraph::new(status_line)
        .style(Style::default().bg(Color::White).fg(Color::Black));

    f.render_widget(status_bar, area);
}
