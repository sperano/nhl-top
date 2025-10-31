use ratatui::style::{Color, Modifier, Style};

// Primary colors
pub const SELECTION_FG: Color = Color::Cyan;
pub const ACCENT_COLOR: Color = Color::Blue;
pub const MUTED_COLOR: Color = Color::DarkGray;
pub const ERROR_BG: Color = Color::Red;
pub const ERROR_FG: Color = Color::White;

// Tab navigation
pub fn tab_active_style() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_inactive_style() -> Style {
    Style::new().fg(Color::White)
}

// Horizontal pills pattern (for Standings)
pub fn pill_normal_border() -> Style {
    Style::new().fg(MUTED_COLOR)
}

pub fn pill_selected_border() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn pill_normal_text() -> Style {
    Style::new().fg(Color::White)
}

pub fn pill_selected_text() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

// Vertical list pattern (for Stats and general use)
pub fn list_header_style() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn list_normal_style() -> Style {
    Style::new().fg(Color::White)
}

pub fn list_selected_style() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

pub const LIST_HIGHLIGHT_SYMBOL: &str = "▶ ";

// Cards and blocks
pub fn card_border_style() -> Style {
    Style::new().fg(MUTED_COLOR)
}

pub fn card_title_style() -> Style {
    Style::new()
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

// Status and hints
pub fn status_normal_style() -> Style {
    Style::new().fg(Color::White)
}

pub fn status_error_style() -> Style {
    Style::new()
        .bg(ERROR_BG)
        .fg(ERROR_FG)
}

pub fn hint_style() -> Style {
    Style::new()
        .fg(MUTED_COLOR)
        .add_modifier(Modifier::DIM)
}
