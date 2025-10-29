mod common;
mod scores;
mod standings;
mod stats;
mod settings;
mod app;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use crate::SharedDataHandle;
use app::{AppState, CurrentTab};
use tokio::sync::mpsc;

async fn handle_key_event(
    key: KeyEvent,
    app_state: &mut AppState,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // ESC key handling
    if key.code == KeyCode::Esc {
        if app_state.is_subtab_focused() {
            app_state.exit_subtab_mode();
            return false;
        } else {
            return true; // Exit the app
        }
    }

    // Number keys for tab switching (only when not in subtab mode)
    if !app_state.is_subtab_focused() {
        match key.code {
            KeyCode::Char('1') => {
                app_state.current_tab = CurrentTab::Scores;
                return false;
            }
            KeyCode::Char('2') => {
                app_state.current_tab = CurrentTab::Standings;
                return false;
            }
            KeyCode::Char('3') => {
                app_state.current_tab = CurrentTab::Stats;
                return false;
            }
            KeyCode::Char('4') => {
                app_state.current_tab = CurrentTab::Settings;
                return false;
            }
            _ => {}
        }
    }

    // Down arrow or Enter to enter subtab mode
    if !app_state.is_subtab_focused() {
        match key.code {
            KeyCode::Down | KeyCode::Enter => {
                if app_state.has_subtabs() {
                    app_state.enter_subtab_mode();
                }
                return false;
            }
            _ => {}
        }
    }

    // Arrow keys for navigation - delegate to tab handlers when in subtab mode
    match key.code {
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            if app_state.is_subtab_focused() {
                // Delegate to tab-specific handler
                let handled = match app_state.current_tab {
                    CurrentTab::Scores => {
                        scores::handle_key(key, &mut app_state.scores, shared_data, refresh_tx).await
                    }
                    CurrentTab::Standings => {
                        standings::handle_key(key, &mut app_state.standings)
                    }
                    CurrentTab::Stats => false,
                    CurrentTab::Settings => false,
                };

                // If handler didn't handle the key, apply default behavior
                if !handled && key.code == KeyCode::Up {
                    // Up exits subtab mode by default
                    app_state.exit_subtab_mode();
                }
            } else {
                // Navigate between main tabs
                match key.code {
                    KeyCode::Left => app_state.navigate_tab_left(),
                    KeyCode::Right => app_state.navigate_tab_right(),
                    _ => {}
                }
            }
        }
        _ => {}
    }

    false // Continue running
}

pub async fn run(shared_data: SharedDataHandle, refresh_tx: mpsc::Sender<()>) -> Result<(), io::Error> {
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
        let (standings_data, schedule_data, period_scores_data, game_info_data, western_first, last_refresh, time_format, game_date, error_message) = {
            let data = shared_data.read().await;
            (
                data.standings.clone(),
                data.schedule.clone(),
                data.period_scores.clone(),
                data.game_info.clone(),
                data.config.display_standings_western_first,
                data.last_refresh,
                data.config.time_format.clone(),
                data.game_date.clone(),
                data.error_message.clone(),
            )
        };

        terminal.draw(|f| {
            let size = f.area();

            // Create main layout - add space for sub-tabs if on Scores or Standings, and status bar at bottom
            let has_subtabs = app_state.has_subtabs();
            let constraints = if has_subtabs {
                vec![
                    Constraint::Length(2), // Main tab bar
                    Constraint::Length(2), // Sub-tab bar
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
            let tab_names = CurrentTab::all_names();
            common::tab_bar::render(f, chunks[0], &tab_names, app_state.current_tab.index(), !app_state.is_subtab_focused());

            // Render sub-tabs and content based on current tab
            let content_chunk_idx = match app_state.current_tab {
                CurrentTab::Scores => {
                    scores::render_subtabs(f, chunks[1], &app_state.scores, &game_date);
                    2
                }
                CurrentTab::Standings => {
                    standings::render_subtabs(f, chunks[1], &app_state.standings);
                    2
                }
                CurrentTab::Stats => 1,
                CurrentTab::Settings => 1,
            };

            // Render content for current tab
            match app_state.current_tab {
                CurrentTab::Scores => {
                    scores::render_content(
                        f,
                        chunks[content_chunk_idx],
                        &mut app_state.scores,
                        &schedule_data,
                        &period_scores_data,
                        &game_info_data,
                    );
                }
                CurrentTab::Standings => {
                    standings::render_content(
                        f,
                        chunks[content_chunk_idx],
                        &standings_data,
                        &app_state.standings,
                        western_first,
                    );
                }
                CurrentTab::Stats => {
                    stats::render_content(f, chunks[content_chunk_idx]);
                }
                CurrentTab::Settings => {
                    settings::render_content(f, chunks[content_chunk_idx]);
                }
            }

            // Render status bar at the bottom
            let status_chunk_idx = chunks.len() - 1;
            common::status_bar::render(f, chunks[status_chunk_idx], last_refresh, &time_format, error_message.as_deref());
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(key, &mut app_state, &shared_data, &refresh_tx).await {
                    break;
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
