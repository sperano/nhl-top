//! # TUI Trait Abstractions
//!
//! This module defines trait-based abstractions for the TUI tab system.
//! These traits provide a consistent interface for implementing new tabs
//! and ensure all tabs follow the same patterns.
//!
//! ## Design Philosophy
//!
//! The trait system separates concerns into:
//! - **Rendering**: `TabContent`, `TabViewWithSubtabs`
//! - **Input Handling**: `SyncKeyHandler`, `AsyncKeyHandler`
//! - **State Management**: Each tab owns its state
//!
//! ## Usage Example
//!
//! ```ignore
//! use crate::tui::traits::{TabContent, SyncKeyHandler};
//!
//! pub struct MyTabState {
//!     selected_index: usize,
//! }
//!
//! impl TabContent for MyTabState {
//!     fn render_content(&mut self, f: &mut Frame, area: Rect, selection_fg: Color) {
//!         // Render tab content
//!     }
//! }
//!
//! impl SyncKeyHandler for MyTabState {
//!     fn handle_key(&mut self, key: KeyEvent) -> bool {
//!         // Handle keyboard input
//!         match key.code {
//!             KeyCode::Up => { self.selected_index -= 1; true }
//!             _ => false
//!         }
//!     }
//! }
//! ```

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, style::Color};
use tokio::sync::mpsc;
use crate::SharedDataHandle;

/// Result type for key handling - indicates whether the key was handled
///
/// `true` means the key was consumed by the handler
/// `false` means the key was not recognized and should be passed to parent handlers
pub type KeyHandled = bool;

/// Trait for tab views that have subtabs (like Scores and Standings)
///
/// Tabs implementing this trait have two navigation modes:
/// 1. **Main tab mode**: Navigate between main tabs
/// 2. **Subtab mode**: Navigate within the tab's subtabs
///
/// Examples:
/// - **Scores**: Date navigation subtabs
/// - **Standings**: View selection subtabs (Division/Conference/League/Wildcard)
pub trait TabViewWithSubtabs {
    /// Render the subtab bar (date selector, view selector, etc.)
    ///
    /// This should be rendered between the main tab bar and content area.
    ///
    /// # Arguments
    /// - `f`: Frame to render to
    /// - `area`: Rectangle to render within
    /// - `selection_fg`: Foreground color for selected items (when focused)
    /// - `unfocused_selection_fg`: Foreground color for selected items (when unfocused)
    fn render_subtabs(
        &self,
        f: &mut Frame,
        area: Rect,
        selection_fg: Color,
        unfocused_selection_fg: Color,
    );

    /// Check if subtab mode is currently focused
    ///
    /// When `true`, arrow keys navigate subtabs instead of main tabs.
    fn is_subtab_focused(&self) -> bool;

    /// Enter subtab focus mode
    ///
    /// Called when user presses Down/Enter from main tab navigation.
    fn enter_subtab_mode(&mut self);

    /// Exit subtab focus mode
    ///
    /// Called when user presses Up/Esc from subtab navigation.
    fn exit_subtab_mode(&mut self);
}

/// Trait for synchronous tab key handling
///
/// Used by tabs that don't need async operations (Standings, Stats, Players, Settings).
/// For tabs that need network calls, use `AsyncKeyHandler` instead.
pub trait SyncKeyHandler {
    /// Handle a key event synchronously
    ///
    /// # Arguments
    /// - `key`: The keyboard event to handle
    ///
    /// # Returns
    /// - `true` if the key was handled (consumed)
    /// - `false` if the key was not recognized (pass to parent)
    ///
    /// # Example
    /// ```ignore
    /// fn handle_key(&mut self, key: KeyEvent) -> bool {
    ///     match key.code {
    ///         KeyCode::Up => {
    ///             self.selected_index = self.selected_index.saturating_sub(1);
    ///             true
    ///         }
    ///         KeyCode::Down => {
    ///             self.selected_index += 1;
    ///             true
    ///         }
    ///         _ => false
    ///     }
    /// }
    /// ```
    fn handle_key(&mut self, key: KeyEvent) -> KeyHandled;
}

/// Trait for asynchronous tab key handling
///
/// Used by tabs that need network operations (Scores tab for fetching game data).
/// For purely UI-based tabs, use `SyncKeyHandler` instead.
#[async_trait::async_trait]
pub trait AsyncKeyHandler {
    /// Handle a key event asynchronously
    ///
    /// # Arguments
    /// - `key`: The keyboard event to handle
    /// - `shared_data`: Shared application data (for reading/writing game data, config, etc.)
    /// - `refresh_tx`: Channel for triggering background data refresh
    ///
    /// # Returns
    /// - `true` if the key was handled (consumed)
    /// - `false` if the key was not recognized (pass to parent)
    ///
    /// # Example
    /// ```ignore
    /// async fn handle_key(
    ///     &mut self,
    ///     key: KeyEvent,
    ///     shared_data: &SharedDataHandle,
    ///     refresh_tx: &mpsc::Sender<()>,
    /// ) -> bool {
    ///     match key.code {
    ///         KeyCode::Enter => {
    ///             // Trigger data fetch
    ///             refresh_tx.send(()).await.ok();
    ///             true
    ///         }
    ///         _ => false
    ///     }
    /// }
    /// ```
    async fn handle_key(
        &mut self,
        key: KeyEvent,
        shared_data: &SharedDataHandle,
        refresh_tx: &mpsc::Sender<()>,
    ) -> KeyHandled;
}

/// Trait for tab content rendering
///
/// All tabs must implement this to render their main content area.
/// This is the area below the tab bar (and subtab bar if present).
pub trait TabContent {
    /// Render the main content area of the tab
    ///
    /// # Arguments
    /// - `f`: Frame to render to
    /// - `area`: Rectangle to render within (main content area)
    /// - `selection_fg`: Foreground color for selected items
    ///
    /// # Implementation Notes
    /// - Content should be scrollable if it exceeds `area.height`
    /// - Use the `Scrollable` widget from `common::scrollable` for scrolling
    /// - Render "Loading..." or empty state messages when appropriate
    fn render_content(
        &mut self,
        f: &mut Frame,
        area: Rect,
        selection_fg: Color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example test structure for trait implementations
    // (actual tabs would have their own test modules)

    #[test]
    fn test_key_handled_type() {
        let handled: KeyHandled = true;
        assert!(handled);

        let not_handled: KeyHandled = false;
        assert!(!not_handled);
    }
}
