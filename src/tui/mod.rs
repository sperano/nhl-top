mod tabs;
mod widgets;
mod events;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use crate::SharedDataHandle;
use tabs::{AppState, Tab};
use widgets::{render_tab_bar, render_standings_subtabs, render_status_bar, render_content};
use events::{handle_key_event, AppAction};

pub async fn run(shared_data: SharedDataHandle) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::default();

    // Main loop
    loop {
        // Read data from shared state
        let (standings_data, schedule_data, period_scores_data, western_first, last_refresh, time_format) = {
            let data = shared_data.read().await;
            (
                data.standings.clone(),
                data.schedule.clone(),
                data.period_scores.clone(),
                data.config.display_standings_western_first,
                data.last_refresh,
                data.config.time_format.clone(),
            )
        };

        terminal.draw(|f| {
            let size = f.area();

            // Create main layout - add space for sub-tabs if on Standings, and status bar at bottom
            let constraints = if app_state.current_tab == Tab::Standings {
                vec![
                    Constraint::Length(2), // Main tab bar
                    Constraint::Length(2), // Sub-tab bar for standings
                    Constraint::Min(0),    // Content
                    Constraint::Length(1), // Status bar
                ]
            } else {
                vec![
                    Constraint::Length(2), // Main tab bar
                    Constraint::Min(0),    // Content
                    Constraint::Length(1), // Status bar
                ]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(size);

            // Render main tab bar
            render_tab_bar(f, chunks[0], app_state.current_tab, !app_state.subtab_focused);

            // Render sub-tabs and content based on current tab
            let content_chunk_idx = if app_state.current_tab == Tab::Standings {
                render_standings_subtabs(f, chunks[1], app_state.standings_view, app_state.subtab_focused);
                2
            } else {
                1
            };

            render_content(
                f,
                chunks[content_chunk_idx],
                app_state.current_tab,
                &standings_data,
                &schedule_data,
                &period_scores_data,
                app_state.standings_view,
                western_first,
            );

            // Render status bar at the bottom
            let status_chunk_idx = chunks.len() - 1;
            render_status_bar(f, chunks[status_chunk_idx], last_refresh, &time_format);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_key_event(key, &mut app_state) {
                    AppAction::Exit => break,
                    AppAction::Continue => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
