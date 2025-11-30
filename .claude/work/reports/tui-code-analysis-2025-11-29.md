# TUI Code Analysis Report

**Date:** 2025-11-29
**Scope:** Comprehensive analysis of `src/tui/` codebase

---

## Executive Summary

The TUI codebase implements a React/Redux-inspired architecture for a terminal UI. While the design is sophisticated, there are significant opportunities for consolidation, performance improvement, and simplification. The document system is well-designed but underutilized, and the autoscrolling implementation is fragmented across multiple locations with inconsistent behavior.

---

## 1. Document System Usage

### Overview

The document system (`src/tui/document/`) provides viewport-based scrolling, focus management, and declarative element composition. It's a powerful abstraction but only partially adopted.

### Components Using Document System (10 of 19)

| Component | File | Usage Pattern |
|-----------|------|---------------|
| BoxscoreDocument | `components/boxscore_document.rs` | Full Document impl with tables |
| LeagueStandingsDocument | `components/standings_documents.rs:33` | Single table |
| ConferenceStandingsDocument | `components/standings_documents.rs:93` | Two side-by-side tables via Row |
| DivisionStandingsDocument | `components/standings_documents.rs:220` | 2x2 grid layout |
| WildcardStandingsDocument | `components/standings_documents.rs:358` | Wildcard view |
| PlayerDetailDocument | `components/player_detail_document.rs:188` | Career stats with TeamLink cells |
| TeamDetailDocument | `components/team_detail_document.rs:110` | Team roster tables |
| SettingsDocument | `components/settings_document.rs:103` | Focusable links |
| ScoresGridDocument | `components/scores_grid_document.rs:119` | GameBox grid |
| DemoDocument | `components/demo_tab.rs:323` | Showcase document |

### Components NOT Using Document System (9 of 19)

| Component | File | Could Benefit? |
|-----------|------|----------------|
| Breadcrumb | `breadcrumb.rs` | No - simple widget |
| GoalieStatsTable | `goalie_stats_table.rs` | Possibly - lacks scroll/focus |
| SkaterStatsTable | `skater_stats_table.rs` | Possibly - lacks scroll/focus |
| StandingsTable | `standings_table.rs` | No - used within documents |
| StatusBar | `status_bar.rs` | No - simple widget |
| Table | `table.rs` | No - base widget |
| TabbedPanel | `tabbed_panel.rs` | No - container |
| App | `app.rs` | No - root component |

### Document System Adoption Rate

**52% of components use the document system** (10/19).

The standalone table components (`GoalieStatsTable`, `SkaterStatsTable`) are used within documents, so this is reasonable. However, the existence of two navigation patterns (DocumentNavState for tabs, DocumentStackEntry for overlays) creates inconsistency.

---

## 2. React/Redux Pattern Adoption

### Pattern Implementation

The codebase implements a React-inspired component system with:

- **Component trait** (`component.rs:15-50`): `init()`, `update()`, `view()`, `did_update()`, `should_update()`
- **Action/Message system** (`action.rs:20-84`): Global actions + component messages
- **Reducers** (`reducers/`): State transformation functions
- **Effects** (`component.rs`): Async side effects

### Pattern Adherence by Component

**Well-Implemented:**
- `ScoresTab`: Local state (date window, focus), messages, proper effect returns
- `StandingsTab`: Local state (view grouping), `DocumentNavState` embedded
- `SettingsTab`: Local state (doc nav, modal), clean message handling

**Deviations:**
- **Document stack navigation uses global actions** (`document_stack.rs:14-16`): `DocumentSelectNext`, `DocumentSelectPrevious` are global actions that directly manipulate `AppState.navigation.document_stack` instead of using component messages
- **Duplicate navigation systems**: Tab content uses `DocumentNavState` (component-embedded), while overlays use `DocumentStackEntry` (global state)

### Boilerplate Problem

Every component repeats this exact boilerplate:

```rust
impl ComponentMessageTrait for ScoresTabMsg {
    fn apply(&self, state: &mut dyn Any) -> Effect {
        if let Some(scores_state) = state.downcast_mut::<ScoresTabState>() {
            let mut component = ScoresTab;
            component.update(self.clone(), scores_state)
        } else {
            Effect::None
        }
    }

    fn clone_box(&self) -> Box<dyn ComponentMessageTrait> {
        Box::new(self.clone())
    }
}
```

**Found in 4 files, totaling ~100 lines of duplicated code.**

---

## 3. Performance Issues

### Critical: State Cloning in Runtime

**File:** `runtime.rs:81-102`

The `dispatch()` method clones `AppState` **3 times per action**:

```rust
// Line 81: First clone
let (new_state, _reducer_effect) = reduce(self.state.clone(), action.clone(), ...);

// Line 90: Second clone
let mut new_state = self.state.clone();

// Line 102: Third clone (else branch)
let (new_state, reducer_effect) = reduce(self.state.clone(), action, ...);
```

**Impact:** Every key press, every async callback triggers multiple deep clones. Even with `Arc` wrappers for inner data, the top-level struct clone is expensive.

### Effect Combination Verbosity

**File:** `runtime.rs:119-144`

```rust
let mut effects = Vec::new();
if !matches!(reducer_effect, Effect::None) { effects.push(reducer_effect); }
if !matches!(boxscore_effect, Effect::None) { effects.push(boxscore_effect); }
// ... repeated 5 times
```

**Better:** Use iterator adapters:
```rust
[reducer_effect, boxscore_effect, team_detail_effect, ...]
    .into_iter()
    .filter(|e| !matches!(e, Effect::None))
    .collect()
```

### Conservative Widget Diffing

**File:** `renderer.rs:194-198`

```rust
(Element::Widget(_), Element::Widget(_)) => {
    // Conservative: assume widgets are always different
    false
}
```

All widgets re-render every frame even when unchanged.

### Reducer Cloning Pattern

**File:** `reducers/navigation.rs:11-18`

Every reducer path clones the state even if it won't modify it:
```rust
Action::NavigateTab(tab) => Some(navigate_to_tab(state.clone(), *tab)),
Action::NavigateTabLeft => Some(navigate_tab_left(state.clone())),
```

---

## 4. Code Extraction/Simplification Opportunities

### 4.1 ComponentMessageTrait Derive Macro

Create a macro to eliminate the 100+ lines of boilerplate:

```rust
#[derive(ComponentMessage)]
#[component(ScoresTab, ScoresTabState)]
pub enum ScoresTabMsg { ... }
```

### 4.2 Tab Component Generic

All tabs follow the same pattern:
- Props struct with API data + focus state
- Local state with `DocumentNavState` embedded
- Message enum with document navigation variants

Extract a generic `TabComponent<S, M>` trait.

### 4.3 Table Column Definition Builder

`skater_stats_table.rs` and `goalie_stats_table.rs` use nearly identical `ColumnDef::new()` patterns. Extract a shared column builder.

### 4.4 Unified Document Navigation

Merge `DocumentNavState` (component-embedded) and `DocumentStackEntry` navigation into a single abstraction. Currently:

- `document_nav.rs:264-313`: `autoscroll_to_focus()` with 3-line padding
- `document_stack.rs:117-157`: `ensure_focused_visible()` with 2-line padding

These are functionally identical with different padding values.

---

## 5. Idiomatic Rust Issues

### 5.1 Missing Iterator Adapters

**File:** `runtime.rs:246-258`
```rust
for game in &schedule.games {
    if game.game_state != GameState::Future && game.game_state != GameState::PreGame {
        effects.push(self.data_effects.fetch_game_details(game.id));
    }
}
```

**Should be:**
```rust
effects.extend(
    schedule.games.iter()
        .filter(|g| !matches!(g.game_state, GameState::Future | GameState::PreGame))
        .map(|g| self.data_effects.fetch_game_details(g.id))
);
```

### 5.2 Manual Downcast Pattern

The `ComponentMessageTrait::apply()` pattern repeats the same downcast logic everywhere. Should use a generic helper.

### 5.3 Vec Allocation for 0-2 Items

Multiple places allocate `Vec` for typically 0-2 effects. Consider `SmallVec<[Effect; 4]>`.

### 5.4 Excessive `.clone()` in Reducers

Reducers clone state at the start even if the action won't match. Pattern should be:
1. Match action first
2. Clone only when needed

---

## 6. Autoscrolling Analysis

### The Problem

Autoscrolling code is scattered across multiple locations with inconsistent behavior:

### Location 1: Document Stack (Global State)

**File:** `reducers/document_stack.rs:72-157`

```rust
const AUTOSCROLL_PADDING: u16 = 2;  // 2-line padding

fn ensure_focused_visible(doc_entry: &mut DocumentStackEntry) {
    // ... scrolls document_stack[].scroll_offset
}
```

**Used by:** Boxscore, TeamDetail, PlayerDetail overlays
**Triggered by:** `DocumentSelectNext`, `DocumentSelectPrevious` global actions

### Location 2: Component-Embedded (Local State)

**File:** `document_nav.rs:264-313`

```rust
const AUTOSCROLL_PADDING: u16 = 3;  // 3-line padding (different!)

pub fn autoscroll_to_focus(state: &mut DocumentNavState) {
    // ... scrolls DocumentNavState.scroll_offset
}
```

**Used by:** StandingsTab, ScoresTab, SettingsTab
**Triggered by:** `DocumentNavMsg::FocusNext`, `FocusPrev` component messages

### Inconsistencies

| Aspect | DocumentStack | DocumentNavState |
|--------|---------------|------------------|
| Padding | 2 lines | 3 lines |
| State location | Global `AppState` | Component local |
| Wrapping | No (clamped) | Yes (wraps around) |
| Left/Right nav | Not supported | Supported |
| Page up/down | Not supported | Supported |

### Root Cause

The split exists because:
1. **Overlay documents** (Boxscore, TeamDetail, PlayerDetail) need to persist scroll state when navigating between them, so they live in global state
2. **Tab documents** (Scores, Standings, Settings) are single-instance, so they use component-local state

However, the implementation diverged unnecessarily. They should share the same autoscroll logic.

### Recommended Fix

Extract autoscroll logic into a single function that operates on any `&mut` scroll state:

```rust
pub fn autoscroll_to_focus(
    scroll_offset: &mut u16,
    viewport_height: u16,
    element_y: u16,
    element_height: u16,
    padding: u16,
) { ... }
```

Then both `ensure_focused_visible()` and `DocumentNavState::autoscroll_to_focus()` call this.

---

## 7. Legacy Code to Remove

### 7.1 Disabled Tests

**File:** `keys.rs`
```rust
// TODO: Tests disabled - key handling refactored to use document system and component messages
// TODO: Enter activation not yet implemented
```

Either remove or re-enable these tests.

### 7.2 Commented-Out Code Blocks

**File:** `reducer.rs:91-127`

Large commented blocks for `SelectPlayer` and `SelectTeam` actions. Remove entirely.

### 7.3 Scattered TODO Items

| File | TODO |
|------|------|
| `settings_tab.rs` | "Other settings - TODO: implement text editing" |
| `app.rs:~270` | "Stats" and "Players" tabs with `Element::None` |
| `app.rs` | `team_view: TeamView::Away, // TODO: Store in doc_entry` |
| `data_loading.rs` | "TODO: Remove Schedule loading key" (multiple) |
| `document/mod.rs` | "TODO: Optimize by tracking if focus changed since last render" |

### 7.4 Unused Cfg Conditions

```
warning: unexpected `cfg` condition value: `disabled_tests`
```

Clean up or document these.

---

## 8. Summary of Recommendations

### High Priority

1. **Unify autoscroll logic** - Extract shared function, use consistent padding
2. **Reduce state cloning in Runtime.dispatch()** - Major performance impact
3. **Create ComponentMessageTrait derive macro** - Eliminate 100+ lines boilerplate

### Medium Priority

4. **Replace effect combination with iterator adapters** - Cleaner, more idiomatic
5. **Extract tab component generic** - Reduce duplication across tabs
6. **Clean up legacy code** - Remove TODOs, commented blocks, disabled tests

### Low Priority

7. **Use SmallVec for effect accumulation** - Minor perf improvement
8. **Add widget comparison for rendering** - Minor perf improvement
9. **Consolidate DocumentNavState and DocumentStackEntry** - Architectural cleanup

---

## Appendix: File Metrics

| Directory | Files | Lines (approx) |
|-----------|-------|----------------|
| `src/tui/document/` | 7 | ~3,900 |
| `src/tui/components/` | 20 | ~6,500 |
| `src/tui/reducers/` | 5 | ~1,800 |
| `src/tui/widgets/` | 3 | ~800 |
| `src/tui/` (root) | 12 | ~2,500 |
| **Total** | **47** | **~15,500** |

The codebase is reasonably well-structured but shows signs of rapid iteration (hence the TODOs, dual navigation systems, and disabled tests). The main concerns are performance in the hot path (dispatch/reducer) and maintenance burden from duplication.
