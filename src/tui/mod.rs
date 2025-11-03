mod common;
mod scores;
mod standings;
mod stats;
mod players;
mod settings;
mod app;
mod error;
pub mod traits;
pub mod navigation;

use std::io;
use std::sync::Arc;
use std::time::SystemTime;
use std::collections::HashMap;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
    Frame,
};
use crate::SharedDataHandle;
use crate::commands::scores_format::PeriodScores;
use app::{AppState, CurrentTab};
use tokio::sync::mpsc;
use nhl_api::{Standing, DailySchedule, GameMatchup, GameDate};

// UI Layout Constants
/// Height of main tab bar
const TAB_BAR_HEIGHT: u16 = 2;

/// Height of subtab bar (for tabs with subtabs)
const SUBTAB_BAR_HEIGHT: u16 = 2;

/// Height of status bar at bottom
const STATUS_BAR_HEIGHT: u16 = 1;

/// Event polling interval in milliseconds
const EVENT_POLL_INTERVAL_MS: u64 = 100;

/// Handles ESC key presses - exits subtab mode or signals app exit
/// Returns Some(true) to exit app, Some(false) to continue, None if not ESC
fn handle_esc_key(key: KeyEvent, app_state: &mut AppState) -> Option<bool> {
    if key.code == KeyCode::Esc {
        // If on Scores tab and boxscore view is active, don't handle ESC here
        // Let the scores handler close the boxscore view
        if matches!(app_state.current_tab, CurrentTab::Scores) && app_state.scores.boxscore_view_active {
            return None; // Let scores handler handle ESC
        }

        // If on Scores tab and box selection is active, don't handle ESC here
        // Let the scores handler exit box selection mode
        if matches!(app_state.current_tab, CurrentTab::Scores) && app_state.scores.box_selection_active {
            return None; // Let scores handler handle ESC
        }

        // If on Standings tab and team selection is active, don't handle ESC here
        // Let the standings handler exit team selection mode
        if matches!(app_state.current_tab, CurrentTab::Standings) && app_state.standings.team_selection_active {
            return None; // Let standings handler handle ESC
        }

        if app_state.is_subtab_focused() {
            app_state.exit_subtab_mode();
            Some(false) // Continue running
        } else {
            Some(true) // Exit the app
        }
    } else {
        None // Not an ESC key
    }
}

/// Handles number keys (1-5) for direct tab switching
/// Only works when not in subtab mode
/// Returns true if key was handled
async fn handle_number_keys(key: KeyEvent, app_state: &mut AppState, shared_data: &SharedDataHandle) -> bool {
    if app_state.is_subtab_focused() {
        return false;
    }

    let old_tab = app_state.current_tab;
    let handled = match key.code {
        KeyCode::Char('1') => {
            app_state.current_tab = CurrentTab::Scores;
            true
        }
        KeyCode::Char('2') => {
            app_state.current_tab = CurrentTab::Standings;
            true
        }
        KeyCode::Char('3') => {
            app_state.current_tab = CurrentTab::Stats;
            true
        }
        KeyCode::Char('4') => {
            app_state.current_tab = CurrentTab::Players;
            true
        }
        KeyCode::Char('5') => {
            app_state.current_tab = CurrentTab::Settings;
            true
        }
        _ => false,
    };

    // Reset boxscore state when navigating away from Scores tab
    if handled && matches!(old_tab, CurrentTab::Scores) && !matches!(app_state.current_tab, CurrentTab::Scores) {
        let mut data = shared_data.write().await;
        data.clear_boxscore();
    }

    handled
}

/// Handles Down/Enter keys to enter subtab mode
/// Only works when not already in subtab mode and tab has subtabs
/// Returns true if key was handled
fn handle_enter_subtab_mode(key: KeyEvent, app_state: &mut AppState) -> bool {
    if app_state.is_subtab_focused() {
        return false;
    }

    match key.code {
        KeyCode::Down | KeyCode::Enter => {
            if app_state.has_subtabs() {
                app_state.enter_subtab_mode();
            }
            true
        }
        _ => false,
    }
}

/// Handles arrow key and Enter navigation for both main tabs and subtabs
/// Delegates to tab-specific handlers when in subtab mode
/// Returns true if key was handled
async fn handle_arrow_and_enter_keys(
    key: KeyEvent,
    app_state: &mut AppState,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Esc => {
            if app_state.is_subtab_focused() {
                // Delegate to tab-specific handler
                let handled = match app_state.current_tab {
                    CurrentTab::Scores => {
                        scores::handle_key(key, &mut app_state.scores, shared_data, refresh_tx).await
                    }
                    CurrentTab::Standings => {
                        standings::handle_key(key, &mut app_state.standings, shared_data, refresh_tx).await
                    }
                    CurrentTab::Stats => false,
                    CurrentTab::Players => false,
                    CurrentTab::Settings => false,
                };

                // If handler didn't handle the key, apply default behavior
                if !handled && key.code == KeyCode::Up {
                    // Up exits subtab mode by default
                    app_state.exit_subtab_mode();
                }
            } else {
                // Navigate between main tabs
                let old_tab = app_state.current_tab;
                match key.code {
                    KeyCode::Left => app_state.navigate_tab_left(),
                    KeyCode::Right => app_state.navigate_tab_right(),
                    _ => {}
                }

                // Reset boxscore state when navigating away from Scores tab
                if matches!(old_tab, CurrentTab::Scores) && !matches!(app_state.current_tab, CurrentTab::Scores) {
                    let mut data = shared_data.write().await;
                    data.clear_boxscore();
                }
            }
            true
        }
        _ => false,
    }
}

/// Main key event dispatcher - coordinates all key handling logic
/// Returns true to signal app exit, false to continue running
async fn handle_key_event(
    key: KeyEvent,
    app_state: &mut AppState,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) -> bool {
    // Handle Q key globally to quit from anywhere
    if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
        return true; // Exit app
    }

    // Try ESC key handler first
    if let Some(should_exit) = handle_esc_key(key, app_state) {
        return should_exit;
    }

    // Try other handlers in order
    if handle_number_keys(key, app_state, shared_data).await {
        return false;
    }

    if handle_enter_subtab_mode(key, app_state) {
        return false;
    }

    if handle_arrow_and_enter_keys(key, app_state, shared_data, refresh_tx).await {
        return false;
    }

    false // Continue running - unhandled key
}

/// Calculates layout constraints based on whether the current tab has subtabs
/// Returns a Vec of Constraints for: tab bar, optional subtab bar, content, and status bar
fn calculate_layout_constraints(has_subtabs: bool) -> Vec<Constraint> {
    if has_subtabs {
        vec![
            Constraint::Length(TAB_BAR_HEIGHT),    // Main tab bar
            Constraint::Length(SUBTAB_BAR_HEIGHT), // Sub-tab bar
            Constraint::Min(0),                    // Content
            Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
        ]
    } else {
        vec![
            Constraint::Length(TAB_BAR_HEIGHT),    // Main tab bar
            Constraint::Min(0),                    // Content
            Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
        ]
    }
}

/// Calculates the index of the content chunk based on the current tab
/// Tabs with subtabs use chunk index 2, others use index 1
fn calculate_content_chunk_index(current_tab: &CurrentTab) -> usize {
    match current_tab {
        CurrentTab::Scores | CurrentTab::Standings => 2,
        CurrentTab::Stats | CurrentTab::Players | CurrentTab::Settings => 1,
    }
}

/// Data structure holding all cloned data needed for rendering
/// This avoids holding the RwLock during rendering operations
struct RenderData {
    standings: Arc<Vec<Standing>>,
    schedule: Arc<Option<DailySchedule>>,
    period_scores: Arc<HashMap<i64, PeriodScores>>,
    game_info: Arc<HashMap<i64, GameMatchup>>,
    boxscore: Arc<Option<nhl_api::Boxscore>>,
    club_stats: Arc<HashMap<String, nhl_api::ClubStats>>,
    western_first: bool,
    last_refresh: Option<SystemTime>,
    time_format: String,
    game_date: GameDate,
    error_message: Option<String>,
    theme: crate::config::ThemeConfig,
    boxscore_loading: bool,
    selected_team_abbrev: Option<String>,
}

/// Renders a single frame with the current application state and data
/// Delegates rendering to tab-specific modules
fn render_frame(f: &mut Frame, chunks: &[Rect], app_state: &mut AppState, data: &RenderData) {
    // Render main tab bar
    let tab_names = CurrentTab::all_names();
    common::tab_bar::render(f, chunks[0], &tab_names, app_state.current_tab.index(), !app_state.is_subtab_focused(), data.theme.selection_fg, data.theme.unfocused_selection_fg());

    // Render sub-tabs for tabs that have them
    match app_state.current_tab {
        CurrentTab::Scores => {
            scores::render_subtabs(f, chunks[1], &app_state.scores, &data.game_date, data.theme.selection_fg, data.theme.unfocused_selection_fg());
        }
        CurrentTab::Standings => {
            standings::render_subtabs(f, chunks[1], &app_state.standings, data.theme.selection_fg, data.theme.unfocused_selection_fg());
        }
        CurrentTab::Stats | CurrentTab::Players | CurrentTab::Settings => {
            // No subtabs for these tabs
        }
    }

    // Calculate content chunk index
    let content_chunk_idx = calculate_content_chunk_index(&app_state.current_tab);

    // Render content for current tab
    match app_state.current_tab {
        CurrentTab::Scores => {
            scores::render_content(
                f,
                chunks[content_chunk_idx],
                &mut app_state.scores,
                &data.schedule,
                &data.period_scores,
                &data.game_info,
                data.theme.selection_fg,
                &data.boxscore,
                data.boxscore_loading,
            );
        }
        CurrentTab::Standings => {
            // Update layout cache if needed
            if app_state.standings.layout_cache.is_none() {
                app_state.standings.update_layout(&data.standings, data.western_first);
            }

            standings::render_content(
                f,
                chunks[content_chunk_idx],
                &mut app_state.standings,
                data.theme.selection_fg,
                &data.club_stats,
                &data.selected_team_abbrev,
            );
        }
        CurrentTab::Stats => {
            stats::render_content(f, chunks[content_chunk_idx]);
        }
        CurrentTab::Players => {
            players::render_content(f, chunks[content_chunk_idx]);
        }
        CurrentTab::Settings => {
            settings::render_content(f, chunks[content_chunk_idx]);
        }
    }

    // Render status bar at the bottom
    let status_chunk_idx = chunks.len() - 1;
    common::status_bar::render(f, chunks[status_chunk_idx], data.last_refresh, &data.time_format, data.error_message.as_deref());
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
        let render_data = {
            let data = shared_data.read().await;
            RenderData {
                standings: Arc::clone(&data.standings),
                schedule: Arc::clone(&data.schedule),
                period_scores: Arc::clone(&data.period_scores),
                game_info: Arc::clone(&data.game_info),
                boxscore: Arc::clone(&data.boxscore),
                club_stats: Arc::clone(&data.club_stats),
                western_first: data.config.display_standings_western_first,
                last_refresh: data.last_refresh,
                time_format: data.config.time_format.clone(),
                game_date: data.game_date.clone(),
                error_message: data.error_message.clone(),
                theme: data.config.theme.clone(),
                boxscore_loading: data.boxscore_loading,
                selected_team_abbrev: data.selected_team_abbrev.clone(),
            }
        };

        terminal.draw(|f| {
            let size = f.area();

            // Create main layout - add space for sub-tabs if on Scores or Standings, and status bar at bottom
            let has_subtabs = app_state.has_subtabs();
            let constraints = calculate_layout_constraints(has_subtabs);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(size);

            // Render the frame using extracted function
            render_frame(f, &chunks, &mut app_state, &render_data);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(EVENT_POLL_INTERVAL_MS))? {
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
