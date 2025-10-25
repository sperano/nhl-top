use nhl_api::{Client, GameDate, GameId, Boxscore, GameClock, PeriodDescriptor};
use chrono::NaiveDate;

pub async fn run(client: &Client, date: Option<String>) {
    let game_date = if let Some(date_str) = date {
        let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .expect("Invalid date format. Use YYYY-MM-DD");
        GameDate::Date(parsed_date)
    } else {
        GameDate::today()
    };

    let schedule = client.daily_schedule(Some(&game_date)).await.unwrap();

    // Display header
    println!("\n{}", "═".repeat(90));
    println!("NHL SCORES - {}", schedule.date);
    println!("{}\n", "═".repeat(90));

    if schedule.number_of_games == 0 {
        println!("No games scheduled for this date.\n");
        return;
    }

    // Process each game
    for (i, game) in schedule.games.iter().enumerate() {
        if i > 0 {
            println!();
        }

        // Determine if game has started
        let game_started = game.game_state.has_started();

        if game_started {
            // Fetch detailed boxscore for period information
            let game_id = GameId::new(game.id);
            match client.boxscore(&game_id).await {
                Ok(boxscore) => {
                    display_detailed_score(&boxscore, game.game_state);
                }
                Err(_) => {
                    // Fall back to simple display if boxscore unavailable
                    display_simple_score(game);
                }
            }
        } else {
            // Game hasn't started yet
            display_simple_score(game);
        }
    }

    println!();
}

fn display_detailed_score(boxscore: &Boxscore, game_state: nhl_api::GameState) {
    let away_abbrev = &boxscore.away_team.abbrev;
    let home_abbrev = &boxscore.home_team.abbrev;
    let away_score = boxscore.away_team.score;
    let home_score = boxscore.home_team.score;

    // Box top
    println!("┌{:─<88}┐", "");

    // Teams and final score
    println!("│ {:<15} {:>2}     FINAL     {:>2}  {:<15}                                │",
        away_abbrev, away_score, home_score, home_abbrev);

    // Game status line
    let status_text = format_game_status(boxscore.game_state, &boxscore.period_descriptor.number, &boxscore.clock);
    println!("│ {:<86} │", status_text);

    println!("├{:─<88}┤", "");

    // Period-by-period header
    print!("│ {:<15}   ", "");
    print!("{:^5}", "1");
    print!("{:^5}", "2");
    print!("{:^5}", "3");

    // Check if there were overtime/shootout periods
    let max_period = boxscore.period_descriptor.number;
    if max_period > 3 {
        print!("{:^5}", "OT");
    }
    if max_period > 4 {
        print!("{:^5}", "SO");
    }
    print!("{:^7}", "T");
    println!("                                    │");

    print!("│ {:<15}   {:─<5}{:─<5}{:─<5}", "", "", "", "");
    if max_period > 3 {
        print!("{:─<5}", "");
    }
    if max_period > 4 {
        print!("{:─<5}", "");
    }
    println!("{:─<7}                                    │", "");

    // Get period scores from linescore if available
    // Note: The nhl_api crate may or may not have linescore data
    // We'll display what we can
    display_period_line(away_abbrev, away_score, max_period);
    display_period_line(home_abbrev, home_score, max_period);

    // Box bottom
    println!("└{:─<88}┘", "");
}

fn display_period_line(team_abbrev: &str, total_score: i32, max_period: i32) {
    print!("│ {:<15}   ", team_abbrev);

    // For now, we'll show placeholders for period scores
    // The nhl_api crate's Boxscore might not include detailed linescore
    // We'd need to check the actual structure
    for _ in 1..=3 {
        print!("{:^5}", "-");
    }

    if max_period > 3 {
        print!("{:^5}", "-");
    }
    if max_period > 4 {
        print!("{:^5}", "-");
    }

    print!("{:^7}", total_score);
    println!("                                    │");
}

fn display_simple_score(game: &nhl_api::ScheduleGame) {
    println!("┌{:─<88}┐", "");

    if let (Some(away_score), Some(home_score)) = (game.away_team.score, game.home_team.score) {
        println!("│ {:<15} {:>2}           {:>2}  {:<15}                                    │",
            game.away_team.abbrev, away_score, home_score, game.home_team.abbrev);
    } else {
        println!("│ {:<15}  @  {:<15}                                                │",
            game.away_team.abbrev, game.home_team.abbrev);
    }

    let status = if game.game_state.is_scheduled() {
        format!("Scheduled: {}", game.start_time_utc)
    } else {
        format!("Status: {}", game.game_state)
    };
    println!("│ {:<86} │", status);

    println!("└{:─<88}┘", "");
}

fn format_game_status(state: nhl_api::GameState, period: &i32, clock: &GameClock) -> String {
    use nhl_api::GameState;

    match state {
        GameState::Final | GameState::Off => "FINAL".to_string(),
        GameState::Live => {
            let period_str = match period {
                1 => "1st",
                2 => "2nd",
                3 => "3rd",
                _ => "OT",
            };

            if clock.in_intermission {
                format!("{} Period - Intermission", period_str)
            } else {
                format!("{} Period - {}", period_str, clock.time_remaining)
            }
        }
        GameState::Future | GameState::PreGame => "Scheduled".to_string(),
        GameState::Postponed => "Postponed".to_string(),
        GameState::Suspended => "Suspended".to_string(),
    }
}
