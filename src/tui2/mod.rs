mod traits;
mod app;
mod components;
mod views;
mod theme;

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
use tokio::sync::mpsc;

use crate::SharedDataHandle;
use app::{AppState, Tab};
use traits::{View, KeyResult};
use components::{render_tab_bar, render_breadcrumb, render_status_bar};
use views::scores::GameListView;
use views::standings::ViewSelectorView;
use views::stats::CategoryListView;
use views::settings::SettingsFormView;

const EVENT_POLL_INTERVAL_MS: u64 = 100;

pub async fn run(shared_data: SharedDataHandle, refresh_tx: mpsc::Sender<()>) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize with Scores tab
    let initial_view: Box<dyn View> = Box::new(GameListView::new(shared_data.clone()));
    let mut app_state = AppState::new(Tab::Scores, initial_view);

    // Main event loop
    loop {
        // Get rendering data
        let (last_refresh, time_format, error_message) = {
            let data = shared_data.read().await;
            (
                data.last_refresh,
                data.config.time_format.clone(),
                data.error_message.clone(),
            )
        };

        // Render
        terminal.draw(|f| {
            let size = f.area();

            // Create layout: tab bar, breadcrumb, content, status bar
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),  // Tab bar
                    Constraint::Length(2),  // Breadcrumb
                    Constraint::Min(0),     // Content
                    Constraint::Length(1),  // Status bar
                ])
                .split(size);

            // Render components
            render_tab_bar(f, chunks[0], app_state.current_tab);
            render_breadcrumb(f, chunks[1], &app_state.breadcrumb);

            // Render current view
            app_state.current_view().render(f, chunks[2], true);

            // Render status bar
            render_status_bar(
                f,
                chunks[3],
                app_state.at_root(),
                error_message.as_deref(),
                last_refresh,
                &time_format,
            );
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(EVENT_POLL_INTERVAL_MS))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(key, &mut app_state, &shared_data).await {
                    break; // Exit requested
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

async fn handle_key_event(
    key: KeyEvent,
    app_state: &mut AppState,
    shared_data: &SharedDataHandle,
) -> bool {
    // Handle global quit
    if key.code == KeyCode::Char('q') {
        return true; // Exit
    }

    // Handle tab switching at root level
    if app_state.at_root() {
        match key.code {
            KeyCode::Char('1') => {
                switch_tab(app_state, Tab::Scores, shared_data).await;
                return false;
            }
            KeyCode::Char('2') => {
                switch_tab(app_state, Tab::Standings, shared_data).await;
                return false;
            }
            KeyCode::Char('3') => {
                switch_tab(app_state, Tab::Stats, shared_data).await;
                return false;
            }
            KeyCode::Char('4') => {
                switch_tab(app_state, Tab::Settings, shared_data).await;
                return false;
            }
            KeyCode::Left => {
                let new_tab = app_state.current_tab.prev();
                switch_tab(app_state, new_tab, shared_data).await;
                return false;
            }
            KeyCode::Right => {
                let new_tab = app_state.current_tab.next();
                switch_tab(app_state, new_tab, shared_data).await;
                return false;
            }
            _ => {}
        }
    }

    // Let the current view handle the key
    let result = app_state.current_view().handle_key(key);

    match result {
        KeyResult::Handled => false,
        KeyResult::NotHandled => false,
        KeyResult::DrillDown(new_view) => {
            app_state.push_view(new_view);
            false
        }
        KeyResult::GoBack => {
            app_state.pop_view();
            false
        }
        KeyResult::Quit => true,
    }
}

async fn switch_tab(app_state: &mut AppState, new_tab: Tab, shared_data: &SharedDataHandle) {
    if app_state.current_tab == new_tab {
        return; // Already on this tab
    }

    app_state.current_tab = new_tab;

    // Create root view for the new tab
    let root_view: Box<dyn View> = match new_tab {
        Tab::Scores => Box::new(GameListView::new(shared_data.clone())),
        Tab::Standings => Box::new(ViewSelectorView::new(shared_data.clone())),
        Tab::Stats => Box::new(CategoryListView::new()),
        Tab::Settings => Box::new(SettingsFormView::new()),
    };

    app_state.replace_root(root_view);
}
