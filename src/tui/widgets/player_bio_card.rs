/// PlayerBioCard widget - displays player biographical information
///
/// This widget renders a card showing player biographical details:
/// - Position
/// - Jersey number (optional)
/// - Height (feet and inches)
/// - Weight (pounds)
/// - Birthplace (city, state/province, country)

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use crate::config::DisplayConfig;
use crate::tui::widgets::RenderableWidget;
use crate::tui::widgets::section_header::render_section_header;

/// Widget for displaying player biographical information
#[derive(Debug)]
pub struct PlayerBioCard<'a> {
    /// Player data
    pub player: &'a nhl_api::PlayerLanding,
    /// Optional header text (e.g., "Player Information")
    pub header: Option<&'a str>,
    /// Left margin for indentation
    pub margin: u16,
}

impl<'a> PlayerBioCard<'a> {
    /// Create a new PlayerBioCard widget
    pub fn new(
        player: &'a nhl_api::PlayerLanding,
        header: Option<&'a str>,
        margin: u16,
    ) -> Self {
        Self {
            player,
            header,
            margin,
        }
    }

    /// Calculate the total height needed for this card
    fn calculate_height(&self) -> u16 {
        let mut height = 0;
        if self.header.is_some() {
            height += 2;
        }
        height += 1; // position
        if self.player.sweater_number.is_some() {
            height += 1;
        }
        height += 1; // height
        height += 1; // weight
        if self.player.birth_city.is_some()
            || self.player.birth_state_province.is_some()
            || self.player.birth_country.is_some() {
            height += 1;
        }
        height += 2;
        height
    }

    /// Format birthplace from city, state/province, and country
    fn format_birthplace(&self) -> Option<String> {
        let mut birthplace = String::new();

        if let Some(city) = &self.player.birth_city {
            birthplace.push_str(&city.default);
        }

        if let Some(state_prov) = &self.player.birth_state_province {
            if !birthplace.is_empty() {
                birthplace.push_str(", ");
            }
            birthplace.push_str(&state_prov.default);
        }

        if let Some(country) = &self.player.birth_country {
            if !birthplace.is_empty() {
                birthplace.push_str(", ");
            }
            birthplace.push_str(country);
        }

        if birthplace.is_empty() {
            None
        } else {
            Some(birthplace)
        }
    }
}

impl<'a> RenderableWidget for PlayerBioCard<'a> {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig) {
        let mut y = area.y;
        let margin = self.margin;

        // Render header if present
        if let Some(header_text) = &self.header {
            y += render_section_header(header_text, false, margin, area, y, buf, config);
        }

        // Position
        if y < area.bottom() {
            let line = format!("{}  Position:      {}", " ".repeat(margin as usize), self.player.position);
            buf.set_string(area.x, y, &line, Style::default());
            y += 1;
        }

        // Number (if present)
        if let Some(num) = self.player.sweater_number {
            if y < area.bottom() {
                let line = format!("{}  Number:        #{}", " ".repeat(margin as usize), num);
                buf.set_string(area.x, y, &line, Style::default());
                y += 1;
            }
        }

        // Height
        if y < area.bottom() {
            let height_ft = self.player.height_in_inches / 12;
            let height_in = self.player.height_in_inches % 12;
            let line = format!("{}  Height:        {}'{}\"", " ".repeat(margin as usize), height_ft, height_in);
            buf.set_string(area.x, y, &line, Style::default());
            y += 1;
        }

        // Weight
        if y < area.bottom() {
            let line = format!("{}  Weight:        {} lbs", " ".repeat(margin as usize), self.player.weight_in_pounds);
            buf.set_string(area.x, y, &line, Style::default());
            y += 1;
        }

        // Birthplace (if any data exists)
        if let Some(birthplace) = self.format_birthplace() {
            if y < area.bottom() {
                let line = format!("{}  Birthplace:    {}", " ".repeat(margin as usize), birthplace);
                buf.set_string(area.x, y, &line, Style::default());
                y += 1;
            }
        }

        // Two blank lines
        if y < area.bottom() {
            buf.set_string(area.x, y, "", Style::default());
            y += 1;
        }
        if y < area.bottom() {
            buf.set_string(area.x, y, "", Style::default());
        }
    }

    fn preferred_height(&self) -> Option<u16> {
        Some(self.calculate_height())
    }

    fn preferred_width(&self) -> Option<u16> {
        // Dynamic based on content, but return None to indicate flexible width
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::{assert_buffer, RENDER_WIDTH};
    use crate::tui::widgets::testing::*;
    use nhl_api::LocalizedString;

    fn create_test_player(
        position: &str,
        number: Option<i32>,
        height_inches: i32,
        weight: i32,
        city: Option<&str>,
        state_prov: Option<&str>,
        country: Option<&str>,
    ) -> nhl_api::PlayerLanding {
        nhl_api::PlayerLanding {
            player_id: 1,
            is_active: true,
            current_team_id: None,
            current_team_abbrev: None,
            headshot: "".to_string(),
            hero_image: None,
            first_name: LocalizedString { default: "Test".to_string() },
            last_name: LocalizedString { default: "Player".to_string() },
            sweater_number: number,
            position: position.to_string(),
            height_in_inches: height_inches,
            weight_in_pounds: weight,
            birth_date: "1990-01-01".to_string(),
            birth_city: city.map(|c| LocalizedString { default: c.to_string() }),
            birth_country: country.map(|c| c.to_string()),
            birth_state_province: state_prov.map(|s| LocalizedString { default: s.to_string() }),
            shoots_catches: "R".to_string(),
            draft_details: None,
            player_slug: None,
            featured_stats: None,
            career_totals: None,
            season_totals: None,
            awards: None,
            last_five_games: None,
        }
    }

    #[test]
    fn test_player_bio_card_basic() {
        let player = create_test_player("C", Some(34), 75, 200, Some("Toronto"), Some("ON"), Some("CAN"));
        let widget = PlayerBioCard::new(&player, None, 0);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "  Position:      C",
            "  Number:        #34",
            "  Height:        6'3\"",
            "  Weight:        200 lbs",
            "  Birthplace:    Toronto, ON, CAN",
            "",
            "",
        ]);
    }

    #[test]
    fn test_player_bio_card_with_header() {
        let player = create_test_player("D", Some(2), 72, 190, Some("Boston"), Some("MA"), Some("USA"));
        let widget = PlayerBioCard::new(&player, Some("Player Information"), 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "  Player Information",
            "  ──────────────────",
            "    Position:      D",
            "    Number:        #2",
            "    Height:        6'0\"",
            "    Weight:        190 lbs",
            "    Birthplace:    Boston, MA, USA",
            "",
            "",
        ]);
    }

    #[test]
    fn test_player_bio_card_without_number() {
        let player = create_test_player("G", None, 73, 185, Some("Montreal"), Some("QC"), Some("CAN"));
        let widget = PlayerBioCard::new(&player, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "    Position:      G",
            "    Height:        6'1\"",
            "    Weight:        185 lbs",
            "    Birthplace:    Montreal, QC, CAN",
            "",
            "",
        ]);
    }

    #[test]
    fn test_player_bio_card_partial_birthplace() {
        // Only city and country
        let player = create_test_player("LW", Some(12), 70, 175, Some("Stockholm"), None, Some("SWE"));
        let widget = PlayerBioCard::new(&player, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "    Position:      LW",
            "    Number:        #12",
            "    Height:        5'10\"",
            "    Weight:        175 lbs",
            "    Birthplace:    Stockholm, SWE",
            "",
            "",
        ]);
    }

    #[test]
    fn test_player_bio_card_no_birthplace() {
        let player = create_test_player("RW", Some(9), 74, 210, None, None, None);
        let widget = PlayerBioCard::new(&player, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "    Position:      RW",
            "    Number:        #9",
            "    Height:        6'2\"",
            "    Weight:        210 lbs",
            "",
            "",
        ]);
    }

    #[test]
    fn test_player_bio_card_height_calculation() {
        // With number and birthplace
        let player1 = create_test_player("C", Some(88), 75, 200, Some("City"), None, Some("Country"));
        let widget1 = PlayerBioCard::new(&player1, None, 0);
        // Position(1) + Number(1) + Height(1) + Weight(1) + Birthplace(1) + Blanks(2) = 7
        assert_eq!(widget1.preferred_height(), Some(7));

        // Without number, without birthplace
        let player2 = create_test_player("D", None, 72, 190, None, None, None);
        let widget2 = PlayerBioCard::new(&player2, None, 0);
        // Position(1) + Height(1) + Weight(1) + Blanks(2) = 5
        assert_eq!(widget2.preferred_height(), Some(5));

        // With header, number, and birthplace
        let player3 = create_test_player("G", Some(1), 76, 220, Some("City"), Some("State"), Some("Country"));
        let widget3 = PlayerBioCard::new(&player3, Some("Player Info"), 0);
        // Header(2) + Position(1) + Number(1) + Height(1) + Weight(1) + Birthplace(1) + Blanks(2) = 9
        assert_eq!(widget3.preferred_height(), Some(9));
    }

    #[test]
    fn test_player_bio_card_height_formatting() {
        // 75 inches = 6 feet 3 inches
        let player = create_test_player("C", None, 75, 200, None, None, None);
        let widget = PlayerBioCard::new(&player, None, 2);
        let config = test_config();
        let height = widget.preferred_height().unwrap();
        let buf = render_widget_with_config(&widget, RENDER_WIDTH, height, &config);

        assert_buffer(&buf, &[
            "    Position:      C",
            "    Height:        6'3\"",
            "    Weight:        200 lbs",
            "",
            "",
        ]);
    }
}
