use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
//
use nhl_api::{DailySchedule, GameDate, GameMatchup};
//
use crate::commands::scores_format::{format_period_text, PeriodScores};
use crate::config::DisplayConfig;
use crate::layout_constants::SCORE_BOX_WIDTH;
use crate::tui::action::{Action, ComponentMessageTrait};
use crate::tui::component::{Component, Effect, Element, ElementWidget};
use crate::tui::widgets::{GameBox, GameState as WidgetGameState};
//
use super::{TabItem, TabbedPanel, TabbedPanelProps};
//
/// Component state for ScoresTab - managed by the component itself
#[derive(Clone, Debug)]
pub struct ScoresTabState {
    pub selected_date_index: usize,
    pub game_date: GameDate,
    pub browse_mode: bool,
    pub selected_game_index: Option<usize>,
}

impl Default for ScoresTabState {
    fn default() -> Self {
        Self {
            selected_date_index: 2, // Middle of 5-date window
            game_date: GameDate::today(),
            browse_mode: false,
            selected_game_index: None,
        }
    }
}

/// Messages handled by ScoresTab component
#[derive(Clone, Debug)]
pub enum ScoresTabMsg {
    NavigateLeft,
    NavigateRight,
    EnterBoxSelection,
    ExitBoxSelection,
    MoveGameSelectionUp(u16),    // boxes_per_row
    MoveGameSelectionDown(u16),  // boxes_per_row
    MoveGameSelectionLeft,
    MoveGameSelectionRight,
}

impl ComponentMessageTrait for ScoresTabMsg {
    fn apply(&self, state: &mut dyn Any) -> Effect {
        if let Some(scores_state) = state.downcast_mut::<ScoresTabState>() {
            let mut component = ScoresTab;
            component.update(self.clone(), scores_state)
        } else {
            Effect::None
        }
    }

    fn clone_box(&self) -> Box<dyn ComponentMessageTrait> {
        Box::new(self.clone())
    }
}

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
        match msg {
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
                state.browse_mode = true;
                state.selected_game_index = Some(0);
                Effect::None
            }
            ScoresTabMsg::ExitBoxSelection => {
                state.browse_mode = false;
                state.selected_game_index = None;
                Effect::None
            }
            ScoresTabMsg::MoveGameSelectionUp(boxes_per_row) => {
                if let Some(idx) = state.selected_game_index {
                    if idx >= boxes_per_row as usize {
                        state.selected_game_index = Some(idx - boxes_per_row as usize);
                    }
                }
                Effect::None
            }
            ScoresTabMsg::MoveGameSelectionDown(boxes_per_row) => {
                // TODO: Get game count from schedule and bounds check
                if let Some(idx) = state.selected_game_index {
                    state.selected_game_index = Some(idx + boxes_per_row as usize);
                }
                Effect::None
            }
            ScoresTabMsg::MoveGameSelectionLeft => {
                if let Some(idx) = state.selected_game_index {
                    if idx > 0 {
                        state.selected_game_index = Some(idx - 1);
                    }
                }
                Effect::None
            }
            ScoresTabMsg::MoveGameSelectionRight => {
                if let Some(idx) = state.selected_game_index {
                    state.selected_game_index = Some(idx + 1);
                }
                Effect::None
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
                let content = self.render_game_list_from_state(props, state);
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
                focused: props.focused && !state.browse_mode,
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
    //
    fn render_game_list_from_state(&self, props: &ScoresTabProps, state: &ScoresTabState) -> Element {
        Element::Widget(Box::new(GameListWidget {
            schedule: props.schedule.clone(),
            period_scores: props.period_scores.clone(),
            game_info: props.game_info.clone(),
            selected_game_index: if state.browse_mode {
                state.selected_game_index
            } else {
                None
            },
        }))
    }

}
//
//
/// Game list widget - shows games for selected date as GameBox widgets
struct GameListWidget {
    schedule: Arc<Option<DailySchedule>>,
    period_scores: Arc<HashMap<i64, PeriodScores>>,
    game_info: Arc<HashMap<i64, GameMatchup>>,
    selected_game_index: Option<usize>,
}
//
impl GameListWidget {
    /// Convert schedule game to GameBox widget
    fn create_game_box(&self, game: &nhl_api::ScheduleGame, selected: bool) -> GameBox {
        // Determine game state
        let state = if game.game_state.is_final() {
            WidgetGameState::Final
        } else if game.game_state.has_started() {
            // Get period text and time from game_info
            if let Some(info) = self.game_info.get(&game.id) {
                let period_text = format_period_text(
                    info.period_descriptor.period_type,
                    info.period_descriptor.number,
                );
                let (time_remaining, in_intermission) = if let Some(clock) = &info.clock {
                    (Some(clock.time_remaining.clone()), clock.in_intermission)
                } else {
                    (None, false)
                };
                WidgetGameState::Live {
                    period_text,
                    time_remaining,
                    in_intermission,
                }
            } else {
                WidgetGameState::Live {
                    period_text: "Live".to_string(),
                    time_remaining: None,
                    in_intermission: false,
                }
            }
        } else {
            // Scheduled game - format start time
            let start_time =
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
                    let local_time: chrono::DateTime<chrono::Local> = parsed.into();
                    local_time.format("%I:%M %p").to_string()
                } else {
                    game.start_time_utc.clone()
                };
            WidgetGameState::Scheduled { start_time }
        };
        //
        // Get scores and period details
        let (away_score, home_score, away_periods, home_periods, has_ot, has_so) =
            if let Some(scores) = self.period_scores.get(&game.id) {
                (
                    Some(scores.away_total()),
                    Some(scores.home_total()),
                    Some(scores.away_periods.clone()),
                    Some(scores.home_periods.clone()),
                    scores.has_ot,
                    scores.has_so,
                )
            } else {
                (None, None, None, None, false, false)
            };
        //
        // Get current period
        let current_period = self
            .game_info
            .get(&game.id)
            .map(|info| info.period_descriptor.number);
        //
        GameBox::new(
            game.away_team.abbrev.clone(),
            game.home_team.abbrev.clone(),
            away_score,
            home_score,
            away_periods,
            home_periods,
            has_ot,
            has_so,
            current_period,
            state,
            selected,
        )
    }
}
//
impl ElementWidget for GameListWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        match self.schedule.as_ref().as_ref() {
            None => {
                let widget = Paragraph::new("Loading games...")
                    .block(Block::default().borders(Borders::ALL).title("Games"));
                ratatui::widgets::Widget::render(widget, area, buf);
            }
            Some(schedule) if schedule.games.is_empty() => {
                let widget = Paragraph::new("No games scheduled")
                    .block(Block::default().borders(Borders::ALL).title("Games"));
                ratatui::widgets::Widget::render(widget, area, buf);
            }
            Some(schedule) => {
                const GAME_BOX_HEIGHT: u16 = 7;
                const GAME_BOX_MARGIN: u16 = 2;
                //
                // Calculate how many game boxes fit in a row
                let boxes_per_row = (area.width / (SCORE_BOX_WIDTH + GAME_BOX_MARGIN)).max(1);
                //
                // Create game boxes
                let game_boxes: Vec<(GameBox, usize)> = schedule
                    .games
                    .iter()
                    .enumerate()
                    .map(|(index, game)| {
                        let selected = self.selected_game_index == Some(index);
                        (self.create_game_box(game, selected), index)
                    })
                    .collect();
                //
                // Render in grid layout
                let rows = (game_boxes.len() as u16).div_ceil(boxes_per_row);
                //
                for row_idx in 0..rows {
                    let row_y = area.y + row_idx * (GAME_BOX_HEIGHT + 1);
                    if row_y + GAME_BOX_HEIGHT > area.y + area.height {
                        break; // Don't render outside area
                    }
                    //
                    for col_idx in 0..boxes_per_row {
                        let game_idx = (row_idx * boxes_per_row + col_idx) as usize;
                        if game_idx >= game_boxes.len() {
                            break;
                        }
                        //
                        let col_x = area.x + col_idx * (SCORE_BOX_WIDTH + GAME_BOX_MARGIN);
                        let box_area = Rect::new(col_x, row_y, SCORE_BOX_WIDTH, GAME_BOX_HEIGHT);
                        //
                        if box_area.x + box_area.width <= area.x + area.width {
                            let (game_box, _) = &game_boxes[game_idx];
                            // Use default display config for rendering
                            let config = DisplayConfig::default();
                            crate::tui::widgets::SimpleWidget::render(
                                game_box, box_area, buf, &config,
                            );
                        }
                    }
                }
            }
        }
    }
    //
    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(GameListWidget {
            schedule: self.schedule.clone(),
            period_scores: self.period_scores.clone(),
            game_info: self.game_info.clone(),
            selected_game_index: self.selected_game_index,
        })
    }
}
//
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
