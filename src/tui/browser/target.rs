/// Target represents where a link points to
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Link to a team page
    Team { id: String },
    /// Link to a player page
    Player { id: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_team_creation() {
        let target = Target::Team { id: "MTL".to_string() };
        assert_eq!(target, Target::Team { id: "MTL".to_string() });
    }

    #[test]
    fn test_target_player_creation() {
        let target = Target::Player { id: 8480018 };
        assert_eq!(target, Target::Player { id: 8480018 });
    }

    #[test]
    fn test_target_equality() {
        let team1 = Target::Team { id: "MTL".to_string() };
        let team2 = Target::Team { id: "MTL".to_string() };
        let team3 = Target::Team { id: "VGK".to_string() };

        assert_eq!(team1, team2);
        assert_ne!(team1, team3);

        let player1 = Target::Player { id: 8480018 };
        let player2 = Target::Player { id: 8480018 };
        let player3 = Target::Player { id: 8479318 };

        assert_eq!(player1, player2);
        assert_ne!(player1, player3);
    }

    #[test]
    fn test_target_debug() {
        let team = Target::Team { id: "MTL".to_string() };
        let debug_str = format!("{:?}", team);
        assert!(debug_str.contains("Team"));
        assert!(debug_str.contains("MTL"));
    }

    #[test]
    fn test_target_clone() {
        let team = Target::Team { id: "MTL".to_string() };
        let cloned = team.clone();
        assert_eq!(team, cloned);

        let player = Target::Player { id: 8480018 };
        let cloned_player = player.clone();
        assert_eq!(player, cloned_player);
    }

    #[test]
    fn test_target_team_vs_player() {
        let team = Target::Team { id: "MTL".to_string() };
        let player = Target::Player { id: 8480018 };

        assert_ne!(team, player);
    }
}
