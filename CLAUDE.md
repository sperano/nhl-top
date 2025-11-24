# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NHL CLI tool written in Rust that displays NHL stats, standings, and live scores. The application supports both command-line mode for quick queries and an interactive TUI (Terminal User Interface) mode built with ratatui.

## Specialized Agent Commands

This project includes specialized slash commands that act as domain experts for different aspects of the codebase. Use these commands when you need specialized help:

### Available Agent Commands

- **`/api-integrate`** - API Integration Specialist
  - Use when: Adding new NHL API endpoints to the TUI
  - Expertise: DataEffects, state management, async fetching, caching
  - Example: "I need to integrate the player career stats endpoint"

- **`/fixture-build`** - Fixture Builder Specialist
  - Use when: Creating test fixtures or mock data
  - Expertise: NHL data structures, realistic test data, fixture patterns
  - Example: "Create fixtures for testing overtime game scenarios"

- **`/navigation-debug`** - Navigation Debugger Specialist
  - Use when: Debugging keyboard navigation or focus issues
  - Expertise: Key event handling, focus hierarchy, panel stack
  - Example: "Why isn't the ESC key exiting content focus?"

- **`/test-write`** - Test Writer Specialist
  - Use when: Writing comprehensive unit tests
  - Expertise: assert_buffer, reducer testing, 90%+ coverage patterns
  - Example: "Write tests for the new standings layout reducer"

- **`/tui-architect`** - TUI Architecture Specialist
  - Use when: Making architectural decisions or adding major features
  - Expertise: React-like architecture, state design, component patterns
  - Example: "Should player stats be a new tab or a panel?"

### Using Agent Commands

These commands provide specialized guidance following established patterns in the codebase. When invoked, they will:
1. Analyze your request within their domain of expertise
2. Provide step-by-step implementation guidance
3. Generate code following project conventions
4. Suggest best practices specific to this codebase

Example usage:
```
User: "I need to add a new API endpoint for team rosters"
Assistant: Let me use the API integration specialist to help with this.
[Uses /api-integrate command]
```

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

# Development mode with mock data (requires development feature)
cargo build --features development
cargo run --features development -- --mock           # TUI with mock data
cargo run --features development -- --mock standings # Mock standings
cargo run --features development -- --mock schedule  # Mock schedule
cargo run --features development -- --mock scores    # Mock scores
```

## Development Features

The project includes a "development" feature flag that enables mock mode for development and testing. This is useful for:

- **Screenshots**: Taking consistent screenshots with deterministic data
- **Debugging**: Testing the application without network calls
- **Development**: Working on UI features without API rate limits
- **Testing**: Running integration tests with predictable data

### Mock Mode

When the development feature is enabled and the `--mock` flag is provided:

1. **Data Provider Abstraction**: Uses `NHLDataProvider` trait to abstract over real and mock clients
2. **Mock Client**: `MockClient` returns fixture data from `src/fixtures.rs`
3. **Fixture Data**: Realistic NHL data including all 32 teams, various game states (live, final, future)
4. **Consistent Data**: Always returns the same data for screenshots and testing

### Modules Added for Development

- **`src/data_provider.rs`**: Trait abstracting NHL data operations
- **`src/dev/mock_client.rs`**: Mock implementation of the data provider
- **`src/fixtures.rs`**: Fixture data generators for testing and development

## Architecture

### Two-Mode Design

The application operates in two distinct modes:

1. **CLI Mode**: Invoked when a subcommand is provided. Executes a single command, displays output, and exits.
2. **TUI Mode**: Invoked when no subcommand is provided. Launches an interactive terminal interface with tabs for different views (Scores, Standings, Settings).

The mode is determined in `src/main.rs` by checking if `cli.command.is_none()`.

### React-like Unidirectional Data Flow

The TUI uses a React/Redux-inspired architecture with unidirectional data flow:

```
┌─────────────┐     ┌──────────┐     ┌─────────────┐
│  Key Event  │────>│  Action  │────>│   Reducer   │
└─────────────┘     └──────────┘     └─────────────┘
                                            │
                                            v
┌─────────────┐     ┌──────────┐     ┌─────────────┐
│   Render    │<────│  Element │<────│    State    │
└─────────────┘     │   Tree   │     │  (AppState) │
                    └──────────┘     └─────────────┘
                          ^
                          │
                    ┌─────────────┐
                    │  Component  │
                    │   .view()   │
                    └─────────────┘
```

### Module Structure

#### Core Modules
- **`src/main.rs`**: Entry point with CLI parsing
- **`src/commands/`**: Each subcommand (standings, boxscore, schedule, scores, franchises) as separate modules
- **`src/tui/`**: TUI implementation with React-like framework
- **`src/config.rs`**: Configuration management using XDG base directories
- **`src/cache.rs`**: API response caching

#### TUI Module Structure

```
src/tui/
├── mod.rs              # Main entry point, TUI run loop
├── component.rs        # Core Component trait, Element types, SimpleWidget
├── action.rs           # Action enum (Redux-like actions)
├── state.rs            # AppState - single source of truth
├── reducer.rs          # Main reducer + settings reducer
├── reducers/           # Sub-reducers (modular)
│   ├── mod.rs
│   ├── navigation.rs   # Tab navigation actions
│   ├── panels.rs       # Panel stack management
│   ├── data_loading.rs # Data loaded actions
│   ├── document.rs     # Document navigation actions
│   ├── scores.rs       # Scores tab actions
│   ├── standings.rs    # Standings tab actions
│   └── standings_layout.rs  # Standings layout calculations
├── document/           # Document system for scrollable content
│   ├── mod.rs          # Document trait, DocumentView, FocusContext
│   ├── builder.rs      # DocumentBuilder for declarative document construction
│   ├── elements.rs     # DocumentElement enum (Heading, Text, Link, Row, etc.)
│   ├── focus.rs        # FocusManager, FocusableElement, RowPosition
│   ├── link.rs         # LinkTarget, DocumentLink for navigation
│   ├── viewport.rs     # Viewport for scroll management
│   └── widget.rs       # DocumentElementWidget for Element tree integration
├── runtime.rs          # Runtime - manages component lifecycle
├── renderer.rs         # Renders Element tree to ratatui Buffer
├── effects.rs          # DataEffects - async data fetching
├── keys.rs             # Keyboard event to Action mapping
├── navigation.rs       # Panel stack utilities (breadcrumbs)
├── types.rs            # Tab, Panel, SettingsCategory enums
├── helpers.rs          # UI helper functions
├── settings_helpers.rs # Settings-specific helpers
├── table.rs            # Generic table types (CellValue, ColumnDef)
├── testing.rs          # Test utilities and fixtures
├── integration_tests.rs # Integration tests (cfg(test))
├── components/         # React-like Components
│   ├── mod.rs
│   ├── app.rs          # Root App component
│   ├── tabbed_panel.rs # TabbedPanel (tabs + content)
│   ├── scores_tab.rs   # Scores tab component
│   ├── standings_tab.rs # Standings tab component
│   ├── settings_tab.rs # Settings tab component
│   ├── boxscore_panel.rs # Boxscore drill-down panel
│   ├── team_detail_panel.rs # Team detail drill-down panel
│   ├── player_detail_panel.rs # Player detail drill-down panel
│   ├── standings_panels.rs # Division/Conference/League/Wildcard panels
│   ├── status_bar.rs   # Status bar component
│   ├── breadcrumb.rs   # Breadcrumb navigation widget
│   ├── table.rs        # Generic Table component
│   ├── skater_stats_table.rs # Skater stats table
│   └── goalie_stats_table.rs # Goalie stats table
└── widgets/            # Low-level renderable widgets
    ├── mod.rs          # SimpleWidget trait
    ├── game_box.rs     # GameBox widget (score display)
    ├── score_table.rs  # ScoreTable widget
    ├── settings_list.rs # SettingsListWidget
    ├── list_modal.rs   # ListModalWidget (for selections)
    └── testing.rs      # Widget test utilities
```

### AppState Architecture

The TUI uses a single source of truth `AppState`:

```rust
pub struct AppState {
    pub navigation: NavigationState,  // current_tab, panel_stack, content_focused
    pub data: DataState,              // API data (Arc-wrapped), loading states, errors
    pub ui: UiState,                  // Tab-specific UI state (scores, standings, settings)
    pub system: SystemState,          // last_refresh, config, status_message
}
```

- **NavigationState**: Current tab, panel stack for drill-down views, content focus state
- **DataState**: All API data wrapped in `Arc` for efficient cloning, loading flags, error messages
- **UiState**: Per-tab UI state (selected indices, scroll positions, view modes)
- **SystemState**: Last refresh time, configuration, status messages

### Action/Reducer Pattern

**Actions** (`action.rs`):
```rust
pub enum Action {
    // Navigation
    NavigateTab(Tab), NavigateTabLeft, NavigateTabRight,
    EnterContentFocus, ExitContentFocus,
    PushPanel(Panel), PopPanel,

    // Data
    RefreshData, SetGameDate(GameDate),
    StandingsLoaded(Result<...>), ScheduleLoaded(Result<...>), ...

    // Tab-specific (nested)
    ScoresAction(ScoresAction),
    StandingsAction(StandingsAction),
    SettingsAction(SettingsAction),

    // System
    Quit, Error(String), SetStatusMessage { ... },
}
```

**Reducers** (`reducer.rs`, `reducers/`):
- Main `reduce()` function delegates to sub-reducers
- Each reducer returns `Option<(AppState, Effect)>` - None means didn't handle
- Sub-reducers: `reduce_navigation`, `reduce_panels`, `reduce_data_loading`, `reduce_scores`, `reduce_standings`

### Effects System

```rust
pub enum Effect {
    None,
    Action(Action),           // Dispatch immediately
    Batch(Vec<Effect>),       // Process multiple effects
    Async(Pin<Box<dyn Future<Output = Action> + Send>>),  // Async operation
}
```

`DataEffects` provides async data fetching methods:
- `fetch_standings()` -> Effect
- `fetch_schedule(date)` -> Effect
- `fetch_game_details(game_id)` -> Effect
- `fetch_boxscore(game_id)` -> Effect
- `fetch_team_roster_stats(abbrev)` -> Effect
- `fetch_player_stats(player_id)` -> Effect

All use caching via `cache` module.

### Runtime

The `Runtime` (`runtime.rs`) orchestrates the system:
- Holds the current `AppState`
- Dispatches actions through the reducer
- Executes side effects asynchronously via `DataEffects`
- Builds the virtual Element tree via `App.view()`
- Uses mpsc channels for action queue and effect queue

Key methods:
- `dispatch(action)` - Process action through reducer, queue effects
- `process_actions()` - Drain action queue, returns count processed
- `build()` -> Element - Build virtual tree from current state
- `action_sender()` - Get channel for external action dispatch

### TUI Navigation

#### Key Event Flow

```
KeyEvent -> key_to_action(key, state) -> Option<Action>
```

Priority:
1. Global keys (q=Quit, /=CommandPalette)
2. ESC key (priority-based hierarchy: panel -> modal -> browse mode -> content focus)
3. Panel navigation (when panel open)
4. Number keys (1-6 direct tab switching)
5. Tab bar focused: arrows navigate tabs
6. Content focused: delegated to tab-specific handlers

#### Focus Hierarchy
1. Tab bar (top level)
2. Content area (subtabs)
3. Item selection (within content)
4. Panel stack (drill-down views)

#### Panel Stack
- `panel_stack: Vec<PanelState>`
- Panel types: Boxscore, TeamDetail, PlayerDetail
- Each PanelState has: panel, scroll_offset, selected_index
- Breadcrumb navigation shows path

#### Main Tabs
- Left/Right arrows: Navigate between Scores, Standings, Settings
- Down arrow: Enter content focus (on Scores/Standings tabs)
- ESC: Context-dependent (pop panel, close modal, exit content focus, or quit)
- Number keys 1-6: Direct tab switching

#### Scores Tab Navigation (5-Date Sliding Window)

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

**Date Window Calculation (`src/tui/components/scores_tab.rs`):**
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

**Navigation via Reducer (`src/tui/reducers/scores.rs`):**
- Handles `ScoresAction::NavigateLeft`, `ScoresAction::NavigateRight`
- Updates `game_date` and `selected_index` in `UiState`
- Returns `Effect` to trigger data refresh

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

#### Standings Tab Navigation
- Left/Right arrows cycle through Division → Conference → League → Wildcard views
- Down arrow enters team selection mode
- Enter on team opens TeamDetail panel

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
- All errors stored in `AppState.data.error_message`
- Displayed on status bar with red background and white text
- Format: `"ERROR: <message>"`
- Errors automatically cleared on next successful data fetch

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
- Implementation via `DataEffects` using `futures::future::join_all()`

### TUI Terminal Management
- Uses raw mode, alternate screen, and mouse capture
- 100ms event polling interval
- Proper cleanup on exit to restore terminal state

### Rendering Architecture

The TUI uses a virtual DOM-like rendering approach:

1. **Component Tree**: `App.view()` builds an `Element` tree from current `AppState`
2. **Renderer**: `renderer.rs` takes the `Element` tree and renders to ratatui `Buffer`
3. **Layout**: `vertical()` and `horizontal()` helpers handle layout composition
4. **Widgets**: Leaf nodes implement `SimpleWidget` for direct buffer rendering

**Key rendering components:**
- `components::App` - Root component, composes TabbedPanel + StatusBar
- `components::TabbedPanel` - Tab bar + content area
- `components::ScoresTab`, `StandingsTab`, `SettingsTab` - Tab content
- `components::StatusBar` - Bottom status bar with refresh time

### Component vs Widget Architecture

The TUI has a unified architecture with two abstraction levels:

#### Components (`src/tui/components/`)
Components implement the `Component` trait for complex, composable UI:
```rust
pub trait Component: Send {
    type Props: Clone;
    type State: Default + Clone;
    type Message;

    fn init(_props: &Self::Props) -> Self::State { ... }
    fn update(&mut self, _msg: Self::Message, _state: &mut Self::State) -> Effect { ... }
    fn view(&self, props: &Self::Props, state: &Self::State) -> Element;
}
```

**Use Components for:**
- Composing UI hierarchies with `vertical()`/`horizontal()`
- Building stateful, message-driven UIs
- Tab content, panels, complex layouts
- Examples: `App`, `TabbedPanel`, `ScoresTab`, `StandingsTab`, `BoxscorePanel`

#### ElementWidget (`src/tui/component.rs`)
Widgets that participate in the Element tree implement `ElementWidget`:
```rust
pub trait ElementWidget: Send + Sync {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig);
    fn clone_box(&self) -> Box<dyn ElementWidget>;
    fn preferred_height(&self) -> Option<u16> { None }
    fn preferred_width(&self) -> Option<u16> { None }
}
```

**Use ElementWidget for:**
- Widgets wrapped in `Element::Widget(Box::new(...))`
- Widgets that need to be cloneable and thread-safe
- All component widgets (BoxscorePanelWidget, StatusBarWidget, etc.)

#### Standalone Widgets (`src/tui/widgets/`)
Simple widgets implement `SimpleWidget` (no `Send + Sync`, no `clone_box`):
```rust
pub trait SimpleWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, config: &DisplayConfig);
    fn preferred_height(&self) -> Option<u16> { None }
    fn preferred_width(&self) -> Option<u16> { None }
}
```

**Use SimpleWidget for:**
- Self-contained, reusable rendering primitives
- Direct buffer rendering with no framework overhead
- Widgets not used in the Element tree
- Examples: `GameBox`, `ScoreTable`

### Document System

The document system (`src/tui/document/`) provides scrollable, focusable content views. It's designed for content that exceeds viewport height and requires keyboard navigation through focusable elements.

#### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Document Trait                          │
│  - build(focus) -> Vec<DocumentElement>                     │
│  - focusable_positions() -> Vec<u16>                        │
│  - focusable_ids() -> Vec<FocusableId>                      │
│  - focusable_row_positions() -> Vec<Option<RowPosition>>    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    DocumentView                             │
│  - Holds Arc<dyn Document>                                  │
│  - Manages Viewport (scroll offset, height)                 │
│  - Manages FocusManager (focus state, navigation)           │
│  - Renders visible portion to Buffer                        │
└─────────────────────────────────────────────────────────────┘
```

#### Document Trait

Any struct implementing `Document` can be rendered with scrolling and focus:

```rust
pub trait Document: Send + Sync {
    /// Build the element tree (called on each render)
    fn build(&self, focus: &FocusContext) -> Vec<DocumentElement>;

    /// Document title for navigation/history
    fn title(&self) -> String;

    /// Unique document ID
    fn id(&self) -> String;

    // Default implementations provided:
    fn calculate_height(&self) -> u16;
    fn focusable_positions(&self) -> Vec<u16>;
    fn focusable_ids(&self) -> Vec<FocusableId>;
    fn focusable_row_positions(&self) -> Vec<Option<RowPosition>>;
}
```

The trait provides default implementations for extracting focusable metadata, which is used by reducers for navigation without rebuilding the entire document.

#### DocumentElement Types

```rust
pub enum DocumentElement {
    Heading { level: u8, text: String },
    Text(String),
    Link { id: String, text: String, target: LinkTarget, focused: bool },
    Separator,
    Blank,
    Group { children: Vec<DocumentElement> },
    Table { widget: TableWidget, name: String },
    Row { children: Vec<DocumentElement>, gap: u16 },  // Horizontal layout
    Custom { widget: Box<dyn ElementWidget>, height: u16 },
}
```

**Key element types:**
- **Link**: Focusable element with a `LinkTarget` for activation
- **Table**: Embeds a `TableWidget` with focusable cells
- **Row**: Horizontal layout enabling left/right navigation between children
- **Group**: Vertical container for nested elements

#### Row Navigation (Left/Right)

The `Row` element enables horizontal navigation between side-by-side content:

```rust
DocumentBuilder::new()
    .row(vec![
        DocumentElement::table(left_table, "left"),
        DocumentElement::table(right_table, "right"),
    ])
```

**How it works:**

1. When building focusable elements, each element in a Row gets a `RowPosition`:
   ```rust
   pub struct RowPosition {
       pub row_y: u16,      // Y position identifying the Row
       pub child_idx: usize, // 0 = leftmost child, 1 = next, etc.
       pub idx_within_child: usize, // Position within that child
   }
   ```

2. Left/Right arrows find the element with matching `row_y` and `idx_within_child` but in the adjacent `child_idx`

3. **Wrapping**: Left at leftmost child wraps to rightmost; Right at rightmost wraps to leftmost

**Example navigation:**
```
Row with two tables (5 rows each):
┌─────────────┐  ┌─────────────┐
│ Table Left  │  │ Table Right │
├─────────────┤  ├─────────────┤
│ Row 0 ◄─────┼──┼─► Row 0     │  ← Left/Right moves between tables
│ Row 1 ◄─────┼──┼─► Row 1     │     preserving row position
│ Row 2       │  │   Row 2     │
│ Row 3       │  │   Row 3     │
│ Row 4       │  │   Row 4     │
└─────────────┘  └─────────────┘
```

#### Focus Navigation

**Tab/Shift-Tab (Up/Down direction):**
- Cycles through all focusable elements in document order
- Wraps from last to first (Tab) or first to last (Shift-Tab)
- Autoscrolls viewport to keep focused element visible

**Left/Right (within Rows):**
- Only works when focused element is inside a Row
- Moves to same relative position in adjacent child
- Wraps around at edges

**Enter:**
- Activates the focused element
- Returns `LinkTarget` for navigation actions

#### DocumentBuilder

Declarative API for constructing documents:

```rust
let doc = DocumentBuilder::new()
    .heading(1, "Player Stats")
    .blank()
    .text("Top scorers this season:")
    .table(stats_table, "scorers")
    .separator()
    .link("back", "← Back", LinkTarget::Action("go_back".into()))
    .when(show_details, |b| b.text("Additional details..."))
    .for_each(players, |b, player| b.text(format!("- {}", player.name)))
    .build();
```

#### State Management Pattern

Documents integrate with the Redux-like state flow:

1. **Data loads** → Reducer rebuilds focusable metadata:
   ```rust
   let demo_doc = DemoDocument::new(Some(standings));
   state.ui.demo.focusable_positions = demo_doc.focusable_positions();
   state.ui.demo.focusable_ids = demo_doc.focusable_ids();
   state.ui.demo.focusable_row_positions = demo_doc.focusable_row_positions();
   ```

2. **Key event** → Action dispatched:
   ```rust
   Action::DocumentAction(DocumentAction::FocusNext)
   Action::DocumentAction(DocumentAction::FocusLeft)
   ```

3. **Reducer** updates focus index and scroll offset in state

4. **Render** → Component builds document with current focus, renders to buffer

#### Creating a New Document

1. **Define the document struct**:
   ```rust
   pub struct MyDocument {
       data: Vec<MyData>,
   }
   ```

2. **Implement Document trait**:
   ```rust
   impl Document for MyDocument {
       fn build(&self, focus: &FocusContext) -> Vec<DocumentElement> {
           DocumentBuilder::new()
               .heading(1, "My Document")
               .for_each(&self.data, |b, item| {
                   b.link(&item.id, &item.name, LinkTarget::Document(...))
               })
               .build()
       }

       fn title(&self) -> String { "My Document".into() }
       fn id(&self) -> String { "my_doc".into() }
   }
   ```

3. **Store focusable metadata in state** when data changes

4. **Handle navigation** via `DocumentAction` in reducer

### Testing Infrastructure

**Test utilities** (`tui/testing.rs`):
- `setup_test_render!()` macro - Create test buffer/state/config
- `assert_buffer()` - Compare buffer to expected lines
- `create_client()` - Arc-wrapped NHL API client
- `create_test_standings()` - Full 32-team fixtures
- Widget-specific tests in `widgets/testing.rs`

## Requirements

### General
- Rust 1.65 or later
- Use the tracing and tracing-subscriber crates for logging
- Always use anyhow (especially `anyhow::Result`) when you can
- Never use any unsafe code
- Remember that you cannot launch the TUI because it would not be in a real tty

### Code Style
- Inside the body of a function or struct, only comment things that are really not obvious
- Use imports instead of explicitly calling `crate::formatting::format_header`
- Avoid unnecessary type repetition (e.g., `GroupBy::Division` should be `Self::Division`)
- Avoid writing functions longer than 100 lines unless necessary or really better
- Always be unicode-aware, don't rely on byte length for string length

### Testing
- 90% minimum coverage for all new code
- Always add regression tests after fixing a problem
- Use `tui::testing` utilities when writing tests
- ALWAYS use `assert_buffer` to test rendering - this is a HARD rule
- If modifying a test that doesn't use `assert_buffer`, update it to use `assert_buffer`
- `assert_buffer` doesn't need strings to be padded; put the vec directly in the function call
- When testing rendering, compare with a vector of lines, not substrings or "contains"
- After writing new code, always ask if unit tests should be written. If yes, target 100% coverage. If that's impossible, ask about targeting 90% instead

### Architecture Notes
- The React-like framework is the current architecture (not "experimental")
- What was previously called "legacy" navigation patterns may still be referenced in some places
- put all the new markdown file you create for housekeeping in the new-doc directory
- TODO: there has been confusion about the two SimpleWidget traits. Rename them