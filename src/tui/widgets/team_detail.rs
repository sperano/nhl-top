/// TeamDetail widget - displays team information and roster
///
/// This widget shows:
/// - Team header information (name, conference, division, record)
/// - Roster statistics table with player selection

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::{RenderableWidget, RosterStatsTable};
use std::collections::HashMap;

/// Widget for displaying team detail
pub struct TeamDetail<'a> {
    pub team_name: &'a str,
    pub team_abbrev: &'a str,
    pub conference: &'a str,
    pub division: &'a str,
    pub wins: i32,
    pub losses: i32,
    pub ot_losses: i32,
    pub points: i32,
    pub club_stats: &'a HashMap<String, nhl_api::ClubStats>,
    pub selected_player_index: Option<usize>,
    pub show_instructions: bool,
}

impl<'a> TeamDetail<'a> {
    pub fn new(
        team_name: &'a str,
        team_abbrev: &'a str,
        conference: &'a str,
        division: &'a str,
        wins: i32,
        losses: i32,
        ot_losses: i32,
        points: i32,
        club_stats: &'a HashMap<String, nhl_api::ClubStats>,
    ) -> Self {
        Self {
            team_name,
            team_abbrev,
            conference,
            division,
            wins,
            losses,
            ot_losses,
            points,
            club_stats,
            selected_player_index: None,
            show_instructions: true,
        }
    }

    pub fn with_selection(mut self, selected_index: Option<usize>) -> Self {
        self.selected_player_index = selected_index;
        self
    }

    pub fn with_instructions(mut self, show: bool) -> Self {
        self.show_instructions = show;
        self
    }
}

impl<'a> RenderableWidget for TeamDetail<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;

        // Render header information
        let games_played = self.wins + self.losses + self.ot_losses;

        let header_lines = vec![
            format!("Team: {} ({})", self.team_name, self.team_abbrev),
            String::new(),
            format!("Conference: {}", self.conference),
            format!("Division: {}", self.division),
            String::new(),
            format!("Games Played: {}", games_played),
            format!("Wins: {}", self.wins),
            format!("Losses: {}", self.losses),
            format!("OT Losses: {}", self.ot_losses),
            format!("Points: {}", self.points),
            String::new(),
            String::new(),
        ];

        for line in header_lines {
            if y >= area.bottom() {
                break;
            }
            buf.set_string(area.x, y, &line, Style::default());
            y += 1;
        }

        // Render roster table if available
        if let Some(stats) = self.club_stats.get(self.team_abbrev) {
            if !stats.skaters.is_empty() {
                // Convert skaters to PlayerStat format
                use crate::tui::common::panels::PlayerStat;
                let players: Vec<PlayerStat> = stats.skaters.iter().map(|skater| {
                    PlayerStat {
                        name: format!("{} {}", skater.first_name.default, skater.last_name.default),
                        gp: skater.games_played,
                        g: skater.goals,
                        a: skater.assists,
                        pts: skater.points,
                    }
                }).collect();

                let roster_table = RosterStatsTable::new(&players, Some("Team Roster"), self.selected_player_index, 0);
                let roster_height = roster_table.preferred_height().unwrap_or(20);

                if y < area.bottom() {
                    let widget_area = Rect::new(
                        area.x,
                        y,
                        area.width.min(60),
                        roster_height.min(area.bottom().saturating_sub(y)),
                    );
                    roster_table.render(widget_area, buf, config);
                }
                y += roster_height;
            } else {
                if y < area.bottom() {
                    buf.set_string(area.x, y, "  No player data available", Style::default());
                }
                y += 1;
            }
        } else {
            if y < area.bottom() {
                buf.set_string(area.x, y, "  Loading players...", Style::default());
            }
            y += 1;
        }

        // Render instructions if enabled
        if self.show_instructions {
            let has_players = self.club_stats.get(self.team_abbrev)
                .map(|s| !s.skaters.is_empty())
                .unwrap_or(false);

            let footer_lines = if has_players {
                vec![
                    String::new(),
                    "Press Down to select players, Enter to view details".to_string(),
                    "Press ESC to go back".to_string(),
                ]
            } else {
                vec![String::new(), "Press ESC to go back".to_string()]
            };

            for line in footer_lines {
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x, y, &line, Style::default());
                y += 1;
            }
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        let mut height = 12; // Header lines

        // Add roster height
        if let Some(stats) = self.club_stats.get(self.team_abbrev) {
            if !stats.skaters.is_empty() {
                let roster_table = RosterStatsTable::new(&[], Some("Team Roster"), None, 0);
                height += roster_table.preferred_height().unwrap_or(20);
                height += stats.skaters.len() as u16;
            } else {
                height += 1;
            }
        } else {
            height += 1;
        }

        // Add footer
        if self.show_instructions {
            height += 3;
        }

        Some(height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;
    use std::collections::HashMap;

    #[test]
    fn test_team_detail_loading_players() {
        let club_stats = HashMap::new();

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        );

        let buf = render_widget(&widget, RENDER_WIDTH, 16);
        assert_buffer(&buf, &[
            "Team: Toronto Maple Leafs (TOR)",
            "",
            "Conference: Eastern",
            "Division: Atlantic",
            "",
            "Games Played: 50",
            "Wins: 30",
            "Losses: 15",
            "OT Losses: 5",
            "Points: 65",
            "",
            "",
            "  Loading players...",
            "",
            "Press ESC to go back",
            "",
        ]);
    }

    #[test]
    fn test_team_detail_empty_roster() {
        let mut club_stats = HashMap::new();
        club_stats.insert(
            "TOR".to_string(),
            nhl_api::ClubStats {
                season: "20232024".to_string(),
                game_type: 2,
                skaters: vec![],
                goalies: vec![],
            },
        );

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        );

        let buf = render_widget(&widget, RENDER_WIDTH, 16);
        assert_buffer(&buf, &[
            "Team: Toronto Maple Leafs (TOR)",
            "",
            "Conference: Eastern",
            "Division: Atlantic",
            "",
            "Games Played: 50",
            "Wins: 30",
            "Losses: 15",
            "OT Losses: 5",
            "Points: 65",
            "",
            "",
            "  No player data available",
            "",
            "Press ESC to go back",
            "",
        ]);
    }


    #[test]
    fn test_team_detail_without_instructions() {
        let club_stats = HashMap::new();

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        ).with_instructions(false);

        let buf = render_widget(&widget, RENDER_WIDTH, 13);
        assert_buffer(&buf, &[
            "Team: Toronto Maple Leafs (TOR)",
            "",
            "Conference: Eastern",
            "Division: Atlantic",
            "",
            "Games Played: 50",
            "Wins: 30",
            "Losses: 15",
            "OT Losses: 5",
            "Points: 65",
            "",
            "",
            "  Loading players...",
        ]);
    }

    #[test]
    fn test_team_detail_preferred_height_no_players() {
        let club_stats = HashMap::new();

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        );

        let height = widget.preferred_height();
        assert!(height.is_some());
        // 12 (header) + 1 (loading message) + 3 (instructions)
        assert_eq!(height.unwrap(), 16);
    }

    #[test]
    fn test_team_detail_preferred_height_no_instructions() {
        let club_stats = HashMap::new();

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        ).with_instructions(false);

        let height = widget.preferred_height();
        assert!(height.is_some());
        // 12 (header) + 1 (loading message) + 0 (no instructions)
        assert_eq!(height.unwrap(), 13);
    }

    #[test]
    fn test_team_detail_builder_pattern() {
        let club_stats = HashMap::new();

        let widget = TeamDetail::new(
            "Toronto Maple Leafs",
            "TOR",
            "Eastern",
            "Atlantic",
            30,
            15,
            5,
            65,
            &club_stats,
        )
        .with_selection(Some(1))
        .with_instructions(false);

        assert_eq!(widget.selected_player_index, Some(1));
        assert!(!widget.show_instructions);
    }
}
