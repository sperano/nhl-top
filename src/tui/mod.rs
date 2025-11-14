mod common;
mod scores;
mod standings;
mod settings;
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

use crate::types::SharedDataHandle;

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

