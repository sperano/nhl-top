use crate::tui::widgets::{Action, KeyHint, KeyHintStyle};
use crate::tui::app::CurrentTab;
use crate::commands::standings::GroupBy;
use nhl_api::GameDate;

/// Trait for providing navigation context information
///
/// Each tab state implements this trait to expose its current navigation context,
/// including breadcrumb items, available actions, and keyboard hints.
pub trait NavigationContextProvider {
    /// Get available actions for the current state
    ///
    /// Returns a list of actions that can be performed in the current context.
    /// Example: [Action { key: "Enter", label: "View Details", enabled: true }]
    fn get_available_actions(&self) -> Vec<Action>;

    /// Get keyboard hints for the status bar
    ///
    /// Returns a list of keyboard shortcuts to display in the status bar.
    /// Example: [KeyHint { key: "↓", action: "Select Team", style: Important }]
    fn get_keyboard_hints(&self) -> Vec<KeyHint>;

    /// Get searchable items for the command palette
    ///
    /// Returns a list of items that can be searched in the command palette.
    /// This will be used in Phase 5.
    fn get_searchable_items(&self) -> Vec<SearchableItem> {
        vec![] // Default: no searchable items
    }
}

/// Extended trait for tabs that need additional data for breadcrumbs
pub trait BreadcrumbProvider {
    /// Get breadcrumb items for the current navigation state
    ///
    /// Returns a list of strings representing the navigation path.
    /// Example: ["Standings", "Division", "Toronto Maple Leafs"]
    fn get_breadcrumb_items(&self) -> Vec<String>;
}

/// Scores tab breadcrumb provider that needs game_date
pub struct ScoresBreadcrumbProvider<'a> {
    pub state: &'a crate::tui::scores::State,
    pub game_date: &'a GameDate,
}

/// Format a GameDate as "Mon DD, YYYY"
fn format_date_full(date: &GameDate) -> String {
    match date {
        GameDate::Date(naive_date) => naive_date.format("%b %d, %Y").to_string(),
        GameDate::Now => chrono::Local::now().date_naive().format("%b %d, %Y").to_string(),
    }
}

impl BreadcrumbProvider for ScoresBreadcrumbProvider<'_> {
    fn get_breadcrumb_items(&self) -> Vec<String> {
        let mut items = vec!["Scores".to_string()];

        if self.state.subtab_focused {
            items.push(format_date_full(self.game_date));
        }

        items
    }
}

/// An item that can be searched in the command palette
#[derive(Debug, Clone)]
pub struct SearchableItem {
    pub label: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub navigation_command: NavigationCommand,
}

/// Navigation commands that can be executed
#[derive(Debug, Clone)]
pub enum NavigationCommand {
    GoToTab(CurrentTab),
    GoToTeam(String),           // Team abbreviation
    GoToPlayer(i64),            // Player ID
    GoToGame(i64),              // Game ID
    GoToDate(GameDate),         // Navigate to date in scores
    GoToStandingsView(GroupBy), // Change standings view
    GoToSettings(String),       // Settings category
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_navigation_command_variants() {
        let cmd1 = NavigationCommand::GoToTab(CurrentTab::Scores);
        let cmd2 = NavigationCommand::GoToTeam("TOR".to_string());
        let cmd3 = NavigationCommand::GoToPlayer(12345);

        match cmd1 {
            NavigationCommand::GoToTab(_) => assert!(true),
            _ => panic!("Wrong variant"),
        }

        match cmd2 {
            NavigationCommand::GoToTeam(ref team) => assert_eq!(team, "TOR"),
            _ => panic!("Wrong variant"),
        }

        match cmd3 {
            NavigationCommand::GoToPlayer(id) => assert_eq!(id, 12345),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_navigation_command_all_variants() {
        let date = GameDate::from_ymd(2024, 1, 15).unwrap();
        let commands = vec![
            NavigationCommand::GoToTab(CurrentTab::Standings),
            NavigationCommand::GoToTeam("MTL".to_string()),
            NavigationCommand::GoToPlayer(8478402),
            NavigationCommand::GoToGame(2024020001),
            NavigationCommand::GoToDate(date),
            NavigationCommand::GoToStandingsView(GroupBy::Division),
            NavigationCommand::GoToSettings("display".to_string()),
        ];

        assert_eq!(commands.len(), 7);
    }

    #[test]
    fn test_searchable_item_creation() {
        let item = SearchableItem {
            label: "Toronto Maple Leafs".to_string(),
            category: "Team".to_string(),
            keywords: vec!["leafs".to_string(), "toronto".to_string(), "tor".to_string()],
            navigation_command: NavigationCommand::GoToTeam("TOR".to_string()),
        };

        assert_eq!(item.label, "Toronto Maple Leafs");
        assert_eq!(item.category, "Team");
        assert_eq!(item.keywords.len(), 3);
    }

    #[test]
    fn test_searchable_item_with_multiple_keywords() {
        let item = SearchableItem {
            label: "Montreal Canadiens".to_string(),
            category: "Team".to_string(),
            keywords: vec![
                "montreal".to_string(),
                "canadiens".to_string(),
                "habs".to_string(),
                "mtl".to_string(),
            ],
            navigation_command: NavigationCommand::GoToTeam("MTL".to_string()),
        };

        assert_eq!(item.keywords.len(), 4);
        assert!(item.keywords.contains(&"habs".to_string()));
    }

    #[test]
    fn test_scores_breadcrumb_provider_not_focused() {
        use crate::tui::scores::State as ScoresState;

        let state = ScoresState::new();
        let date = GameDate::from_ymd(2024, 11, 8).unwrap();
        let provider = ScoresBreadcrumbProvider {
            state: &state,
            game_date: &date,
        };

        let items = provider.get_breadcrumb_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "Scores");
    }

    #[test]
    fn test_scores_breadcrumb_provider_focused() {
        use crate::tui::scores::State as ScoresState;

        let mut state = ScoresState::new();
        state.subtab_focused = true;
        let date = GameDate::from_ymd(2024, 11, 8).unwrap();
        let provider = ScoresBreadcrumbProvider {
            state: &state,
            game_date: &date,
        };

        let items = provider.get_breadcrumb_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "Scores");
        assert_eq!(items[1], "Nov 08, 2024");
    }

    #[test]
    fn test_standings_breadcrumb_division() {
        use crate::tui::standings::State as StandingsState;
        use crate::commands::standings::GroupBy;

        let mut state = StandingsState::new();
        state.view = GroupBy::Division;

        let items = state.get_breadcrumb_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "Standings");
        assert_eq!(items[1], "Division");
    }

    #[test]
    fn test_standings_breadcrumb_conference() {
        use crate::tui::standings::State as StandingsState;
        use crate::commands::standings::GroupBy;

        let mut state = StandingsState::new();
        state.view = GroupBy::Conference;

        let items = state.get_breadcrumb_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "Standings");
        assert_eq!(items[1], "Conference");
    }

    #[test]
    fn test_settings_breadcrumb() {
        use crate::tui::settings::State as SettingsState;

        let state = SettingsState::new();
        let items = state.get_breadcrumb_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "Settings");
    }

    #[test]
    fn test_scores_keyboard_hints_not_focused() {
        use crate::tui::scores::State as ScoresState;

        let state = ScoresState::new();
        let hints = state.get_keyboard_hints();

        assert!(hints.len() >= 2);
        assert!(hints.iter().any(|h| h.key == "↓"));
        assert!(hints.iter().any(|h| h.key == "ESC"));
    }

    #[test]
    fn test_scores_keyboard_hints_focused() {
        use crate::tui::scores::State as ScoresState;

        let mut state = ScoresState::new();
        state.subtab_focused = true;
        let hints = state.get_keyboard_hints();

        assert!(hints.len() >= 3);
        assert!(hints.iter().any(|h| h.key == "←→"));
        assert!(hints.iter().any(|h| h.key == "↑"));
        assert!(hints.iter().any(|h| h.key == "ESC"));
    }

    #[test]
    fn test_standings_available_actions_not_focused() {
        use crate::tui::standings::State as StandingsState;

        let state = StandingsState::new();
        let actions = state.get_available_actions();

        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_standings_available_actions_focused() {
        use crate::tui::standings::State as StandingsState;

        let mut state = StandingsState::new();
        state.subtab_focused = true;
        let actions = state.get_available_actions();

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].key, "←→");
        assert_eq!(actions[0].label, "Change View");
        assert!(actions[0].enabled);
    }

    #[test]
    fn test_settings_keyboard_hints() {
        use crate::tui::settings::State as SettingsState;

        let state = SettingsState::new();
        let hints = state.get_keyboard_hints();

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].key, "ESC");
        assert_eq!(hints[0].action, "Back");
    }
}
