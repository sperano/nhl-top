use super::{scores, standings, stats, players, settings};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentTab {
    Scores,
    Standings,
    Stats,
    Players,
    Settings,
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

    pub fn all_names() -> [&'static str; 5] {
        ["Scores", "Standings", "Stats", "Players", "Settings"]
    }

    pub fn index(&self) -> usize {
        match self {
            CurrentTab::Scores => 0,
            CurrentTab::Standings => 1,
            CurrentTab::Stats => 2,
            CurrentTab::Players => 3,
            CurrentTab::Settings => 4,
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
        }
    }

    pub fn navigate_tab_left(&mut self) {
        self.current_tab = match self.current_tab {
            CurrentTab::Scores => CurrentTab::Settings,
            CurrentTab::Standings => CurrentTab::Scores,
            CurrentTab::Stats => CurrentTab::Standings,
            CurrentTab::Players => CurrentTab::Stats,
            CurrentTab::Settings => CurrentTab::Players,
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
            CurrentTab::Settings => CurrentTab::Scores,
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
            CurrentTab::Settings => {} // No subtabs for settings
        }
    }

    pub fn exit_subtab_mode(&mut self) {
        self.scores.subtab_focused = false;
        self.scores.box_selection_active = false;
        self.standings.subtab_focused = false;
    }

    pub fn is_subtab_focused(&self) -> bool {
        match self.current_tab {
            CurrentTab::Scores => self.scores.subtab_focused,
            CurrentTab::Standings => self.standings.subtab_focused,
            CurrentTab::Stats => false,
            CurrentTab::Players => false,
            CurrentTab::Settings => false,
        }
    }

    pub fn has_subtabs(&self) -> bool {
        matches!(self.current_tab, CurrentTab::Scores | CurrentTab::Standings)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
