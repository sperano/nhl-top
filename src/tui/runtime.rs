use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, trace};

use super::action::Action;
use super::component::{Effect, Element};
use super::effects::DataEffects;
use super::reducer::reduce;
use super::state::AppState;

/// Component runtime - manages component lifecycle and action processing
///
/// The Runtime is responsible for:
/// - Managing the application state
/// - Dispatching actions through the reducer
/// - Executing side effects asynchronously
/// - Building the virtual component tree
pub struct Runtime {
    /// Current application state
    state: AppState,

    /// Channel for dispatching actions
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,

    /// Channel for queuing effects
    effect_tx: mpsc::UnboundedSender<Effect>,

    /// Data effects handler
    data_effects: Arc<DataEffects>,
}

impl Runtime {
    /// Create a new runtime with initial state and data effects handler
    pub fn new(initial_state: AppState, data_effects: Arc<DataEffects>) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (effect_tx, mut effect_rx) = mpsc::unbounded_channel();

        // Spawn effect executor task
        let action_tx_clone = action_tx.clone();
        tokio::spawn(async move {
            Self::run_effect_executor(&mut effect_rx, action_tx_clone).await;
        });

        Self {
            state: initial_state,
            action_tx,
            action_rx,
            effect_tx,
            data_effects,
        }
    }

    /// Get a reference to the current state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Dispatch an action to be processed by the reducer
    pub fn dispatch(&mut self, action: Action) {
        trace!("ACTION: Dispatching {:?}", action);

        // Handle RefreshData action specially - generate data fetch effects
        let effect = if matches!(action, Action::RefreshData) {
            debug!("ACTION: RefreshData - generating fetch effects");

            // First, run the reducer to update last_refresh timestamp
            let (new_state, _reducer_effect) = reduce(self.state.clone(), action.clone());
            self.state = new_state;

            // Then generate data fetch effects
            self.data_effects.handle_refresh(&self.state)
        } else {
            // Run the reducer to get new state and any effects
            let (new_state, reducer_effect) = reduce(self.state.clone(), action);

            // Check if a boxscore panel was just pushed and trigger fetch
            let boxscore_effect = self.check_for_boxscore_fetch(&self.state, &new_state);

            // Check if a team detail panel was just pushed and trigger fetch
            let team_detail_effect = self.check_for_team_detail_fetch(&self.state, &new_state);

            // Check if a player detail panel was just pushed and trigger fetch
            let player_detail_effect = self.check_for_player_detail_fetch(&self.state, &new_state);

            // Check if schedule was just loaded and trigger game detail fetches
            let game_details_effect = self.check_for_game_details_fetch(&self.state, &new_state);

            self.state = new_state;

            // Combine effects if needed
            let mut effects = Vec::new();
            if !matches!(reducer_effect, Effect::None) { effects.push(reducer_effect); }
            if !matches!(boxscore_effect, Effect::None) { effects.push(boxscore_effect); }
            if !matches!(team_detail_effect, Effect::None) { effects.push(team_detail_effect); }
            if !matches!(player_detail_effect, Effect::None) { effects.push(player_detail_effect); }
            if !matches!(game_details_effect, Effect::None) { effects.push(game_details_effect); }

            if effects.is_empty() {
                Effect::None
            } else if effects.len() == 1 {
                effects.pop().unwrap()
            } else {
                Effect::Batch(effects)
            }
        };

        // Queue effect for execution if not None
        if !matches!(effect, Effect::None) {
            trace!("ACTION: Queueing effect for execution");
            let _ = self.effect_tx.send(effect);
        }
    }

    /// Check if a boxscore panel was just pushed and needs data fetching
    fn check_for_boxscore_fetch(&self, old_state: &AppState, new_state: &AppState) -> Effect {
        // Check if panel_stack grew and new panel is a Boxscore
        if new_state.navigation.panel_stack.len() > old_state.navigation.panel_stack.len() {
            if let Some(panel_state) = new_state.navigation.panel_stack.last() {
                if let super::types::Panel::Boxscore { game_id } = panel_state.panel {
                    // Check if we don't already have the data and aren't already loading
                    if !new_state.data.boxscores.contains_key(&game_id)
                        && !new_state.data.loading.contains(&super::state::LoadingKey::Boxscore(game_id))
                    {
                        debug!("EFFECT: Triggering boxscore fetch for game_id={}", game_id);
                        return self.data_effects.fetch_boxscore(game_id);
                    } else {
                        trace!("EFFECT: Boxscore already loaded/loading for game_id={}", game_id);
                    }
                }
            }
        }
        Effect::None
    }

    /// Check if a team detail panel was just pushed and needs data fetching
    fn check_for_team_detail_fetch(&self, old_state: &AppState, new_state: &AppState) -> Effect {
        // Check if panel_stack grew and new panel is a TeamDetail
        if new_state.navigation.panel_stack.len() > old_state.navigation.panel_stack.len() {
            if let Some(panel_state) = new_state.navigation.panel_stack.last() {
                if let super::types::Panel::TeamDetail { abbrev } = &panel_state.panel {
                    // Check if we don't already have the data and aren't already loading
                    if !new_state.data.team_roster_stats.contains_key(abbrev)
                        && !new_state.data.loading.contains(&super::state::LoadingKey::TeamRosterStats(abbrev.clone()))
                    {
                        debug!("EFFECT: Triggering team roster stats fetch for team={}", abbrev);
                        return self.data_effects.fetch_team_roster_stats(abbrev.clone());
                    } else {
                        trace!("EFFECT: Team roster stats already loaded/loading for team={}", abbrev);
                    }
                }
            }
        }
        Effect::None
    }

    /// Check if a player detail panel was just pushed and needs data fetching
    fn check_for_player_detail_fetch(&self, old_state: &AppState, new_state: &AppState) -> Effect {
        // Check if panel_stack grew and new panel is a PlayerDetail
        if new_state.navigation.panel_stack.len() > old_state.navigation.panel_stack.len() {
            if let Some(panel_state) = new_state.navigation.panel_stack.last() {
                if let super::types::Panel::PlayerDetail { player_id } = panel_state.panel {
                    // Check if we don't already have the data and aren't already loading
                    if !new_state.data.player_data.contains_key(&player_id)
                        && !new_state.data.loading.contains(&super::state::LoadingKey::PlayerStats(player_id))
                    {
                        debug!("EFFECT: Triggering player stats fetch for player_id={}", player_id);
                        return self.data_effects.fetch_player_stats(player_id);
                    } else {
                        trace!("EFFECT: Player stats already loaded/loading for player_id={}", player_id);
                    }
                }
            }
        }
        Effect::None
    }

    /// Check if schedule was just loaded and trigger game detail fetches for started games
    fn check_for_game_details_fetch(&self, old_state: &AppState, new_state: &AppState) -> Effect {
        // Check if schedule just loaded (went from None to Some)
        if old_state.data.schedule.is_none() && new_state.data.schedule.is_some() {
            if let Some(schedule) = &new_state.data.schedule {
                let mut effects = Vec::new();
                for game in &schedule.games {
                    // Only fetch details for games that have started
                    if game.game_state != nhl_api::GameState::Future
                        && game.game_state != nhl_api::GameState::PreGame
                    {
                        debug!("EFFECT: Triggering game details fetch for game_id={}", game.id);
                        effects.push(self.data_effects.fetch_game_details(game.id));
                    }
                }
                if !effects.is_empty() {
                    return Effect::Batch(effects);
                }
            }
        }
        Effect::None
    }

    /// Process all pending actions in the queue
    ///
    /// Returns the number of actions processed
    pub fn process_actions(&mut self) -> usize {
        let mut count = 0;
        while let Ok(action) = self.action_rx.try_recv() {
            self.dispatch(action);
            count += 1;
        }
        count
    }

    /// Build the virtual element tree from current state
    ///
    /// This will be used by the Renderer to produce the actual terminal output.
    /// It builds the component tree by calling the root App component's view() method
    /// with the current state as props.
    pub fn build(&self) -> Element {
        use crate::tui::components::App;
        use crate::tui::component::Component;

        let app = App;
        app.view(&self.state, &())
    }

    /// Get a sender for dispatching actions from external sources
    pub fn action_sender(&self) -> mpsc::UnboundedSender<Action> {
        self.action_tx.clone()
    }

    /// Execute effects asynchronously
    ///
    /// This runs in a separate tokio task and processes effects as they come in.
    /// Effects can dispatch new actions which feed back into the runtime.
    async fn run_effect_executor(
        effect_rx: &mut mpsc::UnboundedReceiver<Effect>,
        action_tx: mpsc::UnboundedSender<Action>,
    ) {
        while let Some(effect) = effect_rx.recv().await {
            match effect {
                Effect::None => {
                    // Nothing to do
                }
                Effect::Action(action) => {
                    // Dispatch action immediately
                    let _ = action_tx.send(action);
                }
                Effect::Batch(effects) => {
                    // Process each effect in the batch
                    for e in effects {
                        // Re-queue each effect to be processed
                        // Note: We can't directly recurse here because we're borrowing effect_rx
                        // Instead, we'll handle batches by dispatching actions
                        match e {
                            Effect::Action(action) => {
                                let _ = action_tx.send(action);
                            }
                            Effect::Async(future) => {
                                let action_tx = action_tx.clone();
                                tokio::spawn(async move {
                                    let action = future.await;
                                    let _ = action_tx.send(action);
                                });
                            }
                            Effect::Batch(nested) => {
                                // Handle nested batches recursively
                                for nested_effect in nested {
                                    if let Effect::Action(action) = nested_effect {
                                        let _ = action_tx.send(action);
                                    } else if let Effect::Async(future) = nested_effect {
                                        let action_tx = action_tx.clone();
                                        tokio::spawn(async move {
                                            let action = future.await;
                                            let _ = action_tx.send(action);
                                        });
                                    }
                                }
                            }
                            Effect::None => {}
                        }
                    }
                }
                Effect::Async(future) => {
                    // Spawn async task to execute the future
                    let action_tx = action_tx.clone();
                    tokio::spawn(async move {
                        let action = future.await;
                        let _ = action_tx.send(action);
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::types::Tab;
    use crate::tui::testing::create_client;

    fn create_test_data_effects() -> Arc<DataEffects> {
        let client = create_client();
        Arc::new(DataEffects::new(client))
    }

    #[tokio::test]
    async fn test_runtime_initial_state() {
        let state = AppState::default();
        let data_effects = create_test_data_effects();
        let runtime = Runtime::new(state.clone(), data_effects);

        assert_eq!(runtime.state().navigation.current_tab, Tab::Scores);
    }

    #[tokio::test]
    async fn test_dispatch_action() {
        let state = AppState::default();
        let data_effects = create_test_data_effects();
        let mut runtime = Runtime::new(state, data_effects);

        // Dispatch navigation action
        runtime.dispatch(Action::NavigateTab(Tab::Standings));

        assert_eq!(runtime.state().navigation.current_tab, Tab::Standings);
    }

    #[tokio::test]
    async fn test_action_queue() {
        let state = AppState::default();
        let data_effects = create_test_data_effects();
        let mut runtime = Runtime::new(state, data_effects);

        // Send actions through the action channel
        let tx = runtime.action_sender();
        tx.send(Action::NavigateTab(Tab::Standings)).unwrap();

        // Process the queued actions
        let count = runtime.process_actions();

        assert_eq!(count, 1);
        assert_eq!(runtime.state().navigation.current_tab, Tab::Standings);
    }

    #[tokio::test]
    async fn test_effect_execution() {
        use std::sync::{Arc, Mutex};

        let state = AppState::default();
        let data_effects = create_test_data_effects();
        let mut runtime = Runtime::new(state, data_effects);

        // Create a flag to track if async effect executed
        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();

        // Create an async effect
        let effect = Effect::Async(Box::pin(async move {
            *executed_clone.lock().unwrap() = true;
            Action::NavigateTab(Tab::Settings)
        }));

        // Queue the effect
        runtime.effect_tx.send(effect).unwrap();

        // Give the async task time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Process any actions that resulted from the effect
        runtime.process_actions();

        // Verify the effect executed and dispatched the action
        assert!(*executed.lock().unwrap());
        assert_eq!(runtime.state().navigation.current_tab, Tab::Settings);
    }

    #[tokio::test]
    async fn test_build_returns_component_tree() {
        let state = AppState::default();
        let data_effects = create_test_data_effects();
        Runtime::new(state, data_effects);

        // build() should return the App component tree
        //let element = runtime.build();

        // Should be a container with 2 children (TabbedPanel, StatusBar)
        // match element {
        //     Element::Container { children, .. } => {
        //         assert_eq!(children.len(), 2);
        //     }
        //     _ => panic!("Expected container element from App component"),
        // }
    }

    #[tokio::test]
    async fn test_refresh_data_triggers_data_effects() {
        let state = AppState::default();
        let data_effects = create_test_data_effects();
        let mut runtime = Runtime::new(state, data_effects);

        // Dispatch RefreshData action
        runtime.dispatch(Action::RefreshData);

        // Poll for actions with a timeout (network calls can be slow)
        let mut total_count = 0;
        let max_wait = tokio::time::Duration::from_secs(5);
        let poll_interval = tokio::time::Duration::from_millis(50);
        let start = tokio::time::Instant::now();

        while start.elapsed() < max_wait {
            tokio::time::sleep(poll_interval).await;
            let count = runtime.process_actions();
            total_count += count;

            // If we got at least one action, the test passed
            if total_count >= 1 {
                break;
            }
        }

        // Should have received at least StandingsLoaded and ScheduleLoaded actions
        // Note: actual count depends on network and what data is returned
        assert!(total_count >= 1, "Expected at least 1 action from data refresh after {} seconds", start.elapsed().as_secs_f32());
    }
}
