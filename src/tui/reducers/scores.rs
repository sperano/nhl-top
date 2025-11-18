use tracing::{debug, trace};

use crate::tui::action::{Action, ScoresAction};
use crate::tui::types::Panel;
use crate::tui::component::Effect;
use crate::tui::state::{AppState, PanelState};

/// Sub-reducer for scores tab
pub fn reduce_scores(state: AppState, action: ScoresAction) -> (AppState, Effect) {
    match action {
        ScoresAction::DateLeft => handle_date_left(state),
        ScoresAction::DateRight => handle_date_right(state),
        ScoresAction::SelectGame => handle_select_game(state),
        ScoresAction::SelectGameById(game_id) => handle_select_game_by_id(state, game_id),
        ScoresAction::EnterBoxSelection => handle_enter_box_selection(state),
        ScoresAction::ExitBoxSelection => handle_exit_box_selection(state),
        ScoresAction::MoveGameSelectionUp => handle_move_game_selection_up(state),
        ScoresAction::MoveGameSelectionDown => handle_move_game_selection_down(state),
        ScoresAction::MoveGameSelectionLeft => handle_move_game_selection_left(state),
        ScoresAction::MoveGameSelectionRight => handle_move_game_selection_right(state),
        ScoresAction::UpdateBoxesPerRow(boxes_per_row) => handle_update_boxes_per_row(state, boxes_per_row),
    }
}

fn handle_date_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    let ui = &mut new_state.ui.scores;

    if ui.selected_date_index == 0 {
        // At edge - shift window left
        ui.game_date = ui.game_date.add_days(-1);
    } else {
        // Within window - move index
        ui.selected_date_index -= 1;
        let window_base = ui.game_date.add_days(-(ui.selected_date_index as i64 + 1));
        ui.game_date = window_base.add_days(ui.selected_date_index as i64);
    }

    // Clear old data
    clear_schedule_data(&mut new_state);

    // Effect: fetch schedule for new date
    (new_state, Effect::Action(Action::RefreshData))
}

fn handle_date_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    let ui = &mut new_state.ui.scores;

    if ui.selected_date_index == 4 {
        // At edge - shift window right
        ui.game_date = ui.game_date.add_days(1);
    } else {
        // Within window - move index
        ui.selected_date_index += 1;
        let window_base = ui.game_date.add_days(-(ui.selected_date_index as i64 - 1));
        ui.game_date = window_base.add_days(ui.selected_date_index as i64);
    }

    // Clear old data
    clear_schedule_data(&mut new_state);

    // Effect: fetch schedule for new date
    (new_state, Effect::Action(Action::RefreshData))
}

fn handle_select_game(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    if let Some(selected_index) = new_state.ui.scores.selected_game_index {
        if let Some(schedule) = &new_state.data.schedule {
            if let Some(game) = schedule.games.get(selected_index) {
                let game_id = game.id;

                // Push boxscore panel onto stack
                new_state.navigation.panel_stack.push(PanelState {
                    panel: Panel::Boxscore { game_id },
                    scroll_offset: 0,
                    selected_index: Some(0), // Start with first player selected
                });

                return (new_state, Effect::None);
            }
        }
    }

    (new_state, Effect::None)
}

fn handle_select_game_by_id(state: AppState, game_id: i64) -> (AppState, Effect) {
    let mut new_state = state;

    // Push boxscore panel onto stack
    new_state.navigation.panel_stack.push(PanelState {
        panel: Panel::Boxscore { game_id },
        scroll_offset: 0,
        selected_index: Some(0), // Start with first player selected
    });

    (new_state, Effect::None)
}

fn handle_enter_box_selection(state: AppState) -> (AppState, Effect) {
    debug!("FOCUS: Entering box selection mode (Scores tab)");
    let mut new_state = state;
    new_state.ui.scores.box_selection_active = true;

    // Initialize selection to first game if we have games
    if new_state.ui.scores.selected_game_index.is_none() {
        if let Some(schedule) = &new_state.data.schedule {
            if !schedule.games.is_empty() {
                new_state.ui.scores.selected_game_index = Some(0);
                trace!("  Initialized game selection to index 0");
            }
        }
    }
    trace!("  Selected game index: {:?}", new_state.ui.scores.selected_game_index);
    (new_state, Effect::None)
}

fn handle_exit_box_selection(state: AppState) -> (AppState, Effect) {
    debug!("FOCUS: Exiting box selection mode (Scores tab)");
    let mut new_state = state;
    new_state.ui.scores.box_selection_active = false;
    (new_state, Effect::None)
}

fn handle_move_game_selection_up(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    if !new_state.ui.scores.box_selection_active {
        return (new_state, Effect::None);
    }

    let old_index = new_state.ui.scores.selected_game_index;
    if let Some(schedule) = &new_state.data.schedule {
        if let Some(current_index) = new_state.ui.scores.selected_game_index {
            let num_games = schedule.games.len();
            let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
            if num_games > 0 && current_index >= boxes_per_row {
                new_state.ui.scores.selected_game_index = Some(current_index - boxes_per_row);
                trace!("Game selection: moved up from {} to {}", current_index, current_index - boxes_per_row);
            }
        }
    }
    if old_index != new_state.ui.scores.selected_game_index {
        debug!("SELECTION: Game index changed: {:?} -> {:?}", old_index, new_state.ui.scores.selected_game_index);
    }
    (new_state, Effect::None)
}

fn handle_move_game_selection_down(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    if !new_state.ui.scores.box_selection_active {
        return (new_state, Effect::None);
    }

    if let Some(schedule) = &new_state.data.schedule {
        if let Some(current_index) = new_state.ui.scores.selected_game_index {
            let num_games = schedule.games.len();
            let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
            if num_games > 0 {
                let new_index = current_index + boxes_per_row;
                if new_index < num_games {
                    new_state.ui.scores.selected_game_index = Some(new_index);
                }
            }
        }
    }
    (new_state, Effect::None)
}

fn handle_move_game_selection_left(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    if !new_state.ui.scores.box_selection_active {
        return (new_state, Effect::None);
    }

    if let Some(current_index) = new_state.ui.scores.selected_game_index {
        let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
        // Get current column position (0-indexed within row)
        let col = current_index % boxes_per_row;
        // Only move left if not already in leftmost column
        if col > 0 {
            new_state.ui.scores.selected_game_index = Some(current_index - 1);
        }
    }
    (new_state, Effect::None)
}

fn handle_move_game_selection_right(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;
    if !new_state.ui.scores.box_selection_active {
        return (new_state, Effect::None);
    }

    if let Some(schedule) = &new_state.data.schedule {
        if let Some(current_index) = new_state.ui.scores.selected_game_index {
            let num_games = schedule.games.len();
            let boxes_per_row = new_state.ui.scores.boxes_per_row as usize;
            if num_games > 0 {
                // Get current column position (0-indexed within row)
                let col = current_index % boxes_per_row;
                // Move right if not at rightmost column and next game exists
                if col < boxes_per_row - 1 && current_index + 1 < num_games {
                    new_state.ui.scores.selected_game_index = Some(current_index + 1);
                }
            }
        }
    }
    (new_state, Effect::None)
}

fn handle_update_boxes_per_row(state: AppState, boxes_per_row: u16) -> (AppState, Effect) {
    let mut new_state = state;
    new_state.ui.scores.boxes_per_row = boxes_per_row;
    (new_state, Effect::None)
}

/// Helper function to clear schedule data when changing dates
fn clear_schedule_data(state: &mut AppState) {
    state.data.schedule = None;
    state.data.game_info.clear();
    state.data.period_scores.clear();
}
