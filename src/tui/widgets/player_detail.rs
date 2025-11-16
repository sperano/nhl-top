/// PlayerDetail widget - displays player biography and career statistics
///
/// This widget shows:
/// - Player bio card (name, position, birth info, etc.)
/// - Career statistics table with season-by-season breakdown

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::{RenderableWidget, PlayerBioCard};

/// Widget for displaying player detail
pub struct PlayerDetail<'a> {
    pub player: &'a nhl_api::PlayerLanding,
    pub player_name: &'a str,
    pub nhl_seasons: Vec<nhl_api::SeasonTotal>,
    pub selected_season_index: Option<usize>,
    pub show_instructions: bool,
}

impl<'a> PlayerDetail<'a> {
    pub fn new(
        player: &'a nhl_api::PlayerLanding,
        player_name: &'a str,
    ) -> Self {
        // Filter to only NHL seasons
        let nhl_seasons = if let Some(season_totals) = &player.season_totals {
            season_totals
                .iter()
                .filter(|s| s.league_abbrev == "NHL")
                .cloned()
                .collect()
        } else {
            vec![]
        };

        Self {
            player,
            player_name,
            nhl_seasons,
            selected_season_index: None,
            show_instructions: true,
        }
    }

    pub fn with_selection(mut self, selected_index: Option<usize>) -> Self {
        self.selected_season_index = selected_index;
        self
    }

    pub fn with_instructions(mut self, show: bool) -> Self {
        self.show_instructions = show;
        self
    }
}

impl<'a> RenderableWidget for PlayerDetail<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;

        // Render bio card
        let bio_card = PlayerBioCard::new(self.player, Some(self.player_name), 0);
        let bio_height = bio_card.preferred_height().unwrap_or(10);

        if y < area.bottom() {
            let widget_area = Rect::new(
                area.x,
                y,
                area.width.min(80),
                bio_height.min(area.bottom().saturating_sub(y)),
            );
            bio_card.render(widget_area, buf, config);
        }
        y += bio_height;


        // Render instructions if enabled
        if self.show_instructions {
            let instruction_lines = if self.selected_season_index.is_some() {
                vec![
                    "",
                    "Press Up/Down to navigate seasons, Enter to view team",
                    "Press ESC to go back",
                ]
            } else {
                vec![
                    "",
                    "Press Down to select seasons, Enter to view team",
                    "Press ESC to go back",
                ]
            };

            for line in instruction_lines {
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x, y, line, Style::default());
                y += 1;
            }
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        let mut height = 0;

        // Bio card height
        let bio_card = PlayerBioCard::new(self.player, Some(self.player_name), 0);
        height += bio_card.preferred_height().unwrap_or(10);

        // Career table height

        // Instructions height
        if self.show_instructions {
            height += 3;
        }

        Some(height)
    }
}
