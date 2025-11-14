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
mod context;
pub mod command_palette;
pub mod framework;
pub mod components;
mod mod_experimental;
pub use context::{NavigationContextProvider, BreadcrumbProvider};
pub use mod_experimental::run_experimental;

use widgets::RenderableWidget;

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

/// Development feature: Save terminal buffer to file
#[cfg(feature = "development")]
fn save_screenshot(buffer: &Buffer, area: Rect) -> std::io::Result<String> {
    use std::io::Write;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("nhl-screenshot-{}.txt", timestamp);

    let mut file = std::fs::File::create(&filename)?;

    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(x, y)];
            write!(file, "{}", cell.symbol())?;
        }
        writeln!(file)?;
    }

    Ok(filename)
}

/// Development feature: Log widget tree structure
#[cfg(feature = "development")]
fn log_widget_tree(app_state: &AppState) {
    tracing::debug!("=== Widget Tree Debug ===");
    tracing::debug!("Current tab: {:?}", app_state.current_tab);
    tracing::debug!("Subtab focused: {}", app_state.is_subtab_focused());
    tracing::debug!("Has subtabs: {}", app_state.has_subtabs());

    match app_state.current_tab {
        CurrentTab::Scores => {
            tracing::debug!("Scores state:");
            tracing::debug!("  - selected_index: {}", app_state.scores.selected_index);
            tracing::debug!("  - box_selection_active: {}", app_state.scores.box_selection_active);
            tracing::debug!("  - selected_box: {:?}", app_state.scores.selected_box);
            tracing::debug!("  - boxscore_view_active: {}", app_state.scores.boxscore_view_active);
            tracing::debug!("  - container: {}", if app_state.scores.container.is_some() { "Some" } else { "None" });
        }
        CurrentTab::Standings => {
            tracing::debug!("Standings state:");
            tracing::debug!("  - view: {:?}", app_state.standings.view);
            tracing::debug!("  - focused_table_index: {:?}", app_state.standings.focused_table_index);
            tracing::debug!("  - num_tables: {}", app_state.standings.team_tables.len());
            tracing::debug!("  - navigation depth: {}", app_state.standings.navigation.stack.depth());
            tracing::debug!("  - container: {}", if app_state.standings.container.is_some() { "Some" } else { "None" });
        }
        CurrentTab::Stats => {
            tracing::debug!("Stats state:");
            tracing::debug!("  - container: {}", if app_state.stats.container.is_some() { "Some" } else { "None" });
        }
        CurrentTab::Players => {
            tracing::debug!("Players state:");
            tracing::debug!("  - container: {}", if app_state.players.container.is_some() { "Some" } else { "None" });
        }
        CurrentTab::Settings => {
            tracing::debug!("Settings state:");
            tracing::debug!("  - editing: {}", app_state.settings.editing.is_some());
            tracing::debug!("  - list_modal: {}", app_state.settings.list_modal.is_some());
            tracing::debug!("  - color_modal: {}", app_state.settings.color_modal.is_some());
        }
        CurrentTab::Browser => {
            tracing::debug!("Browser state:");
            tracing::debug!("  - container: {}", if app_state.browser.container.is_some() { "Some" } else { "None" });
        }
    }

    tracing::debug!("Command palette active: {}", app_state.command_palette_active);
    tracing::debug!("========================");
}

/// Handles ESC key presses - exits subtab mode or signals app exit
/// Returns Some(true) to exit app, Some(false) to continue, None if not ESC
fn handle_esc_key(key: KeyEvent, app_state: &mut AppState) -> Option<bool> {
    if key.code == KeyCode::Esc {
        tracing::debug!("ESC key pressed, subtab_focused={}", app_state.is_subtab_focused());
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

        // If on Standings tab and a table is focused, don't handle ESC here
        // Let the standings handler exit table focus mode
        if matches!(app_state.current_tab, CurrentTab::Standings) && app_state.standings.focused_table_index.is_some() {
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
                    CurrentTab::Stats => {
                        use crate::tui::widgets::focus::InputResult;
                        matches!(stats::handle_key(key, &mut app_state.stats), InputResult::Handled | InputResult::Navigate(_))
                    }
                    CurrentTab::Players => {
                        use crate::tui::widgets::focus::InputResult;
                        matches!(players::handle_key(key, &mut app_state.players), InputResult::Handled | InputResult::Navigate(_))
                    }
                    CurrentTab::Settings => {
                        settings::handle_key(key, &mut app_state.settings, shared_data).await
                    }
                    CurrentTab::Browser => {
                        use crate::tui::widgets::focus::InputResult;
                        matches!(browser::handle_key(key, &mut app_state.browser).await, InputResult::Handled | InputResult::Navigate(_))
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
///
/// Development feature flag: When `screenshot_requested` is Some, it will be set to true
/// when Shift-S is pressed, signaling the main loop to take a screenshot
async fn handle_key_event(
    key: KeyEvent,
    app_state: &mut AppState,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
    #[cfg(feature = "development")]
    screenshot_requested: &mut bool,
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

    // Development features: Screenshot and widget tree debugging
    #[cfg(feature = "development")]
    {
        use crossterm::event::KeyModifiers;

        // Shift-S: Take screenshot (save terminal buffer to file)
        if key.code == KeyCode::Char('S') && key.modifiers.contains(KeyModifiers::SHIFT) {
            tracing::info!("Screenshot requested via Shift-S");
            *screenshot_requested = true;
            return false;
        }

        // Shift-D: Print widget tree to logs
        if key.code == KeyCode::Char('D') && key.modifiers.contains(KeyModifiers::SHIFT) {
            tracing::info!("Widget tree debug requested via Shift-D");
            log_widget_tree(app_state);
            let mut data = shared_data.write().await;
            data.status_message = Some("Widget tree logged to file".to_string());
            data.status_is_error = false;
            return false;
        }
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

// create_breadcrumb, create_action_bar, create_status_bar functions removed - old TUI dead code



// render_frame and run functions removed - old TUI not used, only run_experimental is called


/// OLD TUI - Not used, only run_experimental is called from main.rs
#[allow(dead_code)]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::AppState;
    use crate::tui::common::CommonPanel;
    use crate::tui::context::BreadcrumbProvider;

    #[test]
    fn test_subtab_height_standings_at_root() {
        // Regression test for: "when standings subtab has focus, a new line appears"
        // Bug: Layout was allocating 3 lines for subtabs even when breadcrumb wasn't shown

        let mut app_state = AppState::default();
        app_state.current_tab = CurrentTab::Standings;
        app_state.standings.subtab_focused = true;

        // At root (no panel open), breadcrumb should NOT be shown
        let breadcrumb_items = app_state.standings.get_breadcrumb_items();
        assert!(breadcrumb_items.len() <= crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH);

        // Subtab height calculation (extracted from render_frame logic)
        let subtab_height = if app_state.is_subtab_focused() {
            match app_state.current_tab {
                CurrentTab::Scores => {
                    SUBTAB_BAR_HEIGHT // Scores has 2 items, not shown
                }
                CurrentTab::Standings => {
                    let breadcrumb_items = app_state.standings.get_breadcrumb_items();
                    if breadcrumb_items.len() > crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH {
                        3 // 3 lines (tabs + separator + breadcrumb)
                    } else {
                        SUBTAB_BAR_HEIGHT // 2 lines (tabs + separator)
                    }
                }
                _ => SUBTAB_BAR_HEIGHT,
            }
        } else {
            SUBTAB_BAR_HEIGHT
        };

        // Should be 2 lines (tabs + separator), not 3
        assert_eq!(subtab_height, SUBTAB_BAR_HEIGHT);
        assert_eq!(subtab_height, 2);
    }

    #[test]
    fn test_subtab_height_standings_in_panel() {
        // When in a panel, breadcrumb SHOULD be shown, so height should be 3

        let mut app_state = AppState::default();
        app_state.current_tab = CurrentTab::Standings;
        app_state.standings.subtab_focused = true;

        // Navigate into a panel
        let panel = CommonPanel::TeamDetail {
            team_name: "Canadiens".to_string(),
            team_abbrev: "MTL".to_string(),
            wins: 30,
            losses: 20,
            ot_losses: 5,
            points: 65,
            division_name: "Atlantic".to_string(),
            conference_name: Some("Eastern".to_string()),
        };
        app_state.standings.navigation.navigate_to(panel);

        // In a panel - breadcrumb items should exceed BREADCRUMB_MIN_DEPTH
        let breadcrumb_items = app_state.standings.get_breadcrumb_items();
        assert!(breadcrumb_items.len() > crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH);

        // Subtab height calculation
        let subtab_height = if app_state.is_subtab_focused() {
            match app_state.current_tab {
                CurrentTab::Scores => {
                    SUBTAB_BAR_HEIGHT // Scores has 2 items, not shown
                }
                CurrentTab::Standings => {
                    let breadcrumb_items = app_state.standings.get_breadcrumb_items();
                    if breadcrumb_items.len() > crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH {
                        3 // Should be 3 lines (breadcrumb shown)
                    } else {
                        SUBTAB_BAR_HEIGHT
                    }
                }
                _ => SUBTAB_BAR_HEIGHT,
            }
        } else {
            SUBTAB_BAR_HEIGHT
        };

        // Should be 3 lines (tabs + separator + breadcrumb)
        assert_eq!(subtab_height, 3);
    }

    #[test]
    fn test_subtab_height_standings_not_focused() {
        // When subtab is NOT focused, should always be 2 lines

        let mut app_state = AppState::default();
        app_state.current_tab = CurrentTab::Standings;
        app_state.standings.subtab_focused = false; // NOT focused

        let subtab_height = if app_state.is_subtab_focused() {
            match app_state.current_tab {
                CurrentTab::Scores => {
                    SUBTAB_BAR_HEIGHT // Scores has 2 items, not shown
                }
                CurrentTab::Standings => {
                    let breadcrumb_items = app_state.standings.get_breadcrumb_items();
                    if breadcrumb_items.len() > crate::tui::common::subtab::BREADCRUMB_MIN_DEPTH {
                        3
                    } else {
                        SUBTAB_BAR_HEIGHT
                    }
                }
                _ => SUBTAB_BAR_HEIGHT,
            }
        } else {
            SUBTAB_BAR_HEIGHT
        };

        // Should be 2 lines
        assert_eq!(subtab_height, SUBTAB_BAR_HEIGHT);
        assert_eq!(subtab_height, 2);
    }
}
