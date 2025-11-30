use tracing::debug;

use crate::tui::action::Action;
use crate::tui::component::Effect;
use crate::tui::helpers::{ClubGoalieStatsSorting, ClubSkaterStatsSorting, SeasonSorting};
use crate::tui::state::{AppState, DocumentStackEntry, LoadingKey};
use crate::tui::types::StackedDocument;

/// Handle all document stack management actions
pub fn reduce_document_stack(state: &AppState, action: &Action) -> Option<(AppState, Effect)> {
    match action {
        Action::PushDocument(doc) => Some(push_document(state.clone(), doc.clone())),
        Action::PopDocument => Some(pop_document(state.clone())),
        Action::DocumentSelectNext => Some(document_select_next(state.clone())),
        Action::DocumentSelectPrevious => Some(document_select_previous(state.clone())),
        Action::DocumentSelectItem => Some(document_select_item(state.clone())),
        _ => None,
    }
}

fn push_document(state: AppState, doc: StackedDocument) -> (AppState, Effect) {
    debug!("DOCUMENT_STACK: Pushing document onto stack: {:?}", doc);
    let mut new_state = state;
    new_state
        .navigation
        .document_stack
        .push(DocumentStackEntry::new(doc));
    (new_state, Effect::None)
}

fn pop_document(state: AppState) -> (AppState, Effect) {
    debug!("DOCUMENT_STACK: Popping document from stack");
    let mut new_state = state;

    if let Some(doc_entry) = new_state.navigation.document_stack.pop() {
        // Clear the loading state for the document being popped
        match &doc_entry.document {
            StackedDocument::Boxscore { game_id } => {
                new_state
                    .data
                    .loading
                    .remove(&LoadingKey::Boxscore(*game_id));
            }
            StackedDocument::TeamDetail { abbrev } => {
                new_state
                    .data
                    .loading
                    .remove(&LoadingKey::TeamRosterStats(abbrev.clone()));
            }
            StackedDocument::PlayerDetail { player_id } => {
                new_state
                    .data
                    .loading
                    .remove(&LoadingKey::PlayerStats(*player_id));
            }
        }

        debug!(
            "DOCUMENT_STACK: Popped document, {} remaining",
            new_state.navigation.document_stack.len()
        );
    }

    // If no documents left, return focus to content
    if new_state.navigation.document_stack.is_empty() {
        debug!("DOCUMENT_STACK: Document stack empty, returning focus to content");
    }

    (new_state, Effect::None)
}

/// Autoscroll padding - number of lines to keep visible above/below focused element
const AUTOSCROLL_PADDING: u16 = 2;

fn document_select_next(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Move selection in the current document
    if let Some(doc_entry) = new_state.navigation.document_stack.last_mut() {
        let max_index = doc_entry.focusable_positions.len().saturating_sub(1);
        if let Some(idx) = doc_entry.selected_index {
            // Clamp to max index
            let new_idx = (idx + 1).min(max_index);
            doc_entry.selected_index = Some(new_idx);
            debug!(
                "DOCUMENT_STACK: Selected next item, index: {:?}",
                doc_entry.selected_index
            );
        } else {
            doc_entry.selected_index = Some(0);
        }
        ensure_focused_visible(doc_entry);
    }

    (new_state, Effect::None)
}

fn document_select_previous(state: AppState) -> (AppState, Effect) {
    let mut new_state = state;

    // Move selection in the current document
    if let Some(doc_entry) = new_state.navigation.document_stack.last_mut() {
        if let Some(idx) = doc_entry.selected_index {
            doc_entry.selected_index = Some(idx.saturating_sub(1));
            debug!(
                "DOCUMENT_STACK: Selected previous item, index: {:?}",
                doc_entry.selected_index
            );
        }
        ensure_focused_visible(doc_entry);
    }

    (new_state, Effect::None)
}

/// Ensure the focused element is visible in the viewport
fn ensure_focused_visible(doc_entry: &mut crate::tui::state::DocumentStackEntry) {
    let focus_idx = match doc_entry.selected_index {
        Some(idx) => idx,
        None => return,
    };

    let focused_y = match doc_entry.focusable_positions.get(focus_idx) {
        Some(&y) => y,
        None => return,
    };

    let focused_height = doc_entry
        .focusable_heights
        .get(focus_idx)
        .copied()
        .unwrap_or(1);

    let viewport_height = doc_entry.viewport_height.max(10);
    let scroll_offset = doc_entry.scroll_offset;

    // Calculate viewport bounds
    let viewport_top = scroll_offset;
    let viewport_bottom = scroll_offset.saturating_add(viewport_height);

    // Calculate element bounds
    let element_top = focused_y;
    let element_bottom = focused_y.saturating_add(focused_height);

    // Only scroll if element is actually outside the viewport
    if element_top < viewport_top {
        // Element top is above viewport - scroll up to show it with padding
        let new_offset = element_top.saturating_sub(AUTOSCROLL_PADDING);
        doc_entry.scroll_offset = new_offset;
    } else if element_bottom > viewport_bottom {
        // Element bottom is below viewport - scroll down to show entire element
        let new_offset = element_bottom
            .saturating_add(AUTOSCROLL_PADDING)
            .saturating_sub(viewport_height);
        doc_entry.scroll_offset = new_offset;
    }
}

/// Handle selecting a player from a team roster document
fn handle_team_roster_selection(
    state: AppState,
    abbrev: &str,
    selected_index: usize,
) -> Option<(AppState, Effect)> {
    if let Some(roster) = state.data.team_roster_stats.get(abbrev) {
        // CRITICAL: Must sort the same way as team_detail_document.rs does for display
        // Otherwise visual position won't match data array index

        // Sort skaters by points descending (matching team_detail_document.rs)
        let mut sorted_skaters = roster.skaters.clone();
        sorted_skaters.sort_by_points_desc();

        // Sort goalies by games played descending (matching team_detail_document.rs)
        let mut sorted_goalies = roster.goalies.clone();
        sorted_goalies.sort_by_games_played_desc();

        let num_skaters = sorted_skaters.len();

        // Check if selecting a skater
        if selected_index < num_skaters {
            if let Some(player) = sorted_skaters.get(selected_index) {
                let player_id = player.player_id;
                debug!(
                    "DOCUMENT_STACK: Selected skater {} (index {} in sorted list) from team {}",
                    player_id, selected_index, abbrev
                );

                // Push PlayerDetail document
                let mut new_state = state;
                new_state.navigation.document_stack.push(
                    DocumentStackEntry::with_selection(StackedDocument::PlayerDetail { player_id }, None),
                );

                return Some((new_state, Effect::None));
            }
        } else {
            // Check if selecting a goalie
            let goalie_idx = selected_index - num_skaters;
            if let Some(goalie) = sorted_goalies.get(goalie_idx) {
                let player_id = goalie.player_id;
                debug!(
                    "DOCUMENT_STACK: Selected goalie {} (index {} in sorted list) from team {}",
                    player_id, goalie_idx, abbrev
                );

                // Push PlayerDetail document
                let mut new_state = state;
                new_state.navigation.document_stack.push(
                    DocumentStackEntry::with_selection(StackedDocument::PlayerDetail { player_id }, None),
                );

                return Some((new_state, Effect::None));
            }
        }
    }
    None
}

/// Handle selecting a player from a boxscore document
fn handle_boxscore_selection(
    state: AppState,
    game_id: i64,
    selected_index: usize,
) -> Option<(AppState, Effect)> {
    if let Some(boxscore) = state.data.boxscores.get(&game_id) {
        let away_stats = &boxscore.player_by_game_stats.away_team;
        let home_stats = &boxscore.player_by_game_stats.home_team;

        // Calculate section boundaries (same as boxscore_document.rs)
        let away_forwards_count = away_stats.forwards.len();
        let away_defense_count = away_stats.defense.len();
        let away_goalies_count = away_stats.goalies.len();

        let away_total = away_forwards_count + away_defense_count + away_goalies_count;
        let home_forwards_count = home_stats.forwards.len();
        let home_defense_count = home_stats.defense.len();

        // Determine which player was selected
        let player_id = if selected_index < away_forwards_count {
            // Away forward
            away_stats.forwards.get(selected_index).map(|p| p.player_id)
        } else if selected_index < away_forwards_count + away_defense_count {
            // Away defense
            let defense_idx = selected_index - away_forwards_count;
            away_stats.defense.get(defense_idx).map(|p| p.player_id)
        } else if selected_index < away_total {
            // Away goalie
            let goalie_idx = selected_index - away_forwards_count - away_defense_count;
            away_stats.goalies.get(goalie_idx).map(|p| p.player_id)
        } else if selected_index < away_total + home_forwards_count {
            // Home forward
            let forward_idx = selected_index - away_total;
            home_stats.forwards.get(forward_idx).map(|p| p.player_id)
        } else if selected_index < away_total + home_forwards_count + home_defense_count {
            // Home defense
            let defense_idx = selected_index - away_total - home_forwards_count;
            home_stats.defense.get(defense_idx).map(|p| p.player_id)
        } else {
            // Home goalie
            let goalie_idx = selected_index - away_total - home_forwards_count - home_defense_count;
            home_stats.goalies.get(goalie_idx).map(|p| p.player_id)
        };

        if let Some(player_id) = player_id {
            debug!(
                "DOCUMENT_STACK: Selected player {} (index {}) from boxscore game {}",
                player_id, selected_index, game_id
            );

            // Push PlayerDetail document
            let mut new_state = state;
            new_state.navigation.document_stack.push(
                DocumentStackEntry::with_selection(StackedDocument::PlayerDetail { player_id }, None),
            );

            return Some((new_state, Effect::None));
        }
    }
    None
}

/// Handle selecting a season from a player detail document
fn handle_player_season_selection(
    state: AppState,
    player_id: i64,
    selected_index: usize,
) -> Option<(AppState, Effect)> {
    if let Some(player) = state.data.player_data.get(&player_id) {
        if let Some(seasons) = &player.season_totals {
            // Filter to NHL regular season only and sort by season descending (latest first)
            let mut nhl_seasons: Vec<_> = seasons
                .iter()
                .filter(|s| {
                    s.game_type == nhl_api::GameType::RegularSeason && s.league_abbrev == "NHL"
                })
                .collect();
            nhl_seasons.sort_by_season_desc();

            if let Some(season) = nhl_seasons.get(selected_index) {
                // Extract team abbreviation from common name
                if let Some(ref common_name) = season.team_common_name {
                    if let Some(abbrev) =
                        crate::team_abbrev::common_name_to_abbrev(&common_name.default)
                    {
                        debug!(
                            "DOCUMENT_STACK: Selected season {} (index {}) from player {}, navigating to team {}",
                            season.season, selected_index, player_id, abbrev
                        );

                        // Push TeamDetail document
                        let mut new_state = state;
                        new_state.navigation.document_stack.push(
                            DocumentStackEntry::new(StackedDocument::TeamDetail {
                                abbrev: abbrev.to_string(),
                            }),
                        );

                        return Some((new_state, Effect::None));
                    }
                }
            }
        }
    }
    None
}

fn document_select_item(state: AppState) -> (AppState, Effect) {
    // Get information about the current document
    let doc_info = state
        .navigation
        .document_stack
        .last()
        .map(|d| (d.document.clone(), d.selected_index));

    if let Some((doc, Some(idx))) = doc_info {
        // Delegate to document-specific handlers
        let result = match doc {
            StackedDocument::TeamDetail { ref abbrev } => {
                handle_team_roster_selection(state.clone(), abbrev, idx)
            }
            StackedDocument::Boxscore { game_id } => {
                handle_boxscore_selection(state.clone(), game_id, idx)
            }
            StackedDocument::PlayerDetail { player_id } => {
                handle_player_season_selection(state.clone(), player_id, idx)
            }
        };

        if let Some((new_state, effect)) = result {
            return (new_state, effect);
        }
    }

    (state, Effect::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_push_document() {
        let state = AppState::default();
        let panel = StackedDocument::TeamDetail {
            abbrev: "BOS".to_string(),
        };

        let (new_state, _) = push_document(state, panel.clone());

        assert_eq!(new_state.navigation.document_stack.len(), 1);
        assert_eq!(new_state.navigation.document_stack[0].selected_index, Some(0));
    }

    #[test]
    fn test_pop_document_clears_loading_state() {
        let mut state = AppState::default();
        let game_id = 2024020001;

        // Push a boxscore panel and add loading state
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::Boxscore { game_id },
            selected_index: None,
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });
        state.data.loading.insert(LoadingKey::Boxscore(game_id));

        let (new_state, _) = pop_document(state);

        assert!(new_state.navigation.document_stack.is_empty());
        assert!(!new_state
            .data
            .loading
            .contains(&LoadingKey::Boxscore(game_id)));
    }

    #[test]
    fn test_document_selection() {
        let mut state = AppState::default();
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "BOS".to_string(),
            },
            selected_index: Some(0),
            scroll_offset: 0,
            // Provide focusable positions so navigation can work
            focusable_positions: vec![0, 5, 10, 15, 20],
            focusable_heights: vec![1, 1, 1, 1, 1],
            viewport_height: 30,
        });

        let (state, _) = document_select_next(state);
        assert_eq!(state.navigation.document_stack[0].selected_index, Some(1));

        let (state, _) = document_select_next(state);
        assert_eq!(state.navigation.document_stack[0].selected_index, Some(2));

        let (state, _) = document_select_previous(state);
        assert_eq!(state.navigation.document_stack[0].selected_index, Some(1));

        // Test saturating subtraction
        let (state, _) = document_select_previous(state);
        let (state, _) = document_select_previous(state);
        assert_eq!(state.navigation.document_stack[0].selected_index, Some(0));
    }

    #[test]
    fn test_document_select_item_skater() {
        // Regression test: Ensure selecting a skater from team detail pushes PlayerDetail panel
        use nhl_api::{ClubGoalieStats, ClubSkaterStats, ClubStats, LocalizedString, Position};

        let mut state = AppState::default();

        // Create test roster with skaters and goalies
        let skaters = vec![
            ClubSkaterStats {
                player_id: 8478402,
                first_name: LocalizedString {
                    default: "Connor".to_string(),
                },
                last_name: LocalizedString {
                    default: "McDavid".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 15,
                assists: 25,
                points: 40,
                plus_minus: 10,
                penalty_minutes: 10,
                power_play_goals: 5,
                shorthanded_goals: 0,
                game_winning_goals: 3,
                overtime_goals: 1,
                shots: 80,
                shooting_pctg: 0.1875,
                avg_time_on_ice_per_game: 22.5,
                avg_shifts_per_game: 25.0,
                faceoff_win_pctg: 0.55,
            },
            ClubSkaterStats {
                player_id: 8477934,
                first_name: LocalizedString {
                    default: "Leon".to_string(),
                },
                last_name: LocalizedString {
                    default: "Draisaitl".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 12,
                assists: 20,
                points: 32,
                plus_minus: 8,
                penalty_minutes: 15,
                power_play_goals: 4,
                shorthanded_goals: 1,
                game_winning_goals: 2,
                overtime_goals: 0,
                shots: 70,
                shooting_pctg: 0.1714,
                avg_time_on_ice_per_game: 21.0,
                avg_shifts_per_game: 24.0,
                faceoff_win_pctg: 0.52,
            },
        ];

        let goalies = vec![ClubGoalieStats {
            player_id: 8471469,
            first_name: LocalizedString {
                default: "Stuart".to_string(),
            },
            last_name: LocalizedString {
                default: "Skinner".to_string(),
            },
            headshot: String::new(),
            games_played: 15,
            games_started: 15,
            wins: 10,
            losses: 3,
            overtime_losses: 2,
            goals_against_average: 2.45,
            save_percentage: 0.915,
            shots_against: 450,
            saves: 412,
            goals_against: 38,
            shutouts: 2,
            goals: 0,
            assists: 0,
            points: 0,
            penalty_minutes: 0,
            time_on_ice: 900,
        }];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters: skaters.clone(),
            goalies: goalies.clone(),
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("EDM".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "EDM".to_string(),
            },
            selected_index: Some(0), // Select first skater
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        // Select the first skater
        let (new_state, _) = document_select_item(state);

        // Should have pushed a PlayerDetail panel
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8478402); // Connor McDavid
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_goalie() {
        // Regression test: Ensure selecting a goalie from team detail pushes PlayerDetail panel
        // Bug: Previously only skaters were handled, selecting a goalie did nothing
        use nhl_api::{ClubGoalieStats, ClubSkaterStats, ClubStats, LocalizedString, Position};

        let mut state = AppState::default();

        let skaters = vec![ClubSkaterStats {
            player_id: 8478402,
            first_name: LocalizedString {
                default: "Connor".to_string(),
            },
            last_name: LocalizedString {
                default: "McDavid".to_string(),
            },
            headshot: String::new(),
            position: Position::Center,
            games_played: 20,
            goals: 15,
            assists: 25,
            points: 40,
            plus_minus: 10,
            penalty_minutes: 10,
            power_play_goals: 5,
            shorthanded_goals: 0,
            game_winning_goals: 3,
            overtime_goals: 1,
            shots: 80,
            shooting_pctg: 0.1875,
            avg_time_on_ice_per_game: 22.5,
            avg_shifts_per_game: 25.0,
            faceoff_win_pctg: 0.55,
        }];

        let goalies = vec![
            ClubGoalieStats {
                player_id: 8471469,
                first_name: LocalizedString {
                    default: "Stuart".to_string(),
                },
                last_name: LocalizedString {
                    default: "Skinner".to_string(),
                },
                headshot: String::new(),
                games_played: 15,
                games_started: 15,
                wins: 10,
                losses: 3,
                overtime_losses: 2,
                goals_against_average: 2.45,
                save_percentage: 0.915,
                shots_against: 450,
                saves: 412,
                goals_against: 38,
                shutouts: 2,
                goals: 0,
                assists: 0,
                points: 0,
                penalty_minutes: 0,
                time_on_ice: 900,
            },
            ClubGoalieStats {
                player_id: 8476999,
                first_name: LocalizedString {
                    default: "Calvin".to_string(),
                },
                last_name: LocalizedString {
                    default: "Pickard".to_string(),
                },
                headshot: String::new(),
                games_played: 5,
                games_started: 5,
                wins: 3,
                losses: 2,
                overtime_losses: 0,
                goals_against_average: 2.80,
                save_percentage: 0.905,
                shots_against: 150,
                saves: 136,
                goals_against: 14,
                shutouts: 0,
                goals: 0,
                assists: 1,
                points: 1,
                penalty_minutes: 0,
                time_on_ice: 300,
            },
        ];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters,
            goalies,
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("EDM".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "EDM".to_string(),
            },
            selected_index: Some(1), // Select first goalie (index 1, after 1 skater)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        // Select the first goalie
        let (new_state, _) = document_select_item(state);

        // Should have pushed a PlayerDetail panel for the goalie
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8471469); // Stuart Skinner
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_second_goalie() {
        // Test selecting the second goalie in the list
        use nhl_api::{ClubGoalieStats, ClubSkaterStats, ClubStats, LocalizedString, Position};

        let mut state = AppState::default();

        let skaters = vec![ClubSkaterStats {
            player_id: 8478402,
            first_name: LocalizedString {
                default: "Player".to_string(),
            },
            last_name: LocalizedString {
                default: "One".to_string(),
            },
            headshot: String::new(),
            position: Position::Center,
            games_played: 20,
            goals: 15,
            assists: 25,
            points: 40,
            plus_minus: 10,
            penalty_minutes: 10,
            power_play_goals: 5,
            shorthanded_goals: 0,
            game_winning_goals: 3,
            overtime_goals: 1,
            shots: 80,
            shooting_pctg: 0.1875,
            avg_time_on_ice_per_game: 22.5,
            avg_shifts_per_game: 25.0,
            faceoff_win_pctg: 0.55,
        }];

        let goalies = vec![
            ClubGoalieStats {
                player_id: 8471469,
                first_name: LocalizedString {
                    default: "Goalie".to_string(),
                },
                last_name: LocalizedString {
                    default: "One".to_string(),
                },
                headshot: String::new(),
                games_played: 15,
                games_started: 15,
                wins: 10,
                losses: 3,
                overtime_losses: 2,
                goals_against_average: 2.45,
                save_percentage: 0.915,
                shots_against: 450,
                saves: 412,
                goals_against: 38,
                shutouts: 2,
                goals: 0,
                assists: 0,
                points: 0,
                penalty_minutes: 0,
                time_on_ice: 900,
            },
            ClubGoalieStats {
                player_id: 8476999,
                first_name: LocalizedString {
                    default: "Goalie".to_string(),
                },
                last_name: LocalizedString {
                    default: "Two".to_string(),
                },
                headshot: String::new(),
                games_played: 5,
                games_started: 5,
                wins: 3,
                losses: 2,
                overtime_losses: 0,
                goals_against_average: 2.80,
                save_percentage: 0.905,
                shots_against: 150,
                saves: 136,
                goals_against: 14,
                shutouts: 0,
                goals: 0,
                assists: 1,
                points: 1,
                penalty_minutes: 0,
                time_on_ice: 300,
            },
        ];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters,
            goalies,
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("TST".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "TST".to_string(),
            },
            selected_index: Some(2), // Select second goalie (index 2 = 1 skater + 1 goalie)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        // Select the second goalie
        let (new_state, _) = document_select_item(state);

        // Should have pushed a PlayerDetail panel for the second goalie
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8476999); // Second goalie
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_uses_sorted_roster() {
        // Regression test for bug: Selecting Martin Necas showed Brock Nelson
        // Root cause: UI displays sorted roster (by points) but selection used unsorted data
        //
        // This test creates a roster where sorted order != data order
        // Then verifies that selecting visual position 0 gets the highest-points player,
        // not the first player in the data array
        use nhl_api::{ClubSkaterStats, ClubStats, LocalizedString, Position};

        let mut state = AppState::default();

        // Create skaters with DIFFERENT order when sorted by points
        // Data order: [Brent Burns (10 pts), Brock Nelson (5 pts), Martin Necas (40 pts)]
        // Sorted order by points: [Martin Necas (40 pts), Brent Burns (10 pts), Brock Nelson (5 pts)]
        let skaters = vec![
            ClubSkaterStats {
                player_id: 8470613, // Brent Burns
                first_name: LocalizedString {
                    default: "Brent".to_string(),
                },
                last_name: LocalizedString {
                    default: "Burns".to_string(),
                },
                headshot: String::new(),
                position: Position::Defense,
                games_played: 20,
                goals: 3,
                assists: 7,
                points: 10, // Middle points
                plus_minus: 5,
                penalty_minutes: 20,
                power_play_goals: 1,
                shorthanded_goals: 0,
                game_winning_goals: 0,
                overtime_goals: 0,
                shots: 60,
                shooting_pctg: 0.05,
                avg_time_on_ice_per_game: 22.0,
                avg_shifts_per_game: 25.0,
                faceoff_win_pctg: 0.0,
            },
            ClubSkaterStats {
                player_id: 8475754, // Brock Nelson
                first_name: LocalizedString {
                    default: "Brock".to_string(),
                },
                last_name: LocalizedString {
                    default: "Nelson".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 2,
                assists: 3,
                points: 5, // Lowest points
                plus_minus: -2,
                penalty_minutes: 4,
                power_play_goals: 1,
                shorthanded_goals: 0,
                game_winning_goals: 0,
                overtime_goals: 0,
                shots: 40,
                shooting_pctg: 0.05,
                avg_time_on_ice_per_game: 18.0,
                avg_shifts_per_game: 22.0,
                faceoff_win_pctg: 0.48,
            },
            ClubSkaterStats {
                player_id: 8480039, // Martin Necas
                first_name: LocalizedString {
                    default: "Martin".to_string(),
                },
                last_name: LocalizedString {
                    default: "Necas".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 15,
                assists: 25,
                points: 40, // Highest points - should be first when sorted!
                plus_minus: 12,
                penalty_minutes: 8,
                power_play_goals: 5,
                shorthanded_goals: 1,
                game_winning_goals: 3,
                overtime_goals: 1,
                shots: 80,
                shooting_pctg: 0.1875,
                avg_time_on_ice_per_game: 20.5,
                avg_shifts_per_game: 24.0,
                faceoff_win_pctg: 0.52,
            },
        ];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters,
            goalies: vec![],
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("COL".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "COL".to_string(),
            },
            selected_index: Some(0), // Select first VISUAL position (highest points)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        // Select the first visual position
        let (new_state, _) = document_select_item(state);

        // Should have selected Martin Necas (highest points), NOT Brent Burns (first in data)
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8480039, "Should select Martin Necas (40 pts), not Brent Burns (10 pts) or Brock Nelson (5 pts)");
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_sorted_second_position() {
        // Test selecting second visual position in sorted roster
        use nhl_api::{ClubSkaterStats, ClubStats, LocalizedString, Position};

        let mut state = AppState::default();

        let skaters = vec![
            ClubSkaterStats {
                player_id: 8470613,
                first_name: LocalizedString {
                    default: "Brent".to_string(),
                },
                last_name: LocalizedString {
                    default: "Burns".to_string(),
                },
                headshot: String::new(),
                position: Position::Defense,
                games_played: 20,
                goals: 3,
                assists: 7,
                points: 10, // Second highest
                plus_minus: 5,
                penalty_minutes: 20,
                power_play_goals: 1,
                shorthanded_goals: 0,
                game_winning_goals: 0,
                overtime_goals: 0,
                shots: 60,
                shooting_pctg: 0.05,
                avg_time_on_ice_per_game: 22.0,
                avg_shifts_per_game: 25.0,
                faceoff_win_pctg: 0.0,
            },
            ClubSkaterStats {
                player_id: 8475754,
                first_name: LocalizedString {
                    default: "Brock".to_string(),
                },
                last_name: LocalizedString {
                    default: "Nelson".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 2,
                assists: 3,
                points: 5, // Lowest
                plus_minus: -2,
                penalty_minutes: 4,
                power_play_goals: 1,
                shorthanded_goals: 0,
                game_winning_goals: 0,
                overtime_goals: 0,
                shots: 40,
                shooting_pctg: 0.05,
                avg_time_on_ice_per_game: 18.0,
                avg_shifts_per_game: 22.0,
                faceoff_win_pctg: 0.48,
            },
            ClubSkaterStats {
                player_id: 8480039,
                first_name: LocalizedString {
                    default: "Martin".to_string(),
                },
                last_name: LocalizedString {
                    default: "Necas".to_string(),
                },
                headshot: String::new(),
                position: Position::Center,
                games_played: 20,
                goals: 15,
                assists: 25,
                points: 40, // Highest
                plus_minus: 12,
                penalty_minutes: 8,
                power_play_goals: 5,
                shorthanded_goals: 1,
                game_winning_goals: 3,
                overtime_goals: 1,
                shots: 80,
                shooting_pctg: 0.1875,
                avg_time_on_ice_per_game: 20.5,
                avg_shifts_per_game: 24.0,
                faceoff_win_pctg: 0.52,
            },
        ];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters,
            goalies: vec![],
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("COL".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "COL".to_string(),
            },
            selected_index: Some(1), // Select second VISUAL position
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should select Brent Burns (second highest points = 10), not Brock Nelson
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(
                    *player_id, 8470613,
                    "Should select Brent Burns (10 pts, second in sorted list)"
                );
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_goalies_sorted_by_games_played() {
        // Regression test: Goalies should be sorted by games_played, not data order
        use nhl_api::{ClubGoalieStats, ClubStats, LocalizedString};

        let mut state = AppState::default();

        // Create goalies with different order when sorted by games_played
        // Data order: [Goalie A (5 GP), Goalie B (15 GP), Goalie C (10 GP)]
        // Sorted order: [Goalie B (15 GP), Goalie C (10 GP), Goalie A (5 GP)]
        let goalies = vec![
            ClubGoalieStats {
                player_id: 1001,
                first_name: LocalizedString {
                    default: "Goalie".to_string(),
                },
                last_name: LocalizedString {
                    default: "A".to_string(),
                },
                headshot: String::new(),
                games_played: 5, // Lowest GP
                games_started: 5,
                wins: 3,
                losses: 2,
                overtime_losses: 0,
                goals_against_average: 2.50,
                save_percentage: 0.910,
                shots_against: 150,
                saves: 136,
                goals_against: 14,
                shutouts: 0,
                goals: 0,
                assists: 0,
                points: 0,
                penalty_minutes: 0,
                time_on_ice: 300,
            },
            ClubGoalieStats {
                player_id: 1002,
                first_name: LocalizedString {
                    default: "Goalie".to_string(),
                },
                last_name: LocalizedString {
                    default: "B".to_string(),
                },
                headshot: String::new(),
                games_played: 15, // Highest GP - should be first!
                games_started: 15,
                wins: 10,
                losses: 3,
                overtime_losses: 2,
                goals_against_average: 2.45,
                save_percentage: 0.915,
                shots_against: 450,
                saves: 412,
                goals_against: 38,
                shutouts: 2,
                goals: 0,
                assists: 0,
                points: 0,
                penalty_minutes: 0,
                time_on_ice: 900,
            },
            ClubGoalieStats {
                player_id: 1003,
                first_name: LocalizedString {
                    default: "Goalie".to_string(),
                },
                last_name: LocalizedString {
                    default: "C".to_string(),
                },
                headshot: String::new(),
                games_played: 10, // Middle GP
                games_started: 10,
                wins: 6,
                losses: 3,
                overtime_losses: 1,
                goals_against_average: 2.60,
                save_percentage: 0.908,
                shots_against: 300,
                saves: 272,
                goals_against: 28,
                shutouts: 1,
                goals: 0,
                assists: 1,
                points: 1,
                penalty_minutes: 0,
                time_on_ice: 600,
            },
        ];

        let roster = ClubStats {
            season: "20242025".to_string(),
            game_type: nhl_api::GameType::RegularSeason,
            skaters: vec![],
            goalies,
        };

        Arc::make_mut(&mut state.data.team_roster_stats).insert("TST".to_string(), roster);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::TeamDetail {
                abbrev: "TST".to_string(),
            },
            selected_index: Some(0), // Select first goalie visually (0 skaters + 0 = index 0)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should select Goalie B (15 GP, first in sorted list), not Goalie A (5 GP, first in data)
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(
                    *player_id, 1002,
                    "Should select Goalie B (15 GP), not Goalie A (5 GP)"
                );
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    /// Helper to create a test SkaterStats
    fn create_test_skater(
        player_id: i64,
        name: &str,
        position: nhl_api::Position,
    ) -> nhl_api::SkaterStats {
        nhl_api::SkaterStats {
            player_id,
            name: nhl_api::LocalizedString {
                default: name.to_string(),
            },
            sweater_number: 87,
            position,
            goals: 1,
            assists: 2,
            points: 3,
            plus_minus: 1,
            pim: 0,
            hits: 2,
            power_play_goals: 0,
            sog: 4,
            faceoff_winning_pctg: 55.5,
            toi: "20:00".to_string(),
            blocked_shots: 1,
            shifts: 15,
            giveaways: 0,
            takeaways: 1,
        }
    }

    /// Helper to create a minimal test Boxscore
    fn create_test_boxscore_with_positions(
        game_id: i64,
        away_forwards: Vec<nhl_api::SkaterStats>,
        away_defense: Vec<nhl_api::SkaterStats>,
        home_forwards: Vec<nhl_api::SkaterStats>,
    ) -> nhl_api::Boxscore {
        nhl_api::Boxscore {
            id: game_id,
            season: 20242025,
            game_type: nhl_api::GameType::RegularSeason,
            limited_scoring: false,
            game_date: "2024-11-16".to_string(),
            venue: nhl_api::LocalizedString {
                default: "Test Arena".to_string(),
            },
            venue_location: nhl_api::LocalizedString {
                default: "Test City".to_string(),
            },
            start_time_utc: "2024-11-16T23:00:00Z".to_string(),
            eastern_utc_offset: "-05:00".to_string(),
            venue_utc_offset: "-05:00".to_string(),
            tv_broadcasts: vec![],
            game_state: nhl_api::GameState::Final,
            game_schedule_state: "OK".to_string(),
            period_descriptor: nhl_api::PeriodDescriptor {
                number: 3,
                period_type: nhl_api::PeriodType::Regulation,
                max_regulation_periods: 3,
            },
            special_event: None,
            away_team: nhl_api::BoxscoreTeam {
                id: 1,
                common_name: nhl_api::LocalizedString {
                    default: "Penguins".to_string(),
                },
                abbrev: "PIT".to_string(),
                score: 3,
                sog: 30,
                logo: String::new(),
                dark_logo: String::new(),
                place_name: nhl_api::LocalizedString {
                    default: "Pittsburgh".to_string(),
                },
                place_name_with_preposition: nhl_api::LocalizedString {
                    default: "Pittsburgh".to_string(),
                },
            },
            home_team: nhl_api::BoxscoreTeam {
                id: 18,
                common_name: nhl_api::LocalizedString {
                    default: "Predators".to_string(),
                },
                abbrev: "NSH".to_string(),
                score: 2,
                sog: 25,
                logo: String::new(),
                dark_logo: String::new(),
                place_name: nhl_api::LocalizedString {
                    default: "Nashville".to_string(),
                },
                place_name_with_preposition: nhl_api::LocalizedString {
                    default: "Nashville".to_string(),
                },
            },
            clock: nhl_api::GameClock {
                time_remaining: "00:00".to_string(),
                seconds_remaining: 0,
                running: false,
                in_intermission: false,
            },
            player_by_game_stats: nhl_api::PlayerByGameStats {
                away_team: nhl_api::TeamPlayerStats {
                    forwards: away_forwards,
                    defense: away_defense,
                    goalies: vec![],
                },
                home_team: nhl_api::TeamPlayerStats {
                    forwards: home_forwards,
                    defense: vec![],
                    goalies: vec![],
                },
            },
        }
    }

    /// Helper to create a minimal test Boxscore (simple version)
    fn create_test_boxscore(
        game_id: i64,
        away_forwards: Vec<nhl_api::SkaterStats>,
        home_forwards: Vec<nhl_api::SkaterStats>,
    ) -> nhl_api::Boxscore {
        create_test_boxscore_with_positions(game_id, away_forwards, vec![], home_forwards)
    }

    #[test]
    fn test_document_select_item_boxscore_away_forward() {
        // Regression test: Selecting a player from boxscore should work
        // Bug: Document selection was not implemented for Boxscore documents
        let mut state = AppState::default();
        const TEST_GAME_ID: i64 = 2024020001;

        let boxscore = create_test_boxscore(
            TEST_GAME_ID,
            vec![create_test_skater(
                8478483,
                "Sidney Crosby",
                nhl_api::Position::Center,
            )],
            vec![],
        );

        Arc::make_mut(&mut state.data.boxscores).insert(TEST_GAME_ID, boxscore);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::Boxscore {
                game_id: TEST_GAME_ID,
            },
            selected_index: Some(0), // Select first away forward
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should have pushed PlayerDetail panel for Crosby
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 8478483, "Should select Crosby");
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_boxscore_home_forward() {
        // Test selecting a home forward (after all away players)
        let mut state = AppState::default();
        const TEST_GAME_ID: i64 = 2024020002;

        let boxscore = create_test_boxscore(
            TEST_GAME_ID,
            vec![create_test_skater(
                8478483,
                "Away Forward",
                nhl_api::Position::Center,
            )],
            vec![create_test_skater(
                8476887,
                "Filip Forsberg",
                nhl_api::Position::LeftWing,
            )],
        );

        Arc::make_mut(&mut state.data.boxscores).insert(TEST_GAME_ID, boxscore);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::Boxscore {
                game_id: TEST_GAME_ID,
            },
            selected_index: Some(1), // Index 1 = first home forward (after 1 away forward)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should have pushed PlayerDetail panel for Forsberg
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(
                    *player_id, 8476887,
                    "Should select Forsberg (first home forward)"
                );
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_boxscore_away_defense() {
        // Test selecting an away defenseman
        let mut state = AppState::default();
        const TEST_GAME_ID: i64 = 2024020003;

        let boxscore = create_test_boxscore_with_positions(
            TEST_GAME_ID,
            vec![create_test_skater(
                100,
                "Forward One",
                nhl_api::Position::Center,
            )],
            vec![create_test_skater(
                200,
                "Defense One",
                nhl_api::Position::Defense,
            )],
            vec![],
        );

        Arc::make_mut(&mut state.data.boxscores).insert(TEST_GAME_ID, boxscore);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::Boxscore {
                game_id: TEST_GAME_ID,
            },
            selected_index: Some(1), // Index 1 = first away defenseman (after 1 forward)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should have pushed PlayerDetail panel for the defenseman
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::PlayerDetail { player_id } => {
                assert_eq!(*player_id, 200, "Should select away defenseman");
            }
            _ => panic!("Expected PlayerDetail panel"),
        }
    }

    #[test]
    fn test_document_select_item_player_detail_season() {
        // Regression test: Selecting a season from PlayerDetail should navigate to TeamDetail
        // Bug: PlayerDetail selection was removed during reducer refactoring
        use nhl_api::{LocalizedString, PlayerLanding, SeasonTotal};

        let mut state = AppState::default();
        const TEST_PLAYER_ID: i64 = 8481056;

        // Create player data with multiple seasons
        let player_data = PlayerLanding {
            player_id: TEST_PLAYER_ID,
            is_active: true,
            current_team_id: Some(6),
            current_team_abbrev: Some("BOS".to_string()),
            first_name: LocalizedString {
                default: "Test".to_string(),
            },
            last_name: LocalizedString {
                default: "Player".to_string(),
            },
            sweater_number: Some(34),
            position: nhl_api::Position::Center,
            headshot: String::new(),
            hero_image: None,
            height_in_inches: 72,
            weight_in_pounds: 200,
            birth_date: "1990-01-01".to_string(),
            birth_city: Some(LocalizedString {
                default: "Test City".to_string(),
            }),
            birth_state_province: None,
            birth_country: Some("USA".to_string()),
            shoots_catches: nhl_api::Handedness::Left,
            draft_details: None,
            player_slug: None,
            featured_stats: None,
            career_totals: None,
            season_totals: Some(vec![
                SeasonTotal {
                    season: 20232024,
                    game_type: nhl_api::GameType::RegularSeason,
                    league_abbrev: "NHL".to_string(),
                    team_name: LocalizedString {
                        default: "Boston Bruins".to_string(),
                    },
                    team_common_name: Some(LocalizedString {
                        default: "Bruins".to_string(),
                    }),
                    sequence: Some(1),
                    games_played: 82,
                    goals: Some(30),
                    assists: Some(40),
                    points: Some(70),
                    plus_minus: Some(15),
                    pim: Some(20),
                },
                SeasonTotal {
                    season: 20222023,
                    game_type: nhl_api::GameType::RegularSeason,
                    league_abbrev: "NHL".to_string(),
                    team_name: LocalizedString {
                        default: "Toronto Maple Leafs".to_string(),
                    },
                    team_common_name: Some(LocalizedString {
                        default: "Maple Leafs".to_string(),
                    }),
                    sequence: Some(1),
                    games_played: 75,
                    goals: Some(25),
                    assists: Some(35),
                    points: Some(60),
                    plus_minus: Some(10),
                    pim: Some(15),
                },
            ]),
            awards: None,
            last_five_games: None,
        };

        Arc::make_mut(&mut state.data.player_data).insert(TEST_PLAYER_ID, player_data);
        state.navigation.document_stack.push(DocumentStackEntry {
            document: StackedDocument::PlayerDetail {
                player_id: TEST_PLAYER_ID,
            },
            selected_index: Some(0), // Select first season (2023-2024, Bruins)
            scroll_offset: 0,
            focusable_positions: Vec::new(),
            focusable_heights: Vec::new(),
            viewport_height: 30,
        });

        let (new_state, _) = document_select_item(state);

        // Should have pushed TeamDetail panel for Bruins
        assert_eq!(new_state.navigation.document_stack.len(), 2);
        match &new_state.navigation.document_stack[1].document {
            StackedDocument::TeamDetail { abbrev } => {
                assert_eq!(abbrev, "BOS", "Should navigate to Bruins roster");
            }
            _ => panic!("Expected TeamDetail panel"),
        }
    }
}
