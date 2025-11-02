use crate::commands::standings::GroupBy;
use nhl_api::Standing;

/// Represents the complete layout structure for standings display
/// This is the domain model that separates data organization from presentation
#[derive(Debug, Clone)]
pub struct StandingsLayout {
    pub view: GroupBy,
    pub columns: Vec<StandingsColumn>,
}

/// Represents a single column in the standings display
/// Each column contains teams grouped logically (e.g., one conference, or multiple divisions)
#[derive(Debug, Clone)]
pub struct StandingsColumn {
    /// Header groups (e.g., "Atlantic", "Metropolitan") - can be multiple per column
    pub groups: Vec<TeamGroup>,
    /// Total number of teams in this column (sum of all groups)
    pub team_count: usize,
}

/// Represents a logical group of teams within a column
/// For Division view: one group per division
/// For Conference/League view: one group per column
#[derive(Debug, Clone)]
pub struct TeamGroup {
    /// Group header (e.g., "Eastern Conference", "Atlantic", or empty for League view)
    pub header: String,
    /// Teams in this group, in display order
    pub teams: Vec<Standing>,
    /// For wildcard view: index after which to draw playoff cutoff line (0-indexed)
    /// None for other views. The line is drawn AFTER this team index.
    /// For wildcard: this should be 1 (after the 2nd wildcard team, which is index 1)
    pub playoff_cutoff_after: Option<usize>,
}

impl StandingsLayout {
    /// Build a standings layout from raw standings data
    ///
    /// # Arguments
    /// * `standings` - Raw standings data from the NHL API
    /// * `view` - How to group the standings (Division/Conference/League/Wildcard)
    /// * `western_first` - Whether to display Western teams in the left column
    pub fn build(standings: &[Standing], view: GroupBy, western_first: bool) -> Self {
        let columns = match view {
            GroupBy::League => Self::build_league_layout(standings),
            GroupBy::Conference => Self::build_conference_layout(standings, western_first),
            GroupBy::Division => Self::build_division_layout(standings, western_first),
            GroupBy::Wildcard => Self::build_wildcard_layout(standings, western_first),
        };

        StandingsLayout { view, columns }
    }

    /// Build single-column layout with all teams sorted by points
    fn build_league_layout(standings: &[Standing]) -> Vec<StandingsColumn> {
        let mut sorted = standings.to_vec();
        sorted.sort_by(|a, b| b.points.cmp(&a.points));

        let group = TeamGroup {
            header: String::new(), // No header for league view
            teams: sorted,
            playoff_cutoff_after: None,
        };

        vec![StandingsColumn {
            groups: vec![group],
            team_count: standings.len(),
        }]
    }

    /// Build two-column layout with conferences
    fn build_conference_layout(standings: &[Standing], western_first: bool) -> Vec<StandingsColumn> {
        let mut eastern_teams: Vec<_> = standings
            .iter()
            .filter(|s| s.conference_name.as_deref() == Some("Eastern"))
            .cloned()
            .collect();
        eastern_teams.sort_by(|a, b| b.points.cmp(&a.points));

        let mut western_teams: Vec<_> = standings
            .iter()
            .filter(|s| s.conference_name.as_deref() == Some("Western"))
            .cloned()
            .collect();
        western_teams.sort_by(|a, b| b.points.cmp(&a.points));

        let eastern_group = TeamGroup {
            header: "Eastern Conference".to_string(),
            teams: eastern_teams.clone(),
            playoff_cutoff_after: None,
        };

        let western_group = TeamGroup {
            header: "Western Conference".to_string(),
            teams: western_teams.clone(),
            playoff_cutoff_after: None,
        };

        let eastern_col = StandingsColumn {
            groups: vec![eastern_group],
            team_count: eastern_teams.len(),
        };

        let western_col = StandingsColumn {
            groups: vec![western_group],
            team_count: western_teams.len(),
        };

        if western_first {
            vec![western_col, eastern_col]
        } else {
            vec![eastern_col, western_col]
        }
    }

    /// Build two-column layout with divisions grouped by conference
    fn build_division_layout(standings: &[Standing], western_first: bool) -> Vec<StandingsColumn> {
        // Group teams by division
        let mut atlantic: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Atlantic")
            .cloned()
            .collect();
        atlantic.sort_by(|a, b| b.points.cmp(&a.points));

        let mut metropolitan: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Metropolitan")
            .cloned()
            .collect();
        metropolitan.sort_by(|a, b| b.points.cmp(&a.points));

        let mut central: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Central")
            .cloned()
            .collect();
        central.sort_by(|a, b| b.points.cmp(&a.points));

        let mut pacific: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Pacific")
            .cloned()
            .collect();
        pacific.sort_by(|a, b| b.points.cmp(&a.points));

        // Create groups
        let atlantic_group = TeamGroup {
            header: "Atlantic".to_string(),
            teams: atlantic.clone(),
            playoff_cutoff_after: None,
        };

        let metro_group = TeamGroup {
            header: "Metropolitan".to_string(),
            teams: metropolitan.clone(),
            playoff_cutoff_after: None,
        };

        let central_group = TeamGroup {
            header: "Central".to_string(),
            teams: central.clone(),
            playoff_cutoff_after: None,
        };

        let pacific_group = TeamGroup {
            header: "Pacific".to_string(),
            teams: pacific.clone(),
            playoff_cutoff_after: None,
        };

        // Build columns (each column contains multiple division groups)
        let eastern_col = StandingsColumn {
            groups: vec![atlantic_group, metro_group],
            team_count: atlantic.len() + metropolitan.len(),
        };

        let western_col = StandingsColumn {
            groups: vec![central_group, pacific_group],
            team_count: central.len() + pacific.len(),
        };

        if western_first {
            vec![western_col, eastern_col]
        } else {
            vec![eastern_col, western_col]
        }
    }

    /// Build two-column wildcard layout with playoff positioning
    /// Each column shows:
    /// - Top 3 from Division 1
    /// - Top 3 from Division 2
    /// - Remaining teams (wildcards and out of playoffs)
    /// - Visual separator after 2nd wildcard (8th team overall)
    fn build_wildcard_layout(standings: &[Standing], western_first: bool) -> Vec<StandingsColumn> {
        // Group teams by division
        let mut atlantic: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Atlantic")
            .cloned()
            .collect();
        atlantic.sort_by(|a, b| b.points.cmp(&a.points));

        let mut metropolitan: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Metropolitan")
            .cloned()
            .collect();
        metropolitan.sort_by(|a, b| b.points.cmp(&a.points));

        let mut central: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Central")
            .cloned()
            .collect();
        central.sort_by(|a, b| b.points.cmp(&a.points));

        let mut pacific: Vec<_> = standings
            .iter()
            .filter(|s| s.division_name == "Pacific")
            .cloned()
            .collect();
        pacific.sort_by(|a, b| b.points.cmp(&a.points));

        // Build Eastern Conference wildcard groups
        let eastern_groups = Self::build_wildcard_groups(
            "Atlantic",
            &atlantic,
            "Metropolitan",
            &metropolitan,
        );

        // Build Western Conference wildcard groups
        let western_groups = Self::build_wildcard_groups(
            "Central",
            &central,
            "Pacific",
            &pacific,
        );

        let eastern_col = StandingsColumn {
            groups: eastern_groups,
            team_count: atlantic.len() + metropolitan.len(),
        };

        let western_col = StandingsColumn {
            groups: western_groups,
            team_count: central.len() + pacific.len(),
        };

        if western_first {
            vec![western_col, eastern_col]
        } else {
            vec![eastern_col, western_col]
        }
    }

    /// Helper to build wildcard groups for a conference
    /// Takes top 3 from each division, then remaining teams sorted by points
    fn build_wildcard_groups(
        div1_name: &str,
        div1_teams: &[Standing],
        div2_name: &str,
        div2_teams: &[Standing],
    ) -> Vec<TeamGroup> {
        let mut groups = Vec::new();

        // Division 1 - top 3
        let div1_top3: Vec<_> = div1_teams.iter().take(3).cloned().collect();
        groups.push(TeamGroup {
            header: div1_name.to_string(),
            teams: div1_top3,
            playoff_cutoff_after: None,
        });

        // Division 2 - top 3
        let div2_top3: Vec<_> = div2_teams.iter().take(3).cloned().collect();
        groups.push(TeamGroup {
            header: div2_name.to_string(),
            teams: div2_top3,
            playoff_cutoff_after: None,
        });

        // Remaining teams (wildcards and out of playoffs) - sorted by points
        let div1_remaining: Vec<_> = div1_teams.iter().skip(3).cloned().collect();
        let div2_remaining: Vec<_> = div2_teams.iter().skip(3).cloned().collect();

        let mut wildcard_teams: Vec<_> = div1_remaining
            .into_iter()
            .chain(div2_remaining)
            .collect();
        wildcard_teams.sort_by(|a, b| b.points.cmp(&a.points));

        groups.push(TeamGroup {
            header: "Wildcard".to_string(),
            teams: wildcard_teams,
            // Draw playoff cutoff line after 2nd wildcard (index 1)
            playoff_cutoff_after: Some(1),
        });

        groups
    }

    /// Get the total number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get a team by column and index
    /// Returns None if the indices are out of bounds
    pub fn get_team(&self, column_idx: usize, team_idx: usize) -> Option<&Standing> {
        let column = self.columns.get(column_idx)?;

        // Iterate through groups to find the team at the given index
        let mut current_idx = 0;
        for group in &column.groups {
            let group_size = group.teams.len();
            if team_idx < current_idx + group_size {
                return group.teams.get(team_idx - current_idx);
            }
            current_idx += group_size;
        }

        None
    }

    /// Find the position (column_idx, team_idx) of a team by name
    /// Returns None if the team is not found
    pub fn find_team_position(&self, team_name: &str) -> Option<(usize, usize)> {
        for (col_idx, column) in self.columns.iter().enumerate() {
            let mut team_idx = 0;
            for group in &column.groups {
                for team in &group.teams {
                    if team.team_common_name.default == team_name {
                        return Some((col_idx, team_idx));
                    }
                    team_idx += 1;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nhl_api::LocalizedString;

    fn mock_standing(name: &str, division: &str, conference: &str, points: i32) -> Standing {
        Standing {
            conference_abbrev: Some(conference.chars().next().unwrap().to_string()),
            conference_name: Some(conference.to_string()),
            division_abbrev: division.chars().next().unwrap().to_string(),
            division_name: division.to_string(),
            team_name: LocalizedString {
                default: name.to_string(),
            },
            team_common_name: LocalizedString {
                default: name.to_string(),
            },
            team_abbrev: LocalizedString {
                default: name.chars().take(3).collect(),
            },
            team_logo: "".to_string(),
            wins: 0,
            losses: 0,
            ot_losses: 0,
            points,
        }
    }

    #[test]
    fn test_league_layout_single_column() {
        let standings = vec![
            mock_standing("Team A", "Atlantic", "Eastern", 100),
            mock_standing("Team B", "Pacific", "Western", 90),
        ];

        let layout = StandingsLayout::build(&standings, GroupBy::League, false);

        assert_eq!(layout.column_count(), 1);
        assert_eq!(layout.columns[0].team_count, 2);
    }

    #[test]
    fn test_conference_layout_two_columns() {
        let standings = vec![
            mock_standing("Team A", "Atlantic", "Eastern", 100),
            mock_standing("Team B", "Pacific", "Western", 90),
        ];

        let layout = StandingsLayout::build(&standings, GroupBy::Conference, false);

        assert_eq!(layout.column_count(), 2);
        assert_eq!(layout.columns[0].groups[0].header, "Eastern Conference");
        assert_eq!(layout.columns[1].groups[0].header, "Western Conference");
    }

    #[test]
    fn test_division_layout_western_first() {
        let standings = vec![
            mock_standing("Team A", "Atlantic", "Eastern", 100),
            mock_standing("Team B", "Central", "Western", 90),
        ];

        let layout = StandingsLayout::build(&standings, GroupBy::Division, true);

        // Western divisions should be in column 0 when western_first is true
        assert_eq!(layout.columns[0].groups[0].header, "Central");
    }

    #[test]
    fn test_get_team() {
        let standings = vec![
            mock_standing("Team A", "Atlantic", "Eastern", 100),
            mock_standing("Team B", "Atlantic", "Eastern", 90),
        ];

        let layout = StandingsLayout::build(&standings, GroupBy::League, false);

        let team = layout.get_team(0, 0);
        assert!(team.is_some());
        assert_eq!(team.unwrap().team_common_name.default, "Team A");
    }

    #[test]
    fn test_find_team_position() {
        let standings = vec![
            mock_standing("Team A", "Atlantic", "Eastern", 100),
            mock_standing("Team B", "Atlantic", "Eastern", 90),
        ];

        let layout = StandingsLayout::build(&standings, GroupBy::League, false);

        let pos = layout.find_team_position("Team B");
        assert_eq!(pos, Some((0, 1)));
    }
}
