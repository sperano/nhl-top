# Navigation Framework

A generic, reusable navigation system for drill-down views with breadcrumb trails.

## Overview

This framework provides:
- **Stack-based navigation**: Like browser history, navigate forward and backward
- **Breadcrumb trails**: Visual representation of navigation path
- **Type-safe panel definitions**: Define your own panel types
- **Data caching**: Cache data per panel to avoid re-fetching
- **Separation of concerns**: Navigation logic separate from UI rendering

## Core Concepts

### Panel
A `Panel` represents a navigable screen/view. You define your own panel enum:

```rust
use crate::tui::navigation::Panel;

#[derive(Clone, Debug, PartialEq)]
enum StandingsPanel {
    TeamDetail {
        team_id: i64,
        team_name: String,
    },
    PlayerDetail {
        player_id: i64,
        player_name: String,
    },
    TeamHistory {
        team_id: i64,
        team_name: String,
        season: String,
    },
}

impl Panel for StandingsPanel {
    fn breadcrumb_label(&self) -> String {
        match self {
            StandingsPanel::TeamDetail { team_name, .. } => team_name.clone(),
            StandingsPanel::PlayerDetail { player_name, .. } => player_name.clone(),
            StandingsPanel::TeamHistory { team_name, season, .. } =>
                format!("{} ({})", team_name, season),
        }
    }

    fn cache_key(&self) -> String {
        match self {
            StandingsPanel::TeamDetail { team_id, .. } => format!("team:{}", team_id),
            StandingsPanel::PlayerDetail { player_id, .. } => format!("player:{}", player_id),
            StandingsPanel::TeamHistory { team_id, season, .. } =>
                format!("team:{}:season:{}", team_id, season),
        }
    }
}
```

### NavigationStack
Manages the stack of panels:

```rust
use crate::tui::navigation::NavigationStack;

let mut nav = NavigationStack::new();

// Navigate forward
nav.push(StandingsPanel::TeamDetail {
    team_id: 1,
    team_name: "Canadiens".into(),
});

nav.push(StandingsPanel::PlayerDetail {
    player_id: 42,
    player_name: "Anderson".into(),
});

// Get current panel
if let Some(current) = nav.current() {
    println!("Current: {:?}", current);
}

// Get breadcrumb trail
let breadcrumb = nav.breadcrumb_string(" >> ");
// Result: "Canadiens >> Anderson"

// Navigate backward
nav.pop(); // Back to Canadiens

// Jump to specific depth
nav.go_to_depth(1); // Back to root level
```

### NavigationDataCache
Cache data associated with panels:

```rust
use crate::tui::navigation::NavigationDataCache;

#[derive(Clone)]
struct TeamData {
    players: Vec<String>,
    stats: HashMap<String, i64>,
}

let mut cache: NavigationDataCache<String, TeamData> = NavigationDataCache::new();

// Store data
cache.insert("team:1".into(), TeamData {
    players: vec!["Player1".into(), "Player2".into()],
    stats: HashMap::new(),
});

// Retrieve data
if let Some(data) = cache.get(&"team:1".into()) {
    println!("Players: {:?}", data.players);
}

// Check if data exists
if cache.contains_key(&"team:1".into()) {
    println!("Data cached!");
}
```

### NavigationContext
Combines stack and cache:

```rust
use crate::tui::navigation::NavigationContext;

let mut ctx: NavigationContext<StandingsPanel, String, TeamData> =
    NavigationContext::new();

// Navigate
ctx.navigate_to(StandingsPanel::TeamDetail {
    team_id: 1,
    team_name: "Canadiens".into(),
});

// Cache data for current panel
if let Some(panel) = ctx.stack.current() {
    ctx.data.insert(panel.cache_key(), fetch_team_data());
}

// Go back
ctx.go_back();

// Reset everything
ctx.reset(true); // true = also clear cache
```

## Integration with TUI

### State Structure

Add navigation context to your tab's state:

```rust
use crate::tui::navigation::NavigationContext;

pub struct State {
    // Existing fields...
    pub view: GroupBy,
    pub subtab_focused: bool,

    // Add navigation
    pub navigation: Option<NavigationContext<StandingsPanel, String, PanelData>>,

    // For UI state
    pub scrollable: Scrollable,
}
```

### View Rendering

Use breadcrumb helper in your view:

```rust
use crate::tui::common::breadcrumb::render_breadcrumb_simple;

pub fn render_subtabs(f: &mut Frame, area: Rect, state: &State, selection_fg: Color) {
    let base_style = base_tab_style(state.subtab_focused);

    // Check if navigation is active
    if let Some(nav_ctx) = &state.navigation {
        if !nav_ctx.is_at_root() {
            // Render breadcrumb
            let trail = nav_ctx.stack.breadcrumb_trail();
            render_breadcrumb_simple(f, area, &trail, selection_fg, base_style);
            return;
        }
    }

    // Otherwise render normal subtabs...
}
```

### Handler Logic

```rust
pub fn handle_key(key: KeyEvent, state: &mut State) -> bool {
    // Check if in navigation mode
    if let Some(nav_ctx) = &mut state.navigation {
        if !nav_ctx.is_at_root() {
            match key.code {
                KeyCode::Esc => {
                    // Go back in navigation
                    nav_ctx.go_back();
                    return true;
                }
                KeyCode::Enter => {
                    // Navigate to selected item
                    if let Some(selected) = get_selected_item(state) {
                        nav_ctx.navigate_to(selected);
                    }
                    return true;
                }
                _ => {}
            }
        }
    }

    // Regular handling...
    false
}
```

## Example: Complete Flow

```rust
// User flow: Standings -> Team -> Player -> Team History

// 1. User selects team in standings
ctx.navigate_to(StandingsPanel::TeamDetail {
    team_id: 1,
    team_name: "Canadiens".into(),
});
// Breadcrumb: "Canadiens"

// 2. User selects player
ctx.navigate_to(StandingsPanel::PlayerDetail {
    player_id: 42,
    player_name: "Anderson".into(),
});
// Breadcrumb: "Canadiens >> Anderson"

// 3. User selects team from player's history
ctx.navigate_to(StandingsPanel::TeamDetail {
    team_id: 2,
    team_name: "Columbus".into(),
});
// Breadcrumb: "Canadiens >> Anderson >> Columbus"

// 4. User presses ESC to go back
ctx.go_back();
// Breadcrumb: "Canadiens >> Anderson"

// 5. User presses ESC again
ctx.go_back();
// Breadcrumb: "Canadiens"

// 6. User presses ESC again
ctx.go_back();
// Back to root standings view (no navigation active)
```

## Data Fetching Integration

The navigation system works with background data fetching:

```rust
// In SharedData (main.rs)
pub struct SharedData {
    // Existing fields...

    // Add navigation-specific cached data
    pub team_details: Arc<HashMap<i64, TeamDetail>>,
    pub player_stats: Arc<HashMap<i64, PlayerStats>>,
}

// In background fetch loop
async fn fetch_data_loop(client: Client, shared_data: SharedDataHandle) {
    loop {
        // Check if navigation requires data
        let data = shared_data.read().await;

        // Fetch team details if needed
        if let Some(team_id) = get_requested_team_id(&data) {
            if let Ok(team_detail) = client.team_detail(team_id).await {
                let mut data = shared_data.write().await;
                data.team_details.insert(team_id, team_detail);
            }
        }

        // Similar for player stats, etc.
    }
}
```

## Best Practices

1. **Keep panels immutable**: Once created, panel data shouldn't change
2. **Use cache_key() wisely**: Make it unique but consistent for the same logical panel
3. **Clear cache strategically**: Clear when data becomes stale, keep when possible
4. **Render breadcrumbs clearly**: Use clear separators and colors
5. **Handle empty stack**: Always check `is_at_root()` before accessing navigation

## Testing

The framework includes comprehensive tests. See `src/tui/navigation.rs` for examples.

```bash
cargo test tui::navigation
```
