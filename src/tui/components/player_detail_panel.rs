use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Style, Modifier},
};

use nhl_api::{PlayerLanding, SeasonTotal};

use crate::config::DisplayConfig;
use crate::team_abbrev::common_name_to_abbrev;
use crate::tui::framework::{
    component::{Component, Element, RenderableWidget},
    Alignment, CellValue, ColumnDef,
};
use super::table::TableWidget;

/// Props for PlayerDetailPanel component
#[derive(Clone)]
pub struct PlayerDetailPanelProps {
    pub player_id: i64,
    pub player_data: Option<PlayerLanding>,
    pub loading: bool,
    pub scroll_offset: usize,
    pub selected_index: Option<usize>,
}

/// PlayerDetailPanel component - renders player info and career stats
pub struct PlayerDetailPanel;

impl Component for PlayerDetailPanel {
    type Props = PlayerDetailPanelProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(PlayerDetailPanelWidget {
            player_id: props.player_id,
            player_data: props.player_data.clone(),
            loading: props.loading,
            scroll_offset: props.scroll_offset,
            selected_index: props.selected_index,
        }))
    }
}

/// Widget for rendering the player detail panel
#[derive(Clone)]
struct PlayerDetailPanelWidget {
    player_id: i64,
    player_data: Option<PlayerLanding>,
    loading: bool,
    scroll_offset: usize,
    selected_index: Option<usize>,
}

impl RenderableWidget for PlayerDetailPanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.loading {
            let text = format!("Loading player {} details...", self.player_id);
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Player Detail"));
            ratatui::widgets::Widget::render(widget, area, buf);
            return;
        }

        let Some(ref player) = self.player_data else {
            let text = format!("No data available for player {}", self.player_id);
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Player Detail"));
            ratatui::widgets::Widget::render(widget, area, buf);
            return;
        };

        let mut y = area.y + 1; // Leave space for border
        let x = area.x + 2; // Left margin inside border

        // Render player info header
        let full_name = format!("{} {}",
            player.first_name.default,
            player.last_name.default
        );
        buf.set_string(x, y, &full_name, Style::default().add_modifier(Modifier::BOLD));
        y += 1;

        // Player details line 1
        let team_info = if let Some(ref team_abbrev) = player.current_team_abbrev {
            format!("Team: {} | ", team_abbrev)
        } else {
            String::new()
        };
        let sweater = if let Some(num) = player.sweater_number {
            format!("#{} | ", num)
        } else {
            String::new()
        };
        let details1 = format!(
            "{}{}{} | {}/{}",
            team_info,
            sweater,
            player.position,
            player.shoots_catches,
            if player.position == "G" { "Catches" } else { "Shoots" }
        );
        buf.set_string(x, y, &details1, Style::default());
        y += 1;

        // Player details line 2
        let height_feet = player.height_in_inches / 12;
        let height_inches = player.height_in_inches % 12;
        let details2 = format!(
            "Height: {}'{}\" | Weight: {} lbs | Born: {}",
            height_feet,
            height_inches,
            player.weight_in_pounds,
            player.birth_date
        );
        buf.set_string(x, y, &details2, Style::default());
        y += 2;

        // Draft details if available
        if let Some(ref draft) = player.draft_details {
            let draft_info = format!(
                "Draft: {} - Round {}, Pick {} (#{} overall) by {}",
                draft.year,
                draft.round,
                draft.pick_in_round,
                draft.overall_pick,
                draft.team_abbrev
            );
            buf.set_string(x, y, &draft_info, Style::default());
            y += 2;
        }

        // Display career totals if available
        if let Some(ref career) = player.career_totals {
            let career_header = "CAREER TOTALS - Regular Season";
            buf.set_string(x, y, career_header, Style::default().add_modifier(Modifier::BOLD));
            y += 1;

            let career_stats = if player.position == "G" {
                format!(
                    "GP: {} | W: {} | L: {} | OTL: {} | GAA: {:.2} | SV%: {:.3} | SO: {}",
                    career.regular_season.games_played.unwrap_or(0),
                    career.regular_season.wins.unwrap_or(0),
                    career.regular_season.losses.unwrap_or(0),
                    career.regular_season.ot_losses.unwrap_or(0),
                    career.regular_season.goals_against_avg.unwrap_or(0.0),
                    career.regular_season.save_pctg.unwrap_or(0.0),
                    career.regular_season.shutouts.unwrap_or(0)
                )
            } else {
                format!(
                    "GP: {} | G: {} | A: {} | PTS: {} | +/-: {} | PIM: {}",
                    career.regular_season.games_played.unwrap_or(0),
                    career.regular_season.goals.unwrap_or(0),
                    career.regular_season.assists.unwrap_or(0),
                    career.regular_season.points.unwrap_or(0),
                    career.regular_season.plus_minus.unwrap_or(0),
                    career.regular_season.pim.unwrap_or(0)
                )
            };
            buf.set_string(x, y, &career_stats, Style::default());
            y += 2;
        }

        // Get regular season stats only (game_type_id == 2)
        let mut season_stats: Vec<SeasonTotal> = player.season_totals
            .as_ref()
            .map(|seasons| {
                seasons.iter()
                    .filter(|s| s.game_type_id == 2 && s.league_abbrev == "NHL")
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        // Sort by season descending (latest first)
        season_stats.sort_by(|a, b| b.season.cmp(&a.season));

        if !season_stats.is_empty() {
            // Create season-by-season table
            let is_goalie = player.position == "G";

            let columns = if is_goalie {
                vec![
                    ColumnDef::new("Season", 8, Alignment::Left, |s: &SeasonTotal| {
                        let season_str = s.season.to_string();
                        let formatted = format!("{}-{}",
                            &season_str[0..4],
                            &season_str[4..8]
                        );
                        CellValue::Text(formatted)
                    }),
                    ColumnDef::new("Team", 25, Alignment::Left, |s: &SeasonTotal| {
                        // Try to get team abbreviation from common name, fallback to full name
                        if let Some(ref common_name) = s.team_common_name {
                            if let Some(abbrev) = common_name_to_abbrev(&common_name.default) {
                                CellValue::TeamLink {
                                    display: s.team_name.default.clone(),
                                    team_abbrev: abbrev.to_string(),
                                }
                            } else {
                                // No mapping found, just show text
                                CellValue::Text(s.team_name.default.clone())
                            }
                        } else {
                            // No common name available, just show text
                            CellValue::Text(s.team_name.default.clone())
                        }
                    }),
                    ColumnDef::new("GP", 4, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.games_played.to_string())
                    }),
                    // Goalie stats would go here - but SeasonTotal doesn't have them
                    // We'll just show GP for now
                ]
            } else {
                vec![
                    ColumnDef::new("Season", 8, Alignment::Left, |s: &SeasonTotal| {
                        let season_str = s.season.to_string();
                        let formatted = format!("{}-{}",
                            &season_str[0..4],
                            &season_str[4..8]
                        );
                        CellValue::Text(formatted)
                    }),
                    ColumnDef::new("Team", 25, Alignment::Left, |s: &SeasonTotal| {
                        // Try to get team abbreviation from common name, fallback to full name
                        if let Some(ref common_name) = s.team_common_name {
                            if let Some(abbrev) = common_name_to_abbrev(&common_name.default) {
                                CellValue::TeamLink {
                                    display: s.team_name.default.clone(),
                                    team_abbrev: abbrev.to_string(),
                                }
                            } else {
                                // No mapping found, just show text
                                CellValue::Text(s.team_name.default.clone())
                            }
                        } else {
                            // No common name available, just show text
                            CellValue::Text(s.team_name.default.clone())
                        }
                    }),
                    ColumnDef::new("GP", 4, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.games_played.to_string())
                    }),
                    ColumnDef::new("G", 3, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.goals.unwrap_or(0).to_string())
                    }),
                    ColumnDef::new("A", 3, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.assists.unwrap_or(0).to_string())
                    }),
                    ColumnDef::new("PTS", 4, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.points.unwrap_or(0).to_string())
                    }),
                    ColumnDef::new("+/-", 4, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(
                            s.plus_minus
                                .map(|v| format!("{:+}", v))
                                .unwrap_or_else(|| "0".to_string())
                        )
                    }),
                    ColumnDef::new("PIM", 4, Alignment::Right, |s: &SeasonTotal| {
                        CellValue::Text(s.pim.unwrap_or(0).to_string())
                    }),
                ]
            };

            // Calculate windowing for scrolling
            let total_seasons = season_stats.len();
            let available_height = area.height.saturating_sub(y - area.y).saturating_sub(4) as usize;
            let visible_end = (self.scroll_offset + available_height).min(total_seasons);
            let show_from = self.scroll_offset.min(total_seasons);
            let show_to = visible_end;

            let windowed_seasons: Vec<_> = season_stats[show_from..show_to].to_vec();

            // Build table first to find the link column
            let mut seasons_table = TableWidget::from_data(columns, windowed_seasons);

            // Find the first link column (should be Team column, index 1)
            let link_column = seasons_table.find_first_link_column().unwrap_or(0);

            // Adjust selection for windowing
            let selected_row = self.selected_index
                .filter(|&idx| idx >= show_from && idx < show_to)
                .map(|idx| idx - show_from);

            seasons_table = seasons_table
                .with_selection_opt(selected_row, Some(link_column))
                .with_focused(true)
                .with_header(format!("SEASON BY SEASON ({} NHL seasons)", total_seasons))
                .with_margin(2);

            let table_height = seasons_table.preferred_height().unwrap_or(0);
            let available_height_rect = area.height.saturating_sub(y - area.y).saturating_sub(2);
            let clamped_height = table_height.min(available_height_rect);
            let table_area = Rect::new(x, y, area.width.saturating_sub(4), clamped_height);
            seasons_table.render(table_area, buf, config);
        }

        // Render border and title
        let title = format!(
            "Player {} - ↑↓: Navigate | Enter: View Team | ESC: Back",
            self.player_id
        );
        let block = Block::default().borders(Borders::ALL).title(title);
        ratatui::widgets::Widget::render(block, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::{LocalizedString, SeasonTotal};
    use ratatui::{buffer::Buffer, layout::Rect};

    fn create_test_player(player_id: i64, position: &str) -> PlayerLanding {
        PlayerLanding {
            player_id,
            is_active: true,
            current_team_id: Some(10),
            current_team_abbrev: Some("TOR".to_string()),
            first_name: LocalizedString {
                default: "Test".to_string(),
            },
            last_name: LocalizedString {
                default: "Player".to_string(),
            },
            sweater_number: Some(34),
            position: position.to_string(),
            headshot: String::new(),
            hero_image: None,
            height_in_inches: 73,
            weight_in_pounds: 200,
            birth_date: "1997-09-15".to_string(),
            birth_city: Some(LocalizedString {
                default: "Toronto".to_string(),
            }),
            birth_state_province: Some(LocalizedString {
                default: "ON".to_string(),
            }),
            birth_country: Some("CAN".to_string()),
            shoots_catches: "L".to_string(),
            draft_details: None,
            player_slug: None,
            featured_stats: None,
            career_totals: None, // Skip career totals in test (types not exported from nhl_api)
            season_totals: Some(vec![
                SeasonTotal {
                    season: 20232024,
                    game_type_id: 2,
                    league_abbrev: "NHL".to_string(),
                    team_name: LocalizedString {
                        default: "Toronto Maple Leafs".to_string(),
                    },
                    team_common_name: Some(LocalizedString {
                        default: "Maple Leafs".to_string(),
                    }),
                    sequence: Some(1),
                    games_played: 82,
                    goals: Some(40),
                    assists: Some(50),
                    points: Some(90),
                    plus_minus: Some(10),
                    pim: Some(20),
                },
                SeasonTotal {
                    season: 20222023,
                    game_type_id: 2,
                    league_abbrev: "NHL".to_string(),
                    team_name: LocalizedString {
                        default: "Toronto Maple Leafs".to_string(),
                    },
                    team_common_name: Some(LocalizedString {
                        default: "Maple Leafs".to_string(),
                    }),
                    sequence: Some(1),
                    games_played: 78,
                    goals: Some(35),
                    assists: Some(45),
                    points: Some(80),
                    plus_minus: Some(8),
                    pim: Some(18),
                },
            ]),
            awards: None,
            last_five_games: None,
        }
    }

    #[test]
    fn test_player_panel_renders_with_data() {
        let player = create_test_player(8479318, "C");

        let widget = PlayerDetailPanelWidget {
            player_id: 8479318,
            player_data: Some(player),
            loading: false,
            scroll_offset: 0,
            selected_index: None,
        };

        let area = Rect::new(0, 0, 80, 30);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        widget.render(area, &mut buf, &config);

        // Verify rendering completed without panic
        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_player_panel_shows_loading() {
        let widget = PlayerDetailPanelWidget {
            player_id: 8479318,
            player_data: None,
            loading: true,
            scroll_offset: 0,
            selected_index: None,
        };

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        widget.render(area, &mut buf, &config);

        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_player_panel_handles_no_data() {
        let widget = PlayerDetailPanelWidget {
            player_id: 8479318,
            player_data: None,
            loading: false,
            scroll_offset: 0,
            selected_index: None,
        };

        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        widget.render(area, &mut buf, &config);

        assert_eq!(*buf.area(), area);
    }

    #[test]
    fn test_player_panel_with_limited_height() {
        let player = create_test_player(8479318, "C");

        let widget = PlayerDetailPanelWidget {
            player_id: 8479318,
            player_data: Some(player),
            loading: false,
            scroll_offset: 0,
            selected_index: None,
        };

        // Small area to test clamping
        let area = Rect::new(0, 0, 80, 15);
        let mut buf = Buffer::empty(area);
        let config = DisplayConfig::default();

        widget.render(area, &mut buf, &config);

        assert_eq!(*buf.area(), area);
    }
}
