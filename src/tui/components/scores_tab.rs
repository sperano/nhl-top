use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
//
use nhl_api::{DailySchedule, GameDate, GameMatchup};
//
use crate::config::DisplayConfig;
use crate::tui::{
    component::{Component, Element, RenderableWidget},
};
use crate::tui::widgets::{GameBox, GameState as WidgetGameState};
use crate::commands::scores_format::{PeriodScores, format_period_text};
//
use super::{TabbedPanel, TabbedPanelProps, TabItem};
//
/// Props for ScoresTab component
#[derive(Clone)]
pub struct ScoresTabProps {
    pub game_date: GameDate,
    pub selected_index: usize,
    pub schedule: Option<DailySchedule>,
    pub game_info: HashMap<i64, GameMatchup>,
    pub period_scores: HashMap<i64, PeriodScores>,
    pub box_selection_active: bool,
    pub selected_game_index: Option<usize>,
    pub focused: bool,
}
//
/// ScoresTab component - renders scores with date selector
pub struct ScoresTab;
//
impl Component for ScoresTab {
    type Props = ScoresTabProps;
    type State = ();
    type Message = ();
//
    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        // Use TabbedPanel for date navigation
        self.render_date_tabs(props)
    }
}
//
impl ScoresTab {
    /// Render date tabs using TabbedPanel
    fn render_date_tabs(&self, props: &ScoresTabProps) -> Element {
        const DATE_WINDOW_SIZE: usize = 5;
//
        // Calculate the 5-date window
        let window_base_date = props.game_date.add_days(-(props.selected_index as i64));
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
                let content = self.render_game_list(props);
//
                TabItem::new(key, title, content)
            })
            .collect();
//
        // Active key is the current game_date
        let active_key = self.date_to_key(&props.game_date);
//
        TabbedPanel.view(
            &TabbedPanelProps {
                active_key,
                tabs,
                focused: props.focused,
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
            GameDate::Now => chrono::Local::now().date_naive().format("%m/%d").to_string(),
        }
    }
//
    fn render_game_list(&self, props: &ScoresTabProps) -> Element {
        Element::Widget(Box::new(GameListWidget {
            schedule: props.schedule.clone(),
            period_scores: props.period_scores.clone(),
            game_info: props.game_info.clone(),
            selected_game_index: if props.box_selection_active {
                props.selected_game_index
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
    schedule: Option<DailySchedule>,
    period_scores: HashMap<i64, PeriodScores>,
    game_info: HashMap<i64, GameMatchup>,
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
                    &info.period_descriptor.period_type,
                    info.period_descriptor.number
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
            let start_time = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&game.start_time_utc) {
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
        let current_period = self.game_info.get(&game.id).map(|info| info.period_descriptor.number);
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
impl RenderableWidget for GameListWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        match &self.schedule {
            None => {
                let widget = Paragraph::new("Loading games...").block(
                    Block::default().borders(Borders::ALL).title("Games"),
                );
                ratatui::widgets::Widget::render(widget, area, buf);
            }
            Some(schedule) if schedule.games.is_empty() => {
                let widget = Paragraph::new("No games scheduled").block(
                    Block::default().borders(Borders::ALL).title("Games"),
                );
                ratatui::widgets::Widget::render(widget, area, buf);
            }
            Some(schedule) => {
                // GameBox dimensions: 37 wide × 7 tall
                const GAME_BOX_WIDTH: u16 = 37;
                const GAME_BOX_HEIGHT: u16 = 7;
                const GAME_BOX_MARGIN: u16 = 2;
//
                // Calculate how many game boxes fit in a row
                let boxes_per_row = (area.width / (GAME_BOX_WIDTH + GAME_BOX_MARGIN)).max(1);
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
                let rows = (game_boxes.len() as u16 + boxes_per_row - 1) / boxes_per_row;
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
                        let col_x = area.x + col_idx * (GAME_BOX_WIDTH + GAME_BOX_MARGIN);
                        let box_area = Rect::new(col_x, row_y, GAME_BOX_WIDTH, GAME_BOX_HEIGHT);
//
                        if box_area.x + box_area.width <= area.x + area.width {
                            let (game_box, _) = &game_boxes[game_idx];
                            // Use default display config for rendering
                            let config = DisplayConfig::default();
                            crate::tui::widgets::RenderableWidget::render(game_box, box_area, buf, &config);
                        }
                    }
                }
            }
        }
    }
//
    fn clone_box(&self) -> Box<dyn RenderableWidget> {
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
            game_date: GameDate::today(),
            selected_index: 2,
            schedule: None,
            game_info: HashMap::new(),
            period_scores: HashMap::new(),
            box_selection_active: false,
            selected_game_index: None,
            focused: false,
        };
//
        let element = scores_tab.view(&props, &());
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
