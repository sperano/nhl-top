use nhl_api::{Client, GameId, Boxscore};
use anyhow::{Context, Result};

/// Format skater (forwards/defense) stats table
fn format_skater_stats(
    output: &mut String,
    team_abbrev: &str,
    position_name: &str,
    players: &[nhl_api::SkaterStats]
) {
    output.push_str(&format!("\n{} - {}\n", team_abbrev, position_name));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}\n",
        "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
    ));
    for player in players {
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
}

/// Format goalie stats table
fn format_goalie_stats(
    output: &mut String,
    team_abbrev: &str,
    goalies: &[nhl_api::GoalieStats]
) {
    output.push_str(&format!("\n{} - Goalies\n", team_abbrev));
    output.push_str(&format!("{}\n", "─".repeat(80)));
    output.push_str(&format!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}\n",
        "#", "Name", "SA", "Saves", "GA", "SV%"
    ));
    for goalie in goalies {
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
}

/// Format all player stats for a team
fn format_team_stats(
    output: &mut String,
    team_abbrev: &str,
    stats: &nhl_api::TeamPlayerStats
) {
    format_skater_stats(output, team_abbrev, "Forwards", &stats.forwards);
    format_skater_stats(output, team_abbrev, "Defense", &stats.defense);
    format_goalie_stats(output, team_abbrev, &stats.goalies);
}

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

    // Display player stats using extracted helper functions
    format_team_stats(&mut output, &boxscore.away_team.abbrev, &boxscore.player_by_game_stats.away_team);
    format_team_stats(&mut output, &boxscore.home_team.abbrev, &boxscore.player_by_game_stats.home_team);

    output
}

pub async fn run(client: &Client, game_id: i64) -> Result<()> {
    let game_id = GameId::new(game_id);
    let boxscore = client.boxscore(&game_id).await
        .context("Failed to fetch boxscore")?;
    print!("{}", format_boxscore(&boxscore));

    Ok(())
}
