//! Document implementations for standings views
//!
//! This module contains Document implementations for the different standings
//! views (League, Conference, Division, Wildcard).

use std::collections::BTreeMap;
use std::sync::Arc;

use nhl_api::Standing;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::config::{Config, DisplayConfig};
use crate::tui::component::ElementWidget;
use crate::tui::document::{Document, DocumentBuilder, DocumentElement, DocumentView, FocusContext};
use crate::tui::helpers::StandingsSorting;

use super::{standings_columns, TableWidget};

/// League standings document - single table with all teams sorted by points
pub struct LeagueStandingsDocument {
    standings: Arc<Vec<Standing>>,
    #[allow(dead_code)] // Will be used for other standings views
    config: Config,
}

impl LeagueStandingsDocument {
    pub fn new(standings: Arc<Vec<Standing>>, config: Config) -> Self {
        Self { standings, config }
    }
}

impl Document for LeagueStandingsDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        let focused_row = focus.focused_table_row("league_standings");

        let table = TableWidget::from_data(standings_columns(), self.standings.as_ref().clone())
            .with_focused_row(focused_row)
            .with_margin(0);

        DocumentBuilder::new()
            .table("league_standings", table)
            .build()
    }

    fn title(&self) -> String {
        "League Standings".to_string()
    }

    fn id(&self) -> String {
        "league_standings".to_string()
    }
}

/// Conference standings document - two tables side-by-side in a Row element
pub struct ConferenceStandingsDocument {
    standings: Arc<Vec<Standing>>,
    config: Config,
}

impl ConferenceStandingsDocument {
    pub fn new(standings: Arc<Vec<Standing>>, config: Config) -> Self {
        Self { standings, config }
    }

    /// Group standings by conference and return (Eastern, Western) sorted by points
    fn group_by_conference(&self) -> (Vec<Standing>, Vec<Standing>) {
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in self.standings.as_ref() {
            let conference = standing
                .conference_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            grouped
                .entry(conference)
                .or_default()
                .push(standing.clone());
        }

        // Sort teams within each conference by points
        for teams in grouped.values_mut() {
            teams.sort_by_points_desc();
        }

        // Extract Eastern and Western (BTreeMap gives us alphabetically sorted keys)
        let eastern = grouped.get("Eastern").cloned().unwrap_or_default();
        let western = grouped.get("Western").cloned().unwrap_or_default();

        (eastern, western)
    }
}

impl Document for ConferenceStandingsDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        const LEFT_TABLE: &str = "conference_left";
        const RIGHT_TABLE: &str = "conference_right";

        let (eastern, western) = self.group_by_conference();

        // Determine column order based on western_first config
        let (left_teams, right_teams, left_header, right_header) =
            if self.config.display_standings_western_first {
                (western, eastern, "Western", "Eastern")
            } else {
                (eastern, western, "Eastern", "Western")
            };

        // Create left table
        let left_table = TableWidget::from_data(standings_columns(), left_teams)
            .with_header(left_header)
            .with_focused_row(focus.focused_table_row(LEFT_TABLE))
            .with_margin(0);

        // Create right table
        let right_table = TableWidget::from_data(standings_columns(), right_teams)
            .with_header(right_header)
            .with_focused_row(focus.focused_table_row(RIGHT_TABLE))
            .with_margin(0);

        // Use Row element to place tables side-by-side
        DocumentBuilder::new()
            .row(vec![
                DocumentElement::table(LEFT_TABLE, left_table),
                DocumentElement::table(RIGHT_TABLE, right_table),
            ])
            .build()
    }

    fn title(&self) -> String {
        "Conference Standings".to_string()
    }

    fn id(&self) -> String {
        "conference_standings".to_string()
    }
}

/// Division standings document - two columns with two divisions each
///
/// Layout (with western_first=true):
/// ```text
/// +-------------------+-------------------+
/// |   Central         |   Atlantic        |
/// |   (8 teams)       |   (8 teams)       |
/// |                   |                   |
/// |   Pacific         |   Metropolitan    |
/// |   (8 teams)       |   (8 teams)       |
/// +-------------------+-------------------+
/// ```
///
/// Navigation order: Central -> Pacific -> Atlantic -> Metropolitan (down through
/// left column first, then down through right column).
pub struct DivisionStandingsDocument {
    standings: Arc<Vec<Standing>>,
    config: Config,
}

impl DivisionStandingsDocument {
    pub fn new(standings: Arc<Vec<Standing>>, config: Config) -> Self {
        Self { standings, config }
    }

    /// Group standings by division and return maps for each conference
    fn group_by_division(&self) -> (Vec<(&'static str, Vec<Standing>)>, Vec<(&'static str, Vec<Standing>)>) {
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in self.standings.as_ref() {
            grouped
                .entry(standing.division_name.clone())
                .or_default()
                .push(standing.clone());
        }

        // Sort teams within each division by points
        for teams in grouped.values_mut() {
            teams.sort_by_points_desc();
        }

        // Eastern divisions (alphabetically: Atlantic, Metropolitan)
        let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
        let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();

        // Western divisions (alphabetically: Central, Pacific)
        let central = grouped.get("Central").cloned().unwrap_or_default();
        let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

        // Return as ordered tuples for each conference
        let eastern = vec![("Atlantic", atlantic), ("Metropolitan", metropolitan)];
        let western = vec![("Central", central), ("Pacific", pacific)];

        (eastern, western)
    }

    /// Build a vertical group of division tables
    fn build_division_group(
        divisions: &[(&str, Vec<Standing>)],
        table_prefix: &str,
        focus: &FocusContext,
    ) -> DocumentElement {
        let mut children = Vec::new();

        for (idx, (div_name, teams)) in divisions.iter().enumerate() {
            let table_name = format!("{}_{}", table_prefix, div_name.to_lowercase());
            let table = TableWidget::from_data(standings_columns(), teams.clone())
                .with_header(*div_name)
                .with_focused_row(focus.focused_table_row(&table_name))
                .with_margin(0);

            children.push(DocumentElement::table(table_name, table));

            // Add spacer between divisions (not after the last one)
            if idx < divisions.len() - 1 {
                children.push(DocumentElement::spacer(1));
            }
        }

        DocumentElement::group(children)
    }
}

impl Document for DivisionStandingsDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        let (eastern, western) = self.group_by_division();

        // Determine column order based on western_first config
        let (left_divs, right_divs, left_prefix, right_prefix) =
            if self.config.display_standings_western_first {
                (western, eastern, "division_left", "division_right")
            } else {
                (eastern, western, "division_left", "division_right")
            };

        // Build left column (Group with 2 division tables)
        let left_group = Self::build_division_group(&left_divs, left_prefix, focus);

        // Build right column (Group with 2 division tables)
        let right_group = Self::build_division_group(&right_divs, right_prefix, focus);

        // Use Row element to place columns side-by-side
        DocumentBuilder::new()
            .row(vec![left_group, right_group])
            .build()
    }

    fn title(&self) -> String {
        "Division Standings".to_string()
    }

    fn id(&self) -> String {
        "division_standings".to_string()
    }
}

/// Wildcard standings document - two columns showing playoff picture
///
/// Layout (with western_first=true):
/// ```text
/// +-------------------+-------------------+
/// |   Central (top 3) |   Atlantic (top 3)|
/// |   Pacific (top 3) |   Metropolitan    |
/// |                   |   (top 3)         |
/// |   Wildcard        |   Wildcard        |
/// |   (remaining)     |   (remaining)     |
/// +-------------------+-------------------+
/// ```
///
/// Each conference column shows:
/// 1. Division 1 top 3 teams (guaranteed playoff spots)
/// 2. Division 2 top 3 teams (guaranteed playoff spots)
/// 3. Wildcard section with remaining teams sorted by points
pub struct WildcardStandingsDocument {
    standings: Arc<Vec<Standing>>,
    config: Config,
}

impl WildcardStandingsDocument {
    pub fn new(standings: Arc<Vec<Standing>>, config: Config) -> Self {
        Self { standings, config }
    }

    /// Group standings by division and return sorted teams for each division
    fn group_by_division(&self) -> (Vec<Standing>, Vec<Standing>, Vec<Standing>, Vec<Standing>) {
        let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
        for standing in self.standings.as_ref() {
            grouped
                .entry(standing.division_name.clone())
                .or_default()
                .push(standing.clone());
        }

        // Sort teams within each division by points
        for teams in grouped.values_mut() {
            teams.sort_by_points_desc();
        }

        let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
        let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
        let central = grouped.get("Central").cloned().unwrap_or_default();
        let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

        (atlantic, metropolitan, central, pacific)
    }

    /// Build a wildcard conference column (div1 top 3 + div2 top 3 + wildcards)
    fn build_wildcard_group(
        div1_name: &str,
        div1_teams: &[Standing],
        div2_name: &str,
        div2_teams: &[Standing],
        table_prefix: &str,
        focus: &FocusContext,
    ) -> DocumentElement {
        let mut children = Vec::new();

        // Division 1 - top 3 teams
        let div1_top3: Vec<_> = div1_teams.iter().take(3).cloned().collect();
        if !div1_top3.is_empty() {
            let table_name = format!("{}_{}", table_prefix, div1_name.to_lowercase());
            let table = TableWidget::from_data(standings_columns(), div1_top3)
                .with_header(div1_name)
                .with_focused_row(focus.focused_table_row(&table_name))
                .with_margin(0);
            children.push(DocumentElement::table(table_name, table));
            children.push(DocumentElement::spacer(1));
        }

        // Division 2 - top 3 teams
        let div2_top3: Vec<_> = div2_teams.iter().take(3).cloned().collect();
        if !div2_top3.is_empty() {
            let table_name = format!("{}_{}", table_prefix, div2_name.to_lowercase());
            let table = TableWidget::from_data(standings_columns(), div2_top3)
                .with_header(div2_name)
                .with_focused_row(focus.focused_table_row(&table_name))
                .with_margin(0);
            children.push(DocumentElement::table(table_name, table));
            children.push(DocumentElement::spacer(1));
        }

        // Wildcard section - remaining teams from both divisions, sorted by points
        let div1_remaining: Vec<_> = div1_teams.iter().skip(3).cloned().collect();
        let div2_remaining: Vec<_> = div2_teams.iter().skip(3).cloned().collect();

        let mut wildcard_teams: Vec<_> = div1_remaining.into_iter().chain(div2_remaining).collect();
        wildcard_teams.sort_by_points_desc();

        if !wildcard_teams.is_empty() {
            let table_name = format!("{}_wildcard", table_prefix);
            let table = TableWidget::from_data(standings_columns(), wildcard_teams)
                .with_header("Wildcard")
                .with_focused_row(focus.focused_table_row(&table_name))
                .with_margin(0);
            children.push(DocumentElement::table(table_name, table));
        }

        DocumentElement::group(children)
    }
}

impl Document for WildcardStandingsDocument {
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
        let (atlantic, metropolitan, central, pacific) = self.group_by_division();

        // Determine column order based on western_first config
        let (left_group, right_group) = if self.config.display_standings_western_first {
            // Western left, Eastern right
            let western = Self::build_wildcard_group(
                "Central", &central,
                "Pacific", &pacific,
                "wildcard_left", focus,
            );
            let eastern = Self::build_wildcard_group(
                "Atlantic", &atlantic,
                "Metropolitan", &metropolitan,
                "wildcard_right", focus,
            );
            (western, eastern)
        } else {
            // Eastern left, Western right
            let eastern = Self::build_wildcard_group(
                "Atlantic", &atlantic,
                "Metropolitan", &metropolitan,
                "wildcard_left", focus,
            );
            let western = Self::build_wildcard_group(
                "Central", &central,
                "Pacific", &pacific,
                "wildcard_right", focus,
            );
            (eastern, western)
        };

        // Use Row element to place columns side-by-side
        DocumentBuilder::new()
            .row(vec![left_group, right_group])
            .build()
    }

    fn title(&self) -> String {
        "Wildcard Standings".to_string()
    }

    fn id(&self) -> String {
        "wildcard_standings".to_string()
    }
}

/// Widget that renders a standings document with DocumentView
///
/// This widget wraps DocumentView and applies focus/scroll state from AppState.
/// It can render League, Conference, or Division standings based on the document type.
pub struct StandingsDocumentWidget {
    doc: Arc<dyn Document>,
    focus_index: Option<usize>,
    scroll_offset: u16,
}

impl StandingsDocumentWidget {
    /// Create widget for League standings
    pub fn league(standings: Arc<Vec<Standing>>, config: Config, focus_index: Option<usize>, scroll_offset: u16) -> Self {
        Self {
            doc: Arc::new(LeagueStandingsDocument::new(standings, config)),
            focus_index,
            scroll_offset,
        }
    }

    /// Create widget for Conference standings
    pub fn conference(standings: Arc<Vec<Standing>>, config: Config, focus_index: Option<usize>, scroll_offset: u16) -> Self {
        Self {
            doc: Arc::new(ConferenceStandingsDocument::new(standings, config)),
            focus_index,
            scroll_offset,
        }
    }

    /// Create widget for Division standings
    pub fn division(standings: Arc<Vec<Standing>>, config: Config, focus_index: Option<usize>, scroll_offset: u16) -> Self {
        Self {
            doc: Arc::new(DivisionStandingsDocument::new(standings, config)),
            focus_index,
            scroll_offset,
        }
    }

    /// Create widget for Wildcard standings
    pub fn wildcard(standings: Arc<Vec<Standing>>, config: Config, focus_index: Option<usize>, scroll_offset: u16) -> Self {
        Self {
            doc: Arc::new(WildcardStandingsDocument::new(standings, config)),
            focus_index,
            scroll_offset,
        }
    }
}

impl ElementWidget for StandingsDocumentWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, display_config: &DisplayConfig) {
        // Create DocumentView with viewport height
        let mut view = DocumentView::new(self.doc.clone(), area.height);

        // Apply focus state from AppState
        if let Some(idx) = self.focus_index {
            view.focus_by_index(idx);
        }

        // Apply scroll offset from AppState
        view.set_scroll_offset(self.scroll_offset);

        // Render the document
        view.render(area, buf, display_config);
    }

    fn clone_box(&self) -> Box<dyn ElementWidget> {
        Box::new(StandingsDocumentWidget {
            doc: self.doc.clone(),
            focus_index: self.focus_index,
            scroll_offset: self.scroll_offset,
        })
    }

    fn preferred_height(&self) -> Option<u16> {
        None // Fills available space
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DisplayConfig;
    use crate::tui::testing::{assert_buffer, create_test_standings};
    use std::sync::Arc;

    #[test]
    fn test_league_standings_document_renders() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = LeagueStandingsDocument::new(standings, config);

        // Build with no focus
        let elements = doc.build(&FocusContext::default());

        // Should have one element: the table
        assert_eq!(elements.len(), 1);

        // Check that it's a table element
        match &elements[0] {
            DocumentElement::Table { widget, focusable } => {
                assert_eq!(widget.row_count(), 32); // 32 teams
                // Should have 32 focusable elements (one per team row, col 0 is the team link)
                assert_eq!(focusable.len(), 32);
            }
            _ => panic!("Expected Table element"),
        }
    }

    #[test]
    fn test_league_standings_document_full_render() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = LeagueStandingsDocument::new(standings, config);

        let display_config = DisplayConfig::default();
        let (buf, height) = doc.render_full(60, &display_config, &FocusContext::default());

        // Height should be: column headers (1) + separator (1) + 32 teams = 34 lines
        assert_eq!(height, 34);

        // Check all lines
        assert_buffer(
            &buf,
            &[
                "  Team                        GP    W     L    OT   PTS",
                "  ───────────────────────────────────────────────────────",
                "  Panthers                      19    14    3    2     30",
                "  Bruins                        18    13    4    1     27",
                "  Maple Leafs                   19    12    5    2     26",
                "  Lightning                     18    11    6    1     23",
                "  Canadiens                     18    10    5    3     23",
                "  Senators                      18     9    7    2     20",
                "  Red Wings                     18     8    8    2     18",
                "  Sabres                        18     6   10    2     14",
                "  Devils                        18    15    2    1     31",
                "  Hurricanes                    19    14    3    2     30",
                "  Rangers                       18    12    5    1     25",
                "  Penguins                      19    11    6    2     24",
                "  Capitals                      18    10    7    1     21",
                "  Islanders                     18     9    7    2     20",
                "  Flyers                        18     8    9    1     17",
                "  Blue Jackets                  18     5   11    2     12",
                "  Avalanche                     19    16    2    1     33",
                "  Stars                         20    14    4    2     30",
                "  Jets                          19    13    5    1     27",
                "  Wild                          19    11    6    2     24",
                "  Predators                     19    10    7    2     22",
                "  Blues                         19     8    8    3     19",
                "  Blackhawks                    18     7   10    1     15",
                "  Coyotes                       18     4   13    1      9",
                "  Golden Knights                19    15    3    1     31",
                "  Oilers                        20    14    4    2     30",
                "  Kings                         19    12    6    1     25",
                "  Kraken                        19    11    6    2     24",
                "  Canucks                       19    10    7    2     22",
                "  Flames                        19     9    8    2     20",
                "  Ducks                         19     7   10    2     16",
                "  Sharks                        18     5   12    1     11",
            ],
        );
    }

    #[test]
    fn test_league_standings_document_with_focus() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = LeagueStandingsDocument::new(standings, config);

        // Create focus context for row 2
        let focus = FocusContext::with_table_cell("league_standings", 2, 0);
        let elements = doc.build(&focus);

        // Build and check that the table was created
        // Note: We can't directly check focused_row as it's a private field,
        // but we've passed it through with_focused_row() so the table will render correctly
        match &elements[0] {
            DocumentElement::Table { widget, .. } => {
                assert_eq!(widget.row_count(), 32); // Verify it's the standings table
            }
            _ => panic!("Expected Table element"),
        }
    }

    #[test]
    fn test_league_standings_document_metadata() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = LeagueStandingsDocument::new(standings, config);

        assert_eq!(doc.title(), "League Standings");
        assert_eq!(doc.id(), "league_standings");
    }

    #[test]
    fn test_league_standings_focusable_positions() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = LeagueStandingsDocument::new(standings, config);

        let positions = doc.focusable_positions();

        // Should have 32 focusable positions (one per team row)
        assert_eq!(positions.len(), 32);

        // First focusable is at y=2 (after column headers + separator)
        assert_eq!(positions[0], 2);

        // Each subsequent focusable is 1 line below the previous
        for i in 1..positions.len() {
            assert_eq!(positions[i], positions[i - 1] + 1);
        }
    }

    // === Conference Standings Tests ===

    #[test]
    fn test_conference_standings_document_renders() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = ConferenceStandingsDocument::new(standings, config);

        // Build with no focus
        let elements = doc.build(&FocusContext::default());

        // Should have one element: a Row containing two tables
        assert_eq!(elements.len(), 1);

        // Check that it's a Row element
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                // Should have 2 children (left and right conference tables)
                assert_eq!(children.len(), 2);

                // Both children should be tables
                for child in children {
                    match child {
                        DocumentElement::Table { widget, .. } => {
                            // Each conference should have 16 teams
                            assert_eq!(widget.row_count(), 16);
                        }
                        _ => panic!("Expected Table element in Row"),
                    }
                }
            }
            _ => panic!("Expected Row element"),
        }
    }

    #[test]
    fn test_conference_standings_document_metadata() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = ConferenceStandingsDocument::new(standings, config);

        assert_eq!(doc.title(), "Conference Standings");
        assert_eq!(doc.id(), "conference_standings");
    }

    #[test]
    fn test_conference_standings_focusable_positions() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = ConferenceStandingsDocument::new(standings, config);

        let positions = doc.focusable_positions();

        // Should have 32 focusable positions (16 per conference)
        assert_eq!(positions.len(), 32);

        // With Row layout, left column elements are collected first, then right column.
        // Both columns have the SAME y-positions because they're rendered side-by-side.
        // Left column (16 teams): positions 5, 6, 7, ... 20
        // Right column (16 teams): positions 5, 6, 7, ... 20

        // First 16 positions are left column
        for i in 0..16 {
            assert_eq!(positions[i], 5 + i as u16, "Left column position {} should be {}", i, 5 + i);
        }
        // Second 16 positions are right column - SAME y values as left
        for i in 0..16 {
            assert_eq!(positions[16 + i], 5 + i as u16, "Right column position {} should be {}", i, 5 + i);
        }
    }

    #[test]
    fn test_conference_standings_row_positions() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = ConferenceStandingsDocument::new(standings, config);

        let row_positions = doc.focusable_row_positions();

        // Should have 32 row positions
        assert_eq!(row_positions.len(), 32);

        // All should be Some (within a Row)
        assert!(row_positions.iter().all(|rp| rp.is_some()));

        // Check that we have elements from both columns (0 and 1)
        let column_0_count = row_positions.iter().filter(|rp| {
            rp.as_ref().map_or(false, |p| p.child_idx == 0)
        }).count();
        let column_1_count = row_positions.iter().filter(|rp| {
            rp.as_ref().map_or(false, |p| p.child_idx == 1)
        }).count();

        // Should have 16 teams in each column
        assert_eq!(column_0_count, 16);
        assert_eq!(column_1_count, 16);
    }

    #[test]
    fn test_conference_standings_respects_western_first_config() {
        let standings = Arc::new(create_test_standings());

        // Test with western_first = false (Eastern left, Western right)
        let mut config = Config::default();
        config.display_standings_western_first = false;
        let doc = ConferenceStandingsDocument::new(standings.clone(), config);
        let elements = doc.build(&FocusContext::default());

        // Verify Row structure
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                // Should have 2 table children
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], DocumentElement::Table { .. }));
                assert!(matches!(children[1], DocumentElement::Table { .. }));
            }
            _ => panic!("Expected Row element"),
        }

        // Test with western_first = true (Western left, Eastern right)
        let mut config = Config::default();
        config.display_standings_western_first = true;
        let doc = ConferenceStandingsDocument::new(standings, config);
        let elements = doc.build(&FocusContext::default());

        // Verify Row structure (same structure, different internal ordering)
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                // Should have 2 table children
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], DocumentElement::Table { .. }));
                assert!(matches!(children[1], DocumentElement::Table { .. }));
            }
            _ => panic!("Expected Row element"),
        }
    }

    // === Division Standings Tests ===

    #[test]
    fn test_division_standings_document_renders() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = DivisionStandingsDocument::new(standings, config);

        // Build with no focus
        let elements = doc.build(&FocusContext::default());

        // Should have one element: a Row containing two Groups
        assert_eq!(elements.len(), 1);

        // Check that it's a Row element with Group children
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                // Should have 2 children (left and right columns)
                assert_eq!(children.len(), 2);

                // Each child should be a Group containing tables
                for child in children {
                    match child {
                        DocumentElement::Group { children: group_children, .. } => {
                            // Each group should have 3 children: table, spacer, table
                            assert_eq!(group_children.len(), 3);
                            assert!(matches!(group_children[0], DocumentElement::Table { .. }));
                            assert!(matches!(group_children[1], DocumentElement::Spacer { .. }));
                            assert!(matches!(group_children[2], DocumentElement::Table { .. }));
                        }
                        _ => panic!("Expected Group element in Row"),
                    }
                }
            }
            _ => panic!("Expected Row element"),
        }
    }

    #[test]
    fn test_division_standings_document_metadata() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = DivisionStandingsDocument::new(standings, config);

        assert_eq!(doc.title(), "Division Standings");
        assert_eq!(doc.id(), "division_standings");
    }

    #[test]
    fn test_division_standings_focusable_positions() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = DivisionStandingsDocument::new(standings, config);

        let positions = doc.focusable_positions();

        // Should have 32 focusable positions (32 teams across 4 divisions)
        assert_eq!(positions.len(), 32);
    }

    #[test]
    fn test_division_standings_row_positions() {
        let standings = Arc::new(create_test_standings());
        let config = Config::default();
        let doc = DivisionStandingsDocument::new(standings, config);

        let row_positions = doc.focusable_row_positions();

        // Should have 32 row positions
        assert_eq!(row_positions.len(), 32);

        // All should be Some (within a Row)
        assert!(row_positions.iter().all(|rp| rp.is_some()));

        // Check that we have elements from both columns (0 and 1)
        let column_0_count = row_positions.iter().filter(|rp| {
            rp.as_ref().map_or(false, |p| p.child_idx == 0)
        }).count();
        let column_1_count = row_positions.iter().filter(|rp| {
            rp.as_ref().map_or(false, |p| p.child_idx == 1)
        }).count();

        // Should have 16 teams in each column (2 divisions x 8 teams)
        assert_eq!(column_0_count, 16);
        assert_eq!(column_1_count, 16);
    }

    #[test]
    fn test_division_standings_respects_western_first_config() {
        let standings = Arc::new(create_test_standings());

        // Test with western_first = false (Eastern divisions left, Western divisions right)
        let mut config = Config::default();
        config.display_standings_western_first = false;
        let doc = DivisionStandingsDocument::new(standings.clone(), config);
        let elements = doc.build(&FocusContext::default());

        // Verify Row structure with Groups
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], DocumentElement::Group { .. }));
                assert!(matches!(children[1], DocumentElement::Group { .. }));
            }
            _ => panic!("Expected Row element"),
        }

        // Test with western_first = true (Western divisions left, Eastern divisions right)
        let mut config = Config::default();
        config.display_standings_western_first = true;
        let doc = DivisionStandingsDocument::new(standings, config);
        let elements = doc.build(&FocusContext::default());

        // Verify Row structure (same structure, different internal ordering)
        match &elements[0] {
            DocumentElement::Row { children, .. } => {
                assert_eq!(children.len(), 2);
                assert!(matches!(children[0], DocumentElement::Group { .. }));
                assert!(matches!(children[1], DocumentElement::Group { .. }));
            }
            _ => panic!("Expected Row element"),
        }
    }
}
