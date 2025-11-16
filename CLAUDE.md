# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NHL CLI tool written in Rust that displays NHL stats, standings, and live scores. The application supports both command-line mode for quick queries and an interactive TUI (Terminal User Interface) mode built with ratatui.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run in interactive TUI mode (default)
cargo run

# Run with specific commands
cargo run -- standings                    # Current NHL standings by division
cargo run -- standings -s 20232024       # Standings for specific season
cargo run -- standings -d 2024-01-15     # Standings for specific date
cargo run -- standings -b l              # League-wide standings (d=division, c=conference, l=league)
cargo run -- boxscore 2024020001         # Boxscore for specific game
cargo run -- schedule                     # Today's schedule
cargo run -- schedule -d 2024-01-15      # Schedule for specific date
cargo run -- scores                       # Today's scores
cargo run -- scores -d 2024-01-15        # Scores for specific date

# Enable debug mode with verbose NHL API output
cargo run -- --debug standings

# Run performance test example
cargo run --example test_parallel
```

## Architecture

### Two-Mode Design

The application operates in two distinct modes:

1. **CLI Mode**: Invoked when a subcommand is provided. Executes a single command, displays output, and exits.
2. **TUI Mode**: Invoked when no subcommand is provided. Launches an interactive terminal interface with tabs for different views (Scores, Standings, Settings).

The mode is determined in `src/main.rs` by checking if `cli.command.is_none()`.

### Module Structure

#### Core Modules
- **`src/main.rs`**: Entry point with CLI parsing, SharedData struct, and background data fetching loop
- **`src/commands/`**: Each subcommand (standings, boxscore, schedule, scores) as separate modules
- **`src/tui/`**: TUI implementation split into submodules (tabs, widgets, events, mod)
- **`src/config.rs`**: Configuration management using XDG base directories

#### TUI Submodules (Modular Tab-Based Architecture)

The TUI is organized into a modular tab-based architecture where each tab is self-contained:

**Common modules:**
- **`tui/common/tab_bar.rs`**: Renders main navigation tabs with box-drawing characters
- **`tui/common/status_bar.rs`**: Renders status bar with refresh time or error messages
- **`tui/common/mod.rs`**: Common module exports

**Tab modules** (each follows state/view/handler pattern):
- **`tui/scores/`**: Scores tab with date navigation
  - `state.rs`: `State { selected_index, subtab_focused }`
  - `view.rs`: `render_subtabs()` (3-date sliding window), `render_content()` (game scores)
  - `handler.rs`: `handle_key()` for left/right date navigation
  - `mod.rs`: Public exports

- **`tui/standings/`**: Standings tab with view selection
  - `state.rs`: `State { view: GroupBy, subtab_focused }`
  - `view.rs`: `render_subtabs()` (Division/Conference/League), `render_content()` (standings table)
  - `handler.rs`: `handle_key()` for cycling through views
  - `mod.rs`: Public exports

- **`tui/settings/`**: Settings tab (placeholder)
  - `state.rs`: Empty state struct for future settings
  - `view.rs`: Minimal rendering
  - `handler.rs`: No key handling yet
  - `mod.rs`: Public exports

**Core TUI files:**
- **`tui/app.rs`**: Composable `AppState` containing all tab states and `CurrentTab` enum
  - `AppState { current_tab, scores, standings, settings }`
  - Navigation methods: `navigate_tab_left()`, `navigate_tab_right()`, `enter_subtab_mode()`, `exit_subtab_mode()`
  - Helper methods: `is_subtab_focused()`, `has_subtabs()`
- **`tui/mod.rs`**: Main event loop, rendering orchestration, and top-level event dispatcher

### SharedData Architecture

The TUI mode uses a shared state pattern with `Arc<RwLock<SharedData>>`:

```rust
pub struct SharedData {
    pub standings: Vec<Standing>,
    pub schedule: Option<DailySchedule>,
    pub period_scores: HashMap<i64, PeriodScores>,
    pub game_info: HashMap<i64, GameMatchup>,
    pub config: Config,
    pub last_refresh: Option<SystemTime>,
    pub game_date: GameDate,
    pub error_message: Option<String>,
}
```

- Shared between the background data fetching loop and the TUI rendering loop
- Background loop (`fetch_data_loop`) periodically fetches standings and schedule data
- TUI reads from SharedData for rendering
- Uses `mpsc` channel for manual refresh triggers

### Background Data Fetching

The `fetch_data_loop` function runs in a separate tokio task:
- Fetches standings via `client.current_league_standings()`
- Fetches daily schedule via `client.daily_schedule()`
- For started games, fetches game details via `client.landing()` **in parallel** using `futures::future::join_all()`
- Updates SharedData with fetched data
- Stores errors in `SharedData.error_message` for display in status bar
- Responds to manual refresh triggers from the TUI

### TUI Navigation

#### Main Tabs
- Left/Right arrows: Navigate between Scores, Standings, Settings
- Down arrow: Enter subtab mode (on Scores/Standings tabs)
- Up arrow: Exit subtab mode back to main tabs
- ESC: Exit application

#### Scores Subtab Navigation (5-Date Sliding Window)

**CRITICAL: This specification is MANDATORY and must be followed exactly when modifying date navigation code.**

##### Architecture

The window has a **sticky base date** (leftmost date) that only shifts when reaching edges:

- **window_base_date** = game_date - selected_index
- **Window** = `[base, base+1, base+2, base+3, base+4]` (always 5 dates)
- **game_date** = `window_base_date + selected_index` (the date being viewed)
- **selected_index** = position within the window (0-4)

##### Navigation Behavior

**Within Window (index 1-3 → 0-4):**
- **selected_index** changes
- **game_date** changes to match the new date at selected_index
- **Window stays the same** (window_base unchanged)
- **Refresh triggered** every time

**At Left Edge (index = 0, press Left):**
- **selected_index** stays at 0
- **game_date** decrements by 1 day
- **Window shifts left** by 1 day (window_base decrements)
- **Refresh triggered**

**At Right Edge (index = 4, press Right):**
- **selected_index** stays at 4
- **game_date** increments by 1 day
- **Window shifts right** by 1 day (window_base increments)
- **Refresh triggered**

##### Complete Example Sequence (MEMORIZE THIS)

```
Start: game_date=11/02, selected_index=2, refresh
  Window: [10/31, 11/01, 11/02, 11/03, 11/04]

Press Left: selected_index=1, game_date=11/01, refresh
  Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Same window!

Press Left: selected_index=0, game_date=10/31, refresh
  Window: [10/31, 11/01, 11/02, 11/03, 11/04] ← Still same window!

Press Left at edge: game_date=10/30, selected_index=0, refresh
  Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!

Press Left at edge: game_date=10/29, selected_index=0, refresh
  Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Window shifted!

Press Right: game_date=10/30, selected_index=1, refresh
  Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Same window!

Press Right: game_date=10/31, selected_index=2, refresh
  Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Same window!

Press Right: game_date=11/01, selected_index=3, refresh
  Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Same window!

Press Right: game_date=11/02, selected_index=4, refresh
  Window: [10/29, 10/30, 10/31, 11/01, 11/02] ← Same window!

Press Right at edge: game_date=11/03, selected_index=4, refresh
  Window: [10/30, 10/31, 11/01, 11/02, 11/03] ← Window shifted!
```

##### Implementation Files

**View Calculation (`src/tui/scores/view.rs`):**
```rust
fn calculate_date_window(game_date: &GameDate, selected_index: usize) -> [GameDate; 5] {
    let window_base_date = game_date.add_days(-(selected_index as i64));
    [
        window_base_date.add_days(0),
        window_base_date.add_days(1),
        window_base_date.add_days(2),
        window_base_date.add_days(3),
        window_base_date.add_days(4),
    ]
}
```

**Navigation Handler (`src/tui/scores/handler.rs`):**
```rust
async fn navigate_within_window(
    old_index: usize,
    new_index: usize,
    shared_data: &SharedDataHandle,
    refresh_tx: &mpsc::Sender<()>,
) {
    let mut data = shared_data.write().await;
    // Calculate window base: leftmost date in current window
    let window_base = data.game_date.add_days(-(old_index as i64));
    // Update game_date to the new selected position in the same window
    data.game_date = window_base.add_days(new_index as i64);
    clear_schedule_data(&mut data);
    // Triggers refresh
}
```

##### MANDATORY Tests

**When modifying date navigation code, you MUST run:**
```bash
cargo test --bin nhl handler::tests
```

**All tests must pass, especially:**
- `test_complete_navigation_sequence_from_spec` - verifies the exact sequence above
- `test_navigation_within_window` - verifies window stays same when navigating within
- `test_navigation_at_left_edge_shifts_window` - verifies left edge shifting
- `test_navigation_at_right_edge_shifts_window` - verifies right edge shifting
- `test_window_calculation_from_base` - verifies window calculation formula

##### Critical Rules

1. **NEVER** change the window calculation formula without updating this spec
2. **ALWAYS** ensure game_date updates on every arrow key press (for refresh)
3. **ALWAYS** maintain window_base = game_date - selected_index invariant
4. **ALWAYS** run tests after any navigation-related changes
5. **REFRESH** must trigger on every date change (within window OR at edge)

##### Common Mistakes to Avoid

❌ Making game_date always at center (index 2)
❌ Not updating game_date when navigating within window
❌ Forgetting to trigger refresh on within-window navigation
❌ Shifting window when navigating within it
❌ Not keeping selected_index at edge when window shifts

✅ Window base is sticky until edge is reached
✅ game_date moves within the window
✅ Refresh triggers on every date change
✅ Window only shifts at edges (index 0 or 4)

#### Standings Subtab Navigation
- Left/Right arrows cycle through Division → Conference → League views

## User Navigation Behavior

This section documents the expected navigation behavior from a user's perspective. This serves as a specification for the current behavior and should be consulted when refactoring or adding features.

### Standings Tab Navigation

#### View Selection Mode (Initial State)

When the Standings tab is selected, the user starts in **view selection mode** focused on the subtab bar showing three view options:
- **Division** (default)
- **Conference**
- **League**

**Navigation keys in view selection mode:**

- **Left/Right arrows**: Cycle through views (Division → Conference → League → Division)
- **Down arrow**: Enter team selection mode (focus moves from view tabs to the standings content itself, highlighting the first team in the top-left column)
- **ESC**: Exit to main tabs (returns focus to the main tab bar: Scores, Standings, Stats, Players, Settings)

#### Team Selection Mode

When the user presses **Down** from view selection mode, they enter **team selection mode** where individual teams can be navigated and selected.

**Visual indicator**: The selected team's name is highlighted in the configured `selection_fg` color.

**Navigation keys in team selection mode:**

- **Up arrow**:
  - Move selection up to the previous team in the current column
  - If already on the first team in the column: Exit team selection mode and return to view selection mode

- **Down arrow**:
  - Move selection down to the next team in the current column
  - If already on the last team: No action (selection stays on last team)

- **Left arrow** (only in Conference and Division views with 2 columns):
  - Switch to the left column
  - Preserve the same row/rank position when switching columns
  - If the new column has fewer teams than the current row position, clamp to the last team in that column
  - In League view (single column): No action

- **Right arrow** (only in Conference and Division views with 2 columns):
  - Switch to the right column
  - Preserve the same row/rank position when switching columns
  - If the new column has fewer teams than the current row position, clamp to the last team in that column
  - In League view (single column): No action

- **ESC**: Exit team selection mode and return to view selection mode

- **Enter**: Log the selected team information (for debugging purposes - no visible action to user)

- **PageUp/PageDown/Home/End**: Scroll the viewport without changing team selection

#### Column Behavior by View

**League View:**
- Single column containing all 32 teams
- Teams sorted by points (highest to lowest)
- Left/Right arrows have no effect in team selection mode

**Conference View:**
- Two columns side-by-side
- Column order depends on `display_standings_western_first` config setting:
  - If `false` (default): Left column = Eastern Conference, Right column = Western Conference
  - If `true`: Left column = Western Conference, Right column = Eastern Conference
- Teams within each conference sorted by points (highest to lowest)
- Left/Right arrows switch between columns, preserving row position

**Division View:**
- Two columns side-by-side
- Column order depends on `display_standings_western_first` config setting:
  - If `false` (default): Left column = Eastern divisions (Atlantic, Metropolitan), Right column = Western divisions (Central, Pacific)
  - If `true`: Left column = Western divisions (Central, Pacific), Right column = Eastern divisions (Atlantic, Metropolitan)
- Teams grouped by division first, then sorted by points within each division
- Visual layout shows division headers and teams grouped together
- Left/Right arrows switch between columns, preserving row position within the multi-division column

#### Auto-scrolling Behavior

When navigating teams with Up/Down arrows:
- The viewport automatically scrolls to keep the selected team visible
- If the selected team would scroll above the visible area: viewport scrolls up to show it at the top
- If the selected team would scroll below the visible area: viewport scrolls down to show it at the bottom
- Scrolling is smooth and immediate (happens during navigation, not after)

**Manual scrolling** (PageUp/PageDown/Home/End):
- Available in both view selection mode and team selection mode
- Does not change which team is selected
- PageUp/PageDown: Scroll by 10 lines
- Home: Scroll to top
- End: Scroll to bottom

### Error Handling

Network and deserialization errors are **never** output to stderr/stdout (conflicts with ratatui):
- All errors stored in `SharedData.error_message`
- Displayed on status bar with red background and white text
- Format: `"ERROR: <message>"`
- Errors automatically cleared on next successful standings fetch

### NHL API Integration

Uses the `nhl_api` crate (local path dependency at `../nhl-api`):
- Client created with `Client::new()` (returns `Result`)
- Debug mode available via `ClientConfig` (not currently used in this project)
- All API calls return `Result` types

### Command Pattern

All CLI commands follow the pattern:
```rust
pub async fn run(client: &Client, /* command-specific params */)
```

### Configuration

Config file location: `~/.config/nhl/config.toml` (XDG standard)

Available settings:
- `debug`: Enable debug logging
- `refresh_interval`: Seconds between background data refreshes (default: 60)
- `display_standings_western_first`: Display Western Conference first in standings
- `time_format`: Time format string for status bar (default: "%H:%M:%S")

## Dependencies

Key dependencies:
- `nhl_api`: NHL API client (local path dependency)
- `clap` (4.5.40): CLI argument parsing with derive macros
- `ratatui` (0.29.0): Terminal UI framework
- `crossterm` (0.28.1): Cross-platform terminal manipulation
- `tokio` (1.x): Async runtime with full features
- `chrono` (0.4.42): Date/time handling
- `xdg` (3.0.0): XDG base directory specification support
- `futures` (0.3): For parallel async execution with `join_all`
- `serde` (1.0): Serialization/deserialization
- `toml` (0.8): TOML config file parsing

## Important Implementation Details

### Parallel API Requests
Game data fetching uses parallel execution for performance:
- Sequential: ~1730ms for 13 games
- Parallel: ~170ms for 13 games (10x speedup)
- Implementation in `fetch_data_loop` using `futures::future::join_all()`

### TUI Terminal Management
- Uses raw mode, alternate screen, and mouse capture
- 100ms event polling interval
- Proper cleanup on exit to restore terminal state

### Rendering Architecture

The rendering is delegated to tab-specific modules:

**Common rendering:**
- `common::tab_bar::render()`: Main navigation tabs with box-drawing characters
- `common::status_bar::render()`: Status bar with last refresh time or error messages

**Tab-specific rendering** (in `tui/mod.rs` main loop):
- `scores::render_subtabs()`: 3-date sliding window navigation
- `scores::render_content()`: Game scores display
- `standings::render_subtabs()`: Division/Conference/League selector
- `standings::render_content()`: Standings table
- `settings::render_content()`: Settings display (placeholder)

### State Management

**TUI State (AppState):**
- Composable design: each tab owns its own state struct
- `AppState` in `tui/app.rs` contains tab states: `scores`, `standings`, `settings`
- Tab navigation state: `current_tab: CurrentTab` enum (Scores, Standings, Settings)
- Each tab's state contains its specific UI state (e.g., `scores.selected_index`, `standings.view`)

**Application Data (SharedData):**
- Tracks application data fetched from API
- Shared between background fetch loop and TUI via `Arc<RwLock<SharedData>>`
- Clear separation: TUI state vs application data state

**Benefits of modular architecture:**
- Each tab is self-contained with its own state, view, and handler
- Easy to add new tabs or modify existing ones
- Clear separation of concerns
- Better code organization and maintainability

### Component vs Widget Architecture

The codebase has two distinct UI architectures:

#### Production TUI (`src/tui/mod.rs`)
Uses **widgets** from `src/tui/widgets/`:
- Direct implementation of `RenderableWidget` trait
- Render directly to ratatui `Buffer` with `DisplayConfig`
- Self-contained, reusable UI primitives with no framework overhead
- Used in production code
- Example: `widgets::TabBar`, `widgets::StatusBar`, `widgets::GameBox`

#### Experimental React-like Framework (`src/tui/mod_experimental.rs`)
Uses **components** from `src/tui/components/`:
- Implement the `Component` trait from `framework/component.rs`
- Build virtual `Element` trees (like React's virtual DOM)
- Compose using `vertical()` and `horizontal()` layout helpers
- Managed by `Runtime` with reducer pattern for state updates
- Experimental architecture - not used in production
- Example: `components::App`, `components::ScoresTab`, `components::StandingsTab`

#### RenderableWidget Trait Unification
Both architectures now share a **unified `RenderableWidget` trait**:
```rust
pub trait RenderableWidget: Send + Sync {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig);
    fn clone_box(&self) -> Box<dyn RenderableWidget>;
    fn preferred_height(&self) -> Option<u16> { None }
    fn preferred_width(&self) -> Option<u16> { None }
}
```

This allows widgets to work in both systems.

#### When to Use Each

**Use Widgets** (`src/tui/widgets/`):
- For production TUI code
- When you need a self-contained, reusable rendering primitive
- When performance matters (direct rendering, no virtual DOM overhead)
- For leaf-level display logic (tables, boxes, panels)

**Use Components** (`src/tui/components/`):
- For experimental React-like framework only
- When composing UI hierarchies with `vertical()`/`horizontal()`
- When building stateful, message-driven UIs
- When experimenting with new architectural patterns

**Note:** Legacy code in `src/tui/common/` (function-based renderers) has been removed. Only `widgets/` and `components/` remain.

## Requirements

- Rust 1.65 or later
- Use the tracing and tracing-subscriber crates for logging 
- Always use anyhow (especially error! and result) when you can
- inside the body of a function or struct, only comment things that are really not obvious.
- do an import instead of explicitely calling crate::formatting::format_header
- to only comment the non obvious inside a function body or structure
- remember that you cannot launch the tui because it would not be in a real tty
- avoid Unnecessary type repetition (e.g., GroupBy::Division should be Self::Division)%
- 90% minimum coverage for all new code
- avoid writing functions longer than 100 lines unless necessary or really better
- the lessons learned from this migration
- always add regression tests after fixing a problem
- remember to always be unicode-aware, dont rely on byte length for string length
- when i ask to do test of rendering, don't use "contains" or comparing with substrings. I want you to compare with literal strings like this 
let expected = "\ 
foo 
bar";
- when i ask to do test of rendering, don't use "contains" or comparing with substrings. I want you to compare with an array or a vector. If your rect is 80x3, then your array should have 3 lines of 80 chars to compare againast.
- never use any unsafe code
- ALWAYS user assert_buffer to test rendering
- if you have to modify a test and the test is not testing the rendering with assert_buffer, modify the test so it uses assert_buffer
- if you have to modify a test and the test is not testing the rendering with assert_buffer, modify the test so it uses assert_buffer. this is a HARD rule.
- mod_experinmental is gone, everything moved in mod.rs
- from now on, what you call "Production", I want you to call it "legacy". What you call experimental, i want you to call it "current"
- use the stuff in tui::testing when writing tests