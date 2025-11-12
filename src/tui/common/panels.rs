//! Generic panel definitions for navigation across tabs

use crate::tui::navigation::Panel;

/// Generic panels that can be navigated to across different tabs
#[derive(Clone, Debug, PartialEq)]
pub enum CommonPanel {
    /// Team detail view showing team info and roster
    TeamDetail {
        team_name: String,
        team_abbrev: String,
        wins: i32,
        losses: i32,
        ot_losses: i32,
        points: i32,
        division_name: String,
        conference_name: Option<String>,
    },
    /// Player detail view showing player stats and career
    PlayerDetail {
        player_id: i64,
        player_name: String,
        /// Generic context string: "from team TOR", "from game 2024020001", etc.
        from_context: String,
    },
}

impl Panel for CommonPanel {
    fn breadcrumb_label(&self) -> String {
        match self {
            Self::TeamDetail { team_name, .. } => team_name.clone(),
            Self::PlayerDetail { player_name, .. } => player_name.clone(),
        }
    }

    fn cache_key(&self) -> String {
        match self {
            Self::TeamDetail { team_abbrev, .. } => format!("team:{}", team_abbrev),
            Self::PlayerDetail { player_id, .. } => format!("player:{}", player_id),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TeamDetailData {
    pub team_name: String,
    pub city: String,
    pub arena: String,
    pub founded: String,
    pub conference: String,
    pub division: String,
    pub players: Vec<PlayerStat>,
    pub goalies: Vec<GoalieStat>,
}

#[derive(Clone, Debug)]
pub struct PlayerStat {
    pub name: String,
    pub gp: i32,
    pub g: i32,
    pub a: i32,
    pub pts: i32,
}

#[derive(Clone, Debug)]
pub struct GoalieStat {
    pub name: String,
    pub gp: i32,
    pub gaa: String,
    pub sv_pct: String,
    pub so: i32,
}

/// Generate fake team data for testing
#[cfg(test)]
pub fn fake_team_data(team_name: &str) -> TeamDetailData {
    TeamDetailData {
        team_name: team_name.to_string(),
        city: "Toronto".to_string(),
        arena: "Scotiabank Arena".to_string(),
        founded: "1917".to_string(),
        conference: "Eastern".to_string(),
        division: "Atlantic".to_string(),
        players: vec![
            PlayerStat { name: "Auston Matthews".into(), gp: 58, g: 42, a: 31, pts: 73 },
            PlayerStat { name: "Mitchell Marner".into(), gp: 58, g: 18, a: 48, pts: 66 },
            PlayerStat { name: "William Nylander".into(), gp: 56, g: 28, a: 35, pts: 63 },
            PlayerStat { name: "John Tavares".into(), gp: 58, g: 22, a: 28, pts: 50 },
            PlayerStat { name: "Morgan Rielly".into(), gp: 58, g: 8, a: 32, pts: 40 },
            PlayerStat { name: "Matthew Knies".into(), gp: 52, g: 15, a: 18, pts: 33 },
            PlayerStat { name: "Tyler Bertuzzi".into(), gp: 55, g: 14, a: 16, pts: 30 },
            PlayerStat { name: "Max Domi".into(), gp: 54, g: 12, a: 16, pts: 28 },
            PlayerStat { name: "Jake McCabe".into(), gp: 58, g: 3, a: 15, pts: 18 },
            PlayerStat { name: "T.J. Brodie".into(), gp: 56, g: 2, a: 14, pts: 16 },
            PlayerStat { name: "Calle Jarnkrok".into(), gp: 42, g: 6, a: 8, pts: 14 },
            PlayerStat { name: "Bobby McMann".into(), gp: 38, g: 7, a: 5, pts: 12 },
            PlayerStat { name: "David Kampf".into(), gp: 48, g: 4, a: 6, pts: 10 },
            PlayerStat { name: "Timothy Liljegren".into(), gp: 35, g: 2, a: 8, pts: 10 },
            PlayerStat { name: "Noah Gregor".into(), gp: 40, g: 5, a: 4, pts: 9 },
        ],
        goalies: vec![
            GoalieStat { name: "Ilya Samsonov".into(), gp: 35, gaa: "2.89".into(), sv_pct: ".903".into(), so: 2 },
            GoalieStat { name: "Joseph Woll".into(), gp: 23, gaa: "2.52".into(), sv_pct: ".915".into(), so: 1 },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_detail_breadcrumb_label() {
        let panel = CommonPanel::TeamDetail {
            team_name: "Toronto Maple Leafs".to_string(),
            team_abbrev: "TOR".to_string(),
            wins: 50,
            losses: 20,
            ot_losses: 12,
            points: 112,
            division_name: "Atlantic".to_string(),
            conference_name: Some("Eastern".to_string()),
        };

        assert_eq!(panel.breadcrumb_label(), "Toronto Maple Leafs");
        assert_eq!(panel.cache_key(), "team:TOR");
    }

    #[test]
    fn test_player_detail_breadcrumb_label() {
        let panel = CommonPanel::PlayerDetail {
            player_id: 8478402,
            player_name: "Auston Matthews".to_string(),
            from_context: "from team TOR".to_string(),
        };

        assert_eq!(panel.breadcrumb_label(), "Auston Matthews");
        assert_eq!(panel.cache_key(), "player:8478402");
    }

    #[test]
    fn test_fake_team_data() {
        let data = fake_team_data("Maple Leafs");
        assert_eq!(data.team_name, "Maple Leafs");
        assert_eq!(data.city, "Toronto");
        assert_eq!(data.conference, "Eastern");
        assert_eq!(data.division, "Atlantic");
        assert!(!data.players.is_empty());
        assert!(!data.goalies.is_empty());
    }
}
