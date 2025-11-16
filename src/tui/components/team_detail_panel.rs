use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    style::{Style, Modifier},
};

use nhl_api::{ClubStats, Standing};

use crate::config::DisplayConfig;
use crate::tui::framework::{
    component::{Component, Element, RenderableWidget},
    Alignment, CellValue, ColumnDef,
};
use super::table::TableWidget;

/// Props for TeamDetailPanel component
#[derive(Clone)]
pub struct TeamDetailPanelProps {
    pub team_abbrev: String,
    pub standing: Option<Standing>,
    pub club_stats: Option<ClubStats>,
    pub loading: bool,
    pub scroll_offset: usize,
    pub selected_index: Option<usize>,
}

/// TeamDetailPanel component - renders team info and season player stats
pub struct TeamDetailPanel;

impl Component for TeamDetailPanel {
    type Props = TeamDetailPanelProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::Widget(Box::new(TeamDetailPanelWidget {
            team_abbrev: props.team_abbrev.clone(),
            standing: props.standing.clone(),
            club_stats: props.club_stats.clone(),
            loading: props.loading,
            scroll_offset: props.scroll_offset,
            selected_index: props.selected_index,
        }))
    }
}

/// Widget for rendering the team detail panel
struct TeamDetailPanelWidget {
    team_abbrev: String,
    standing: Option<Standing>,
    club_stats: Option<ClubStats>,
    loading: bool,
    scroll_offset: usize,
    selected_index: Option<usize>,
}

impl RenderableWidget for TeamDetailPanelWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        if self.loading {
            let text = format!("Loading {} team details...", self.team_abbrev);
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Team Detail"));
            ratatui::widgets::Widget::render(widget, area, buf);
            return;
        }

        let Some(ref stats) = self.club_stats else {
            let text = format!("No stats available for {}", self.team_abbrev);
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Team Detail"));
            ratatui::widgets::Widget::render(widget, area, buf);
            return;
        };

        let mut y = area.y + 1; // Leave space for border
        let x = area.x + 2; // Left margin inside border

        // Render team info header
        if let Some(ref standing) = self.standing {
            let team_name = &standing.team_name.default;
            let common_name = &standing.team_common_name.default;

            let header = format!("{} {}", team_name, common_name);
            buf.set_string(x, y, &header, Style::default().add_modifier(Modifier::BOLD));
            y += 2;

            let record = format!(
                "Record: {}-{}-{} ({} pts) | Division: {} | Conference: {}",
                standing.wins,
                standing.losses,
                standing.ot_losses,
                standing.points,
                standing.division_name,
                standing.conference_name.as_deref().unwrap_or("Unknown")
            );
            buf.set_string(x, y, &record, Style::default());
            y += 2;
        }

        // Sort skaters by points descending
        let mut sorted_skaters = stats.skaters.clone();
        sorted_skaters.sort_by(|a, b| b.points.cmp(&a.points));

        // Sort goalies by games played descending
        let mut sorted_goalies = stats.goalies.clone();
        sorted_goalies.sort_by(|a, b| b.games_played.cmp(&a.games_played));

        let total_skaters = sorted_skaters.len();
        let total_players = total_skaters + sorted_goalies.len();

        // Calculate visible window based on scroll_offset
        // Available height for content (subtract borders, header, etc.)
        let available_height = area.height.saturating_sub(10) as usize; // Account for border, team info, table headers
        let visible_end = (self.scroll_offset + available_height).min(total_players);

        // Determine which table(s) and rows to show
        let show_skaters_from = self.scroll_offset.min(total_skaters);
        let show_skaters_to = visible_end.min(total_skaters);
        let show_goalies_from = self.scroll_offset.saturating_sub(total_skaters);
        let show_goalies_to = visible_end.saturating_sub(total_skaters).min(sorted_goalies.len());

        // Create skaters table (windowed)
        let skater_columns = vec![
            ColumnDef::new("Player", 20, Alignment::Left, |s: &nhl_api::ClubSkaterStats| {
                CellValue::PlayerLink {
                    display: format!("{} {}", s.first_name.default, s.last_name.default),
                    player_id: s.player_id,
                }
            }),
            ColumnDef::new("Pos", 3, Alignment::Left, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.position_code.clone())
            }),
            ColumnDef::new("GP", 4, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.games_played.to_string())
            }),
            ColumnDef::new("G", 3, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.goals.to_string())
            }),
            ColumnDef::new("A", 3, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.assists.to_string())
            }),
            ColumnDef::new("PTS", 4, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.points.to_string())
            }),
            ColumnDef::new("+/-", 4, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(format!("{:+}", s.plus_minus))
            }),
            ColumnDef::new("PIM", 4, Alignment::Right, |s: &nhl_api::ClubSkaterStats| {
                CellValue::Text(s.penalty_minutes.to_string())
            }),
        ];

        // Only render skaters table if any skaters are visible in window
        let skaters_visible = show_skaters_to > show_skaters_from;
        let windowed_skaters: Vec<_> = if skaters_visible {
            sorted_skaters[show_skaters_from..show_skaters_to].to_vec()
        } else {
            vec![]
        };

        // Determine which row is selected in skaters table (if any)
        // Adjust selection index to account for windowing
        let skater_selected_row = self.selected_index
            .filter(|&idx| idx >= show_skaters_from && idx < show_skaters_to)
            .map(|idx| idx - show_skaters_from);

        let skaters_table = TableWidget::from_data(skater_columns, windowed_skaters)
            .with_selection_opt(skater_selected_row, Some(0))
            .with_focused(true)
            .with_header(format!("SKATERS ({}) - Regular Season", stats.skaters.len()))
            .with_margin(2);

        // Render skaters table if visible
        if skaters_visible {
            let skaters_height = skaters_table.preferred_height().unwrap_or(0);
            let skaters_area = Rect::new(x, y, area.width.saturating_sub(4), skaters_height);
            skaters_table.render(skaters_area, buf, config);
            y += skaters_height + 1;
        }

        // Create goalies table
        let goalie_columns = vec![
            ColumnDef::new("Player", 20, Alignment::Left, |g: &nhl_api::ClubGoalieStats| {
                CellValue::PlayerLink {
                    display: format!("{} {}", g.first_name.default, g.last_name.default),
                    player_id: g.player_id,
                }
            }),
            ColumnDef::new("GP", 4, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(g.games_played.to_string())
            }),
            ColumnDef::new("W", 3, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(g.wins.to_string())
            }),
            ColumnDef::new("L", 3, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(g.losses.to_string())
            }),
            ColumnDef::new("OTL", 3, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(g.overtime_losses.to_string())
            }),
            ColumnDef::new("GAA", 5, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(format!("{:.2}", g.goals_against_average))
            }),
            ColumnDef::new("SV%", 5, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(format!("{:.3}", g.save_percentage))
            }),
            ColumnDef::new("SO", 3, Alignment::Right, |g: &nhl_api::ClubGoalieStats| {
                CellValue::Text(g.shutouts.to_string())
            }),
        ];

        // Only render goalies table if any goalies are visible in window
        let goalies_visible = show_goalies_to > show_goalies_from;
        let windowed_goalies: Vec<_> = if goalies_visible {
            sorted_goalies[show_goalies_from..show_goalies_to].to_vec()
        } else {
            vec![]
        };

        // Determine which row is selected in goalies table (if any)
        // Adjust selection index to account for windowing
        let goalie_selected_row = self.selected_index
            .and_then(|idx| idx.checked_sub(total_skaters))
            .filter(|&idx| idx >= show_goalies_from && idx < show_goalies_to)
            .map(|idx| idx - show_goalies_from);

        let goalies_table = TableWidget::from_data(goalie_columns, windowed_goalies)
            .with_selection_opt(goalie_selected_row, Some(0))
            .with_focused(true)
            .with_header(format!("GOALIES ({}) - Regular Season", stats.goalies.len()))
            .with_margin(2);

        // Render goalies table if visible
        if goalies_visible {
            let goalies_height = goalies_table.preferred_height().unwrap_or(0);
            let goalies_area = Rect::new(x, y, area.width.saturating_sub(4), goalies_height);
            goalies_table.render(goalies_area, buf, config);
        }

        // Render border and title
        let title = format!(
            "{} - ↑↓: Navigate | Enter: View Player | ESC: Back",
            self.team_abbrev
        );
        let block = Block::default().borders(Borders::ALL).title(title);
        ratatui::widgets::Widget::render(block, area, buf);
    }

    fn clone_box(&self) -> Box<dyn RenderableWidget> {
        Box::new(TeamDetailPanelWidget {
            team_abbrev: self.team_abbrev.clone(),
            standing: self.standing.clone(),
            club_stats: self.club_stats.clone(),
            loading: self.loading,
            scroll_offset: self.scroll_offset,
            selected_index: self.selected_index,
        })
    }
}
