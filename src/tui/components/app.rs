use crate::tui::component::{vertical, Component, Constraint, Element};
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
        use crate::tui::Tab;
        //
        // Convert Tab enum to string key
        let active_key = match state.navigation.current_tab {
            Tab::Scores => "scores",
            Tab::Standings => "standings",
            Tab::Stats => "stats",
            Tab::Players => "players",
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
                    self.render_scores_tab(state),
                    self.render_standings_tab(state),
                    self.render_settings_tab(state),
                )
            };
        //
        // Build Demo tab content
        let demo_content = DemoTab.view(
            &DemoTabProps {
                content_focused: state.navigation.content_focused,
                focus_index: state.ui.demo.focus_index,
                scroll_offset: state.ui.demo.scroll_offset,
                standings: state.data.standings.clone(),
            },
            &Default::default(),
        );

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
                    scroll_offset: panel_state.scroll_offset,
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
                    scroll_offset: panel_state.scroll_offset,
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
        let props = StandingsTabProps {
            view: state.ui.standings.view,
            browse_mode: state.ui.standings.browse_mode,
            selected_column: state.ui.standings.selected_column,
            selected_row: state.ui.standings.selected_row,
            scroll_offset: state.ui.standings.scroll_offset,
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
            editing: state.ui.settings.editing,
            edit_buffer: state.ui.settings.edit_buffer.clone(),
            modal_open: state.ui.settings.modal_open,
            modal_selected_index: state.ui.settings.modal_selected_index,
        };
        SettingsTab.view(&props, &())
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
