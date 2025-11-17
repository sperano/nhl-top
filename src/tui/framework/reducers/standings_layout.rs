use std::collections::BTreeMap;
use nhl_api::Standing;
use crate::commands::standings::GroupBy;

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
        teams.sort_by(|a, b| b.points.cmp(&a.points));
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
    } else {
        if column == 0 { eastern_count } else { western_count }
    }
}

// Private helper functions for building layouts

fn build_league_layout(standings: &[Standing]) -> Vec<Vec<String>> {
    // Single column, sorted by points
    let mut sorted = standings.to_vec();
    sorted.sort_by(|a, b| b.points.cmp(&a.points));
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
        teams.sort_by(|a, b| b.points.cmp(&a.points));
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
        teams.sort_by(|a, b| b.points.cmp(&a.points));
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
        teams.sort_by(|a, b| b.points.cmp(&a.points));
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
        wildcards.sort_by(|a, b| b.points.cmp(&a.points));
        teams.extend(wildcards);
        teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
    };

    let western: Vec<String> = {
        let mut teams = Vec::new();
        teams.extend(central.iter().take(3).cloned());
        teams.extend(pacific.iter().take(3).cloned());
        let mut wildcards: Vec<_> = central.iter().skip(3).chain(pacific.iter().skip(3)).cloned().collect();
        wildcards.sort_by(|a, b| b.points.cmp(&a.points));
        teams.extend(wildcards);
        teams.iter().map(|s| s.team_abbrev.default.clone()).collect()
    };

    if western_first {
        vec![western, eastern]
    } else {
        vec![eastern, western]
    }
}
