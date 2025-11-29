use crate::tui::component::{vertical, Component, Constraint, Element};
use crate::tui::component_store::ComponentStateStore;
use crate::tui::state::{AppState, LoadingKey};
//
use super::{
    boxscore_panel::{BoxscorePanel, BoxscorePanelProps, TeamView},
    demo_tab::DemoTabProps,
    player_detail_panel::PlayerDetailPanelProps,
    scores_tab::ScoresTabProps,
    settings_tab::SettingsTabProps,
    standings_tab::StandingsTabProps,
    team_detail_panel::TeamDetailPanelProps,
    BreadcrumbWidget, DemoTab, PlayerDetailPanel, ScoresTab, SettingsTab, StandingsTab, StatusBar,
    TabItem, TabbedPanel, TabbedPanelProps, TeamDetailPanel,
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
        // Note: This is kept for backward compatibility with the Component trait,
        // but Runtime now calls build_with_component_states() directly
        vertical(
            [
                Constraint::Min(0),    // TabbedPanel (tabs + content)
                Constraint::Length(2), // StatusBar (2 lines: separator + content)
            ],
            vec![
                self.render_main_tabs_without_states(props),
                StatusBar.view(&props.system, &()),
            ],
        )
    }
}
//
impl App {
    /// Build the app element tree with access to component states
    ///
    /// This is called by Runtime instead of the normal view() method,
    /// allowing App to access component_states for child components.
    pub fn build_with_component_states(
        &self,
        state: &AppState,
        component_states: &mut ComponentStateStore,
    ) -> Element {
        vertical(
            [
                Constraint::Min(0),    // TabbedPanel (tabs + content)
                Constraint::Length(2), // StatusBar (2 lines: separator + content)
            ],
            vec![
                self.render_main_tabs_with_states(state, component_states),
                StatusBar.view(&state.system, &()),
            ],
        )
    }

    /// Render main navigation tabs using TabbedPanel (without component states - for tests)
    fn render_main_tabs_without_states(&self, state: &AppState) -> Element {
        use crate::tui::Tab;

        let active_key = match state.navigation.current_tab {
            Tab::Scores => "scores",
            Tab::Standings => "standings",
            Tab::Settings => "settings",
            Tab::Demo => "demo",
        };

        let (scores_content, standings_content, settings_content) =
            if let Some(panel_state) = state.navigation.panel_stack.last() {
                let panel_element = self.render_panel(state, panel_state);
                let breadcrumb_element = self.render_breadcrumb(state);

                let content_with_breadcrumb = vertical(
                    [
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ],
                    vec![breadcrumb_element, panel_element],
                );

                match state.navigation.current_tab {
                    Tab::Scores => (content_with_breadcrumb, Element::None, Element::None),
                    Tab::Standings => (Element::None, content_with_breadcrumb, Element::None),
                    Tab::Settings => (Element::None, Element::None, content_with_breadcrumb),
                    _ => (Element::None, Element::None, Element::None),
                }
            } else {
                (
                    self.render_scores_tab(state),
                    self.render_standings_tab(state),
                    self.render_settings_tab(state),
                )
            };

        let demo_content = DemoTab.view(
            &DemoTabProps {
                content_focused: state.navigation.content_focused,
                standings: state.data.standings.clone(),
            },
            &Default::default(),
        );

        let tabs = vec![
            TabItem::new("scores", "Scores", scores_content),
            TabItem::new("standings", "Standings", standings_content),
            TabItem::new("stats", "Stats", Element::None),
            TabItem::new("players", "Players", Element::None),
            TabItem::new("settings", "Settings", settings_content),
            TabItem::new("demo", "Demo", demo_content),
        ];

        TabbedPanel.view(
            &TabbedPanelProps {
                active_key: active_key.into(),
                tabs,
                focused: !state.navigation.content_focused
                    && state.navigation.panel_stack.is_empty(),
            },
            &(),
        )
    }

    /// Render main navigation tabs using TabbedPanel (with component states)
    fn render_main_tabs_with_states(
        &self,
        state: &AppState,
        component_states: &mut ComponentStateStore,
    ) -> Element {
        use crate::tui::Tab;
        //
        // Convert Tab enum to string key
        let active_key = match state.navigation.current_tab {
            Tab::Scores => "scores",
            Tab::Standings => "standings",
            Tab::Settings => "settings",
            Tab::Demo => "demo",
        };
        //
        // Determine content for active tab - if panel is open, show panel instead
        let (scores_content, standings_content, settings_content) =
            if let Some(panel_state) = state.navigation.panel_stack.last() {
                // Panel is open - render it with breadcrumb in the active tab's content area
                let panel_element = self.render_panel(state, panel_state);
                let breadcrumb_element = self.render_breadcrumb(state);

                // Wrap panel with breadcrumb
                let content_with_breadcrumb = vertical(
                    [
                        Constraint::Length(1), // Breadcrumb (1 line)
                        Constraint::Min(0),    // Panel content
                    ],
                    vec![breadcrumb_element, panel_element],
                );

                match state.navigation.current_tab {
                    Tab::Scores => (content_with_breadcrumb, Element::None, Element::None),
                    Tab::Standings => (Element::None, content_with_breadcrumb, Element::None),
                    Tab::Settings => (Element::None, Element::None, content_with_breadcrumb),
                    _ => (Element::None, Element::None, Element::None),
                }
            } else {
                // No panel - render normal tab content
                (
                    self.render_scores_tab_with_states(state, component_states),
                    self.render_standings_tab_with_states(state, component_states),
                    self.render_settings_tab_with_states(state, component_states),
                )
            };
        //
        // Build Demo tab content
        let demo_props = DemoTabProps {
            content_focused: state.navigation.content_focused,
            standings: state.data.standings.clone(),
        };
        let demo_state = component_states.get_or_init::<DemoTab>("app/demo_tab", &demo_props);
        let demo_content = DemoTab.view(&demo_props, demo_state);

        // Build tabs with their content
        let tabs = vec![
            TabItem::new("scores", "Scores", scores_content),
            TabItem::new("standings", "Standings", standings_content),
            TabItem::new("stats", "Stats", Element::None), // TODO
            TabItem::new("players", "Players", Element::None), // TODO
            TabItem::new("settings", "Settings", settings_content),
            TabItem::new("demo", "Demo", demo_content),
        ];
        //
        TabbedPanel.view(
            &TabbedPanelProps {
                active_key: active_key.into(),
                tabs,
                focused: !state.navigation.content_focused
                    && state.navigation.panel_stack.is_empty(),
            },
            &(),
        )
    }
    //
    /// Render a panel overlay
    fn render_panel(
        &self,
        state: &AppState,
        panel_state: &crate::tui::state::PanelState,
    ) -> Element {
        use crate::tui::types::Panel;
        //
        match &panel_state.panel {
            Panel::Boxscore { game_id } => {
                let props = BoxscorePanelProps {
                    game_id: *game_id,
                    boxscore: state.data.boxscores.get(game_id).cloned(),
                    loading: state.data.loading.contains(&LoadingKey::Boxscore(*game_id)),
                    team_view: TeamView::Away, // TODO: Store in panel state to allow switching
                    selected_index: panel_state.selected_index,
                    focused: true, // Panel has focus when it's on the stack
                };
                BoxscorePanel.view(&props, &())
            }
            Panel::TeamDetail { abbrev } => {
                // Find the standing for this team
                let standing = state
                    .data
                    .standings
                    .as_ref()
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
                    loading: state
                        .data
                        .loading
                        .contains(&LoadingKey::TeamRosterStats(abbrev.clone())),
                    selected_index: panel_state.selected_index,
                };
                TeamDetailPanel.view(&props, &())
            }
            Panel::PlayerDetail { player_id } => {
                let props = PlayerDetailPanelProps {
                    player_id: *player_id,
                    player_data: state.data.player_data.get(player_id).cloned(),
                    loading: state
                        .data
                        .loading
                        .contains(&LoadingKey::PlayerStats(*player_id)),
                    selected_index: panel_state.selected_index,
                };
                PlayerDetailPanel.view(&props, &())
            }
        }
    }
    //
    /// Render Scores tab content (with component states - Phase 3.5)
    fn render_scores_tab_with_states(
        &self,
        state: &AppState,
        component_states: &mut ComponentStateStore,
    ) -> Element {
        use crate::tui::components::scores_tab::ScoresTab;

        let props = ScoresTabProps {
            schedule: state.data.schedule.clone(),
            game_info: state.data.game_info.clone(),
            period_scores: state.data.period_scores.clone(),
            focused: state.navigation.content_focused,
        };

        // Get or initialize component state from the component store
        let scores_state =
            component_states.get_or_init::<ScoresTab>("app/scores_tab", &props);
        ScoresTab.view(&props, scores_state)
    }

    /// Render Scores tab content (old method - kept for compatibility during migration)
    #[allow(dead_code)]
    fn render_scores_tab(&self, state: &AppState) -> Element {
        use crate::tui::components::scores_tab::ScoresTabState;

        let props = ScoresTabProps {
            schedule: state.data.schedule.clone(),
            game_info: state.data.game_info.clone(),
            period_scores: state.data.period_scores.clone(),
            focused: state.navigation.content_focused,
        };
        let component_state = ScoresTabState::default();
        ScoresTab.view(&props, &component_state)
    }
    //
    /// Render Standings tab content (with component states - Phase 4)
    fn render_standings_tab_with_states(
        &self,
        state: &AppState,
        component_states: &mut ComponentStateStore,
    ) -> Element {
        use crate::tui::components::standings_tab::StandingsTab;

        let props = StandingsTabProps {
            standings: state.data.standings.clone(),
            panel_stack: state.navigation.panel_stack.clone(),
            focused: state.navigation.content_focused,
            config: state.system.config.clone(),
        };

        let standings_state =
            component_states.get_or_init::<StandingsTab>("app/standings_tab", &props);
        StandingsTab.view(&props, standings_state)
    }

    /// Render Standings tab content (old method - kept for compatibility during migration)
    #[allow(dead_code)]
    fn render_standings_tab(&self, state: &AppState) -> Element {
        use crate::tui::components::standings_tab::StandingsTabState;

        let props = StandingsTabProps {
            standings: state.data.standings.clone(),
            panel_stack: state.navigation.panel_stack.clone(),
            focused: state.navigation.content_focused,
            config: state.system.config.clone(),
        };
        let component_state = StandingsTabState::default();
        StandingsTab.view(&props, &component_state)
    }
    //
    /// Render Settings tab content with component state management
    fn render_settings_tab_with_states(
        &self,
        state: &AppState,
        component_states: &mut ComponentStateStore,
    ) -> Element {
        let props = SettingsTabProps {
            config: state.system.config.clone(),
            selected_category: state.ui.settings.selected_category,
            focused: state.navigation.content_focused,
        };

        let settings_state = component_states.get_or_init::<SettingsTab>("app/settings_tab", &props);
        SettingsTab.view(&props, settings_state)
    }

    /// Render Settings tab content (legacy - without state management)
    fn render_settings_tab(&self, state: &AppState) -> Element {
        use crate::tui::components::SettingsTabState;
        let props = SettingsTabProps {
            config: state.system.config.clone(),
            selected_category: state.ui.settings.selected_category,
            focused: state.navigation.content_focused,
        };
        let component_state = SettingsTabState::default();
        SettingsTab.view(&props, &component_state)
    }
    //
    /// Render breadcrumb navigation
    fn render_breadcrumb(&self, state: &AppState) -> Element {
        Element::Widget(Box::new(BreadcrumbWidget::new(
            state.navigation.current_tab,
            state.navigation.panel_stack.clone(),
        )))
    }
}
//
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::AppState;
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
                    crate::tui::component::ContainerLayout::Vertical(constraints) => {
                        assert_eq!(constraints.len(), 2);
                    }
                    _ => panic!("Expected vertical layout"),
                }
            }
            _ => panic!("Expected container element"),
        }
    }
}
