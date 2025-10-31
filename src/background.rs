use nhl_api::{Client, DailySchedule};
use std::collections::HashMap;
use tokio::sync::mpsc;
use std::time::{Duration, SystemTime};
use futures::future::join_all;
use crate::{SharedDataHandle, commands};

/// Fetch league standings and update shared state
pub async fn fetch_standings(client: &Client, shared_data: &SharedDataHandle) {
    match client.current_league_standings().await {
        Ok(data) => {
            let mut shared = shared_data.write().await;
            shared.standings = data;
            shared.last_refresh = Some(SystemTime::now());
            shared.error_message = None; // Clear any previous errors
        }
        Err(e) => {
            let mut shared = shared_data.write().await;
            shared.error_message = Some(format!("Failed to fetch standings: {}", e));
        }
    }
}

/// Fetch landing data for all started games in parallel
async fn fetch_all_started_games(client: &Client, schedule: &DailySchedule) -> HashMap<i64, nhl_api::GameMatchup> {
    let mut game_info = HashMap::new();

    // Filter games that have started
    let games_to_fetch: Vec<_> = schedule.games.iter()
        .filter(|game| game.game_state.has_started())
        .collect();

    // Create futures for all landing requests
    let fetch_futures = games_to_fetch.iter().map(|game| {
        let game_id = nhl_api::GameId::new(game.id);
        let game_clone = (*game).clone();
        async move {
            let result = client.landing(&game_id).await;
            (game_clone.id, result)
        }
    });

    // Execute all requests in parallel
    let results = join_all(fetch_futures).await;

    // Collect successful results
    for (id, result) in results {
        if let Ok(landing) = result {
            game_info.insert(id, landing);
        }
        // Silently skip failed individual game fetches
    }

    game_info
}

/// Extract period scores from game matchup data
fn extract_game_data(game_info: &HashMap<i64, nhl_api::GameMatchup>) -> HashMap<i64, commands::scores_format::PeriodScores> {
    let mut period_scores = HashMap::new();

    for (game_id, matchup) in game_info {
        if let Some(summary) = &matchup.summary {
            let scores = commands::scores_format::extract_period_scores(summary);
            period_scores.insert(*game_id, scores);
        }
    }

    period_scores
}

/// Fetch daily schedule and all game details in parallel
pub async fn fetch_schedule_with_games(client: &Client, shared_data: &SharedDataHandle) {
    // Get current game date
    let date = {
        let shared = shared_data.read().await;
        shared.game_date.clone()
    };

    match client.daily_schedule(Some(date)).await {
        Ok(schedule) => {
            // Fetch all started games in parallel
            let game_info = fetch_all_started_games(client, &schedule).await;

            // Extract period scores from fetched game data
            let period_scores = extract_game_data(&game_info);

            // Update shared state with all data
            let mut shared = shared_data.write().await;
            shared.schedule = Some(schedule);
            shared.period_scores = period_scores;
            shared.game_info = game_info;
            shared.error_message = None; // Clear errors on successful fetch
        }
        Err(e) => {
            let mut shared = shared_data.write().await;
            shared.error_message = Some(format!("Failed to fetch schedule: {}", e));
        }
    }
}

/// Fetch boxscore for selected game
async fn fetch_boxscore(client: &Client, shared_data: &SharedDataHandle) {
    // Get the selected game ID
    let selected_game_id = {
        let shared = shared_data.read().await;
        shared.selected_game_id
    };

    if let Some(game_id) = selected_game_id {
        // Set loading state
        {
            let mut shared = shared_data.write().await;
            shared.boxscore_loading = true;
            shared.boxscore = None;
        }

        // Fetch the boxscore
        match client.boxscore(&nhl_api::GameId::new(game_id)).await {
            Ok(boxscore) => {
                let mut shared = shared_data.write().await;
                shared.boxscore = Some(boxscore);
                shared.boxscore_loading = false;
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.boxscore_loading = false;
                shared.error_message = Some(format!("Failed to fetch boxscore: {}", e));
            }
        }
    }
}

/// Background task loop that periodically fetches NHL data
pub async fn fetch_data_loop(client: Client, shared_data: SharedDataHandle, interval: u64, mut refresh_rx: mpsc::Receiver<()>) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
    interval_timer.tick().await; // First tick completes immediately

    loop {
        // Fetch standings and schedule with game details
        fetch_standings(&client, &shared_data).await;
        fetch_schedule_with_games(&client, &shared_data).await;

        // Fetch boxscore if a game is selected
        fetch_boxscore(&client, &shared_data).await;

        // Wait for either the interval timer or a manual refresh signal
        tokio::select! {
            _ = interval_timer.tick() => {
                // Regular interval refresh
            }
            _ = refresh_rx.recv() => {
                // Manual refresh triggered
            }
        }
    }
}
