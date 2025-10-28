use nhl_api::{Client, GameDate, DailySchedule};
use chrono::NaiveDate;

pub fn format_schedule(schedule: &DailySchedule) -> String {
    let mut output = String::new();

    // Display schedule header
    output.push_str(&format!("\nNHL Games - {}\n", schedule.date));
    output.push_str(&format!("{}\n\n", "═".repeat(80)));

    if schedule.number_of_games == 0 {
        output.push_str("No games scheduled for today.\n");
    } else {
        // Display each game in a box
        for (i, game) in schedule.games.iter().enumerate() {
            if i > 0 {
                output.push_str("\n");
            }

            // Game box header
            output.push_str(&format!("┌{:─<78}┐\n", ""));
            output.push_str(&format!("│ {} @ {:66} │\n",
                game.away_team.abbrev,
                game.home_team.abbrev
            ));
            output.push_str(&format!("├{:─<78}┤\n", ""));

            // Game status and time
            output.push_str(&format!("│ Status: {:<70} │\n", game.game_state));
            output.push_str(&format!("│ Time: {:<72} │\n", game.start_time_utc));

            // Display scores if available
            if let (Some(away_score), Some(home_score)) = (game.away_team.score, game.home_team.score) {
                output.push_str(&format!("├{:─<78}┤\n", ""));
                output.push_str(&format!("│ {:<30} {:>3}  -  {:<3} {:>30} │\n",
                    game.away_team.abbrev,
                    away_score,
                    home_score,
                    game.home_team.abbrev
                ));
            } else {
                output.push_str(&format!("│ {:<76} │\n", "Game not started"));
            }

            output.push_str(&format!("└{:─<78}┘\n", ""));
        }
    }

    output
}

pub async fn run(client: &Client, date: Option<String>) {
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
