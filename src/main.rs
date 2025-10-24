use nhl_api::{Client, ClientConfig, GameDate, GameId};
use chrono::NaiveDate;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;

#[derive(Parser)]
#[command(name = "nhl-top")]
#[command(about = "NHL stats and standings CLI", long_about = None)]
struct Cli {
    /// Enable debug mode with verbose output
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum GroupBy {
    /// Group by division
    #[value(name = "d")]
    Division,
    /// Group by conference
    #[value(name = "c")]
    Conference,
    /// Show league-wide standings
    #[value(name = "l")]
    League,
}

#[derive(Subcommand)]
enum Commands {
    /// Display NHL standings
    Standings {
        /// Season year (optional, defaults to current standings)
        #[arg(short, long)]
        season: Option<i64>,

        /// Date in YYYY-MM-DD format (optional)
        #[arg(short, long)]
        date: Option<String>,

        /// Group standings by: d=division, c=conference, l=league
        #[arg(short, long, default_value = "d")]
        by: GroupBy,
    },
    /// Display boxscore for a specific game
    Boxscore {
        /// Game ID (e.g., 2024020001)
        game_id: i64,
    },
    /// Display daily schedule of games
    Schedule {
        /// Date in YYYY-MM-DD format (optional, defaults to today)
        #[arg(short, long)]
        date: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Create client with debug mode if flag is set
    let client = if cli.debug {
        let config = ClientConfig::default().with_debug();
        Client::with_config(config).unwrap()
    } else {
        Client::new().unwrap()
    };

    match cli.command {
        Commands::Standings { season, date, by } => {
            let mut standings = if let Some(date_str) = date {
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

            // Sort by points (descending)
            standings.sort_by(|a, b| b.points.cmp(&a.points));

            match by {
                GroupBy::Division => {
                    // Group by division
                    let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
                    for standing in standings {
                        grouped
                            .entry(standing.division_name.clone())
                            .or_default()
                            .push(standing);
                    }

                    for (division, teams) in grouped {
                        println!("\n{}", division);
                        println!("{}", "=".repeat(division.len()));
                        for standing in teams {
                            println!("{}", standing);
                        }
                    }
                }
                GroupBy::Conference => {
                    // Group by conference
                    let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
                    for standing in standings {
                        let conference = standing.conference_name
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        grouped
                            .entry(conference)
                            .or_default()
                            .push(standing);
                    }

                    for (conference, teams) in grouped {
                        println!("\n{}", conference);
                        println!("{}", "=".repeat(conference.len()));
                        for standing in teams {
                            println!("{}", standing);
                        }
                    }
                }
                GroupBy::League => {
                    // Show all teams in a single list
                    println!("\nNHL Standings");
                    println!("=============");
                    for standing in standings {
                        println!("{}", standing);
                    }
                }
            }
        }
        Commands::Boxscore { game_id } => {
            let game_id = GameId::new(game_id);
            let boxscore = client.boxscore(&game_id).await.unwrap();

            // Display game header
            println!("\n{} @ {}",
                boxscore.away_team.common_name.default,
                boxscore.home_team.common_name.default
            );
            println!("{}", "=".repeat(60));
            println!("Date: {} | Venue: {}",
                boxscore.game_date,
                boxscore.venue.default
            );
            println!("Status: {} | Period: {}",
                boxscore.game_state,
                boxscore.period_descriptor.number
            );
            if boxscore.clock.running || !boxscore.clock.in_intermission {
                println!("Time: {}", boxscore.clock.time_remaining);
            }

            // Display score
            println!("\n{:<20} {:>3}", "Team", "Score");
            println!("{}", "-".repeat(25));
            println!("{:<20} {:>3}",
                boxscore.away_team.abbrev,
                boxscore.away_team.score
            );
            println!("{:<20} {:>3}",
                boxscore.home_team.abbrev,
                boxscore.home_team.score
            );

            // Display shots on goal
            println!("\n{:<20} {:>3}", "Team", "SOG");
            println!("{}", "-".repeat(25));
            println!("{:<20} {:>3}",
                boxscore.away_team.abbrev,
                boxscore.away_team.sog
            );
            println!("{:<20} {:>3}",
                boxscore.home_team.abbrev,
                boxscore.home_team.sog
            );

            // Display player stats - Away Team
            println!("\n{} - Forwards", boxscore.away_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
            );
            for player in &boxscore.player_by_game_stats.away_team.forwards {
                println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                    player.sweater_number,
                    player.name.default,
                    player.position,
                    player.goals,
                    player.assists,
                    player.points,
                    player.plus_minus,
                    player.toi
                );
            }

            println!("\n{} - Defense", boxscore.away_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
            );
            for player in &boxscore.player_by_game_stats.away_team.defense {
                println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                    player.sweater_number,
                    player.name.default,
                    player.position,
                    player.goals,
                    player.assists,
                    player.points,
                    player.plus_minus,
                    player.toi
                );
            }

            println!("\n{} - Goalies", boxscore.away_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}",
                "#", "Name", "SA", "Saves", "GA", "SV%"
            );
            for goalie in &boxscore.player_by_game_stats.away_team.goalies {
                let sv_pct = goalie.save_pctg
                    .map(|p| format!("{:.3}", p))
                    .unwrap_or_else(|| "-".to_string());
                println!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}",
                    goalie.sweater_number,
                    goalie.name.default,
                    goalie.shots_against,
                    goalie.saves,
                    goalie.goals_against,
                    sv_pct
                );
            }

            // Display player stats - Home Team
            println!("\n{} - Forwards", boxscore.home_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
            );
            for player in &boxscore.player_by_game_stats.home_team.forwards {
                println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                    player.sweater_number,
                    player.name.default,
                    player.position,
                    player.goals,
                    player.assists,
                    player.points,
                    player.plus_minus,
                    player.toi
                );
            }

            println!("\n{} - Defense", boxscore.home_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                "#", "Name", "Pos", "G", "A", "P", "+/-", "TOI"
            );
            for player in &boxscore.player_by_game_stats.home_team.defense {
                println!("{:<3} {:<20} {:<4} {:>3} {:>3} {:>3} {:>4} {:>6}",
                    player.sweater_number,
                    player.name.default,
                    player.position,
                    player.goals,
                    player.assists,
                    player.points,
                    player.plus_minus,
                    player.toi
                );
            }

            println!("\n{} - Goalies", boxscore.home_team.abbrev);
            println!("{}", "-".repeat(80));
            println!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}",
                "#", "Name", "SA", "Saves", "GA", "SV%"
            );
            for goalie in &boxscore.player_by_game_stats.home_team.goalies {
                let sv_pct = goalie.save_pctg
                    .map(|p| format!("{:.3}", p))
                    .unwrap_or_else(|| "-".to_string());
                println!("{:<3} {:<20} {:>4} {:>6} {:>6} {:>6}",
                    goalie.sweater_number,
                    goalie.name.default,
                    goalie.shots_against,
                    goalie.saves,
                    goalie.goals_against,
                    sv_pct
                );
            }
        }
        Commands::Schedule { date } => {
            let game_date = if let Some(date_str) = date {
                // Parse date string
                let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date format. Use YYYY-MM-DD");
                GameDate::Date(parsed_date)
            } else {
                // Use today's date
                GameDate::today()
            };

            let schedule = client.daily_schedule(Some(&game_date)).await.unwrap();

            // Display schedule header
            println!("\nNHL Schedule - {}", schedule.date);
            println!("{}", "=".repeat(80));

            if schedule.number_of_games == 0 {
                println!("No games scheduled for this date.");
            } else {
                println!("Games: {}\n", schedule.number_of_games);

                // Display each game
                for game in &schedule.games {
                    println!("Game ID: {}", game.id);
                    println!("  {} @ {}",
                        game.away_team.abbrev,
                        game.home_team.abbrev
                    );
                    println!("  Time: {} (UTC)", game.start_time_utc);
                    println!("  Status: {}", game.game_state);

                    // Display scores if available
                    if let (Some(away_score), Some(home_score)) = (game.away_team.score, game.home_team.score) {
                        println!("  Score: {} - {}", away_score, home_score);
                    }
                    println!();
                }
            }

            // Display navigation info
            if let Some(prev) = schedule.previous_start_date {
                println!("Previous date with games: {}", prev);
            }
            if let Some(next) = schedule.next_start_date {
                println!("Next date with games: {}", next);
            }
        }
    }
}
