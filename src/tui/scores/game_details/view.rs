use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
    style::{Style, Modifier},
};

use crate::config::DisplayConfig;
use crate::tui::scores::game_details::state::{GameDetailsState};
use crate::tui::scores::game_details::players;

/// Render the game details view with player selection support
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut GameDetailsState,
    boxscore_text: &str,
    boxscore: Option<&nhl_api::Boxscore>,
    display: &DisplayConfig,
) {
    // Parse boxscore text and apply styling to selected player
    let styled_lines = if state.player_selection_active && boxscore.is_some() {
        highlight_selected_player(boxscore_text, state, boxscore.unwrap(), display)
    } else {
        // No selection, just convert text to lines
        boxscore_text.lines().map(|line| Line::raw(line.to_string())).collect()
    };

    state.scrollable.update_viewport_height(area.height);
    state.scrollable.update_content_height(styled_lines.len());

    // Create paragraph with styled lines
    let paragraph = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: true })
        .scroll((state.scrollable.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

/// Highlight the selected player in the boxscore text
fn highlight_selected_player(
    boxscore_text: &str,
    state: &GameDetailsState,
    boxscore: &nhl_api::Boxscore,
    display: &DisplayConfig,
) -> Vec<Line<'static>> {
    // Get the selected player info
    let selected_player = players::find_player(boxscore, state.selected_section, state.selected_index);

    if selected_player.is_none() {
        // No valid player selected, return plain text
        return boxscore_text.lines().map(|line| Line::raw(line.to_string())).collect();
    }

    let player = selected_player.unwrap();
    let player_name = &player.name;

    // Parse the boxscore text and highlight lines containing the selected player
    let mut lines = Vec::new();
    for line in boxscore_text.lines() {
        if line.contains(player_name) {
            // This line contains the selected player's name - highlight it
            let style = Style::default()
                .fg(display.selection_fg)
                .add_modifier(Modifier::BOLD);
            lines.push(Line::styled(line.to_string(), style));
        } else {
            lines.push(Line::raw(line.to_string()));
        }
    }

    lines
}

/// Render visual indicator for the currently selected player
fn render_selection_indicator(
    _frame: &mut Frame,
    _area: Rect,
    _state: &GameDetailsState,
    _display: &DisplayConfig,
) {
    // This will be implemented to highlight the selected player row
    // For now, this is a placeholder
}

/// Render keyboard hints for game details navigation
pub fn render_hints(state: &GameDetailsState) -> Vec<Line<'static>> {
    let mut hints = vec![];

    if state.player_selection_active {
        hints.push(Line::from(vec![
            Span::raw("↑↓ Navigate • "),
            Span::raw("Tab Next Section • "),
            Span::raw("Enter View Player • "),
            Span::raw("Esc Back"),
        ]));
    } else {
        hints.push(Line::from(vec![
            Span::raw("↓ Select Player • "),
            Span::raw("PgUp/PgDn Scroll • "),
            Span::raw("Esc Back"),
        ]));
    }

    hints
}
