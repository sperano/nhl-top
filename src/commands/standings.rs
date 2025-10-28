use nhl_api::{Client, GameDate, Standing};
use chrono::NaiveDate;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Division,
    Conference,
    League,
}

impl GroupBy {
    pub fn name(&self) -> &str {
        match self {
            GroupBy::Division => "Division",
            GroupBy::Conference => "Conference",
            GroupBy::League => "League",
        }
    }

    pub fn all() -> [GroupBy; 3] {
        [GroupBy::Division, GroupBy::Conference, GroupBy::League]
    }
}

pub fn format_standings_table(standings: &[Standing]) -> String {
    let mut output = String::new();

    // Print table header
    output.push_str(&format!("{:<25} {:>3} {:>3} {:>3} {:>3} {:>4}\n", "Team", "GP", "W", "L", "OT", "PTS"));
    output.push_str(&format!("{}\n", "─".repeat(46)));

    // Print each team's stats
    for standing in standings {
        let team_name = &standing.team_common_name.default;
        output.push_str(&format!(
            "{:<25} {:>3} {:>3} {:>3} {:>3} {:>4}\n",
            team_name,
            standing.games_played(),
            standing.wins,
            standing.losses,
            standing.ot_losses,
            standing.points
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

fn merge_columns(left_lines: Vec<String>, right_lines: Vec<String>, column_width: usize) -> String {
    let mut output = String::new();
    let max_len = left_lines.len().max(right_lines.len());

    for i in 0..max_len {
        let left = left_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let right = right_lines.get(i).map(|s| s.as_str()).unwrap_or("");

        // Pad left column to column_width
        output.push_str(&format!("{:<width$}    {}\n", left, right, width = column_width));
    }

    output
}

pub fn format_standings_by_group(standings: &[Standing], by: GroupBy, western_first: bool) -> String {
    if standings.is_empty() {
        return "Loading standings...".to_string();
    }

    let mut output = String::new();
    let mut sorted_standings = standings.to_vec();
    sorted_standings.sort_by(|a, b| b.points.cmp(&a.points));

    match by {
        GroupBy::Division => {
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
            let mut col1_lines = Vec::new();
            for (div_name, teams) in &col1_divs {
                if !col1_lines.is_empty() {
                    col1_lines.push(String::new()); // Add blank line between divisions
                }
                col1_lines.extend(format_group_with_header(div_name, teams));
            }

            let mut col2_lines = Vec::new();
            for (div_name, teams) in &col2_divs {
                if !col2_lines.is_empty() {
                    col2_lines.push(String::new()); // Add blank line between divisions
                }
                col2_lines.extend(format_group_with_header(div_name, teams));
            }

            output.push('\n');
            output.push_str(&merge_columns(col1_lines, col2_lines, 46));
        }
        GroupBy::Conference => {
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

            output.push('\n');

            if groups.len() == 2 {
                let left_lines = format_group_with_header(&groups[0].0, &groups[0].1);
                let right_lines = format_group_with_header(&groups[1].0, &groups[1].1);
                output.push_str(&merge_columns(left_lines, right_lines, 46));
            } else {
                // Fallback to single column if not exactly 2 conferences
                for (conference, teams) in groups {
                    output.push_str(&format!("\n{}\n", conference));
                    output.push_str(&format!("{}\n", "═".repeat(conference.len())));
                    output.push_str(&format_standings_table(&teams));
                }
            }
        }
        GroupBy::League => {
            output.push('\n');
            output.push_str(&format_standings_table(&sorted_standings));
        }
    }

    output
}

pub async fn run(client: &Client, season: Option<i64>, date: Option<String>, by: GroupBy) {
    let standings = if let Some(date_str) = date {
        // Parse date string and get standings for that date
        let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .expect("Invalid date format. Use YYYY-MM-DD");
        let game_date = GameDate::Date(parsed_date);
        client.league_standings_for_date(&game_date).await.unwrap()
    } else if let Some(season_year) = season {
        // Get standings for specific season
        client.league_standings_for_season(season_year).await.unwrap()
    } else {
        // Get current standings
        client.current_league_standings().await.unwrap()
    };

    // Use the shared formatting function (CLI always uses default order)
    let output = format_standings_by_group(&standings, by, false);
    print!("{}", output);
}
