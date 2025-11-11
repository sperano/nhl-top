mod common;
mod scores;
mod standings;
mod stats;
mod players;
mod settings;
mod browser;
mod app;
pub mod navigation;
pub mod widgets;
mod layout;
mod context;
pub mod command_palette;
pub use context::{NavigationContextProvider};

use widgets::RenderableWidget;
use layout::{Layout as LayoutManager};

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
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
    Frame,
};
use crate::types::SharedDataHandle;
use crate::commands::scores_format::PeriodScores;
use crate::config::{self, DisplayConfig};
use app::{AppState, CurrentTab};
use tokio::sync::mpsc;
use nhl_api::{Standing, DailySchedule, GameMatchup, GameDate};

// UI Layout Constants
/// Height of main tab bar
//const TAB_BAR_HEIGHT: u16 = 2;

/// Height of subtab bar (for tabs with subtabs)
const SUBTAB_BAR_HEIGHT: u16 = 2;

/// Height of status bar at bottom
const STATUS_BAR_HEIGHT: u16 = 2;

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

/// Handles number keys (1-6) for direct tab switching
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
        KeyCode::Char('6') => {
            app_state.current_tab = CurrentTab::Browser;
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
    // Special case: if boxscore view is active in Scores tab, pass ALL keys to it
    if matches!(app_state.current_tab, CurrentTab::Scores) && app_state.scores.boxscore_view_active {
        return scores::handle_key(key, &mut app_state.scores, shared_data, refresh_tx).await;
    }

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
                    CurrentTab::Settings => {
                        settings::handle_key(key, &mut app_state.settings, shared_data).await
                    }
                    CurrentTab::Browser => {
                        browser::handle_key(key, &mut app_state.browser, shared_data).await
                    }
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
    // Handle command palette if active
    if app_state.command_palette_active {
        if let Some(ref palette) = app_state.command_palette {
            if palette.is_visible {
                if let Err(e) = command_palette::handler::handle_key(app_state, key, shared_data, refresh_tx).await {
                    tracing::error!("Error handling command palette key: {}", e);
                }
                return false; // Skip normal key handling when palette is active
            }
        }
    }

    // Handle '/' key to open command palette
    if key.code == KeyCode::Char('/') && !app_state.command_palette_active {
        app_state.open_command_palette();
        return false;
    }

    // Special case: if Settings tab is in editing mode or showing modal, route ALL keys to it
    if matches!(app_state.current_tab, CurrentTab::Settings)
        && (app_state.settings.editing.is_some()
            || app_state.settings.list_modal.is_some()
            || app_state.settings.color_modal.is_some()) {
        settings::handle_key(key, &mut app_state.settings, shared_data).await;
        return false;
    }

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

/// Data structure holding all cloned data needed for rendering
/// This avoids holding the RwLock during rendering operations
struct RenderData {
    standings: Arc<Vec<Standing>>,
    schedule: Arc<Option<DailySchedule>>,
    period_scores: Arc<HashMap<i64, PeriodScores>>,
    game_info: Arc<HashMap<i64, GameMatchup>>,
    boxscore: Arc<Option<nhl_api::Boxscore>>,
    club_stats: Arc<HashMap<String, nhl_api::ClubStats>>,
    player_info: Arc<HashMap<i64, nhl_api::PlayerLanding>>,
    western_first: bool,
    last_refresh: Option<SystemTime>,
    #[allow(dead_code)]
    time_format: String,
    game_date: GameDate,
    status_message: Option<String>,
    status_is_error: bool,
    display: Arc<DisplayConfig>,
    config: Arc<config::Config>,
    boxscore_loading: bool,
    selected_team_abbrev: Option<String>,
}

/// Create breadcrumb widget if navigation depth > 0
fn create_breadcrumb(app_state: &AppState, data: &RenderData) -> Option<widgets::EnhancedBreadcrumb> {
    use context::BreadcrumbProvider;

    if !app_state.is_subtab_focused() {
        return None;
    }

    let items = match app_state.current_tab {
        CurrentTab::Scores => {
            let provider = context::ScoresBreadcrumbProvider {
                state: &app_state.scores,
                game_date: &data.game_date,
            };
            provider.get_breadcrumb_items()
        }
        CurrentTab::Standings => app_state.standings.get_breadcrumb_items(),
        CurrentTab::Stats => app_state.stats.get_breadcrumb_items(),
        CurrentTab::Players => app_state.players.get_breadcrumb_items(),
        CurrentTab::Settings => app_state.settings.get_breadcrumb_items(),
        CurrentTab::Browser => vec![],
    };

    if items.is_empty() {
        return None;
    }

    Some(widgets::EnhancedBreadcrumb {
        items,
        separator: " ▸ ".to_string(),
        icon: None,
    })
}

/// Create action bar widget based on current context
fn create_action_bar(app_state: &AppState, data: &RenderData) -> Option<widgets::ActionBar> {
    use context::NavigationContextProvider;

    if !data.display.show_action_bar {
        return None;
    }

    let actions = match app_state.current_tab {
        CurrentTab::Scores => app_state.scores.get_available_actions(),
        CurrentTab::Standings => app_state.standings.get_available_actions(),
        CurrentTab::Stats => app_state.stats.get_available_actions(),
        CurrentTab::Players => app_state.players.get_available_actions(),
        CurrentTab::Settings => app_state.settings.get_available_actions(),
        CurrentTab::Browser => vec![],
    };

    if actions.is_empty() {
        return None;
    }

    Some(widgets::ActionBar { actions })
}

/// Create status bar with refresh info and error messages
fn create_status_bar(data: &RenderData, app_state: &AppState) -> widgets::StatusBar {
    use context::NavigationContextProvider;

    let mut status_bar = widgets::StatusBar::new();

    let hints = match app_state.current_tab {
        CurrentTab::Scores => app_state.scores.get_keyboard_hints(),
        CurrentTab::Standings => app_state.standings.get_keyboard_hints(),
        CurrentTab::Stats => app_state.stats.get_keyboard_hints(),
        CurrentTab::Players => app_state.players.get_keyboard_hints(),
        CurrentTab::Settings => app_state.settings.get_keyboard_hints(),
        CurrentTab::Browser => vec![],
    };

    if !hints.is_empty() {
        status_bar = status_bar.with_hints(hints);
    }

    if let Some(ref error) = data.status_message {
        if data.status_is_error {
            status_bar = status_bar.with_error(error.clone());
        } else {
            status_bar = status_bar.with_status(error.clone());
        }
    }

    if let Some(last_refresh) = data.last_refresh {
        status_bar = status_bar.with_last_refresh(Some(last_refresh));
    }

    status_bar = status_bar.with_refresh_interval(data.config.refresh_interval);

    status_bar
}

/// Renders a single frame with the current application state and data
/// Delegates rendering to tab-specific modules
fn render_frame(f: &mut Frame, app_state: &mut AppState, data: &RenderData) {
    // Create layout manager with all chrome components
    let layout = LayoutManager {
        tab_bar: widgets::TabBar::new(app_state.current_tab, !app_state.is_subtab_focused()),
        breadcrumb: create_breadcrumb(app_state, data),
        action_bar: create_action_bar(app_state, data),
        status_bar: create_status_bar(data, app_state),
        command_palette: app_state.command_palette.clone(),
    };

    // Calculate areas for all components
    let areas = layout.calculate_areas(f.area(), &data.display);

    // Render chrome (tab bar, breadcrumb, action bar, status bar)
    // Note: Command palette is NOT rendered here - it's rendered last to appear on top
    layout.render_chrome(f, &areas, &data.display);

    // Render sub-tabs for tabs that have them (within content area)
    // Note: subtabs are rendered within the content area allocated by the layout manager
    let has_subtabs = app_state.has_subtabs();
    let (subtab_area, main_content_area) = if has_subtabs {
        // Split content area into subtab bar and actual content
        let constraints = vec![
            Constraint::Length(SUBTAB_BAR_HEIGHT),
            Constraint::Min(0),
        ];
        let sub_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(areas.content);
        (Some(sub_chunks[0]), sub_chunks[1])
    } else {
        (None, areas.content)
    };

    // Render subtabs if present
    if let Some(subtab_area) = subtab_area {
        match app_state.current_tab {
            CurrentTab::Scores => {
                scores::render_subtabs(f, subtab_area, &app_state.scores, &data.game_date, &data.display);
            }
            CurrentTab::Standings => {
                standings::render_subtabs(f, subtab_area, &app_state.standings, &data.display);
            }
            CurrentTab::Settings => {
                // Settings has subtabs according to has_subtabs(), but doesn't render them yet
            }
            CurrentTab::Stats | CurrentTab::Players | CurrentTab::Browser => {
                // No subtabs for these tabs
            }
        }
    }

    // Render content for current tab
    match app_state.current_tab {
        CurrentTab::Scores => {
            scores::render_content(
                f,
                main_content_area,
                &mut app_state.scores,
                &data.schedule,
                &data.period_scores,
                &data.game_info,
                &data.display,
                &data.boxscore,
                data.boxscore_loading,
                &data.player_info,
            );
        }
        CurrentTab::Standings => {
            // Update layout cache if needed
            if app_state.standings.layout_cache.is_none() {
                app_state.standings.update_layout(&data.standings, data.western_first);
            }

            standings::render_content(
                f,
                main_content_area,
                &mut app_state.standings,
                &data.display,
                &data.club_stats,
                &data.selected_team_abbrev,
                &data.player_info,
            );
        }
        CurrentTab::Stats => {
            stats::render_content(f, main_content_area);
        }
        CurrentTab::Players => {
            players::render_content(f, main_content_area);
        }
        CurrentTab::Settings => {
            settings::render_content(f, main_content_area, &mut app_state.settings, &data.config);
        }
        CurrentTab::Browser => {
            browser::render_content(f, main_content_area, &app_state.browser, &data.display);
        }
    }

    // Render command palette LAST so it appears on top of all content
    if let Some(ref palette) = app_state.command_palette {
        if palette.is_visible {
            if let Some(palette_area) = areas.command_palette {
                let render_area = Rect::new(0, 0, palette_area.width, palette_area.height);
                let mut palette_buf = Buffer::empty(render_area);
                palette.render(render_area, &mut palette_buf, &data.display);

                let frame_buf = f.buffer_mut();
                for y in 0..palette_area.height {
                    for x in 0..palette_area.width {
                        let cell = &palette_buf[(x, y)];
                        frame_buf[(palette_area.x + x, palette_area.y + y)]
                            .set_symbol(cell.symbol())
                            .set_style(cell.style());
                    }
                }
            }
        }
    }
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
                player_info: Arc::clone(&data.player_info),
                western_first: data.config.display_standings_western_first,
                last_refresh: data.last_refresh,
                time_format: data.config.time_format.clone(),
                game_date: data.game_date.clone(),
                status_message: data.status_message.clone(),
                status_is_error: data.status_is_error,
                display: Arc::new(data.config.display.clone()),
                config: Arc::new(data.config.clone()),
                boxscore_loading: data.boxscore_loading,
                selected_team_abbrev: data.selected_team_abbrev.clone(),
            }
        };

        terminal.draw(|f| {
            render_frame(f, &mut app_state, &render_data);
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
