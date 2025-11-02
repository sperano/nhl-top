use nhl_api::{Client, Standing};
use std::collections::BTreeMap;
use crate::commands::parse_game_date;
use anyhow::{Context, Result};

// Layout Constants
/// Width of standings table column (for two-column layout)
const STANDINGS_COLUMN_WIDTH: usize = 46;

/// Width of the separator line (matches the table header width)
const SEPARATOR_LINE_WIDTH: usize = 46;

/// Width of team name column
const TEAM_NAME_COL_WIDTH: usize = 25;

/// Width of games played column
const GP_COL_WIDTH: usize = 3;

/// Width of wins column
const W_COL_WIDTH: usize = 3;

/// Width of losses column
const L_COL_WIDTH: usize = 3;

/// Width of OT losses column
const OT_COL_WIDTH: usize = 3;

/// Width of points column
const PTS_COL_WIDTH: usize = 4;

/// Spacing between columns in two-column layout
const COLUMN_SPACING: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Division,
    Conference,
    League,
    Wildcard,
}

impl GroupBy {
    pub fn name(&self) -> &str {
        match self {
            GroupBy::Division => "Division",
            GroupBy::Conference => "Conference",
            GroupBy::League => "League",
            GroupBy::Wildcard => "Wildcard",
        }
    }

    pub fn all() -> [GroupBy; 4] {
        [GroupBy::Wildcard, GroupBy::Division, GroupBy::Conference, GroupBy::League]
    }

    /// Get the next view in the cycle (Wildcard → Division → Conference → League → Wildcard)
    pub fn next(&self) -> Self {
        match self {
            GroupBy::Wildcard => GroupBy::Division,
            GroupBy::Division => GroupBy::Conference,
            GroupBy::Conference => GroupBy::League,
            GroupBy::League => GroupBy::Wildcard,
        }
    }

    /// Get the previous view in the cycle (Wildcard → League → Conference → Division → Wildcard)
    pub fn prev(&self) -> Self {
        match self {
            GroupBy::Wildcard => GroupBy::League,
            GroupBy::Division => GroupBy::Wildcard,
            GroupBy::Conference => GroupBy::Division,
            GroupBy::League => GroupBy::Conference,
        }
    }
}

pub fn format_standings_table(standings: &[Standing]) -> String {
    let mut output = String::new();

    // Print table header
    output.push_str(&format!(
        "{:<team_width$} {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}\n",
        "Team", "GP", "W", "L", "OT", "PTS",
        team_width = TEAM_NAME_COL_WIDTH,
        gp_width = GP_COL_WIDTH,
        w_width = W_COL_WIDTH,
        l_width = L_COL_WIDTH,
        ot_width = OT_COL_WIDTH,
        pts_width = PTS_COL_WIDTH
    ));
    output.push_str(&format!("{}\n", "─".repeat(SEPARATOR_LINE_WIDTH)));

    // Print each team's stats
    for standing in standings {
        let team_name = &standing.team_common_name.default;
        output.push_str(&format!(
            "{:<team_width$} {:>gp_width$} {:>w_width$} {:>l_width$} {:>ot_width$} {:>pts_width$}\n",
            team_name,
            standing.games_played(),
            standing.wins,
            standing.losses,
            standing.ot_losses,
            standing.points,
            team_width = TEAM_NAME_COL_WIDTH,
            gp_width = GP_COL_WIDTH,
            w_width = W_COL_WIDTH,
            l_width = L_COL_WIDTH,
            ot_width = OT_COL_WIDTH,
            pts_width = PTS_COL_WIDTH
        ));
    }

    output
}

fn format_group_with_header(name: &str, teams: &[Standing]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("{}", name));
    lines.push(format!("{}", "═".repeat(name.len())));
    lines.push(String::new()); // Empty line between header and table

    // Add table rows
    let table = format_standings_table(teams);
    lines.extend(table.lines().map(|s| s.to_string()));

    lines
}

/// Formats a column of divisions by stacking them vertically with blank lines between each division.
///
/// # Arguments
/// * `divisions` - A slice of tuples containing division name and team standings
///
/// # Returns
/// A vector of formatted strings representing the division column
fn format_division_column(divisions: &[(String, Vec<Standing>)]) -> Vec<String> {
    let mut lines = Vec::new();

    for (div_name, teams) in divisions {
        if !lines.is_empty() {
            lines.push(String::new()); // Add blank line between divisions
        }
        lines.extend(format_group_with_header(div_name, teams));
    }

    lines
}

fn merge_columns(left_lines: Vec<String>, right_lines: Vec<String>, column_width: usize) -> String {
    let mut output = String::new();
    let max_len = left_lines.len().max(right_lines.len());

    for i in 0..max_len {
        let left = left_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let right = right_lines.get(i).map(|s| s.as_str()).unwrap_or("");

        // Pad left column to column_width with spacing between columns
        output.push_str(&format!(
            "{:<width$}{}{}\n",
            left,
            " ".repeat(COLUMN_SPACING),
            right,
            width = column_width
        ));
    }

    output
}

/// Formats standings in division view with two-column layout
fn format_division_view(sorted_standings: Vec<Standing>, western_first: bool) -> String {
    let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for standing in sorted_standings {
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

    // Sort divisions alphabetically within each conference
    eastern_divs.sort_by(|a, b| a.0.cmp(&b.0));
    western_divs.sort_by(|a, b| a.0.cmp(&b.0));

    // Build column 1 and column 2 based on western_first
    let (col1_divs, col2_divs) = if western_first {
        (western_divs, eastern_divs)
    } else {
        (eastern_divs, western_divs)
    };

    // Format each column as stacked divisions
    let col1_lines = format_division_column(&col1_divs);
    let col2_lines = format_division_column(&col2_divs);

    let mut output = String::new();
    output.push('\n');
    output.push_str(&merge_columns(col1_lines, col2_lines, STANDINGS_COLUMN_WIDTH));
    output
}

/// Formats standings in conference view with two-column layout
fn format_conference_view(sorted_standings: Vec<Standing>, western_first: bool) -> String {
    let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for standing in sorted_standings {
        let conference = standing.conference_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        grouped
            .entry(conference)
            .or_default()
            .push(standing);
    }

    // Convert to vec to split into columns
    let mut groups: Vec<_> = grouped.into_iter().collect();

    // If western_first is true, reverse to show Western first
    if western_first && groups.len() == 2 {
        // BTreeMap gives us Eastern, Western alphabetically
        // Reverse to get Western, Eastern
        groups.reverse();
    }

    let mut output = String::new();
    output.push('\n');

    if groups.len() == 2 {
        let left_lines = format_group_with_header(&groups[0].0, &groups[0].1);
        let right_lines = format_group_with_header(&groups[1].0, &groups[1].1);
        output.push_str(&merge_columns(left_lines, right_lines, STANDINGS_COLUMN_WIDTH));
    } else {
        // Fallback to single column if not exactly 2 conferences
        for (conference, teams) in groups {
            output.push_str(&format!("\n{}\n", conference));
            output.push_str(&format!("{}\n", "═".repeat(conference.len())));
            output.push_str(&format_standings_table(&teams));
        }
    }

    output
}

/// Formats standings in league-wide view (single column, sorted by points)
fn format_league_view(sorted_standings: Vec<Standing>) -> String {
    let mut output = String::new();
    output.push('\n');
    output.push_str(&format_standings_table(&sorted_standings));
    output
}

/// Helper to format wildcard groups for a conference
fn format_wildcard_conference(
    div1_name: &str,
    div1_teams: &[Standing],
    div2_name: &str,
    div2_teams: &[Standing],
) -> Vec<String> {
    let mut lines = Vec::new();

    // Division 1 - top 3
    let div1_top3: Vec<_> = div1_teams.iter().take(3).cloned().collect();
    if !div1_top3.is_empty() {
        lines.extend(format_group_with_header(div1_name, &div1_top3));
        lines.push(String::new()); // Blank line after division
    }

    // Division 2 - top 3
    let div2_top3: Vec<_> = div2_teams.iter().take(3).cloned().collect();
    if !div2_top3.is_empty() {
        lines.extend(format_group_with_header(div2_name, &div2_top3));
        lines.push(String::new()); // Blank line after division
    }

    // Remaining teams (wildcards and out of playoffs) - sorted by points
    let div1_remaining: Vec<_> = div1_teams.iter().skip(3).cloned().collect();
    let div2_remaining: Vec<_> = div2_teams.iter().skip(3).cloned().collect();

    let mut wildcard_teams: Vec<_> = div1_remaining
        .into_iter()
        .chain(div2_remaining)
        .collect();
    wildcard_teams.sort_by(|a, b| b.points.cmp(&a.points));

    if !wildcard_teams.is_empty() {
        lines.extend(format_group_with_header("Wildcard", &wildcard_teams));

        // Add playoff cutoff line after 2nd wildcard team (if there are at least 2)
        if wildcard_teams.len() >= 2 {
            // Find the line with the 2nd wildcard team (accounting for header lines)
            // Header has 3 lines (title, underline, blank) + table header (2 lines) + teams
            let cutoff_line_idx = 3 + 2 + 2; // After 2nd team row
            if lines.len() > cutoff_line_idx {
                lines.insert(cutoff_line_idx, format!("{}", "─".repeat(SEPARATOR_LINE_WIDTH)));
            }
        }
    }

    lines
}

/// Formats standings in wildcard view with two-column layout
fn format_wildcard_view(sorted_standings: Vec<Standing>, western_first: bool) -> String {
    // Group teams by division
    let mut atlantic: Vec<_> = sorted_standings
        .iter()
        .filter(|s| s.division_name == "Atlantic")
        .cloned()
        .collect();
    atlantic.sort_by(|a, b| b.points.cmp(&a.points));

    let mut metropolitan: Vec<_> = sorted_standings
        .iter()
        .filter(|s| s.division_name == "Metropolitan")
        .cloned()
        .collect();
    metropolitan.sort_by(|a, b| b.points.cmp(&a.points));

    let mut central: Vec<_> = sorted_standings
        .iter()
        .filter(|s| s.division_name == "Central")
        .cloned()
        .collect();
    central.sort_by(|a, b| b.points.cmp(&a.points));

    let mut pacific: Vec<_> = sorted_standings
        .iter()
        .filter(|s| s.division_name == "Pacific")
        .cloned()
        .collect();
    pacific.sort_by(|a, b| b.points.cmp(&a.points));

    // Build Eastern Conference wildcard groups
    let eastern_lines = format_wildcard_conference(
        "Atlantic",
        &atlantic,
        "Metropolitan",
        &metropolitan,
    );

    // Build Western Conference wildcard groups
    let western_lines = format_wildcard_conference(
        "Central",
        &central,
        "Pacific",
        &pacific,
    );

    let (col1_lines, col2_lines) = if western_first {
        (western_lines, eastern_lines)
    } else {
        (eastern_lines, western_lines)
    };

    let mut output = String::new();
    output.push('\n');
    output.push_str(&merge_columns(col1_lines, col2_lines, STANDINGS_COLUMN_WIDTH));
    output
}

pub fn format_standings_by_group(standings: &[Standing], by: GroupBy, western_first: bool) -> String {
    if standings.is_empty() {
        return "Loading standings...".to_string();
    }

    let mut sorted_standings = standings.to_vec();
    sorted_standings.sort_by(|a, b| b.points.cmp(&a.points));

    match by {
        GroupBy::Division => format_division_view(sorted_standings, western_first),
        GroupBy::Conference => format_conference_view(sorted_standings, western_first),
        GroupBy::League => format_league_view(sorted_standings),
        GroupBy::Wildcard => format_wildcard_view(sorted_standings, western_first),
    }
}

pub async fn run(client: &Client, season: Option<i64>, date: Option<String>, by: GroupBy) -> Result<()> {
    let standings = if date.is_some() {
        // Parse date string and get standings for that date
        let game_date = parse_game_date(date)?;
        client.league_standings_for_date(&game_date).await
            .context("Failed to fetch standings for date")?
    } else if let Some(season_year) = season {
        // Get standings for specific season
        client.league_standings_for_season(season_year).await
            .context("Failed to fetch standings for season")?
    } else {
        // Get current standings
        client.current_league_standings().await
            .context("Failed to fetch current standings")?
    };

    // Use the shared formatting function (CLI always uses default order)
    let output = format_standings_by_group(&standings, by, false);
    print!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groupby_name() {
        assert_eq!(GroupBy::Division.name(), "Division");
        assert_eq!(GroupBy::Conference.name(), "Conference");
        assert_eq!(GroupBy::League.name(), "League");
    }

    #[test]
    fn test_groupby_all() {
        let all = GroupBy::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], GroupBy::Wildcard);
        assert_eq!(all[1], GroupBy::Division);
        assert_eq!(all[2], GroupBy::Conference);
        assert_eq!(all[3], GroupBy::League);
    }

    #[test]
    fn test_format_standings_by_group_empty() {
        let standings = vec![];
        let output = format_standings_by_group(&standings, GroupBy::Division, false);
        assert_eq!(output, "Loading standings...");
    }

    #[test]
    fn test_merge_columns_equal_length() {
        let left = vec!["Left1".to_string(), "Left2".to_string()];
        let right = vec!["Right1".to_string(), "Right2".to_string()];

        let output = merge_columns(left, right, 10);

        // Should have both columns
        assert!(output.contains("Left1"));
        assert!(output.contains("Right1"));
        assert!(output.contains("Left2"));
        assert!(output.contains("Right2"));
    }

    #[test]
    fn test_merge_columns_unequal_length() {
        let left = vec!["Left1".to_string(), "Left2".to_string(), "Left3".to_string()];
        let right = vec!["Right1".to_string()];

        let output = merge_columns(left, right, 10);

        // Should have all left items
        assert!(output.contains("Left1"));
        assert!(output.contains("Left2"));
        assert!(output.contains("Left3"));

        // Should have right item
        assert!(output.contains("Right1"));
    }
}
