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

// UI Layout Constants

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
