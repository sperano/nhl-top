use nhl_api::Boxscore;

use super::state::PlayerSection;

/// Player information extracted from boxscore
#[derive(Clone, Debug)]
pub struct PlayerInfo {
    pub player_id: i64,
    pub name: String,
    pub section: PlayerSection,
    pub index: usize,
}

/// Extract all players from a boxscore, organized by section
pub fn extract_players(boxscore: &Boxscore) -> Vec<PlayerInfo> {
    let mut players = Vec::new();

    // Away team forwards
    for (index, skater) in boxscore
        .player_by_game_stats
        .away_team
        .forwards
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: skater.player_id,
            name: skater.name.default.clone(),
            section: PlayerSection::AwayForwards,
            index,
        });
    }

    // Away team defense
    for (index, skater) in boxscore
        .player_by_game_stats
        .away_team
        .defense
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: skater.player_id,
            name: skater.name.default.clone(),
            section: PlayerSection::AwayDefense,
            index,
        });
    }

    // Away team goalies
    for (index, goalie) in boxscore
        .player_by_game_stats
        .away_team
        .goalies
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: goalie.player_id,
            name: goalie.name.default.clone(),
            section: PlayerSection::AwayGoalies,
            index,
        });
    }

    // Home team forwards
    for (index, skater) in boxscore
        .player_by_game_stats
        .home_team
        .forwards
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: skater.player_id,
            name: skater.name.default.clone(),
            section: PlayerSection::HomeForwards,
            index,
        });
    }

    // Home team defense
    for (index, skater) in boxscore
        .player_by_game_stats
        .home_team
        .defense
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: skater.player_id,
            name: skater.name.default.clone(),
            section: PlayerSection::HomeDefense,
            index,
        });
    }

    // Home team goalies
    for (index, goalie) in boxscore
        .player_by_game_stats
        .home_team
        .goalies
        .iter()
        .enumerate()
    {
        players.push(PlayerInfo {
            player_id: goalie.player_id,
            name: goalie.name.default.clone(),
            section: PlayerSection::HomeGoalies,
            index,
        });
    }

    players
}

/// Find a player by section and index
pub fn find_player(
    boxscore: &Boxscore,
    section: PlayerSection,
    index: usize,
) -> Option<PlayerInfo> {
    let players = extract_players(boxscore);
    players
        .into_iter()
        .find(|p| p.section == section && p.index == index)
}

/// Get the count of players in a specific section
pub fn section_player_count(boxscore: &Boxscore, section: PlayerSection) -> usize {
    match section {
        PlayerSection::ScoringSummary(_) => {
            // TODO: Parse scoring plays from boxscore
            0
        }
        PlayerSection::AwayForwards => boxscore.player_by_game_stats.away_team.forwards.len(),
        PlayerSection::AwayDefense => boxscore.player_by_game_stats.away_team.defense.len(),
        PlayerSection::AwayGoalies => boxscore.player_by_game_stats.away_team.goalies.len(),
        PlayerSection::HomeForwards => boxscore.player_by_game_stats.home_team.forwards.len(),
        PlayerSection::HomeDefense => boxscore.player_by_game_stats.home_team.defense.len(),
        PlayerSection::HomeGoalies => boxscore.player_by_game_stats.home_team.goalies.len(),
    }
}

