use crate::tui::framework::component::{vertical, Component, Constraint, Element};
use crate::tui::framework::state::{AppState, LoadingKey};
//
use super::{
    boxscore_panel::BoxscorePanelProps, scores_tab::ScoresTabProps,
    settings_tab::SettingsTabProps, standings_tab::StandingsTabProps,
    team_detail_panel::TeamDetailPanelProps, player_detail_panel::PlayerDetailPanelProps,
    BoxscorePanel, ScoresTab, SettingsTab, StandingsTab, StatusBar, TabbedPanel,
    TabbedPanelProps, TabItem, TeamDetailPanel, PlayerDetailPanel,
};
//
/// Root App component
///
/// This is the top-level component that renders the entire application.
/// It uses the global AppState as props and delegates rendering to child components.
pub struct App;
//
impl Component for App {
    type Props = AppState;
    type State = ();
    type Message = ();
//
    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        tracing::trace!("APP: App.view() called with panel_stack.len={}", props.navigation.panel_stack.len());
        vertical(
            [
                Constraint::Min(0),    // TabbedPanel (tabs + content)
                Constraint::Length(2), // StatusBar (2 lines: separator + content)
            ],
            vec![
                self.render_main_tabs(props),
                StatusBar.view(&props.system, &()),
            ],
        )
    }
}
//
impl App {
    /// Render main navigation tabs using TabbedPanel
    fn render_main_tabs(&self, state: &AppState) -> Element {
        use crate::tui::framework::action::Tab;
//
        // Convert Tab enum to string key
        let active_key = match state.navigation.current_tab {
            Tab::Scores => "scores",
            Tab::Standings => "standings",
            Tab::Stats => "stats",
            Tab::Players => "players",
            Tab::Settings => "settings",
            Tab::Browser => "browser",
        };
//
        // Determine content for active tab - if panel is open, show panel instead
        let (scores_content, standings_content, settings_content) = if let Some(panel_state) = state.navigation.panel_stack.last() {
            // Panel is open - render it in the active tab's content area
            let panel_element = self.render_panel(state, panel_state);
            match state.navigation.current_tab {
                Tab::Scores => (panel_element, Element::None, Element::None),
                Tab::Standings => (Element::None, panel_element, Element::None),
                Tab::Settings => (Element::None, Element::None, panel_element),
                _ => (Element::None, Element::None, Element::None),
            }
        } else {
            // No panel - render normal tab content
            (
                self.render_scores_tab(state),
                self.render_standings_tab(state),
                self.render_settings_tab(state),
            )
        };
//
        // Build tabs with their content
        let tabs = vec![
            TabItem::new("scores", "Scores", scores_content),
            TabItem::new("standings", "Standings", standings_content),
            TabItem::new("stats", "Stats", Element::None), // TODO
            TabItem::new("players", "Players", Element::None), // TODO
            TabItem::new("settings", "Settings", settings_content),
            TabItem::new("browser", "Browser", Element::None), // TODO
        ];
//
        TabbedPanel.view(
            &TabbedPanelProps {
                active_key: active_key.into(),
                tabs,
                focused: !state.navigation.content_focused && state.navigation.panel_stack.is_empty(),
            },
            &(),
        )
    }
//
    /// Render a panel overlay
    fn render_panel(
        &self,
        state: &AppState,
        panel_state: &crate::tui::framework::state::PanelState,
    ) -> Element {
        use crate::tui::framework::action::Panel;
//
        match &panel_state.panel {
            Panel::Boxscore { game_id } => {
                let props = BoxscorePanelProps {
                    game_id: *game_id,
                    boxscore: state.data.boxscores.get(game_id).cloned(),
                    loading: state.data.loading.contains(&LoadingKey::Boxscore(*game_id)),
                };
                BoxscorePanel.view(&props, &())
            }
            Panel::TeamDetail { abbrev } => {
                // Find the standing for this team
                let standing = state
                    .data
                    .standings
                    .as_ref()
                    .and_then(|standings| {
                        standings
                            .iter()
                            .find(|s| s.team_abbrev.default == *abbrev)
                            .cloned()
                    });
//
                let props = TeamDetailPanelProps {
                    team_abbrev: abbrev.clone(),
                    standing,
                    club_stats: state.data.team_roster_stats.get(abbrev).cloned(),
                    loading: state.data.loading.contains(&LoadingKey::TeamRosterStats(abbrev.clone())),
                    scroll_offset: panel_state.scroll_offset,
                    selected_index: panel_state.selected_index,
                };
                TeamDetailPanel.view(&props, &())
            }
            Panel::PlayerDetail { player_id } => {
                let props = PlayerDetailPanelProps {
                    player_id: *player_id,
                    player_data: state.data.player_data.get(player_id).cloned(),
                    loading: state.data.loading.contains(&LoadingKey::PlayerStats(*player_id)),
                    scroll_offset: panel_state.scroll_offset,
                    selected_index: panel_state.selected_index,
                };
                PlayerDetailPanel.view(&props, &())
            }
        }
    }
//
    /// Render Scores tab content
    fn render_scores_tab(&self, state: &AppState) -> Element {
        let props = ScoresTabProps {
            game_date: state.ui.scores.game_date.clone(),
            selected_index: state.ui.scores.selected_date_index,
            schedule: state.data.schedule.clone(),
            game_info: state.data.game_info.clone(),
            period_scores: state.data.period_scores.clone(),
            box_selection_active: state.ui.scores.box_selection_active,
            selected_game_index: state.ui.scores.selected_game_index,
            focused: state.navigation.content_focused,
        };
        ScoresTab.view(&props, &())
    }
//
    /// Render Standings tab content
    fn render_standings_tab(&self, state: &AppState) -> Element {
        tracing::debug!(
            "APP: Building StandingsTab with panel_stack.len = {}",
            state.navigation.panel_stack.len()
        );
        let props = StandingsTabProps {
            view: state.ui.standings.view.clone(),
            browse_mode: state.ui.standings.browse_mode,
            selected_column: state.ui.standings.selected_column,
            selected_row: state.ui.standings.selected_row,
            standings: state.data.standings.clone(),
            panel_stack: state.navigation.panel_stack.clone(),
            focused: state.navigation.content_focused,
            config: state.system.config.clone(),
        };
        StandingsTab.view(&props, &())
    }
//
    /// Render Settings tab content
    fn render_settings_tab(&self, state: &AppState) -> Element {
        let props = SettingsTabProps {
            config: state.system.config.clone(),
            selected_category: state.ui.settings.selected_category,
            selected_setting_index: state.ui.settings.selected_setting_index,
            settings_mode: state.ui.settings.settings_mode,
            focused: state.navigation.content_focused,
        };
        SettingsTab.view(&props, &())
    }
}
//
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::framework::state::AppState;
//
    #[test]
    fn test_app_renders_with_default_state() {
        let app = App;
        let state = AppState::default();
//
        let element = app.view(&state, &());
//
        // Should render a vertical container with 2 children (TabbedPanel + StatusBar)
        match element {
            Element::Container {
                children, layout, ..
            } => {
                assert_eq!(children.len(), 2);
                match layout {
                    crate::tui::framework::component::ContainerLayout::Vertical(constraints) => {
                        assert_eq!(constraints.len(), 2);
                    }
                    _ => panic!("Expected vertical layout"),
                }
            }
            _ => panic!("Expected container element"),
        }
    }
}
