use nhl_api::{Client, GameId, Boxscore};

pub fn format_boxscore(boxscore: &Boxscore) -> String {
    let mut output = String::new();

    // Display game header
    output.push_str(&format!("\n{} @ {}\n",
        boxscore.away_team.common_name.default,
        boxscore.home_team.common_name.default
    ));
    output.push_str(&format!("{}\n", "═".repeat(60)));
    output.push_str(&format!("Date: {} | Venue: {}\n",
        boxscore.game_date,
        boxscore.venue.default
    ));
    output.push_str(&format!("Status: {} | Period: {}\n",
        boxscore.game_state,
        boxscore.period_descriptor.number
    ));
    if boxscore.clock.running || !boxscore.clock.in_intermission {
        output.push_str(&format!("Time: {}\n", boxscore.clock.time_remaining));
    }

    // Display score
    output.push_str(&format!("\n{:<20} {:>3}\n", "Team", "Score"));
    output.push_str(&format!("{}\n", "─".repeat(25)));
    output.push_str(&format!("{:<20} {:>3}\n",
        boxscore.away_team.abbrev,
        boxscore.away_team.score
    ));
    output.push_str(&format!("{:<20} {:>3}\n",
        boxscore.home_team.abbrev,
        boxscore.home_team.score
    ));

    // Display shots on goal
    output.push_str(&format!("\n{:<20} {:>3}\n", "Team", "SOG"));
    output.push_str(&format!("{}\n", "─".repeat(25)));
    output.push_str(&format!("{:<20} {:>3}\n",
        boxscore.away_team.abbrev,
        boxscore.away_team.sog
    ));
    output.push_str(&format!("{:<20} {:>3}\n",
        boxscore.home_team.abbrev,
        boxscore.home_team.sog
    ));

    // Display player stats - Away Team
    output.push_str(&format!("\n{} - Forwards\n", boxscore.away_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
        "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
    ));
    for player in &boxscore.player_by_game_stats.away_team.forwards {
        output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
            player.sweater_number,
            player.name.default,
            player.position,
            player.goals,
            player.assists,
            player.points,
            player.plus_minus,
            player.toi
        ));
    }

    output.push_str(&format!("\n{} - Defense\n", boxscore.away_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
        "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
    ));
    for player in &boxscore.player_by_game_stats.away_team.defense {
        output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
            player.sweater_number,
            player.name.default,
            player.position,
            player.goals,
            player.assists,
            player.points,
            player.plus_minus,
            player.toi
        ));
    }

    output.push_str(&format!("\n{} - Goalies\n", boxscore.away_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}\n",
        "#", "Name", "SA", "Saves", "GA", "SV%"
    ));
    for goalie in &boxscore.player_by_game_stats.away_team.goalies {
        let sv_pct = goalie.save_pctg
            .map(|p| format!("{:.3}", p))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}\n",
            goalie.sweater_number,
            goalie.name.default,
            goalie.shots_against,
            goalie.saves,
            goalie.goals_against,
            sv_pct
        ));
    }

    // Display player stats - Home Team
    output.push_str(&format!("\n{} - Forwards\n", boxscore.home_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
        "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
    ));
    for player in &boxscore.player_by_game_stats.home_team.forwards {
        output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
            player.sweater_number,
            player.name.default,
            player.position,
            player.goals,
            player.assists,
            player.points,
            player.plus_minus,
            player.toi
        ));
    }

    output.push_str(&format!("\n{} - Defense\n", boxscore.home_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
        "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
    ));
    for player in &boxscore.player_by_game_stats.home_team.defense {
        output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
            player.sweater_number,
            player.name.default,
            player.position,
            player.goals,
            player.assists,
            player.points,
            player.plus_minus,
            player.toi
        ));
    }

    output.push_str(&format!("\n{} - Goalies\n", boxscore.home_team.abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}\n",
        "#", "Name", "SA", "Saves", "GA", "SV%"
    ));
    for goalie in &boxscore.player_by_game_stats.home_team.goalies {
        let sv_pct = goalie.save_pctg
            .map(|p| format!("{:.3}", p))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}\n",
            goalie.sweater_number,
            goalie.name.default,
            goalie.shots_against,
            goalie.saves,
            goalie.goals_against,
            sv_pct
        ));
    }

    output
}

pub async fn run(client: &Client, game_id: i64) {
    let game_id = GameId::new(game_id);
    let boxscore = client.boxscore(&game_id).await.unwrap();
    print!("{}", format_boxscore(&boxscore));
}
