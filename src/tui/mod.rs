//! TUI module with React-like implementation
//!
//! This module provides the terminal user interface using a React-like architecture.

// Module declarations
mod common;
mod scores;
mod standings;
mod settings;
pub mod navigation;
pub mod widgets;
mod context;
pub mod framework;
pub mod components;

#[cfg(test)]
pub mod testing;

//pub use context::{NavigationContextProvider, BreadcrumbProvider};

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

#[cfg(feature = "development")]
use ratatui::{buffer::Buffer, layout::Rect};
use nhl_api::Client;
use crate::config::Config;

use self::framework::{Runtime, DataEffects, Renderer, Action};
use self::framework::action::ScoresAction;
use self::framework::keys::key_to_action;

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

/// Run the React-like TUI (main entry point)
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
    let mut initial_state = self::framework::AppState::default();
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
        tracing::trace!("LOOP: Start of main loop iteration");

        // Process any actions from effects FIRST (so data loads trigger re-render)
        let actions_processed = runtime.process_actions();
        tracing::trace!("LOOP: Processed {} actions", actions_processed);

        // Render
        tracing::trace!("LOOP: Starting terminal.draw()");
        terminal.draw(|f| {
            tracing::trace!("DRAW: Inside terminal.draw callback");
            let area = f.size();
            tracing::trace!("DRAW: Got terminal area: {:?}", area);

            // Update boxes_per_row for game grid navigation
            // GameBox dimensions: 37 wide + 2 margin = 39 per box
            const GAME_BOX_WIDTH: u16 = 37;
            const GAME_BOX_MARGIN: u16 = 2;
            let boxes_per_row = (area.width / (GAME_BOX_WIDTH + GAME_BOX_MARGIN)).max(1);

            // Dispatch action to update boxes_per_row if it changed
            let current_boxes_per_row = runtime.state().ui.scores.boxes_per_row;
            tracing::trace!("DRAW: boxes_per_row check: current={}, calculated={}", current_boxes_per_row, boxes_per_row);
            if boxes_per_row != current_boxes_per_row {
                tracing::trace!("DRAW: Dispatching UpdateBoxesPerRow");
                runtime.dispatch(Action::ScoresAction(ScoresAction::UpdateBoxesPerRow(boxes_per_row)));
            }

            tracing::trace!("DRAW: About to call runtime.build()");
            // Build virtual tree from current state
            let element = runtime.build();
            tracing::trace!("DRAW: runtime.build() completed");

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
