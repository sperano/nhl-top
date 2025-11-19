// Module declarations
pub mod widgets;
mod context;
pub mod components;

// Core modules (formerly framework/)
pub mod action;
pub mod component;
pub mod effects;
pub mod helpers;
pub mod keys;
pub mod navigation;
pub mod reducer;
pub mod reducers;
pub mod renderer;
pub mod runtime;
pub mod settings_helpers;
pub mod state;
pub mod table;
pub mod types;

#[cfg(test)]
pub mod testing;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod experimental_tests;

pub use action::{Action, ScoresAction};
pub use component::{Component, Effect, Element};
pub use effects::DataEffects;
pub use keys::key_to_action;
pub use reducer::reduce;
pub use renderer::Renderer;
pub use runtime::Runtime;
pub use state::AppState;
pub use table::{Alignment, CellValue, ColumnDef};
pub use types::{Panel, SettingsCategory, Tab};

use std::io;
use std::sync::Arc;
use std::time::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use nhl_api::Client;
use crate::config::Config;

/// Calculate how many game boxes fit per row based on terminal width
/// GameBox dimensions: 37 wide + 2 margin = 39 per box
fn calculate_boxes_per_row(terminal_width: u16) -> u16 {
    const BOX_WIDTH_WITH_MARGIN: u16 = 39;
    (terminal_width / BOX_WIDTH_WITH_MARGIN).max(1)
}

/// Check if an action is a quit action
fn is_quit_action(action: &Action) -> bool {
    matches!(action, Action::Quit)
}

/// Main entry point for TUI mode
pub async fn run(
    client: Arc<Client>,
    config: Config,
) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create DataEffects handler
    let data_effects = Arc::new(DataEffects::new(client));

    // Create initial AppState with config
    let mut initial_state = AppState::default();
    initial_state.system.config = config.clone();

    // Create runtime with DataEffects
    let mut runtime = Runtime::new(initial_state, data_effects);

    // Trigger initial data load
    runtime.dispatch(Action::RefreshData);

    #[cfg(feature = "development")]
    let mut screenshot_requested = false;

    // Main loop
    loop {
        // Process any actions from effects FIRST (so data loads trigger re-render)
        let actions_processed = runtime.process_actions();
        if actions_processed > 0 {
            tracing::debug!("LOOP: Processed {} actions", actions_processed);
        }

        // Render
        #[cfg(feature = "development")]
        let mut screenshot_buffer: Option<ratatui::buffer::Buffer> = None;

        terminal.draw(|f| {
            let area = f.area();

            // Update boxes_per_row for game grid navigation
            let boxes_per_row = calculate_boxes_per_row(area.width);

            // Dispatch action to update boxes_per_row if it changed
            let current_boxes_per_row = runtime.state().ui.scores.boxes_per_row;
            if boxes_per_row != current_boxes_per_row {
                tracing::debug!("DRAW: boxes_per_row changed: {} -> {}", current_boxes_per_row, boxes_per_row);
                runtime.dispatch(Action::ScoresAction(ScoresAction::UpdateBoxesPerRow(boxes_per_row)));
            }

            // Build virtual tree from current state
            let element = runtime.build();

            // Render virtual tree to ratatui buffer
            let config = &runtime.state().system.config.display;
            let mut renderer = Renderer::new();
            renderer.render(element, area, f.buffer_mut(), config);

            // Clone buffer if screenshot requested
            #[cfg(feature = "development")]
            if screenshot_requested {
                screenshot_buffer = Some(f.buffer_mut().clone());
            }
        })?;

        #[cfg(feature = "development")]
        if let Some(buffer) = screenshot_buffer {
            screenshot_requested = false;
            let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let filename = format!("nhl-screenshot-{}.txt", timestamp);
            let area = ratatui::layout::Rect::new(0, 0, buffer.area().width, buffer.area().height);
            if let Err(e) = crate::dev::screenshot::save_buffer_screenshot(&buffer, area, &filename) {
                tracing::error!("Failed to save screenshot: {}", e);
            } else {
                tracing::info!("Screenshot saved to {}", filename);
            }
        }

        // If actions were processed, continue loop immediately to check for more
        // This ensures UI updates immediately when async data arrives
        if actions_processed > 0 {
            tracing::debug!("Processed {} actions, continuing loop immediately for re-render", actions_processed);
            continue;
        }

        // Poll for keyboard events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                #[cfg(feature = "development")]
                {
                    use crossterm::event::{KeyCode, KeyModifiers};
                    if key.code == KeyCode::Char('S') && key.modifiers.contains(KeyModifiers::SHIFT) {
                        tracing::info!("Screenshot requested via Shift-S");
                        screenshot_requested = true;
                        continue;
                    }
                }

                // Convert key to action
                let action = key_to_action(key, runtime.state());

                // Check for quit action before handling
                let should_quit = action.as_ref().is_some_and(is_quit_action);

                // Dispatch action if we have one
                if let Some(act) = action {
                    runtime.dispatch(act);

                    // Trigger immediate re-render to show state changes
                    if !should_quit {
                        tracing::debug!("ACTION: Continuing loop for immediate re-render");
                        continue;
                    }
                }

                if should_quit {
                    tracing::debug!("ACTION: Quitting application");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_boxes_per_row_with_wide_terminal() {
        // Terminal width = 200, box width = 39
        // 200 / 39 = 5.128... = 5 boxes
        assert_eq!(calculate_boxes_per_row(200), 5);
    }

    #[test]
    fn test_calculate_boxes_per_row_with_narrow_terminal() {
        // Terminal width = 80, box width = 39
        // 80 / 39 = 2.051... = 2 boxes
        assert_eq!(calculate_boxes_per_row(80), 2);
    }

    #[test]
    fn test_calculate_boxes_per_row_with_very_narrow_terminal() {
        // Terminal width = 30, box width = 39
        // 30 / 39 = 0.769... = 0, but max(1) = 1
        assert_eq!(calculate_boxes_per_row(30), 1);
    }

    #[test]
    fn test_calculate_boxes_per_row_with_exact_fit() {
        // Terminal width = 39 * 3 = 117
        // 117 / 39 = 3 boxes exactly
        assert_eq!(calculate_boxes_per_row(117), 3);
    }

    #[test]
    fn test_calculate_boxes_per_row_minimum_is_one() {
        // Even with width 0, should return 1
        assert_eq!(calculate_boxes_per_row(0), 1);
        assert_eq!(calculate_boxes_per_row(1), 1);
        assert_eq!(calculate_boxes_per_row(10), 1);
    }

    #[test]
    fn test_is_quit_action_with_quit() {
        assert!(is_quit_action(&Action::Quit));
    }

    #[test]
    fn test_is_quit_action_with_non_quit_actions() {
        assert!(!is_quit_action(&Action::RefreshData));
        assert!(!is_quit_action(&Action::NavigateTab(Tab::Scores)));
        assert!(!is_quit_action(&Action::ToggleCommandPalette));
        assert!(!is_quit_action(&Action::ScoresAction(ScoresAction::DateLeft)));
    }

    #[test]
    fn test_is_quit_action_with_panel_actions() {
        assert!(!is_quit_action(&Action::PopPanel));
        assert!(!is_quit_action(&Action::SelectPlayer(123456)));
        assert!(!is_quit_action(&Action::SelectTeam("BOS".to_string())));
    }
}
