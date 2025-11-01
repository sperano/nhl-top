use crossterm::event::{KeyCode, KeyEvent};
use crate::commands::standings::GroupBy;
use super::State;

pub fn handle_key(
    key: KeyEvent,
    state: &mut State,
    standings_data: &[nhl_api::Standing],
    western_first: bool,
) -> bool {
    // Calculate column structure based on view
    let (teams_in_column, max_columns) = calculate_column_info(standings_data, state.view, western_first);
    let team_count_in_current_column = teams_in_column.get(state.selected_column).copied().unwrap_or(0);

    // If team selection is active, handle team navigation
    if state.team_selection_active {
        match key.code {
            KeyCode::Up => {
                if state.selected_team_index == 0 {
                    // At first team in column, exit team selection mode
                    state.team_selection_active = false;
                    true
                } else {
                    // Navigate up within current column
                    state.selected_team_index = state.selected_team_index.saturating_sub(1);
                    true
                }
            }
            KeyCode::Down => {
                // Navigate down within current column
                if state.selected_team_index + 1 < team_count_in_current_column {
                    state.selected_team_index += 1;
                }
                true
            }
            KeyCode::Left => {
                // Switch to left column, maintaining same rank/index
                if state.selected_column > 0 {
                    state.selected_column -= 1;
                    // Keep same index, but clamp to new column's team count
                    let new_column_team_count = teams_in_column.get(state.selected_column).copied().unwrap_or(0);
                    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
                        state.selected_team_index = new_column_team_count - 1;
                    }
                }
                true
            }
            KeyCode::Right => {
                // Switch to right column, maintaining same rank/index
                if state.selected_column + 1 < max_columns {
                    state.selected_column += 1;
                    // Keep same index, but clamp to new column's team count
                    let new_column_team_count = teams_in_column.get(state.selected_column).copied().unwrap_or(0);
                    if state.selected_team_index >= new_column_team_count && new_column_team_count > 0 {
                        state.selected_team_index = new_column_team_count - 1;
                    }
                }
                true
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Handle scrolling even in team selection mode
                state.scrollable.handle_key(key)
            }
            KeyCode::Enter => {
                // Log the selected team for debugging
                let teams_in_column = get_teams_in_column_for_handler(standings_data, state.view, state.selected_column, western_first);
                if let Some(team) = teams_in_column.get(state.selected_team_index) {
                    let column_name = get_column_name(state.view, state.selected_column, western_first);
                    tracing::info!(
                        "Selected team: {} | View: {:?} | Column: {} ({}) | Index: {} | Division: {} | Conference: {}",
                        team.team_common_name.default,
                        state.view,
                        state.selected_column,
                        column_name,
                        state.selected_team_index,
                        team.division_name,
                        team.conference_name.as_deref().unwrap_or("N/A")
                    );
                }
                true
            }
            KeyCode::Esc => {
                // Exit team selection mode
                state.team_selection_active = false;
                true
            }
            _ => false, // Key not handled in team selection mode
        }
    } else {
        // Subtab focused but team selection not active
        match key.code {
            KeyCode::Left => {
                // Navigate standings view
                state.view = match state.view {
                    GroupBy::Division => GroupBy::League,
                    GroupBy::Conference => GroupBy::Division,
                    GroupBy::League => GroupBy::Conference,
                };
                // Reset scroll and team selection when changing view
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                true
            }
            KeyCode::Right => {
                // Navigate standings view
                state.view = match state.view {
                    GroupBy::Division => GroupBy::Conference,
                    GroupBy::Conference => GroupBy::League,
                    GroupBy::League => GroupBy::Division,
                };
                // Reset scroll and team selection when changing view
                state.scrollable.reset();
                state.selected_team_index = 0;
                state.selected_column = 0;
                true
            }
            KeyCode::Down => {
                // Enter team selection mode if there are teams
                if team_count_in_current_column > 0 {
                    state.team_selection_active = true;
                }
                true
            }
            KeyCode::Up => {
                // If at top of scroll, don't handle (let main handler exit subtab mode)
                // Otherwise handle scrolling
                if state.scrollable.scroll_offset == 0 {
                    false // Not handled - will exit subtab mode
                } else {
                    state.scrollable.handle_key(key)
                }
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                // Handle scrolling
                state.scrollable.handle_key(key)
            }
            _ => false, // Key not handled
        }
    }
}

/// Get the display name for a column based on view and western_first setting
fn get_column_name(view: GroupBy, column: usize, western_first: bool) -> String {
    match view {
        GroupBy::League => "All Teams".to_string(),
        GroupBy::Conference => {
            let (first, second) = if western_first {
                ("Western", "Eastern")
            } else {
                ("Eastern", "Western")
            };
            if column == 0 { first.to_string() } else { second.to_string() }
        }
        GroupBy::Division => {
            let (first, second) = if western_first {
                ("Western Divs", "Eastern Divs")
            } else {
                ("Eastern Divs", "Western Divs")
            };
            if column == 0 { first.to_string() } else { second.to_string() }
        }
    }
}

/// Get teams from a specific column (respects western_first setting)
/// Returns teams in the same order as they appear in the visual display
fn get_teams_in_column_for_handler(
    standings: &[nhl_api::Standing],
    view: GroupBy,
    column: usize,
    western_first: bool,
) -> Vec<nhl_api::Standing> {
    let mut sorted = standings.to_vec();
    sorted.sort_by(|a, b| b.points.cmp(&a.points));

    match view {
        GroupBy::League => {
            // Single column, all teams sorted by points
            sorted
        }
        GroupBy::Conference => {
            // Column 0 = Eastern (or Western if western_first), Column 1 = Western (or Eastern)
            // Teams sorted by points within conference
            let (first_conf, second_conf) = if western_first {
                ("Western", "Eastern")
            } else {
                ("Eastern", "Western")
            };
            let conf_name = if column == 0 { first_conf } else { second_conf };
            sorted.into_iter()
                .filter(|s| s.conference_name.as_deref() == Some(conf_name))
                .collect()
        }
        GroupBy::Division => {
            // Column 0 = Eastern divisions (or Western if western_first), Column 1 = Western (or Eastern)
            // Teams grouped by division, then sorted by points WITHIN each division
            let (first_divs, second_divs) = if western_first {
                (vec!["Central", "Pacific"], vec!["Atlantic", "Metropolitan"])
            } else {
                (vec!["Atlantic", "Metropolitan"], vec!["Central", "Pacific"])
            };
            let divs = if column == 0 { &first_divs } else { &second_divs };

            // Group teams by division and sort each division by points
            let mut result = Vec::new();
            for div_name in divs {
                let mut div_teams: Vec<_> = sorted.iter()
                    .filter(|s| s.division_name == *div_name)
                    .cloned()
                    .collect();
                div_teams.sort_by(|a, b| b.points.cmp(&a.points));
                result.extend(div_teams);
            }
            result
        }
    }
}

/// Calculate team counts per column based on view type (respects western_first)
/// Returns (Vec of team counts per column, total number of columns)
fn calculate_column_info(standings: &[nhl_api::Standing], view: GroupBy, western_first: bool) -> (Vec<usize>, usize) {
    match view {
        GroupBy::League => {
            // Single column with all teams
            (vec![standings.len()], 1)
        }
        GroupBy::Conference => {
            // Two columns: order depends on western_first
            let eastern_count = standings.iter()
                .filter(|s| s.conference_name.as_deref() == Some("Eastern"))
                .count();
            let western_count = standings.iter()
                .filter(|s| s.conference_name.as_deref() == Some("Western"))
                .count();

            if western_first {
                (vec![western_count, eastern_count], 2)
            } else {
                (vec![eastern_count, western_count], 2)
            }
        }
        GroupBy::Division => {
            // Two columns: order depends on western_first
            let eastern_count = standings.iter()
                .filter(|s| s.division_name == "Atlantic" || s.division_name == "Metropolitan")
                .count();
            let western_count = standings.iter()
                .filter(|s| s.division_name == "Central" || s.division_name == "Pacific")
                .count();

            if western_first {
                (vec![western_count, eastern_count], 2)
            } else {
                (vec![eastern_count, western_count], 2)
            }
        }
    }
}
