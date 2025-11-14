use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget as RatatuiWidget},
};

use nhl_api::Boxscore;

use crate::config::DisplayConfig;
use crate::tui::framework::component::{Component, Element, RenderableWidget};

/// BoxscorePanel component props
#[derive(Clone)]
pub struct BoxscorePanelProps {
    pub game_id: i64,
    pub boxscore: Option<Boxscore>,
    pub loading: bool,
}

/// BoxscorePanel component - displays detailed game statistics
pub struct BoxscorePanel;

impl Component for BoxscorePanel {
    type Props = BoxscorePanelProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(BoxscorePanelWidget {
            game_id: props.game_id,
            boxscore: props.boxscore.clone(),
            loading: props.loading,
        }))
    }
}

/// Widget for rendering boxscore panel
struct BoxscorePanelWidget {
    game_id: i64,
    boxscore: Option<Boxscore>,
    loading: bool,
}

impl RenderableWidget for BoxscorePanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, _config: &DisplayConfig) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Boxscore - Game {} ", self.game_id))
            .style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.loading {
            let loading_text = Paragraph::new("Loading boxscore...");
            loading_text.render(inner, buf);
            return;
        }

        if let Some(boxscore) = &self.boxscore {
            self.render_boxscore(boxscore, inner, buf);
        } else {
            let error_text = Paragraph::new("Boxscore not available");
            error_text.render(inner, buf);
        }
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(BoxscorePanelWidget {
            game_id: self.game_id,
            boxscore: self.boxscore.clone(),
            loading: self.loading,
        })
    }
}

impl BoxscorePanelWidget {
    fn render_boxscore(&self, boxscore: &Boxscore, area: Rect, buf: &mut Buffer) {
        // Split area into sections
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Header info
                Constraint::Length(4), // Score
                Constraint::Min(0),    // Player stats (scrollable later)
            ])
            .split(area);

        self.render_header(boxscore, chunks[0], buf);
        self.render_score(boxscore, chunks[1], buf);
        self.render_player_stats_summary(boxscore, chunks[2], buf);
    }

    fn render_header(&self, boxscore: &Boxscore, area: Rect, buf: &mut Buffer) {
        let title = format!(
            "{} @ {}",
            boxscore.away_team.common_name.default, boxscore.home_team.common_name.default
        );

        let date_venue = format!(
            "Date: {} | Venue: {}",
            boxscore.game_date, boxscore.venue.default
        );

        let period_text = format_period_text(&boxscore.period_descriptor.number, &boxscore.period_descriptor.period_type);
        let status_period = format!(
            "Status: {} | Period: {}",
            format_game_state(&boxscore.game_state),
            period_text
        );

        let time_info = if boxscore.clock.running || !boxscore.clock.in_intermission {
            format!("Time: {}", boxscore.clock.time_remaining)
        } else if boxscore.clock.in_intermission {
            "INTERMISSION".to_string()
        } else {
            String::new()
        };

        let lines = vec![
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(date_venue),
            Line::from(status_period),
            Line::from(time_info),
        ];

        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }

    fn render_score(&self, boxscore: &Boxscore, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            Line::from(Span::styled(
                "SCORE",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "{:<6} {:>3}",
                boxscore.away_team.abbrev, boxscore.away_team.score
            )),
            Line::from(format!(
                "{:<6} {:>3}",
                boxscore.home_team.abbrev, boxscore.home_team.score
            )),
        ];

        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }

    fn render_player_stats_summary(&self, boxscore: &Boxscore, area: Rect, buf: &mut Buffer) {
        let away_forwards = boxscore.player_by_game_stats.away_team.forwards.len();
        let away_defense = boxscore.player_by_game_stats.away_team.defense.len();
        let away_goalies = boxscore.player_by_game_stats.away_team.goalies.len();

        let home_forwards = boxscore.player_by_game_stats.home_team.forwards.len();
        let home_defense = boxscore.player_by_game_stats.home_team.defense.len();
        let home_goalies = boxscore.player_by_game_stats.home_team.goalies.len();

        let lines = vec![
            Line::from(Span::styled(
                "PLAYER STATS",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("{} Roster:", boxscore.away_team.abbrev)),
            Line::from(format!(
                "  Forwards: {}, Defense: {}, Goalies: {}",
                away_forwards, away_defense, away_goalies
            )),
            Line::from(""),
            Line::from(format!("{} Roster:", boxscore.home_team.abbrev)),
            Line::from(format!(
                "  Forwards: {}, Defense: {}, Goalies: {}",
                home_forwards, home_defense, home_goalies
            )),
            Line::from(""),
            Line::from(Span::styled(
                "(Full player stats coming soon)",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}

fn format_game_state(state: &nhl_api::GameState) -> &str {
    match state {
        nhl_api::GameState::Future => "SCHEDULED",
        nhl_api::GameState::PreGame => "PRE-GAME",
        nhl_api::GameState::Live => "LIVE",
        nhl_api::GameState::Final => "FINAL",
        nhl_api::GameState::Off => "OFF",
        nhl_api::GameState::Postponed => "POSTPONED",
        nhl_api::GameState::Suspended => "SUSPENDED",
        nhl_api::GameState::Critical => "CRITICAL",
    }
}

fn format_period_text(number: &i32, period_type: &str) -> String {
    match period_type {
        "REG" => format!("{}", number),
        "OT" => "OT".to_string(),
        "SO" => "SO".to_string(),
        _ => format!("{}", number),
    }
}

// TODO: Re-enable tests once we have proper test fixtures for Boxscore type
// The nhl_api types have changed and these tests need to be updated
/*
#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::{BoxscoreTeam, GameClock, GameState, LocalizedString, PeriodDescriptor, PlayerByGameStats, TeamPlayerStats};

    fn create_test_boxscore() -> Boxscore {
        Boxscore {
            id: 2024020001,
            season: 20242025,
            game_type: 2,
            limited_scoring: false,
            game_date: "2024-10-04".to_string(),
            venue: LocalizedString {
                default: "Test Arena".to_string(),
            },
            venue_location: LocalizedString {
                default: "Test City".to_string(),
            },
            start_time_utc: "2024-10-04T19:00:00Z".to_string(),
            eastern_utc_offset: "-04:00".to_string(),
            venue_utc_offset: "-04:00".to_string(),
            tv_broadcasts: vec![],
            game_state: GameState::Live,
            game_schedule_state: "OK".to_string(),
            period_descriptor: PeriodDescriptor {
                number: 2,
                period_type: "REG".to_string(),
                max_regulation_periods: 3,
            },
            special_event: None,
            away_team: BoxscoreTeam {
                id: 1,
                common_name: LocalizedString {
                    default: "Devils".to_string(),
                },
                abbrev: "NJD".to_string(),
                score: 2,
                sog: 15,
                logo: "".to_string(),
                dark_logo: "".to_string(),
                place_name: LocalizedString {
                    default: "New Jersey".to_string(),
                },
                place_name_with_preposition: LocalizedString {
                    default: "New Jersey".to_string(),
                },
            },
            home_team: BoxscoreTeam {
                id: 7,
                common_name: LocalizedString {
                    default: "Sabres".to_string(),
                },
                abbrev: "BUF".to_string(),
                score: 1,
                sog: 12,
                logo: "".to_string(),
                dark_logo: "".to_string(),
                place_name: LocalizedString {
                    default: "Buffalo".to_string(),
                },
                place_name_with_preposition: LocalizedString {
                    default: "Buffalo".to_string(),
                },
            },
            clock: GameClock {
                time_remaining: "10:15".to_string(),
                seconds_remaining: 615,
                running: true,
                in_intermission: false,
            },
            player_by_game_stats: PlayerByGameStats {
                away_team: TeamPlayerStats {
                    forwards: vec![],
                    defense: vec![],
                    goalies: vec![],
                },
                home_team: TeamPlayerStats {
                    forwards: vec![],
                    defense: vec![],
                    goalies: vec![],
                },
            },
        }
    }

    #[test]
    fn test_boxscore_panel_renders() {
        let panel = BoxscorePanel;
        let props = BoxscorePanelProps {
            game_id: 2024020001,
            boxscore: Some(create_test_boxscore()),
            loading: false,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Widget(_) => {
                // Widget created successfully
            }
            _ => panic!("Expected widget element"),
        }
    }

    #[test]
    fn test_boxscore_panel_loading() {
        let panel = BoxscorePanel;
        let props = BoxscorePanelProps {
            game_id: 2024020001,
            boxscore: None,
            loading: true,
        };

        let element = panel.view(&props, &());

        match element {
            Element::Widget(_) => {
                // Widget created successfully
            }
            _ => panic!("Expected widget element"),
        }
    }
}
*/
