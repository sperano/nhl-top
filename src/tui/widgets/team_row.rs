use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;

/// A widget that renders a single team row in the standings table
#[derive(Debug, Clone)]
pub struct TeamRow {
    /// Team name to display
    pub team_name: String,
    /// Games played
    pub games_played: i32,
    /// Wins
    pub wins: i32,
    /// Losses
    pub losses: i32,
    /// Overtime losses (includes ties)
    pub ot_losses: i32,
    /// Points
    pub points: i32,
    /// Whether this row is selected
    pub is_selected: bool,
    /// Left margin in characters
    pub margin: u16,
}

impl TeamRow {
    /// Create from an NHL API Standing
    pub fn from_standing(team: &nhl_api::Standing, is_selected: bool, margin: u16) -> Self {
        Self {
            team_name: team.team_common_name.default.clone(),
            games_played: team.games_played(),
            wins: team.wins,
            losses: team.losses,
            ot_losses: team.ot_losses,
            points: team.points,
            is_selected,
            margin,
        }
    }
}

impl RenderableWidget for TeamRow {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        const TEAM_NAME_COL_WIDTH: usize = 13;
        const STATS_COL_WIDTH: usize = 3;

        let mut x = area.x + self.margin;
        let y = area.y;

        let style = if self.is_selected {
            Style::default()
                .fg(config.selection_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Render team name (left aligned, 13 chars)
        let team_text = format!("{:<width$}", self.team_name, width = TEAM_NAME_COL_WIDTH);
        buf.set_string(x, y, &team_text, style);
        x += TEAM_NAME_COL_WIDTH as u16;

        // Render stats with 1 space between team name and stats
        buf.set_string(x, y, " ", style);
        x += 1;

        // GP (3 chars right-aligned + 1 space)
        let gp_text = format!("{:>width$}", self.games_played, width = STATS_COL_WIDTH);
        buf.set_string(x, y, &gp_text, style);
        x += STATS_COL_WIDTH as u16 + 1;

        // W (3 chars right-aligned + 1 space)
        let w_text = format!("{:>width$}", self.wins, width = STATS_COL_WIDTH);
        buf.set_string(x, y, &w_text, style);
        x += STATS_COL_WIDTH as u16 + 1;

        // L (3 chars right-aligned + 1 space)
        let l_text = format!("{:>width$}", self.losses, width = STATS_COL_WIDTH);
        buf.set_string(x, y, &l_text, style);
        x += STATS_COL_WIDTH as u16 + 1;

        // OT (3 chars right-aligned + 1 space)
        let ot_text = format!("{:>width$}", self.ot_losses, width = STATS_COL_WIDTH);
        buf.set_string(x, y, &ot_text, style);
        x += STATS_COL_WIDTH as u16 + 1;

        // PTS (3 chars right-aligned, no trailing space)
        let pts_text = format!("{:>width$}", self.points, width = STATS_COL_WIDTH);
        buf.set_string(x, y, &pts_text, style);
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(1)
    }

    fn preferred_width(&self) -> Option<u16> {
        // margin + team(13) + space(1) + 5 stats (3 chars + 1 space each)
        Some(self.margin + 13 + 1 + (4 * 5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::testing::*;

    #[test]
    fn test_team_row_basic() {
        let row = TeamRow {
            team_name: "Maple Leafs".to_string(),
            games_played: 44,
            wins: 28,
            losses: 16,
            ot_losses: 0,
            points: 56,
            is_selected: false,
            margin: 2,
        };

        let buf = render_widget(&row, 40, 1);
        let line = buffer_line(&buf, 0);

        // Check formatting
        assert!(line.contains("Maple Leafs"));
        assert!(line.contains(" 44"));
        assert!(line.contains(" 28"));
        assert!(line.contains(" 16"));
        assert!(line.contains("  0"));
        assert!(line.contains(" 56"));
    }

    #[test]
    fn test_team_row_selection() {
        let config = test_config();
        let row = TeamRow {
            team_name: "Oilers".to_string(),
            games_played: 43,
            wins: 29,
            losses: 13,
            ot_losses: 1,
            points: 59,
            is_selected: true,
            margin: 0,
        };

        let buf = render_widget_with_config(&row, 40, 1, &config);

        // Check that selection color is applied
        let cell = &buf[(0, 0)];
        assert_eq!(cell.fg, config.selection_fg);
        assert!(cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_team_row_from_standing() {
        let standing = nhl_api::Standing {
            conference_abbrev: Some("E".to_string()),
            conference_name: Some("Eastern".to_string()),
            division_abbrev: "ATL".to_string(),
            division_name: "Atlantic".to_string(),
            team_name: nhl_api::LocalizedString {
                default: "Boston Bruins".to_string(),
            },
            team_common_name: nhl_api::LocalizedString {
                default: "Bruins".to_string(),
            },
            team_abbrev: nhl_api::LocalizedString {
                default: "BOS".to_string(),
            },
            team_logo: "https://example.com/logo.png".to_string(),
            wins: 27,
            losses: 15,
            ot_losses: 3,
            points: 57,
        };

        let row = TeamRow::from_standing(&standing, false, 4);

        assert_eq!(row.team_name, "Bruins");
        assert_eq!(row.games_played, 45); // 27 + 15 + 3
        assert_eq!(row.wins, 27);
        assert_eq!(row.losses, 15);
        assert_eq!(row.ot_losses, 3);
        assert_eq!(row.points, 57);
        assert_eq!(row.margin, 4);
    }

    #[test]
    fn test_team_row_long_name_truncation() {
        let row = TeamRow {
            team_name: "Really Long Team Name".to_string(),
            games_played: 10,
            wins: 5,
            losses: 5,
            ot_losses: 0,
            points: 10,
            is_selected: false,
            margin: 0,
        };

        let buf = render_widget(&row, 40, 1);
        let line = buffer_line(&buf, 0);

        // Name should be truncated to fit in 13 chars
        assert!(line.starts_with("Really Long T"));
    }

    #[test]
    fn test_preferred_dimensions() {
        let row = TeamRow {
            team_name: "Test".to_string(),
            games_played: 0,
            wins: 0,
            losses: 0,
            ot_losses: 0,
            points: 0,
            is_selected: false,
            margin: 2,
        };

        assert_eq!(row.preferred_height(), Some(1));
        assert_eq!(row.preferred_width(), Some(2 + 13 + 1 + 20)); // margin + team + space + stats
    }
}
