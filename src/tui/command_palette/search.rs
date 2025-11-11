use std::str::FromStr;
use crate::tui::widgets::{CommandPalette, SearchResult};
use crate::tui::SharedDataHandle;
use crate::tui::context::NavigationCommand;
use crate::tui::app::CurrentTab;
use crate::commands::standings::GroupBy;
use nhl_api::GameDate;

const MAX_RESULTS: usize = 10;

/// Update search results based on the current input
pub async fn update_search_results(palette: &mut CommandPalette, shared_data: &SharedDataHandle) {
    let query = palette.input.to_lowercase();

    if query.is_empty() {
        palette.set_results(vec![]);
        return;
    }

    let data = shared_data.read().await;
    let mut results = Vec::new();

    // Search teams
    for standing in data.standings.iter() {
        let team_name_lower = standing.team_name.default.to_lowercase();
        let team_abbrev_lower = standing.team_abbrev.default.to_lowercase();

        if team_name_lower.contains(&query) || team_abbrev_lower.contains(&query) {
            results.push(SearchResult::new(
                standing.team_name.default.clone(),
                "Team",
                vec!["team".to_string(), standing.team_abbrev.default.clone()],
            ).with_icon("🏒"));

            if results.len() >= MAX_RESULTS {
                break;
            }
        }
    }

    // Search players
    if results.len() < MAX_RESULTS {
        for (player_id, player) in data.player_info.iter() {
            let first_name_lower = player.first_name.default.to_lowercase();
            let last_name_lower = player.last_name.default.to_lowercase();
            let full_name = format!("{} {}", player.first_name.default, player.last_name.default).to_lowercase();

            if first_name_lower.contains(&query)
                || last_name_lower.contains(&query)
                || full_name.contains(&query)
            {
                results.push(SearchResult::new(
                    format!("{} {}", player.first_name.default, player.last_name.default),
                    "Player",
                    vec!["player".to_string(), player_id.to_string()],
                ).with_icon("👤"));

                if results.len() >= MAX_RESULTS {
                    break;
                }
            }
        }
    }

    palette.set_results(results);
}

/// Parse a navigation path into a NavigationCommand
pub fn parse_navigation_path(path: &[String]) -> Option<NavigationCommand> {
    if path.is_empty() {
        return None;
    }

    match path[0].as_str() {
        "tab" => {
            if path.len() < 2 {
                return None;
            }
            match path[1].as_str() {
                "scores" => Some(NavigationCommand::GoToTab(CurrentTab::Scores)),
                "standings" => Some(NavigationCommand::GoToTab(CurrentTab::Standings)),
                "stats" => Some(NavigationCommand::GoToTab(CurrentTab::Stats)),
                "players" => Some(NavigationCommand::GoToTab(CurrentTab::Players)),
                "settings" => Some(NavigationCommand::GoToTab(CurrentTab::Settings)),
                _ => None,
            }
        }
        "team" => {
            if path.len() < 2 {
                return None;
            }
            Some(NavigationCommand::GoToTeam(path[1].clone()))
        }
        "player" => {
            if path.len() < 2 {
                return None;
            }
            path[1].parse::<i64>().ok().map(NavigationCommand::GoToPlayer)
        }
        "game" => {
            if path.len() < 2 {
                return None;
            }
            path[1].parse::<i64>().ok().map(NavigationCommand::GoToGame)
        }
        "date" => {
            if path.len() < 2 {
                return None;
            }
            GameDate::from_str(&path[1]).ok().map(NavigationCommand::GoToDate)
        }
        "view" => {
            if path.len() < 2 {
                return None;
            }
            match path[1].as_str() {
                "division" => Some(NavigationCommand::GoToStandingsView(GroupBy::Division)),
                "conference" => Some(NavigationCommand::GoToStandingsView(GroupBy::Conference)),
                "league" => Some(NavigationCommand::GoToStandingsView(GroupBy::League)),
                _ => None,
            }
        }
        "settings" => {
            let category = if path.len() >= 2 {
                path[1].clone()
            } else {
                String::new()
            };
            Some(NavigationCommand::GoToSettings(category))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::types::SharedData;
    use nhl_api::Standing;

    #[tokio::test]
    async fn test_update_search_results_empty_query() {
        let mut palette = CommandPalette::new();
        palette.input = String::new();

        let shared_data = Arc::new(RwLock::new(SharedData::default()));

        update_search_results(&mut palette, &shared_data).await;

        assert_eq!(palette.results.len(), 0);
    }

    #[tokio::test]
    async fn test_update_search_results_team_by_name() {
        let mut palette = CommandPalette::new();
        palette.input = "toronto".to_string();

        let mut shared_data = SharedData::default();
        shared_data.standings = Arc::new(vec![
            create_test_standing("Toronto Maple Leafs", "TOR"),
            create_test_standing("Montreal Canadiens", "MTL"),
        ]);

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].label, "Toronto Maple Leafs");
        assert_eq!(palette.results[0].category, "Team");
        assert_eq!(palette.results[0].navigation_path, vec!["team", "TOR"]);
    }

    fn create_test_standing(name: &str, abbrev: &str) -> Standing {
        use nhl_api::LocalizedString;
        Standing {
            conference_abbrev: Some("E".to_string()),
            conference_name: Some("Eastern".to_string()),
            division_abbrev: "ATL".to_string(),
            division_name: "Atlantic".to_string(),
            team_name: LocalizedString { default: name.to_string() },
            team_common_name: LocalizedString { default: name.to_string() },
            team_abbrev: LocalizedString { default: abbrev.to_string() },
            team_logo: "logo.svg".to_string(),
            wins: 0,
            losses: 0,
            ot_losses: 0,
            points: 0,
        }
    }

    #[tokio::test]
    async fn test_update_search_results_team_by_abbrev() {
        let mut palette = CommandPalette::new();
        palette.input = "mtl".to_string();

        let mut shared_data = SharedData::default();
        shared_data.standings = Arc::new(vec![
            create_test_standing("Toronto Maple Leafs", "TOR"),
            create_test_standing("Montreal Canadiens", "MTL"),
        ]);

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].label, "Montreal Canadiens");
    }

    #[tokio::test]
    async fn test_update_search_results_case_insensitive() {
        let mut palette = CommandPalette::new();
        palette.input = "TORONTO".to_string();

        let mut shared_data = SharedData::default();
        shared_data.standings = Arc::new(vec![
            create_test_standing("Toronto Maple Leafs", "TOR"),
        ]);

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].label, "Toronto Maple Leafs");
    }

    #[tokio::test]
    async fn test_update_search_results_max_limit() {
        let mut palette = CommandPalette::new();
        palette.input = "a".to_string();

        let mut standings = Vec::new();
        for i in 0..15 {
            standings.push(create_test_standing(&format!("Team A{}", i), &format!("TA{}", i)));
        }

        let mut shared_data = SharedData::default();
        shared_data.standings = Arc::new(standings);

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), MAX_RESULTS);
    }

    fn create_test_player(id: i64, first_name: &str, last_name: &str) -> (i64, nhl_api::PlayerLanding) {
        use nhl_api::LocalizedString;
        (
            id,
            nhl_api::PlayerLanding {
                player_id: id,
                is_active: true,
                current_team_id: None,
                current_team_abbrev: None,
                first_name: LocalizedString { default: first_name.to_string() },
                last_name: LocalizedString { default: last_name.to_string() },
                sweater_number: None,
                position: "C".to_string(),
                headshot: "headshot.jpg".to_string(),
                hero_image: None,
                height_in_inches: 72,
                weight_in_pounds: 200,
                birth_date: "2000-01-01".to_string(),
                birth_city: None,
                birth_state_province: None,
                birth_country: None,
                shoots_catches: "L".to_string(),
                draft_details: None,
                player_slug: None,
                featured_stats: None,
                career_totals: None,
                season_totals: None,
                awards: None,
                last_five_games: None,
            },
        )
    }

    #[tokio::test]
    async fn test_update_search_results_player() {
        let mut palette = CommandPalette::new();
        palette.input = "matthews".to_string();

        let mut shared_data = SharedData::default();
        shared_data.player_info = Arc::new(
            [create_test_player(8479318, "Auston", "Matthews")]
            .into_iter()
            .collect(),
        );

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].label, "Auston Matthews");
        assert_eq!(palette.results[0].category, "Player");
        assert_eq!(palette.results[0].navigation_path, vec!["player", "8479318"]);
    }

    #[tokio::test]
    async fn test_update_search_results_player_first_name() {
        let mut palette = CommandPalette::new();
        palette.input = "auston".to_string();

        let mut shared_data = SharedData::default();
        shared_data.player_info = Arc::new(
            [create_test_player(8479318, "Auston", "Matthews")]
            .into_iter()
            .collect(),
        );

        let shared_data_handle = Arc::new(RwLock::new(shared_data));
        update_search_results(&mut palette, &shared_data_handle).await;

        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].label, "Auston Matthews");
    }

    #[test]
    fn test_parse_navigation_path_tab_scores() {
        let path = vec!["tab".to_string(), "scores".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToTab(tab)) = cmd {
            assert_eq!(tab, CurrentTab::Scores);
        } else {
            panic!("Expected GoToTab(Scores)");
        }
    }

    #[test]
    fn test_parse_navigation_path_tab_standings() {
        let path = vec!["tab".to_string(), "standings".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToTab(tab)) = cmd {
            assert_eq!(tab, CurrentTab::Standings);
        } else {
            panic!("Expected GoToTab(Standings)");
        }
    }

    #[test]
    fn test_parse_navigation_path_team() {
        let path = vec!["team".to_string(), "TOR".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToTeam(abbrev)) = cmd {
            assert_eq!(abbrev, "TOR");
        } else {
            panic!("Expected GoToTeam");
        }
    }

    #[test]
    fn test_parse_navigation_path_player() {
        let path = vec!["player".to_string(), "8479318".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToPlayer(id)) = cmd {
            assert_eq!(id, 8479318);
        } else {
            panic!("Expected GoToPlayer");
        }
    }

    #[test]
    fn test_parse_navigation_path_game() {
        let path = vec!["game".to_string(), "2024020001".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToGame(id)) = cmd {
            assert_eq!(id, 2024020001);
        } else {
            panic!("Expected GoToGame");
        }
    }

    #[test]
    fn test_parse_navigation_path_date() {
        let path = vec!["date".to_string(), "2024-11-08".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        matches!(cmd, Some(NavigationCommand::GoToDate(_)));
    }

    #[test]
    fn test_parse_navigation_path_view_division() {
        let path = vec!["view".to_string(), "division".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToStandingsView(view)) = cmd {
            matches!(view, GroupBy::Division);
        } else {
            panic!("Expected GoToStandingsView");
        }
    }

    #[test]
    fn test_parse_navigation_path_settings() {
        let path = vec!["settings".to_string(), "display".to_string()];
        let cmd = parse_navigation_path(&path);

        assert!(cmd.is_some());
        if let Some(NavigationCommand::GoToSettings(category)) = cmd {
            assert_eq!(category, "display");
        } else {
            panic!("Expected GoToSettings");
        }
    }

    #[test]
    fn test_parse_navigation_path_empty() {
        let path: Vec<String> = vec![];
        let cmd = parse_navigation_path(&path);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_navigation_path_invalid_tab() {
        let path = vec!["tab".to_string()];
        let cmd = parse_navigation_path(&path);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_navigation_path_invalid_player_id() {
        let path = vec!["player".to_string(), "not_a_number".to_string()];
        let cmd = parse_navigation_path(&path);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_navigation_path_unknown_type() {
        let path = vec!["unknown".to_string(), "value".to_string()];
        let cmd = parse_navigation_path(&path);
        assert!(cmd.is_none());
    }
}
