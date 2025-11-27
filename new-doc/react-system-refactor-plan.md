# React-Like Component System Refactoring Plan

**Created**: 2025-11-26

## Executive Summary

This document outlines a phased approach to refactor the NHL TUI app to actually use its React-like component system. Currently, the app has a well-designed Component trait with Props, State, Message, and lifecycle methods, but almost all components only use Props and `view()`. State is always `()`, Message is always `()`, and lifecycle methods are unused.

## Current Architecture Analysis

### What Exists

The Component trait in `src/tui/component.rs` defines:
```rust
pub trait Component: Send {
    type Props: Clone;
    type State: Default + Clone;
    type Message;

    fn init(_props: &Self::Props) -> Self::State { Self::State::default() }
    fn update(&mut self, _msg: Self::Message, _state: &mut Self::State) -> Effect { Effect::None }
    fn view(&self, props: &Self::Props, state: &Self::State) -> Element;
    fn should_update(&self, _old_props: &Self::Props, _new_props: &Self::Props) -> bool { true }
    fn did_update(&mut self, _old_props: &Self::Props, _new_props: &Self::Props) -> Effect { Effect::None }
}
```

### What Is Actually Used

| Component | Props | State | Message | init() | update() | should_update() | did_update() |
|-----------|-------|-------|---------|--------|----------|-----------------|--------------|
| App | AppState | () | () | default | default | default | default |
| TabbedPanel | TabbedPanelProps | () | () | default | default | default | default |
| ScoresTab | ScoresTabProps | () | () | default | default | default | default |
| StandingsTab | StandingsTabProps | () | () | default | default | default | default |
| SettingsTab | SettingsTabProps | () | () | default | default | default | default |
| DemoTab | DemoTabProps | DemoTabState | DemoTabMessage | implemented | implemented | default | default |
| StatusBar | SystemState | () | () | default | default | default | default |
| BoxscorePanel | BoxscorePanelProps | () | () | default | default | default | default |
| TeamDetailPanel | TeamDetailPanelProps | () | () | default | default | default | default |
| PlayerDetailPanel | PlayerDetailPanelProps | () | () | default | default | default | default |

**Key Observation**: DemoTab is the only component with actual State and Message types, but even its `update()` just logs messages and returns `Effect::None`.

### Problems with Current Approach

1. **All UI state is global**: `ScoresUiState`, `StandingsUiState`, `SettingsUiState`, `DocumentState` live in `AppState.ui`
2. **Tab-specific logic goes through global reducers**: Focus, scroll, selection all handled by `reducers/scores.rs`, `reducers/standings.rs`, etc.
3. **Components are stateless render functions**: They just transform props to elements
4. **Runtime ignores component lifecycle**: It only calls `app.view()`, never tracks instances or calls lifecycle methods
5. **Document system has parallel state management**: `DocumentState` duplicates focus/scroll state that should be in component State

### Target Architecture

Components should own their local UI state:
- **Component State**: focus index, scroll offset, selection, edit buffer, modal state
- **Component Messages**: FocusNext, FocusPrev, Scroll, Select, Edit, etc.
- **Props from parent**: API data, config, whether focused/active

Global state should only contain:
- Current tab (navigation)
- Panel stack (navigation)
- API data (DataState)
- System config and status

---

## Phase 1: Runtime Foundation

### Goal
Update the Runtime to track component instances and call lifecycle methods.

### Files to Modify

#### `src/tui/runtime.rs`

Current `build()` method (line 257-262):
```rust
pub fn build(&self) -> Element {
    use crate::tui::component::Component;
    use crate::tui::components::App;

    let app = App;
    app.view(&self.state, &())
}
```

New approach:
```rust
pub struct Runtime {
    state: AppState,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    effect_tx: mpsc::UnboundedSender<Effect>,
    data_effects: Arc<DataEffects>,

    // NEW: Component instance storage
    component_states: ComponentStateStore,
}

impl Runtime {
    pub fn build(&mut self) -> Element {
        let app = App;
        let app_state = self.component_states.get_or_init::<App>("app", &self.state);
        app.view(&self.state, app_state)
    }

    pub fn dispatch_to_component<C: Component>(
        &mut self,
        path: &str,
        msg: C::Message,
    ) -> Effect {
        if let Some(state) = self.component_states.get_mut::<C::State>(path) {
            let mut component = C::default();
            component.update(msg, state)
        } else {
            Effect::None
        }
    }
}
```

### New File: `src/tui/component_store.rs`

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Stores component states by path for lifecycle management
pub struct ComponentStateStore {
    states: HashMap<String, (TypeId, Box<dyn Any + Send + Sync>)>,
}

impl ComponentStateStore {
    pub fn new() -> Self {
        Self { states: HashMap::new() }
    }

    /// Get or initialize component state
    pub fn get_or_init<C: Component>(&mut self, path: &str, props: &C::Props) -> &C::State {
        let type_id = TypeId::of::<C::State>();

        self.states.entry(path.to_string())
            .or_insert_with(|| (type_id, Box::new(C::init(props))))
            .1
            .downcast_ref::<C::State>()
            .expect("State type mismatch")
    }

    /// Get mutable state for update
    pub fn get_mut<S: 'static + Send + Sync>(&mut self, path: &str) -> Option<&mut S> {
        self.states.get_mut(path)
            .and_then(|(_, state)| state.downcast_mut())
    }
}
```

### Migration Strategy

1. Add `ComponentStateStore` to Runtime without breaking existing code
2. Initially, all components still use `()` for State, so the store is empty
3. Components can opt-in one at a time to use actual state

### Tests to Write

- `test_component_store_init_creates_default_state` ✅
- `test_component_store_get_returns_same_instance` ✅
- `test_component_store_get_mut_allows_modification` ✅
- `test_runtime_builds_with_component_states` ✅

### ✅ CHECKPOINT: Phase 1 Complete

**Status**: COMPLETE
- ComponentStateStore implemented with full test coverage
- Runtime updated to use ComponentStateStore
- All existing tests pass (713 passed)
- Build succeeds with no errors

**What Changed**:
- Added `src/tui/component_store.rs` with ComponentStateStore
- Updated Component trait to require `State: Send + Sync + 'static`
- Updated Runtime to include `component_states: ComponentStateStore`
- Updated `Runtime::build()` to use component states (now requires `&mut self`)

**Next**: Proceed to Phase 2 when ready

---

## Phase 2: Message Dispatch System

### Goal
Create infrastructure for dispatching Messages to components instead of global Actions.

### Files to Modify

#### `src/tui/action.rs`

Add component message wrapper:
```rust
pub enum Action {
    // ... existing actions ...

    /// Dispatch a message to a specific component
    ComponentMessage {
        path: String,
        message: Box<dyn ComponentMessageTrait>,
    },
}

/// Trait for type-erased component messages
pub trait ComponentMessageTrait: Send + Sync {
    fn apply(&self, state: &mut dyn Any) -> Effect;
    fn clone_box(&self) -> Box<dyn ComponentMessageTrait>;
}
```

#### `src/tui/reducer.rs`

Handle ComponentMessage action:
```rust
pub fn reduce(state: AppState, action: Action, component_store: &mut ComponentStateStore) -> (AppState, Effect) {
    match action {
        Action::ComponentMessage { path, message } => {
            if let Some(component_state) = component_store.get_mut_any(&path) {
                let effect = message.apply(component_state);
                (state, effect)
            } else {
                (state, Effect::None)
            }
        }
        // ... existing reducers ...
    }
}
```

### New Pattern: Message Enum per Component

Each component defines its own Message enum:
```rust
// In scores_tab.rs
pub enum ScoresTabMsg {
    DateLeft,
    DateRight,
    EnterBoxSelection,
    ExitBoxSelection,
    SelectGame(usize),
    MoveSelection { dx: i32, dy: i32 },
}

impl Component for ScoresTab {
    type Message = ScoresTabMsg;

    fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
        match msg {
            ScoresTabMsg::DateLeft => {
                if state.selected_date_index > 0 {
                    state.selected_date_index -= 1;
                    state.game_date = state.game_date.add_days(-1);
                    Effect::Action(Action::RefreshData)
                } else {
                    Effect::None
                }
            }
            // ... other messages ...
        }
    }
}
```

---

## Phase 3: Migrate ScoresTab (Proof of Concept)

### Goal
Convert ScoresTab to use component State and Messages, proving the pattern works.

### Files to Modify

#### `src/tui/components/scores_tab.rs`

**Before (stateless):**
```rust
pub struct ScoresTab;

impl Component for ScoresTab {
    type Props = ScoresTabProps;
    type State = ();
    type Message = ();

    fn view(&self, props: &Self::Props, _state: &Self::State) -> Element { ... }
}
```

**After (stateful):**
```rust
/// State managed by ScoresTab component
#[derive(Clone, Default)]
pub struct ScoresTabState {
    pub selected_date_index: usize,
    pub game_date: GameDate,
    pub box_selection_active: bool,
    pub selected_game_index: Option<usize>,
    pub boxes_per_row: u16,
}

/// Messages handled by ScoresTab
#[derive(Clone, Debug)]
pub enum ScoresTabMsg {
    NavigateLeft,
    NavigateRight,
    EnterBoxSelection,
    ExitBoxSelection,
    SelectGame,
    MoveGameSelection { dx: i32, dy: i32 },
    UpdateLayout { boxes_per_row: u16 },
}

impl Component for ScoresTab {
    type Props = ScoresTabProps;
    type State = ScoresTabState;
    type Message = ScoresTabMsg;

    fn init(props: &Self::Props) -> Self::State {
        ScoresTabState {
            selected_date_index: 2, // Center of 5-date window
            game_date: props.initial_date.clone(),
            box_selection_active: false,
            selected_game_index: None,
            boxes_per_row: 2,
        }
    }

    fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
        match msg {
            ScoresTabMsg::NavigateLeft => {
                if state.selected_date_index > 0 {
                    state.selected_date_index -= 1;
                    state.game_date = state.game_date.add_days(-1);
                } else {
                    state.game_date = state.game_date.add_days(-1);
                }
                Effect::Action(Action::RefreshData)
            }
            // ... other message handlers ...
        }
    }

    fn view(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.render_date_tabs(props, state)
    }
}
```

### Key Pattern: Props vs State Split

**Props** (from parent, read-only):
```rust
pub struct ScoresTabProps {
    pub schedule: Arc<Option<DailySchedule>>,
    pub game_info: Arc<HashMap<i64, GameMatchup>>,
    pub period_scores: Arc<HashMap<i64, PeriodScores>>,
    pub focused: bool,
    pub config: Config,
}
```

**State** (owned by component, mutable via Messages):
```rust
pub struct ScoresTabState {
    pub selected_date_index: usize,
    pub game_date: GameDate,
    pub box_selection_active: bool,
    pub selected_game_index: Option<usize>,
    pub boxes_per_row: u16,
}
```

### Tests to Write

- `test_scores_tab_init_creates_default_state`
- `test_scores_tab_navigate_left_within_window`
- `test_scores_tab_navigate_left_at_edge_shifts_window`
- `test_scores_tab_navigate_right_within_window`
- `test_scores_tab_navigate_right_at_edge_shifts_window`
- `test_scores_tab_enter_box_selection`

---

## Phase 4: Migrate StandingsTab

### Goal
Migrate StandingsTab including integration with the document system.

### Files to Modify

#### `src/tui/components/standings_tab.rs`

```rust
#[derive(Clone, Default)]
pub struct StandingsTabState {
    pub view: GroupBy,
    pub browse_mode: bool,
    pub focus_index: Option<usize>,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    pub focusable_positions: Vec<u16>,
    pub focusable_ids: Vec<FocusableId>,
    pub focusable_row_positions: Vec<Option<RowPosition>>,
}

#[derive(Clone, Debug)]
pub enum StandingsTabMsg {
    CycleViewLeft,
    CycleViewRight,
    EnterBrowseMode,
    ExitBrowseMode,
    FocusNext,
    FocusPrev,
    FocusLeft,
    FocusRight,
    Scroll { delta: i16 },
    DataLoaded { standings: Vec<Standing> },
}

impl Component for StandingsTab {
    type Props = StandingsTabProps;
    type State = StandingsTabState;
    type Message = StandingsTabMsg;

    fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
        match msg {
            StandingsTabMsg::CycleViewLeft => {
                state.view = state.view.prev();
                state.focus_index = None;
                state.scroll_offset = 0;
                Effect::None
            }
            StandingsTabMsg::FocusNext => {
                if state.focusable_positions.is_empty() {
                    return Effect::None;
                }
                state.focus_index = match state.focus_index {
                    None => Some(0),
                    Some(idx) if idx + 1 >= state.focusable_positions.len() => {
                        state.scroll_offset = 0;
                        Some(0)
                    }
                    Some(idx) => Some(idx + 1),
                };
                self.autoscroll_to_focus(state);
                Effect::None
            }
            // ... other handlers ...
        }
    }

    fn did_update(&mut self, old_props: &Self::Props, new_props: &Self::Props) -> Effect {
        // Rebuild focusable metadata when standings data changes
        if !Arc::ptr_eq(&old_props.standings, &new_props.standings) {
            if let Some(standings) = new_props.standings.as_ref().as_ref() {
                return Effect::Action(Action::ComponentMessage {
                    path: "app/standings_tab".to_string(),
                    message: Box::new(StandingsTabMsg::DataLoaded {
                        standings: standings.clone(),
                    }),
                });
            }
        }
        Effect::None
    }
}
```

---

## Phase 5: Migrate SettingsTab

### Goal
SettingsTab is complex with modal state, editing state, and category navigation.

### State and Messages

```rust
#[derive(Clone, Default)]
pub struct SettingsTabState {
    pub selected_category: SettingsCategory,
    pub selected_setting_index: usize,
    pub settings_mode: bool,
    pub editing: bool,
    pub edit_buffer: String,
    pub modal_open: bool,
    pub modal_selected_index: usize,
}

#[derive(Clone, Debug)]
pub enum SettingsTabMsg {
    NavigateCategoryLeft,
    NavigateCategoryRight,
    EnterSettingsMode,
    ExitSettingsMode,
    MoveSelectionUp,
    MoveSelectionDown,
    ToggleBoolean { key: String },
    StartEditing { key: String },
    CancelEditing,
    AppendChar(char),
    DeleteChar,
    ModalMoveUp,
    ModalMoveDown,
    ModalConfirm,
    ModalCancel,
    CommitEdit { key: String },
}
```

### Challenge: Config Updates

Settings changes need to persist to Config in global state. Solution:
1. Keep Config in AppState (it's truly global)
2. Component sends `Action::UpdateConfig` effect
3. Global reducer updates Config and triggers save

```rust
fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
    match msg {
        SettingsTabMsg::ToggleBoolean { key } => {
            Effect::Action(Action::UpdateConfig {
                mutation: ConfigMutation::ToggleBoolean(key)
            })
        }
        // ...
    }
}
```

---

## Phase 6: Implement should_update for Performance

### Goal
Use `should_update()` to prevent unnecessary re-renders.

### Pattern

```rust
impl Component for ScoresTab {
    fn should_update(&self, old_props: &Self::Props, new_props: &Self::Props) -> bool {
        !Arc::ptr_eq(&old_props.schedule, &new_props.schedule)
            || !Arc::ptr_eq(&old_props.game_info, &new_props.game_info)
            || old_props.focused != new_props.focused
    }
}

impl Component for StandingsTab {
    fn should_update(&self, old_props: &Self::Props, new_props: &Self::Props) -> bool {
        !Arc::ptr_eq(&old_props.standings, &new_props.standings)
            || old_props.focused != new_props.focused
    }
}
```

---

## Phase 7: Fix DemoTab

### Goal
DemoTab already has State and Message types but doesn't use them. Make it actually work.

Move the document focus logic from `reducers/document.rs` into DemoTab's `update()`:

```rust
fn update(&mut self, msg: Self::Message, state: &mut Self::State) -> Effect {
    match msg {
        DemoTabMessage::FocusNext => {
            let count = state.focusable_positions.len();
            if count == 0 { return Effect::None; }

            match state.focus_index {
                None => state.focus_index = Some(0),
                Some(idx) if idx + 1 >= count => {
                    state.focus_index = Some(0);
                    state.scroll_offset = 0;
                }
                Some(idx) => state.focus_index = Some(idx + 1),
            }
            self.autoscroll_to_focus(state);
            Effect::None
        }
        // ... other handlers with actual logic ...
    }
}
```

---

## Phase 8: Clean Up Global State

### Goal
Remove deprecated fields from AppState once all components are migrated.

#### `src/tui/state.rs`

**Before:**
```rust
pub struct UiState {
    pub scores: ScoresUiState,        // DEPRECATED
    pub standings: StandingsUiState,  // DEPRECATED
    pub settings: SettingsUiState,    // DEPRECATED
    pub demo: DocumentState,          // DEPRECATED
    pub standings_doc: DocumentState, // DEPRECATED
}
```

**After:**
```rust
pub struct UiState;  // Empty - all UI state is in components
```

### Remove/Consolidate Reducers

- `src/tui/reducers/scores.rs` - Convert to message forwarder or remove
- `src/tui/reducers/standings.rs` - Convert to message forwarder or remove
- `src/tui/reducers/document.rs` - Remove (logic now in components)

---

## Phase 9: Update Key Event Routing

### Goal
Route key events directly to component Messages instead of global Actions.

### Current Flow
```
KeyEvent -> key_to_action(key, state) -> Option<Action> -> Runtime.dispatch() -> reduce()
```

### New Flow
```
KeyEvent -> key_to_message(key, state) -> ComponentMessage -> Runtime.dispatch_to_component()
```

#### `src/tui/keys.rs`

```rust
pub enum KeyResult {
    GlobalAction(Action),
    ComponentMessage { path: String, message: Box<dyn ComponentMessageTrait> },
    None,
}

pub fn key_to_result(key: KeyEvent, state: &AppState) -> KeyResult {
    // Global keys first
    if key.code == KeyCode::Char('q') {
        return KeyResult::GlobalAction(Action::Quit);
    }

    // Route to focused component
    match state.navigation.current_tab {
        Tab::Scores if state.navigation.content_focused => {
            match key.code {
                KeyCode::Left => KeyResult::ComponentMessage {
                    path: "app/scores_tab".to_string(),
                    message: Box::new(ScoresTabMsg::NavigateLeft),
                },
                // ...
            }
        }
        // ...
    }
}
```

---

## Key Design Decisions

### 1. Component Path Naming

Use hierarchical paths to identify components:
- `"app"` - Root App component
- `"app/scores_tab"` - Scores tab
- `"app/standings_tab"` - Standings tab
- `"app/settings_tab"` - Settings tab

### 2. Props vs State Division

**Props** (immutable input from parent):
- API data (schedule, standings, boxscores)
- Configuration
- Focus state from parent (`focused: bool`)

**State** (mutable, owned by component):
- Selection indices
- Scroll offsets
- Modal/editing state
- Derived data (focusable positions)

### 3. Effect Flow

Components return Effects that can:
1. Dispatch global Actions (e.g., `Action::RefreshData`)
2. Dispatch Messages to other components
3. Trigger async operations

### 4. Backward Compatibility Strategy

During migration:
1. Keep old Action types working (map to new Messages)
2. Keep old state fields (mark deprecated)
3. Add assertions to catch accidental old pattern usage
4. Remove old code only when all tests pass

---

## Implementation Order Summary

| Phase | Focus | Key Files |
|-------|-------|-----------|
| 1 | Runtime foundation | `runtime.rs`, new `component_store.rs` |
| 2 | Message dispatch | `action.rs`, `reducer.rs` |
| 3 | ScoresTab migration | `scores_tab.rs`, `reducers/scores.rs` |
| 4 | StandingsTab migration | `standings_tab.rs`, `reducers/standings.rs` |
| 5 | SettingsTab migration | `settings_tab.rs` |
| 6 | Performance optimization | All component files |
| 7 | DemoTab fix | `demo_tab.rs` |
| 8 | Global state cleanup | `state.rs`, `reducers/` |
| 9 | Key event routing | `keys.rs` |

---

## Success Criteria

1. All components use actual State and Message types (not `()`)
2. `AppState.ui` is empty or removed
3. Tab-specific logic lives in component `update()` methods
4. `should_update()` prevents unnecessary re-renders
5. `did_update()` handles side effects for prop changes
6. All existing tests pass
7. Runtime tracks component instances and calls lifecycle methods
