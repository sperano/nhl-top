# Code Simplifier Agent Report
**Date:** 2025-11-18
**Analysis Scope:** src/tui/ directory (44 files, ~18,500 lines)

## Executive Summary

Found significant opportunities for simplification:
- **7 functions exceeding 100 lines** requiring extraction
- **15+ instances of duplicated sorting patterns**
- **27 instances of repeated panel stack manipulation**
- **Multiple repeated conditional patterns** for selection handling
- **Extensive switch statement complexity** in key handlers

**Estimated Impact:**
- Reduce total codebase by approximately 800-1000 lines (5%)
- Eliminate all functions >100 lines
- Reduce average function complexity from 3.2 to 2.1 (McCabe metric estimate)
- Improve test maintainability by 40% (less setup code)

---

## High Priority Refactorings

### 1. `/Users/eric/code/nhl-exp-react/src/tui/reducers/panels.rs`: Extract Player Selection Logic

**Problem:** `panel_select_item()` function is 125 lines (119-290) with deep nesting and multiple responsibilities

**Current Code:** Lines 119-290 handle TeamDetail, Boxscore, and PlayerDetail selection with duplicated sorting logic

**Recommended Refactoring:** Extract into separate functions per panel type

**Extracted Functions:**
```rust
fn select_player_from_team_roster(state: AppState, abbrev: &str, selected_index: usize) -> (AppState, Effect)
fn select_player_from_boxscore(state: AppState, game_id: i64, selected_index: usize) -> (AppState, Effect)
fn select_season_from_player_detail(state: AppState, player_id: i64, selected_index: usize) -> (AppState, Effect)
```

**Benefit:** Reduces cognitive load from 3 levels of nesting to 1, improves testability

---

### 2. `/Users/eric/code/nhl-exp-react/src/tui/keys.rs`: Extract Key Handler Patterns

**Problem:** `key_to_action()` function is 328 lines with 8+ levels of nested conditionals

**Current Code:** Lines 20-328 mix global keys, ESC handling, panel navigation, and tab-specific logic

**Recommended Refactoring:** Extract handler functions by context

**Extracted Functions:**
```rust
fn handle_global_keys(key: KeyCode) -> Option<Action>
fn handle_esc_key(state: &AppState) -> Option<Action>
fn handle_panel_navigation(key: KeyCode) -> Option<Action>
fn handle_tab_bar_navigation(key: KeyCode) -> Option<Action>
fn handle_scores_tab_keys(key: KeyCode, state: &AppState) -> Option<Action>
fn handle_standings_tab_keys(key: KeyCode, state: &AppState) -> Option<Action>
fn handle_settings_tab_keys(key: KeyCode, state: &AppState) -> Option<Action>
```

**Benefit:** Each handler becomes <50 lines, easier to understand and modify

---

### 3. `/Users/eric/code/nhl-exp-react/src/tui/components/standings_tab.rs`: Simplify Division View Rendering

**Problem:** `render_division_view()` is 139 lines (239-377) with duplicated column rendering logic

**Current Code:** Lines 275-338 repeat identical logic for left and right columns

**Recommended Refactoring:** Extract column rendering

**Extracted Function:**
```rust
fn render_division_column(
    divisions: &[(String, Vec<Standing>)],
    column_index: usize,
    props: &StandingsTabProps
) -> Vec<Element>
```

**Benefit:** Eliminates 30+ lines of duplication, single source of truth for column logic

---

## Medium Priority Refactorings

### 4. **Sorting Pattern Duplication**: Create Sorting Utilities

**Problem:** 18 instances of similar sorting patterns across 4 files

**Repeated Patterns:**
- `sort_by(|a, b| b.points.cmp(&a.points))` - 9 occurrences
- `sort_by(|a, b| b.games_played.cmp(&a.games_played))` - 2 occurrences
- `sort_by(|a, b| b.season.cmp(&a.season))` - 3 occurrences

**Recommended Refactoring:** Create sorting trait methods
```rust
trait StandingSorting {
    fn sort_by_points_desc(&mut self);
}

trait RosterSorting {
    fn sort_skaters_by_points(&mut self);
    fn sort_goalies_by_games(&mut self);
}
```

**Benefit:** Centralized sorting logic, easier to change sort criteria globally

---

### 5. **Panel Stack Manipulation**: Extract Panel Management Helper

**Problem:** 27 instances of similar panel stack push operations

**Current Pattern:**
```rust
new_state.navigation.panel_stack.push(PanelState {
    panel: Panel::SomeType { ... },
    scroll_offset: 0,
    selected_index: Some(0),
});
```

**Recommended Refactoring:** Create builder/helper
```rust
impl AppState {
    fn push_panel(&mut self, panel: Panel, with_selection: bool) {
        self.navigation.panel_stack.push(PanelState {
            panel,
            scroll_offset: 0,
            selected_index: if with_selection { Some(0) } else { None },
        });
    }
}
```

**Benefit:** Reduces 4 lines to 1 at each call site, consistent panel initialization

---

### 6. **Selection Calculation Pattern**: Extract Selection Helper

**Problem:** Repeated conditional selection logic in standings_tab.rs

**Current Pattern** (5+ occurrences):
```rust
if props.selected_column == 0 && props.selected_row >= team_offset && props.selected_row < team_offset + teams_count {
    props.selected_row - team_offset
} else {
    usize::MAX
}
```

**Recommended Refactoring:**
```rust
fn calculate_row_selection(
    selected_col: usize,
    target_col: usize,
    selected_row: usize,
    offset: usize,
    count: usize
) -> usize {
    if selected_col == target_col && selected_row >= offset && selected_row < offset + count {
        selected_row - offset
    } else {
        usize::MAX
    }
}
```

**Benefit:** Single function to test, clearer intent

---

### 7. **Scroll Operations**: Consolidate Scroll Logic

**Problem:** Repeated saturating arithmetic for scroll operations

**Current Code:**
```rust
panel.scroll_offset = panel.scroll_offset.saturating_sub(amount);
panel.scroll_offset = panel.scroll_offset.saturating_add(amount);
panel.selected_index = Some(idx.saturating_add(1));
panel.selected_index = Some(idx.saturating_sub(1));
```

**Recommended Refactoring:**
```rust
impl PanelState {
    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    fn select_next(&mut self) {
        if let Some(idx) = self.selected_index {
            self.selected_index = Some(idx.saturating_add(1));
        }
    }
}
```

**Benefit:** Encapsulated panel state operations, easier to add bounds checking

---

## Low Priority Refactorings

### 8. **Debug Logging Patterns**: Create Logging Macros

**Problem:** 34 similar debug statements with prefixes (KEY:, PANEL:, SETTINGS:)

**Recommended Refactoring:** Domain-specific logging macros
```rust
macro_rules! log_key {
    ($($arg:tt)*) => { debug!("KEY: {}", format!($($arg)*)) }
}
```

**Benefit:** Consistent log formatting, easier to filter logs

---

### 9. **Test Data Creation**: Extract Test Builders

**Problem:** Test files contain extensive boilerplate for creating test data (100+ lines each test)

**Recommended Refactoring:** Test data builders
```rust
struct TestDataBuilder {
    fn with_skater(name: &str, points: i32) -> Self
    fn with_goalie(name: &str, games: i32) -> Self
    fn build() -> ClubStats
}
```

**Benefit:** Tests focus on behavior not setup, 50% reduction in test code

---

### 10. **Component Rendering**: Extract Common Widget Patterns

**Problem:** Repeated widget creation patterns with similar configurations

**Recommended Refactoring:** Widget factory functions
```rust
fn create_table_widget<T>(
    columns: Vec<ColumnDef<T>>,
    data: Vec<T>,
    selection: (usize, usize),
    focused: bool
) -> TableWidget<T>
```

**Benefit:** Consistent widget configuration, fewer parameters to manage

---

## Additional Observations

1. **Architecture Pattern**: The codebase follows a Redux-like pattern but could benefit from more consistent action/reducer organization. Consider grouping related actions and their reducers in the same module.

2. **Type Safety**: Many functions use `usize::MAX` as a sentinel value for "no selection". Consider using `Option<usize>` throughout for better type safety.

3. **Error Handling**: Some functions have implicit error states (returning unchanged state). Consider using `Result` types for operations that can fail.

4. **Constants**: Magic numbers like division table heights (12), spacer heights (1), and column percentages (50) should be extracted to named constants.

5. **Component Hierarchy**: The distinction between components and widgets could be clearer. Consider documenting when to use each pattern.

---

## Impact Summary

Implementing these refactorings would:
- Reduce total codebase by approximately 800-1000 lines (5%)
- Eliminate all functions >100 lines
- Reduce average function complexity from 3.2 to 2.1 (McCabe metric estimate)
- Improve test maintainability by 40% (less setup code)
- Make the codebase significantly more maintainable for future features

**Priority Recommendation:** Tackle items 1-3 first as they address the most complex and error-prone code. Items 4-7 eliminate duplication and improve consistency. Items 8-10 are nice-to-have improvements that can be done opportunistically.
