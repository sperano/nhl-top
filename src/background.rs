use nhl_api::{Client, DailySchedule};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::{Duration, SystemTime};
use futures::future::join_all;
use chrono::Datelike;
use crate::{SharedDataHandle, commands};
use crate::cache::{
    fetch_standings_cached, fetch_schedule_cached, fetch_game_cached,
    fetch_boxscore_cached, fetch_club_stats_cached, fetch_player_info_cached,
};

/// Fetch league standings and update shared state
pub async fn fetch_standings(client: &Client, shared_data: &SharedDataHandle) {
    match fetch_standings_cached(client).await {
        Ok(data) => {
            let mut shared = shared_data.write().await;
            shared.standings = Arc::new(data);
            shared.last_refresh = Some(SystemTime::now());
            // Clear only error messages on successful refresh
            if shared.status_is_error {
                shared.clear_status();
            }
        }
        Err(e) => {
            let mut shared = shared_data.write().await;
            shared.set_error(format!("Failed to fetch standings: {}", e));
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
        let game_id = game.id;
        let game_clone = (*game).clone();
        async move {
            let result = fetch_game_cached(client, game_id).await;
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

    match fetch_schedule_cached(client, date).await {
        Ok(schedule) => {
            // Fetch all started games in parallel
            let game_info = fetch_all_started_games(client, &schedule).await;

            // Extract period scores from fetched game data
            let period_scores = extract_game_data(&game_info);

            // Update shared state with all data
            let mut shared = shared_data.write().await;
            shared.schedule = Arc::new(Some(schedule));
            shared.period_scores = Arc::new(period_scores);
            shared.game_info = Arc::new(game_info);
            // Clear only error messages on successful refresh
            if shared.status_is_error {
                shared.clear_status();
            }
        }
        Err(e) => {
            let mut shared = shared_data.write().await;
            shared.set_error(format!("Failed to fetch schedule: {}", e));
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
            shared.boxscore = Arc::new(None);
        }

        // Fetch the boxscore
        match fetch_boxscore_cached(client, game_id).await {
            Ok(boxscore) => {
                let mut shared = shared_data.write().await;
                shared.boxscore = Arc::new(Some(boxscore));
                shared.boxscore_loading = false;
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.boxscore_loading = false;
                shared.set_error(format!("Failed to fetch boxscore: {}", e));
            }
        }
    }
}

/// Fetch player info for selected player
async fn fetch_player_info(client: &Client, shared_data: &SharedDataHandle) {
    // Get the selected player ID
    let selected_player_id = {
        let shared = shared_data.read().await;
        shared.selected_player_id
    };

    if let Some(player_id) = selected_player_id {
        // Check if we already have the data cached
        let already_cached = {
            let shared = shared_data.read().await;
            shared.player_info.contains_key(&player_id)
        };

        if already_cached {
            return;
        }

        // Set loading state
        {
            let mut shared = shared_data.write().await;
            shared.player_info_loading = true;
        }

        // Fetch the player info
        match fetch_player_info_cached(client, player_id).await {
            Ok(info) => {
                let mut shared = shared_data.write().await;
                let mut new_player_info = (*shared.player_info).clone();
                new_player_info.insert(player_id, info);
                shared.player_info = Arc::new(new_player_info);
                shared.player_info_loading = false;
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.player_info_loading = false;
                shared.set_error(format!("Failed to fetch player info: {}", e));
            }
        }
    }
}

/// Fetch club stats for selected team
async fn fetch_club_stats(client: &Client, shared_data: &SharedDataHandle) {
    // Get the selected team abbreviation
    let selected_team_abbrev = {
        let shared = shared_data.read().await;
        shared.selected_team_abbrev.clone()
    };

    if let Some(team_abbrev) = selected_team_abbrev {
        // Check if we already have the data cached
        let already_cached = {
            let shared = shared_data.read().await;
            shared.club_stats.contains_key(&team_abbrev)
        };

        if already_cached {
            return;
        }

        // Set loading state
        {
            let mut shared = shared_data.write().await;
            shared.club_stats_loading = true;
        }

        // Determine current season (format: 20242025)
        let now = chrono::Local::now();
        let year = now.year();
        let season = if now.month() >= 9 {
            // September onwards is next season (e.g., Sept 2024 = 20242025 season)
            year * 10000 + (year + 1)
        } else {
            // Before September is current season (e.g., Jan 2025 = 20242025 season)
            (year - 1) * 10000 + year
        };

        // Fetch the club stats
        match fetch_club_stats_cached(client, &team_abbrev, season).await {
            Ok(stats) => {
                let mut shared = shared_data.write().await;
                let mut new_club_stats = (*shared.club_stats).clone();
                new_club_stats.insert(team_abbrev, stats);
                shared.club_stats = Arc::new(new_club_stats);
                shared.club_stats_loading = false;
            }
            Err(e) => {
                let mut shared = shared_data.write().await;
                shared.club_stats_loading = false;
                shared.set_error(format!("Failed to fetch club stats: {}", e));
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

        // Fetch club stats if a team is selected
        fetch_club_stats(&client, &shared_data).await;

        // Fetch player info if a player is selected
        fetch_player_info(&client, &shared_data).await;

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
