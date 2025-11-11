use super::{scores, standings, stats, players, settings, browser};
use super::widgets::CommandPalette;
use super::context::NavigationCommand;
use super::SharedDataHandle;
use tokio::sync::mpsc;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentTab {
    Scores,
    Standings,
    Stats,
    Players,
    Settings,
    Browser,
}

impl CurrentTab {
    // pub fn name(&self) -> &str {
    //     match self {
    //         CurrentTab::Scores => "Scores",
    //         CurrentTab::Standings => "Standings",
    //         CurrentTab::Stats => "Stats",
    //         CurrentTab::Settings => "Settings",
    //     }
    // }
    //
    // pub fn all() -> [CurrentTab; 4] {
    //     [CurrentTab::Scores, CurrentTab::Standings, CurrentTab::Stats, CurrentTab::Settings]
    // }

    // pub fn all_names() -> [&'static str; 5] {
    //     ["Scores", "Standings", "Stats", "Players", "Settings"]
    // }

    pub fn index(&self) -> usize {
        match self {
            CurrentTab::Scores => 0,
            CurrentTab::Standings => 1,
            CurrentTab::Stats => 2,
            CurrentTab::Players => 3,
            CurrentTab::Settings => 4,
            CurrentTab::Browser => 5,
        }
    }
}

pub struct AppState {
    pub current_tab: CurrentTab,
    pub scores: scores::State,
    pub standings: standings::State,
    pub stats: stats::State,
    pub players: players::State,
    pub settings: settings::State,
    pub browser: browser::State,
    pub command_palette: Option<CommandPalette>,
    pub command_palette_active: bool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            current_tab: CurrentTab::Scores,
            scores: scores::State::new(),
            standings: standings::State::new(),
            stats: stats::State::new(),
            players: players::State::new(),
            settings: settings::State::new(),
            browser: browser::State::new(),
            command_palette: Some(CommandPalette::new()),
            command_palette_active: false,
        }
    }

    /// Open the command palette
    pub fn open_command_palette(&mut self) {
        if let Some(ref mut palette) = self.command_palette {
            palette.show();
            self.command_palette_active = true;
        }
    }

    /// Close the command palette
    pub fn close_command_palette(&mut self) {
        if let Some(ref mut palette) = self.command_palette {
            palette.hide();
        }
        self.command_palette_active = false;
    }

    /// Execute a navigation command
    pub async fn execute_navigation_command(
        &mut self,
        command: NavigationCommand,
        shared_data: &SharedDataHandle,
        refresh_tx: &mpsc::Sender<()>,
    ) -> Result<()> {
        match command {
            NavigationCommand::GoToTab(tab) => {
                self.current_tab = tab;
            }
            NavigationCommand::GoToTeam(abbrev) => {
                self.current_tab = CurrentTab::Standings;
                let mut data = shared_data.write().await;
                data.selected_team_abbrev = Some(abbrev);
                let _ = refresh_tx.send(()).await;
            }
            NavigationCommand::GoToPlayer(player_id) => {
                let mut data = shared_data.write().await;
                data.selected_player_id = Some(player_id);
                let _ = refresh_tx.send(()).await;
            }
            NavigationCommand::GoToGame(game_id) => {
                self.current_tab = CurrentTab::Scores;
                let mut data = shared_data.write().await;
                data.selected_game_id = Some(game_id);
                let _ = refresh_tx.send(()).await;
            }
            NavigationCommand::GoToDate(date) => {
                self.current_tab = CurrentTab::Scores;
                let mut data = shared_data.write().await;
                data.game_date = date;
                self.enter_subtab_mode();
            }
            NavigationCommand::GoToStandingsView(view) => {
                self.current_tab = CurrentTab::Standings;
                self.standings.view = view;
                self.enter_subtab_mode();
            }
            NavigationCommand::GoToSettings(_category) => {
                self.current_tab = CurrentTab::Settings;
            }
        }

        self.close_command_palette();
        Ok(())
    }

    pub fn navigate_tab_left(&mut self) {
        self.current_tab = match self.current_tab {
            CurrentTab::Scores => CurrentTab::Browser,
            CurrentTab::Standings => CurrentTab::Scores,
            CurrentTab::Stats => CurrentTab::Standings,
            CurrentTab::Players => CurrentTab::Stats,
            CurrentTab::Settings => CurrentTab::Players,
            CurrentTab::Browser => CurrentTab::Settings,
        };
        // Reset subtab focus when changing tabs
        self.exit_subtab_mode();
    }

    pub fn navigate_tab_right(&mut self) {
        self.current_tab = match self.current_tab {
            CurrentTab::Scores => CurrentTab::Standings,
            CurrentTab::Standings => CurrentTab::Stats,
            CurrentTab::Stats => CurrentTab::Players,
            CurrentTab::Players => CurrentTab::Settings,
            CurrentTab::Settings => CurrentTab::Browser,
            CurrentTab::Browser => CurrentTab::Scores,
        };
        // Reset subtab focus when changing tabs
        self.exit_subtab_mode();
    }

    pub fn enter_subtab_mode(&mut self) {
        match self.current_tab {
            CurrentTab::Scores => self.scores.subtab_focused = true,
            CurrentTab::Standings => self.standings.subtab_focused = true,
            CurrentTab::Stats => {} // No subtabs for stats
            CurrentTab::Players => {} // No subtabs for players
            CurrentTab::Settings => {
                self.settings.subtab_focused = true;
            }
            CurrentTab::Browser => self.browser.subtab_focused = true,
        }
    }

    pub fn exit_subtab_mode(&mut self) {
        self.scores.subtab_focused = false;
        self.scores.box_selection_active = false;
        self.standings.subtab_focused = false;
        self.standings.team_selection_active = false;
        self.standings.selected_team_index = 0;
        self.standings.selected_column = 0;
        self.settings.subtab_focused = false;
        self.browser.subtab_focused = false;
    }

    pub fn is_subtab_focused(&self) -> bool {
        match self.current_tab {
            CurrentTab::Scores => self.scores.subtab_focused,
            CurrentTab::Standings => self.standings.subtab_focused,
            CurrentTab::Stats => false,
            CurrentTab::Players => false,
            CurrentTab::Settings => self.settings.subtab_focused,
            CurrentTab::Browser => self.browser.subtab_focused,
        }
    }

    pub fn has_subtabs(&self) -> bool {
        matches!(self.current_tab, CurrentTab::Scores | CurrentTab::Standings | CurrentTab::Settings | CurrentTab::Browser)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::types::SharedData;
    use crate::commands::standings::GroupBy;
    use std::str::FromStr;

    #[test]
    fn test_app_state_new() {
        let app_state = AppState::new();
        assert_eq!(app_state.current_tab, CurrentTab::Scores);
        assert!(app_state.command_palette.is_some());
        assert!(!app_state.command_palette_active);
    }

    #[test]
    fn test_open_command_palette() {
        let mut app_state = AppState::new();
        assert!(!app_state.command_palette_active);

        app_state.open_command_palette();

        assert!(app_state.command_palette_active);
        assert!(app_state.command_palette.as_ref().unwrap().is_visible);
    }

    #[test]
    fn test_close_command_palette() {
        let mut app_state = AppState::new();
        app_state.open_command_palette();

        assert!(app_state.command_palette_active);

        app_state.close_command_palette();

        assert!(!app_state.command_palette_active);
        assert!(!app_state.command_palette.as_ref().unwrap().is_visible);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_tab() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Scores;
        app_state.open_command_palette();

        let command = NavigationCommand::GoToTab(CurrentTab::Standings);
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Standings);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_team() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Scores;
        app_state.open_command_palette();

        let command = NavigationCommand::GoToTeam("TOR".to_string());
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Standings);
        assert_eq!(shared_data.read().await.selected_team_abbrev, Some("TOR".to_string()));
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_player() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.open_command_palette();

        let command = NavigationCommand::GoToPlayer(8479318);
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(shared_data.read().await.selected_player_id, Some(8479318));
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_game() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Standings;
        app_state.open_command_palette();

        let command = NavigationCommand::GoToGame(2024020001);
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Scores);
        assert_eq!(shared_data.read().await.selected_game_id, Some(2024020001));
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_date() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Standings;
        app_state.open_command_palette();

        let date = nhl_api::GameDate::from_ymd(2024, 11, 8).unwrap();
        let command = NavigationCommand::GoToDate(date.clone());
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Scores);
        assert!(app_state.scores.subtab_focused);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_standings_view() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Scores;
        app_state.open_command_palette();

        let command = NavigationCommand::GoToStandingsView(GroupBy::Conference);
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Standings);
        assert_eq!(app_state.standings.view, GroupBy::Conference);
        assert!(app_state.standings.subtab_focused);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_command_go_to_settings() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        app_state.current_tab = CurrentTab::Scores;
        app_state.open_command_palette();

        let command = NavigationCommand::GoToSettings("display".to_string());
        app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

        assert_eq!(app_state.current_tab, CurrentTab::Settings);
        assert!(!app_state.command_palette_active);
    }

    #[tokio::test]
    async fn test_execute_navigation_always_closes_palette() {
        let mut app_state = AppState::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let (tx, _rx) = mpsc::channel(10);

        let commands = vec![
            NavigationCommand::GoToTab(CurrentTab::Scores),
            NavigationCommand::GoToTeam("MTL".to_string()),
            NavigationCommand::GoToPlayer(12345),
            NavigationCommand::GoToSettings("theme".to_string()),
        ];

        for command in commands {
            app_state.open_command_palette();
            assert!(app_state.command_palette_active);

            app_state.execute_navigation_command(command, &shared_data, &tx).await.unwrap();

            assert!(!app_state.command_palette_active);
        }
    }
}
