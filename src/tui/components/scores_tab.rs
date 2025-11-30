use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect};
use std::collections::HashMap;
use std::sync::Arc;

use nhl_api::{DailySchedule, GameDate, GameMatchup};

use crate::commands::scores_format::PeriodScores;
use crate::config::DisplayConfig;
use crate::tui::action::Action;
use crate::tui::component::{Component, Effect, Element, ElementWidget};
use crate::tui::document::{Document, DocumentView};
use crate::tui::document_nav::{DocumentNavMsg, DocumentNavState};
use crate::tui::tab_component::{CommonTabMessage, TabMessage, TabState, handle_common_message};
use crate::component_message_impl;

use super::scores_grid_document::ScoresGridDocument;
use super::{TabItem, TabbedPanel, TabbedPanelProps};
//
/// Component state for ScoresTab - managed by the component itself
#[derive(Clone, Debug)]
pub struct ScoresTabState {
    // Date window state
    pub selected_date_index: usize,
    pub game_date: GameDate,

    // Document navigation (replaces browse_mode and selected_game_index)
    pub doc_nav: DocumentNavState,
}

impl Default for ScoresTabState {
    fn default() -> Self {
        Self {
            selected_date_index: 2, // Middle of 5-date window
            game_date: GameDate::today(),
            doc_nav: DocumentNavState::default(),
        }
    }
}

impl TabState for ScoresTabState {
    fn doc_nav(&self) -> &DocumentNavState {
        &self.doc_nav
    }

    fn doc_nav_mut(&mut self) -> &mut DocumentNavState {
        &mut self.doc_nav
    }
}

/// Messages handled by ScoresTab component
#[derive(Clone, Debug)]
pub enum ScoresTabMsg {
    /// Key event when this tab is focused (Phase 3: component handles own keys)
    Key(KeyEvent),

    /// Navigate up request (ESC in browse mode, returns to tab bar otherwise)
    /// Returns Effect::Handled if consumed, Effect::None if should bubble up
    NavigateUp,

    // Date navigation
    NavigateLeft,
    NavigateRight,

    // Browse mode (game selection)
    EnterBoxSelection,
    ExitBoxSelection,

    // Document navigation (delegated)
    DocNav(DocumentNavMsg),

    // Viewport management
    UpdateViewportHeight(u16),

    // Game activation
    ActivateGame,
}

impl TabMessage for ScoresTabMsg {
    fn as_common(&self) -> Option<CommonTabMessage<'_>> {
        match self {
            Self::DocNav(msg) => Some(CommonTabMessage::DocNav(msg)),
            Self::UpdateViewportHeight(h) => Some(CommonTabMessage::UpdateViewportHeight(*h)),
            Self::NavigateUp => Some(CommonTabMessage::NavigateUp),
            _ => None,
        }
    }

    fn from_doc_nav(msg: DocumentNavMsg) -> Self {
        Self::DocNav(msg)
    }
}

// Use macro to eliminate ComponentMessageTrait boilerplate
component_message_impl!(ScoresTabMsg, ScoresTab, ScoresTabState);

/// Props for ScoresTab component (data from parent)
///
/// NOTE: During migration, this still contains UI state fields that should
/// eventually come from ScoresTabState. For Phase 3, we're demonstrating
/// the component can manage its own state, but not fully integrating yet.
#[derive(Clone)]
pub struct ScoresTabProps {
    // API data
    pub schedule: Arc<Option<DailySchedule>>,
    pub game_info: Arc<HashMap<i64, GameMatchup>>,
    pub period_scores: Arc<HashMap<i64, PeriodScores>>,

    // Navigation state
    pub focused: bool,
}
//
/// ScoresTab component - renders scores with date selector
#[derive(Default)]
pub struct ScoresTab;
//
impl Component for ScoresTab {
    type Props = ScoresTabProps;
    type State = ScoresTabState;
    type Message = ScoresTabMsg;

    fn init(_props: &Self::Props) -> Self::State {
        ScoresTabState::default()
    }

    fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
        // Handle common tab messages (DocNav, UpdateViewportHeight, NavigateUp)
        if let Some(effect) = handle_common_message(msg.as_common(), state) {
            return effect;
        }

        // Handle tab-specific messages
        match msg {
            ScoresTabMsg::Key(key) => self.handle_key(key, state),

            ScoresTabMsg::NavigateLeft => {
                // Navigate left in the date window
                if state.selected_date_index > 0 {
                    // Move within the window
                    state.selected_date_index -= 1;
                    state.game_date = state.game_date.add_days(-1);
                } else {
                    // At left edge - shift window left
                    state.game_date = state.game_date.add_days(-1);
                    // selected_date_index stays at 0
                }
                // Refresh schedule for new date (also updates global state and clears old data)
                Effect::Action(Action::RefreshSchedule(state.game_date.clone()))
            }
            ScoresTabMsg::NavigateRight => {
                // Navigate right in the date window
                const DATE_WINDOW_SIZE: usize = 5;
                if state.selected_date_index < DATE_WINDOW_SIZE - 1 {
                    // Move within the window
                    state.selected_date_index += 1;
                    state.game_date = state.game_date.add_days(1);
                } else {
                    // At right edge - shift window right
                    state.game_date = state.game_date.add_days(1);
                    // selected_date_index stays at 4
                }
                // Refresh schedule for new date (also updates global state and clears old data)
                Effect::Action(Action::RefreshSchedule(state.game_date.clone()))
            }
            ScoresTabMsg::EnterBoxSelection => {
                state.enter_browse_mode();
                Effect::None
            }
            ScoresTabMsg::ExitBoxSelection => {
                state.exit_browse_mode();
                Effect::None
            }

            // Game activation
            ScoresTabMsg::ActivateGame => {
                if let Some(focus_idx) = state.doc_nav().focus_index {
                    if let Some(id) = state.doc_nav().focusable_ids.get(focus_idx) {
                        if let crate::tui::document::FocusableId::Link(link_id) = id {
                            // Parse "game_12345" -> 12345
                            if let Some(game_id) = link_id.strip_prefix("game_")
                                .and_then(|s| s.parse::<i64>().ok()) {
                                return Effect::Action(Action::PushDocument(
                                    crate::tui::types::StackedDocument::Boxscore { game_id }
                                ));
                            }
                        }
                    }
                }
                Effect::None
            }

            // Common messages already handled above
            ScoresTabMsg::DocNav(_) | ScoresTabMsg::UpdateViewportHeight(_) | ScoresTabMsg::NavigateUp => {
                unreachable!("Common messages should be handled by handle_common_message")
            }
        }
    }

    fn view(&self, props: &Self::Props, state: &Self::State) -> Element {
        // Phase 7: Now using component state for UI state, props for data
        self.render_date_tabs_from_state(props, state)
    }
}
//
impl ScoresTab {
    /// Render date tabs using component state for UI, props for data (Phase 7)
    fn render_date_tabs_from_state(&self, props: &ScoresTabProps, state: &ScoresTabState) -> Element {
        const DATE_WINDOW_SIZE: usize = 5;
        //
        // Calculate the 5-date window using component state
        let window_base_date = state.game_date.add_days(-(state.selected_date_index as i64));
        let dates: Vec<GameDate> = (0..DATE_WINDOW_SIZE)
            .map(|i| window_base_date.add_days(i as i64))
            .collect();
        //
        // Create TabItems for each date
        let tabs: Vec<TabItem> = dates
            .iter()
            .map(|date| {
                let key = self.date_to_key(date);
                let title = self.format_date_label(date);
                let content = self.render_game_list_from_state(props, state, date);
                //
                TabItem::new(key, title, content)
            })
            .collect();
        //
        // Active key is the current game_date from component state
        let active_key = self.date_to_key(&state.game_date);
        //
        TabbedPanel.view(
            &TabbedPanelProps {
                active_key,
                tabs,
                focused: props.focused && !state.is_browse_mode(),
            },
            &(),
        )
    }

    //
    /// Convert GameDate to string key
    fn date_to_key(&self, date: &GameDate) -> String {
        match date {
            GameDate::Date(naive_date) => naive_date.format("%Y-%m-%d").to_string(),
            GameDate::Now => "now".to_string(),
        }
    }
    //
    /// Format date for tab label (MM/DD)
    fn format_date_label(&self, date: &GameDate) -> String {
        match date {
            GameDate::Date(naive_date) => naive_date.format("%m/%d").to_string(),
            GameDate::Now => chrono::Local::now()
                .date_naive()
                .format("%m/%d")
                .to_string(),
        }
    }
    /// Render game list using the document system with ScoresGridDocument
    fn render_game_list_from_state(&self, props: &ScoresTabProps, state: &ScoresTabState, _date: &GameDate) -> Element {
        // Create the ScoresGridDocument
        // Calculate boxes_per_row (we'll use a reasonable default, terminal width not available here)
        let boxes_per_row = 3; // Will be recalculated based on actual viewport

        let doc = ScoresGridDocument::new(
            props.schedule.clone(),
            props.game_info.clone(),
            props.period_scores.clone(),
            boxes_per_row,
            state.game_date.clone(),
        );

        // Wrap in ScoresDocumentWidget which handles DocumentView rendering
        Element::Widget(Box::new(ScoresDocumentWidget {
            doc: Arc::new(doc),
            focus_index: state.doc_nav.focus_index,
            scroll_offset: state.doc_nav.scroll_offset,
        }))
    }

    /// Handle key events when this tab is focused
    ///
    /// This method handles all key logic that was previously in keys.rs.
    /// Returns an Effect which may be an Action to dispatch.
    fn handle_key(&mut self, key: KeyEvent, state: &mut ScoresTabState) -> Effect {
        if state.is_browse_mode() {
            // Box selection mode - arrow keys navigate games
            match key.code {
                KeyCode::Up => {
                    crate::tui::document_nav::handle_message(
                        &mut state.doc_nav,
                        &DocumentNavMsg::FocusPrev,
                    )
                }
                KeyCode::Down => {
                    crate::tui::document_nav::handle_message(
                        &mut state.doc_nav,
                        &DocumentNavMsg::FocusNext,
                    )
                }
                KeyCode::Left => {
                    crate::tui::document_nav::handle_message(
                        &mut state.doc_nav,
                        &DocumentNavMsg::FocusLeft,
                    )
                }
                KeyCode::Right => {
                    crate::tui::document_nav::handle_message(
                        &mut state.doc_nav,
                        &DocumentNavMsg::FocusRight,
                    )
                }
                KeyCode::Enter => {
                    // Activate the focused game
                    self.update(ScoresTabMsg::ActivateGame, state)
                }
                _ => Effect::None,
            }
        } else {
            // Date navigation mode - arrow keys navigate dates
            match key.code {
                KeyCode::Left => self.update(ScoresTabMsg::NavigateLeft, state),
                KeyCode::Right => self.update(ScoresTabMsg::NavigateRight, state),
                KeyCode::Down | KeyCode::Enter => {
                    // Enter box selection mode
                    self.update(ScoresTabMsg::EnterBoxSelection, state)
                }
                _ => Effect::None,
            }
        }
    }
}

/// Widget that renders a ScoresGridDocument with DocumentView
///
/// This widget wraps DocumentView and applies focus/scroll state from component state.
struct ScoresDocumentWidget {
    doc: Arc<dyn Document>,
    focus_index: Option<usize>,
    scroll_offset: u16,
}

impl ElementWidget for ScoresDocumentWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, display_config: &DisplayConfig) {
        // Create DocumentView with viewport height
        let mut view = DocumentView::new(self.doc.clone(), area.height);

        // Apply focus state
        if let Some(idx) = self.focus_index {
            view.focus_by_index(idx);
        }

        // Apply scroll offset
        view.set_scroll_offset(self.scroll_offset);

        // Render the document
        view.render(area, buf, display_config);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(ScoresDocumentWidget {
            doc: self.doc.clone(),
            focus_index: self.focus_index,
            scroll_offset: self.scroll_offset,
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Fills available space
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //
    #[test]
    fn test_scores_tab_renders_with_no_schedule() {
        let scores_tab = ScoresTab;
        let props = ScoresTabProps {
            schedule: Arc::new(None),
            game_info: Arc::new(HashMap::new()),
            period_scores: Arc::new(HashMap::new()),
            focused: false,
        };
        //
        let state = ScoresTabState::default();
        let element = scores_tab.view(&props, &state);
        //
        match element {
            Element::Container { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected container element"),
        }
    }
    //
}
