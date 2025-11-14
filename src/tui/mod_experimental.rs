//! Experimental React-like TUI implementation
//!
//! This module is a work-in-progress migration to a React-like architecture.
//! It runs alongside the existing TUI code during the migration phase.

use std::io;
use std::sync::Arc;
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

use super::framework::{Runtime, DataEffects, Renderer, Action};
use super::framework::action::ScoresAction;
use super::framework::keys::key_to_action;

/// Event polling interval in milliseconds
const EVENT_POLL_INTERVAL_MS: u64 = 100;

/// Development feature: Save terminal buffer to a text file
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

/// Run the experimental React-like TUI
pub async fn run_experimental(
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
    let mut initial_state = super::framework::AppState::default();
    initial_state.system.config = config.clone();

    // Create runtime with DataEffects
    let mut runtime = Runtime::new(initial_state, data_effects);

    // Create renderer
    let mut renderer = Renderer::new();

    // Trigger initial data load
    runtime.dispatch(Action::RefreshData);

    // Development feature: Screenshot support
    #[cfg(feature = "development")]
    let mut screenshot_requested = false;
    #[cfg(feature = "development")]
    let mut screenshot_buffer: Option<(Buffer, Rect)> = None;

    // Main loop
    loop {
        // Process any actions from effects FIRST (so data loads trigger re-render)
        let actions_processed = runtime.process_actions();

        // Render
        terminal.draw(|f| {
            let area = f.size();

            // Update boxes_per_row for game grid navigation
            // GameBox dimensions: 37 wide + 2 margin = 39 per box
            const GAME_BOX_WIDTH: u16 = 37;
            const GAME_BOX_MARGIN: u16 = 2;
            let boxes_per_row = (area.width / (GAME_BOX_WIDTH + GAME_BOX_MARGIN)).max(1);

            // Dispatch action to update boxes_per_row if it changed
            let current_boxes_per_row = runtime.state().ui.scores.boxes_per_row;
            if boxes_per_row != current_boxes_per_row {
                runtime.dispatch(Action::ScoresAction(ScoresAction::UpdateBoxesPerRow(boxes_per_row)));
            }

            // Build virtual tree from current state
            let element = runtime.build();

            // Render virtual tree to ratatui buffer
            let config = &runtime.state().system.config.display;
            renderer.render(element, area, f.buffer_mut(), config);

            // Development feature: Clone buffer if screenshot requested
            #[cfg(feature = "development")]
            if screenshot_requested {
                screenshot_buffer = Some((f.buffer_mut().clone(), area));
            }
        })?;

        // Development feature: Save screenshot if captured
        #[cfg(feature = "development")]
        if let Some((buffer, area)) = screenshot_buffer.take() {
            screenshot_requested = false;
            match save_screenshot(&buffer, area) {
                Ok(filename) => {
                    tracing::info!("Screenshot saved to: {}", filename);
                    // TODO: Show status message in UI when we have status bar
                }
                Err(e) => {
                    tracing::error!("Failed to save screenshot: {}", e);
                }
            }
        }

        // If actions were processed, continue loop immediately to check for more
        // This ensures UI updates immediately when async data arrives
        if actions_processed > 0 {
            tracing::trace!("Processed {} actions, continuing loop immediately for re-render", actions_processed);
            continue;
        }

        // Handle events (only block if no actions were processed)
        if event::poll(std::time::Duration::from_millis(EVENT_POLL_INTERVAL_MS))? {
            if let Event::Key(key) = event::read()? {
                // Development feature: Shift-S for screenshot
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
                let should_quit = if let Some(ref act) = action {
                    matches!(act, Action::Quit)
                } else {
                    false
                };

                // Dispatch action if we have one
                if let Some(act) = action {
                    runtime.dispatch(act);
                }

                if should_quit {
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
